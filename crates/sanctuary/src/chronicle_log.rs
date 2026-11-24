use world::{PlanetaryWorld, WorldFact};

/// Polymorphic log slot avoiding nullable `Option<WorldFact>` state when sector history is compacted.
#[derive(Clone, Debug)]
pub enum ChronicleEntry {
    Superseded,
    Active(WorldFact),
}

/// Segregated contract for authoritative durable `WorldFact` storage and MTU-bounded sync batches.
pub trait ChronicleStore: Send {
    fn total(&self) -> u32;
    fn record(&mut self, fact: WorldFact);
    fn batch_from(&self, ack_seq: u32) -> Option<(u32, Vec<WorldFact>)>;
    fn fact_at(&self, _index: usize) -> Option<&WorldFact> {
        None
    }
    fn supersede(&mut self, _index: u32) {}
    fn compact_sectors(&mut self, _earth_half: f32, _world: &mut dyn PlanetaryWorld) -> Vec<u32> {
        Vec::new()
    }
    fn flush_tick(&mut self) {}
}

/// Ordered in-memory log of durable `WorldFact`s enabling loss-tolerant UDP streaming and late-joiner sync.
pub struct ChronicleLog {
    entries: Vec<ChronicleEntry>,
}

impl ChronicleLog {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_entries(entries: Vec<ChronicleEntry>) -> Self {
        Self { entries }
    }

    /// Secondary constructor converting active `WorldFact`s into log entries.
    pub fn new(facts: Vec<WorldFact>) -> Self {
        Self::with_entries(facts.into_iter().map(ChronicleEntry::Active).collect())
    }

    /// Secondary constructor for an empty authoritative chronicle.
    pub const fn empty() -> Self {
        Self::with_entries(Vec::new())
    }
}

impl ChronicleStore for ChronicleLog {
    fn total(&self) -> u32 {
        self.entries.len() as u32
    }

    fn record(&mut self, fact: WorldFact) {
        if fact.durable() {
            self.entries.push(ChronicleEntry::Active(fact));
        }
    }

    fn batch_from(&self, ack_seq: u32) -> Option<(u32, Vec<WorldFact>)> {
        let start = ack_seq as usize;
        if start >= self.entries.len() {
            return None;
        }
        let mut scratch = Vec::with_capacity(128);
        let mut used_bytes = 0usize;
        let mut batch = Vec::with_capacity(24);
        for entry in &self.entries[start..] {
            let ChronicleEntry::Active(fact) = entry else { continue };
            scratch.clear();
            fact.inscribe(&mut scratch);
            if !batch.is_empty() && used_bytes + scratch.len() > 1_100 {
                break;
            }
            used_bytes += scratch.len();
            batch.push(fact.clone());
            if batch.len() >= 24 {
                break;
            }
        }
        (!batch.is_empty()).then_some((ack_seq, batch))
    }

    fn fact_at(&self, index: usize) -> Option<&WorldFact> {
        match self.entries.get(index) {
            Some(ChronicleEntry::Active(fact)) => Some(fact),
            _ => None,
        }
    }

    fn supersede(&mut self, index: u32) {
        if let Some(slot) = self.entries.get_mut(index as usize) {
            *slot = ChronicleEntry::Superseded;
        }
    }
}
