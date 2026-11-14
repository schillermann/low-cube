mod activity;
mod habitat;
mod mesh;

use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld};
use crate::cardiac::{AcousticWave, RoeBark, RoeHoofbeats};
use crate::sparrow::SparrowFlock;
use self::activity::RoeDeer;
#[cfg(test)]
pub use self::activity::{RoeActivity, RoeDeer as TestRoeDeer};

/// Small woodland roe deer herd ("Sprung") inhabiting `ClimateZone::Temperate` oak glades and meadows.
#[derive(Clone, Copy, Debug)]
pub struct RoeHerd {
    deer: [RoeDeer; 4],
    sound_cooldown: f32,
    anchored: bool,
    bark_turn: bool,
}

impl RoeHerd {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(deer: [RoeDeer; 4], sound_cooldown: f32, anchored: bool, bark_turn: bool) -> Self {
        Self {
            deer,
            sound_cooldown,
            anchored,
            bark_turn,
        }
    }

    /// Secondary constructor initializing two roebucks and two does ready to anchor on `ClimateZone::Temperate`.
    pub const fn seeded() -> Self {
        Self::new(
            [
                RoeDeer::seeded(1301, true),
                RoeDeer::seeded(2417, false),
                RoeDeer::seeded(3529, true),
                RoeDeer::seeded(4639, false),
            ],
            0.0,
            false,
            false,
        )
    }

    #[cfg(test)]
    pub const fn deer(&self) -> &[RoeDeer; 4] {
        &self.deer
    }

    #[cfg(test)]
    pub const fn bark_queued(&self) -> bool {
        self.bark_turn
    }

    /// Steps all roe deer on `ClimateZone::Temperate`, alternating spatialized hoofbeats and laryngeal alarm barks when startled.
    pub fn step_spatial(
        &mut self,
        dt: f32,
        listener: Vec3,
        view_dir: Vec3,
        crouching: bool,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        if !self.anchored {
            for (idx, member) in self.deer.iter_mut().enumerate() {
                member.anchor_in_temperate(idx, listener, world);
            }
            self.anchored = true;
        }
        self.sound_cooldown = (self.sound_cooldown - dt).max(0.0);
        let mut startled: Option<(usize, Vec3, f32)> = None;
        for (idx, member) in self.deer.iter_mut().enumerate() {
            if member.step(dt, listener, crouching, world) {
                let pos = member.position();
                let dist_sq = (pos - listener).length_squared();
                if dist_sq <= 42.0 * 42.0 && startled.is_none_or(|(_, _, best_sq)| dist_sq < best_sq) {
                    startled = Some((idx, pos, dist_sq));
                }
            }
        }
        if self.sound_cooldown <= 0.0 {
            if let Some((idx, deer_pos, dist_sq)) = startled {
                let up = world.surface_up_at(listener);
                let seed = (idx as u32).wrapping_mul(747_796_405).wrapping_add((dist_sq * 100.0) as u32);
                if self.bark_turn {
                    RoeBark::spatial(deer_pos, listener, view_dir, up, seed).thump();
                    self.sound_cooldown = 2.4 + ((seed % 15) as f32) * 0.10;
                } else {
                    RoeHoofbeats::spatial(deer_pos, listener, view_dir, up, seed).thump();
                    self.sound_cooldown = 1.6 + ((seed % 15) as f32) * 0.08;
                }
                self.bark_turn = !self.bark_turn;
                return true;
            }
        }
        false
    }

    /// Emits 3D faceted geometry for all roe deer in the herd.
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        if self.anchored {
            for member in &self.deer {
                member.emit(target);
            }
        }
    }
}

/// Composite woodland wildlife population combining the house sparrow flock and Temperate roe deer herd.
#[derive(Clone, Copy, Debug)]
pub struct WoodlandFauna {
    sparrows: SparrowFlock,
    roe_herd: RoeHerd,
}

impl WoodlandFauna {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sparrows: SparrowFlock, roe_herd: RoeHerd) -> Self {
        Self { sparrows, roe_herd }
    }

    /// Secondary constructor seeding both the woodland sparrow flock and the Temperate roe deer herd.
    pub const fn seeded() -> Self {
        Self::new(SparrowFlock::seeded(), RoeHerd::seeded())
    }

    /// Steps both sparrows and roe deer with 3D spatialized birdsong and alarm barks.
    pub fn step_spatial(
        &mut self,
        dt: f32,
        listener: Vec3,
        view_dir: Vec3,
        crouching: bool,
        world: &dyn PlanetaryWorld,
    ) {
        let _ = self.sparrows.step_spatial(dt, listener, view_dir, world);
        let _ = self.roe_herd.step_spatial(dt, listener, view_dir, crouching, world);
    }

    /// Emits 3D faceted meshes for all active woodland fauna.
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        self.sparrows.emit(target);
        self.roe_herd.emit(target);
    }
}
