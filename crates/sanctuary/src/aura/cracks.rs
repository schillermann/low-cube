use std::collections::HashMap;
use glam::Vec3;
use world::CosmicFrame;
use crate::audience::NEARBY_SIGHT_METERS;
use crate::envelope::CrackNotice;

/// Author-tagged transient dig crack preventing echo feedback to its originating companion.
#[derive(Clone, Copy, Debug)]
pub struct PeerCrack {
    author_slot: u32,
    notice: CrackNotice,
}

impl PeerCrack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(author_slot: u32, notice: CrackNotice) -> Self {
        Self { author_slot, notice }
    }

    pub const fn authored_by(self, slot: u32) -> bool {
        self.author_slot == slot
    }

    pub const fn notice(self) -> CrackNotice {
        self.notice
    }
}

/// Spatially bucketed transient dig cracks (`16` per `(CosmicFrame, 160 m)` sector) preventing cross-zone eviction.
pub struct SectorCracks {
    buckets: HashMap<(CosmicFrame, i32, i32, i32), Vec<PeerCrack>>,
}

impl SectorCracks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(buckets: HashMap<(CosmicFrame, i32, i32, i32), Vec<PeerCrack>>) -> Self {
        Self { buckets }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new())
    }

    fn cell_key(frame: CosmicFrame, pos: Vec3) -> (CosmicFrame, i32, i32, i32) {
        (
            frame,
            (pos.x / NEARBY_SIGHT_METERS).floor() as i32,
            (pos.y / NEARBY_SIGHT_METERS).floor() as i32,
            (pos.z / NEARBY_SIGHT_METERS).floor() as i32,
        )
    }

    pub fn record(&mut self, author_slot: u32, notice: CrackNotice) {
        self.record_in_frame(author_slot, CosmicFrame::earth(), notice);
    }

    pub fn record_in_frame(&mut self, author_slot: u32, frame: CosmicFrame, notice: CrackNotice) {
        let eye = notice.eye_point();
        if !eye.is_finite() {
            return;
        }
        if self.buckets.len() >= 256 {
            self.buckets.clear();
        }
        let bucket = self.buckets.entry(Self::cell_key(frame, eye)).or_default();
        if bucket.len() >= 16 {
            bucket.remove(0);
        }
        bucket.push(PeerCrack::new(author_slot, notice));
    }

    pub fn drain_into_buckets(&mut self) -> Self {
        Self::new(std::mem::take(&mut self.buckets))
    }

    pub fn collect_near(&self, anchor: Vec3, exclude_slot: u32, radius: f32) -> Vec<CrackNotice> {
        self.collect_in_frame(CosmicFrame::earth(), anchor, exclude_slot, radius)
    }

    pub fn collect_in_frame(&self, frame: CosmicFrame, anchor: Vec3, exclude_slot: u32, radius: f32) -> Vec<CrackNotice> {
        if !anchor.is_finite() || self.buckets.is_empty() {
            return Vec::new();
        }
        let (_, cx, cy, cz) = Self::cell_key(frame, anchor);
        let mut out = Vec::new();
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if let Some(list) = self.buckets.get(&(frame, cx + dx, cy + dy, cz + dz)) {
                        for &c in list {
                            if !c.authored_by(exclude_slot) && c.notice().near(anchor, radius) && out.len() < 16 {
                                out.push(c.notice());
                            }
                        }
                    }
                }
            }
        }
        out
    }
}
