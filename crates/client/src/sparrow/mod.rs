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

    /// Steps all sparrows in the flock and triggers woodland birdsong chirps when a nearby sparrow vocalizes.
    pub fn step(&mut self, dt: f32, listener: Vec3, world: &dyn PlanetaryWorld) -> bool {
        if !self.anchored {
            for (idx, bird) in self.birds.iter_mut().enumerate() {
                bird.anchor_around(idx, listener, world);
            }
            self.anchored = true;
        }
        self.chirp_cooldown = (self.chirp_cooldown - dt).max(0.0);
        let mut vocalized = false;
        for bird in &mut self.birds {
            if bird.step(dt, listener, world) && (bird.position() - listener).length_squared() <= 32.0 * 32.0 {
                vocalized = true;
            }
        }
        if !world.weightless_at(listener) && self.chirp_cooldown <= 0.0 && vocalized {
            SparrowChirp::woodland().thump();
            self.chirp_cooldown = 2.6;
            return true;
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
