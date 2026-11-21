pub const MEADOW_RING_SIDE: i32 = 9;
pub const MEADOW_RING_SLOTS: usize = 81;

/// Stamp identifying the planetary face, sector coordinates, LOD band, and terrain revision in a toroidal grass slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeadowStamp {
    face: u8,
    coords: (i32, i32),
    lod_and_rev: (u8, u32),
}

impl MeadowStamp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(face: u8, coords: (i32, i32), lod_and_rev: (u8, u32)) -> Self {
        Self { face, coords, lod_and_rev }
    }

    pub const fn empty() -> Self {
        Self::new(u8::MAX, (i32::MIN, i32::MIN), (u8::MAX, 0))
    }
}

/// Polymorphic toroidal ring sweep state avoiding raw boolean flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeadowSweep {
    Streaming,
    Settled,
}

/// Toroidal `9×9` grass sector ring tracking which `16×16` meadow VBO slots remain valid in VRAM.
#[derive(Clone, Debug)]
pub struct MeadowRing {
    stamps: [MeadowStamp; MEADOW_RING_SLOTS],
    sweep: MeadowSweep,
}

impl MeadowRing {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(stamps: [MeadowStamp; MEADOW_RING_SLOTS], sweep: MeadowSweep) -> Self {
        Self { stamps, sweep }
    }

    pub const fn cold() -> Self {
        Self::new([MeadowStamp::empty(); MEADOW_RING_SLOTS], MeadowSweep::Streaming)
    }

    pub const fn streaming(&self) -> bool {
        matches!(self.sweep, MeadowSweep::Streaming)
    }

    pub fn reopen(&mut self) {
        self.sweep = MeadowSweep::Streaming;
    }

    pub fn mark_complete(&mut self) {
        self.sweep = MeadowSweep::Settled;
    }

    pub fn invalidate_all(&mut self) {
        self.stamps = [MeadowStamp::empty(); MEADOW_RING_SLOTS];
        self.sweep = MeadowSweep::Streaming;
    }

    pub fn ring_slot(gsu: i32, gsv: i32) -> usize {
        let ru = gsu.rem_euclid(MEADOW_RING_SIDE) as usize;
        let rv = gsv.rem_euclid(MEADOW_RING_SIDE) as usize;
        rv * (MEADOW_RING_SIDE as usize) + ru
    }

    pub fn accept_if_stale(&mut self, slot: usize, next: MeadowStamp) -> bool {
        if self.stamps[slot] == next {
            false
        } else {
            self.stamps[slot] = next;
            self.sweep = MeadowSweep::Streaming;
            true
        }
    }
}
