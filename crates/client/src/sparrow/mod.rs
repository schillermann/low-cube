mod activity;
mod mesh;

use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld};
use crate::cardiac::{AcousticWave, SparrowChirp};
use self::activity::Sparrow;
#[cfg(test)]
pub use self::activity::{Sparrow as TestSparrow, SparrowActivity};

/// Flock of woodland house sparrows flying, pecking seeds on the ground, perching on trees, and chirping.
#[derive(Clone, Copy, Debug)]
pub struct SparrowFlock {
    birds: [Sparrow; 9],
    chirp_cooldown: f32,
    anchored: bool,
}

impl SparrowFlock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(birds: [Sparrow; 9], chirp_cooldown: f32, anchored: bool) -> Self {
        Self {
            birds,
            chirp_cooldown,
            anchored,
        }
    }

    /// Secondary constructor initializing 9 deterministically seeded sparrows ready to anchor near the player.
    pub const fn seeded() -> Self {
        Self::new(
            [
                Sparrow::seeded(101),
                Sparrow::seeded(203),
                Sparrow::seeded(307),
                Sparrow::seeded(409),
                Sparrow::seeded(503),
                Sparrow::seeded(607),
                Sparrow::seeded(709),
                Sparrow::seeded(811),
                Sparrow::seeded(919),
            ],
            0.85,
            false,
        )
    }

    #[cfg(test)]
    pub const fn birds(&self) -> &[Sparrow; 9] {
        &self.birds
    }

    /// Steps all sparrows in the flock and triggers a 3D spatialized birdsong chirp when a nearby sparrow vocalizes.
    pub fn step_spatial(&mut self, dt: f32, listener: Vec3, view_dir: Vec3, world: &dyn PlanetaryWorld) -> bool {
        if !self.anchored {
            for (idx, bird) in self.birds.iter_mut().enumerate() {
                bird.anchor_around(idx, listener, world);
            }
            self.anchored = true;
        }
        self.chirp_cooldown = (self.chirp_cooldown - dt).max(0.0);
        let mut vocalizer: Option<(usize, Vec3, f32)> = None;
        for (idx, bird) in self.birds.iter_mut().enumerate() {
            if bird.step(dt, listener, world) {
                let pos = bird.position();
                let dist_sq = (pos - listener).length_squared();
                if dist_sq <= 30.0 * 30.0 && vocalizer.is_none_or(|(_, _, best_sq)| dist_sq < best_sq) {
                    vocalizer = Some((idx, pos, dist_sq));
                }
            }
        }
        if self.chirp_cooldown <= 0.0 {
            if let Some((idx, bird_pos, dist_sq)) = vocalizer {
                let up = world.surface_up_at(listener);
                let seed = (idx as u32).wrapping_mul(747_796_405).wrapping_add((dist_sq * 100.0) as u32);
                SparrowChirp::spatial(bird_pos, listener, view_dir, up, seed).thump();
                self.chirp_cooldown = 4.2 + ((seed % 35) as f32) * 0.12;
                return true;
            }
        }
        false
    }

    /// Emits 3D faceted geometry for all sparrows in the flock.
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        if self.anchored {
            for bird in &self.birds {
                bird.emit(target);
            }
        }
    }
}
