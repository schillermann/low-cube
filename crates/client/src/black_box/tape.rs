use super::{FlightEvent, FlightTape, IncidentMedium, TickTrace};

const RING_CAPACITY: usize = 1_800;
const EVENT_CAPACITY: usize = 64;

struct TickRing {
    entries: Vec<TickTrace>,
    cursor: usize,
    total_recorded: usize,
}

impl TickRing {
    const fn new(entries: Vec<TickTrace>, cursor: usize, total_recorded: usize) -> Self {
        Self { entries, cursor, total_recorded }
    }

    fn preallocated() -> Self {
        Self::new(Vec::with_capacity(RING_CAPACITY), 0, 0)
    }

    fn count(&self) -> usize {
        self.entries.len()
    }

    const fn total(&self) -> usize {
        self.total_recorded
    }

    fn push(&mut self, trace: TickTrace) {
        if self.entries.len() < RING_CAPACITY {
            self.entries.push(trace);
        } else {
            self.entries[self.cursor] = trace;
        }
        self.cursor = (self.cursor + 1) % RING_CAPACITY;
        self.total_recorded = self.total_recorded.saturating_add(1);
    }

    fn chronological<F: FnMut(usize, TickTrace)>(&self, mut consumer: F) {
        let len = self.entries.len();
        if len == 0 {
            return;
        }
        let start = if len < RING_CAPACITY { 0 } else { self.cursor };
        for idx in 0..len {
            let slot = (start + idx) % len;
            consumer(idx, self.entries[slot]);
        }
    }

    fn boundary_pair(&self) -> Option<(TickTrace, TickTrace)> {
        let len = self.entries.len();
        if len == 0 {
            return None;
        }
        let first_slot = if len < RING_CAPACITY { 0 } else { self.cursor };
        let last_slot = (first_slot + len - 1) % len;
        Some((self.entries[first_slot], self.entries[last_slot]))
    }
}

struct EventRing {
    entries: Vec<(usize, FlightEvent)>,
    cursor: usize,
}

impl EventRing {
    const fn new(entries: Vec<(usize, FlightEvent)>, cursor: usize) -> Self {
        Self { entries, cursor }
    }

    fn preallocated() -> Self {
        Self::new(Vec::with_capacity(EVENT_CAPACITY), 0)
    }

    fn push(&mut self, tick_number: usize, event: FlightEvent) {
        if self.entries.len() < EVENT_CAPACITY {
            self.entries.push((tick_number, event));
        } else {
            self.entries[self.cursor] = (tick_number, event);
        }
        self.cursor = (self.cursor + 1) % EVENT_CAPACITY;
    }

    fn chronological<F: FnMut(usize, FlightEvent)>(&self, window_base: usize, mut consumer: F) {
        let len = self.entries.len();
        if len == 0 {
            return;
        }
        let start = if len < EVENT_CAPACITY { 0 } else { self.cursor };
        for idx in 0..len {
            let (abs_tick, ev) = self.entries[(start + idx) % len];
            if abs_tick >= window_base {
                consumer(abs_tick - window_base, ev);
            }
        }
    }
}

/// Null-Object `FlightTape` discarding all telemetry with zero overhead when switched off.
pub struct SilentTape;

impl SilentTape {
    /// Primary constructor initializing the silent Null-Object tape.
    pub const fn new() -> Self {
        Self
    }
}

impl FlightTape for SilentTape {
    fn active(&self) -> bool {
        false
    }

    fn populated(&self) -> bool {
        false
    }

    fn summary_counts(&self) -> (usize, u8) {
        (0, 0)
    }

    fn record_tick(&mut self, _world_blocks: u32, _trace: TickTrace) {}

    fn record_event(&mut self, _event: FlightEvent) {}

    fn toggled(&self) -> Box<dyn FlightTape> {
        Box::new(LiveTape::fresh())
    }

    fn inscribe_dossier(&self, _medium: &mut dyn IncidentMedium) -> Option<u8> {
        None
    }
}

/// Pre-allocated 15-second (1,800 ticks @ 120 Hz) flight black-box ring buffer performing zero runtime heap allocations.
pub struct LiveTape {
    ticks: TickRing,
    events: EventRing,
    world_blocks: u32,
    anomalies: u8,
}

impl LiveTape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(ticks: TickRing, events: EventRing, world_blocks: u32, anomalies: u8) -> Self {
        Self { ticks, events, world_blocks, anomalies }
    }

    /// Secondary constructor pre-allocating fixed ring buffers once outside the simulation loop.
    pub fn fresh() -> Self {
        Self::new(TickRing::preallocated(), EventRing::preallocated(), 128, 0)
    }
}

impl FlightTape for LiveTape {
    fn active(&self) -> bool {
        true
    }

    fn populated(&self) -> bool {
        self.ticks.count() > 0
    }

    fn summary_counts(&self) -> (usize, u8) {
        (self.ticks.count(), self.anomalies)
    }

    fn record_tick(&mut self, world_blocks: u32, trace: TickTrace) {
        if world_blocks > 0 {
            self.world_blocks = world_blocks;
        }
        if let Some(kind) = trace.anomaly() {
            self.anomalies = self.anomalies.saturating_add(1);
            self.events.push(self.ticks.total(), FlightEvent::Anomaly(kind));
        }
        self.ticks.push(trace);
    }

    fn record_event(&mut self, event: FlightEvent) {
        self.events.push(self.ticks.total(), event);
    }

    fn toggled(&self) -> Box<dyn FlightTape> {
        Box::new(SilentTape::new())
    }

    fn inscribe_dossier(&self, medium: &mut dyn IncidentMedium) -> Option<u8> {
        let (first, last) = self.ticks.boundary_pair()?;
        let count = self.ticks.count();
        medium.record_header(self.world_blocks, count, self.anomalies);
        medium.record_boundary("window_start", first, first.climate_zone());
        medium.record_boundary("incident_moment", last, last.climate_zone());
        let window_base = self.ticks.total().saturating_sub(count);
        self.events.chronological(window_base, |off, ev| medium.record_event(off, ev));
        self.ticks.chronological(|idx, trace| medium.record_tick(idx, trace));
        Some(self.anomalies)
    }
}
