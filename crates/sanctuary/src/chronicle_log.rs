use std::cell::Cell;
use glam::Vec3;
use world::{ClimateZone, WorldFact};

/// Interior-mutable bitmask of GPU face/flora slices modified since the last mesh upload.
pub struct DirtySlices {
    bits: Cell<u16>,
}

impl DirtySlices {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bits: Cell<u16>) -> Self {
        Self { bits }
    }

    pub const fn clean() -> Self {
        Self::new(Cell::new(0))
    }

    pub fn pending(&self) -> bool {
        self.bits.get() != 0
    }

    pub fn mark_slice(&self, slice_idx: usize) {
        if slice_idx < 16 {
            self.bits.set(self.bits.get() | (1u16 << slice_idx));
        }
    }

    fn zone_index(pos: Vec3) -> usize {
        let dom = ClimateZone::dominant(pos);
        ClimateZone::all().iter().position(|&z| z == dom).unwrap_or(0)
    }

    pub fn mark_fact(&self, fact: &WorldFact, face_count: usize) {
        let count = face_count.max(1);
        match *fact {
            WorldFact::StratumDisclosed { face, .. } => {
                self.mark_slice((face as usize).min(count - 1));
            }
            WorldFact::TimberStruck { eye, direction }
            | WorldFact::Foraged { eye, direction }
            | WorldFact::LitterCast { eye, direction, .. } => {
                let p0 = Self::zone_index(eye).min(count - 1);
                let p1 = Self::zone_index(eye + direction.normalize_or_zero() * 1.25).min(count - 1);
                self.mark_slice(count + p0);
                self.mark_slice(count + p1);
            }
            WorldFact::Loosened { eye, direction, .. }
            | WorldFact::Harvested { eye, direction }
            | WorldFact::MoldPlaced { eye, direction, .. }
            | WorldFact::RigAdjusted { eye, direction, .. }
            | WorldFact::RigLowered { eye, direction } => {
                let p0 = Self::zone_index(eye).min(count - 1);
                let p1 = Self::zone_index(eye + direction.normalize_or_zero() * 1.25).min(count - 1);
                self.mark_slice(p0);
                self.mark_slice(p1);
            }
        }
    }

    pub fn pop_next(&self) -> Option<usize> {
        let cur = self.bits.get();
        if cur == 0 {
            return None;
        }
        let idx = cur.trailing_zeros() as usize;
        self.bits.set(cur & !(1u16 << idx));
        Some(idx)
    }
}

/// Ordered log of durable `WorldFact`s enabling loss-tolerant UDP streaming and late-joiner sync.
pub struct ChronicleLog {
    facts: Vec<WorldFact>,
    slices: DirtySlices,
}

impl ChronicleLog {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(facts: Vec<WorldFact>, slices: DirtySlices) -> Self {
        Self { facts, slices }
    }

    pub const fn empty() -> Self {
        Self::new(Vec::new(), DirtySlices::clean())
    }

    pub fn total(&self) -> u32 {
        self.facts.len() as u32
    }

    pub fn slices(&self) -> &DirtySlices {
        &self.slices
    }

    pub fn record(&mut self, fact: WorldFact, face_count: usize) {
        self.slices.mark_fact(&fact, face_count);
        if fact.durable() {
            self.facts.push(fact);
        }
    }

    /// Collects an MTU-safe batch of durable facts starting at `ack_seq` (`<= 24` facts per datagram).
    pub fn batch_from(&self, ack_seq: u32) -> Option<(u32, Vec<WorldFact>)> {
        let start = ack_seq as usize;
        if start >= self.facts.len() {
            return None;
        }
        let end = (start + 24).min(self.facts.len());
        Some((ack_seq, self.facts[start..end].to_vec()))
    }
}
