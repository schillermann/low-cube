mod activity;
mod mesh;

use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, SightCone};
use acoustics::{AcousticWave, SparrowChirp};
use self::activity::Sparrow;
pub use self::activity::{Sparrow as TestSparrow, SparrowActivity};

/// Polymorphic sparrow flock state avoiding `anchored: bool` flags and half-born entities.
#[derive(Clone, Copy, Debug)]
enum FlockRoster {
    Seeds([u32; 9]),
    Active([Sparrow; 9]),
}

/// Flock of woodland house sparrows flying, pecking seeds on the ground, perching on trees, and chirping.
#[derive(Clone, Copy, Debug)]
pub struct SparrowFlock {
    roster: FlockRoster,
    chirp_cooldown: f32,
}

impl SparrowFlock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn from_roster(roster: FlockRoster, chirp_cooldown: f32) -> Self {
        Self { roster, chirp_cooldown }
    }

    /// Secondary constructor storing 9 deterministic sparrow seeds prior to first world contact.
    pub const fn seeded() -> Self {
        Self::from_roster(
            FlockRoster::Seeds([101, 203, 307, 409, 503, 607, 709, 811, 919]),
            0.85,
        )
    }

    #[cfg(test)]
    pub const fn birds(&self) -> &[Sparrow] {
        match &self.roster {
            FlockRoster::Seeds(_) => &[],
            FlockRoster::Active(birds) => birds,
        }
    }

    fn ensure_active(&mut self, listener: Vec3, world: &dyn PlanetaryWorld) -> &mut [Sparrow; 9] {
        if let FlockRoster::Seeds(seeds) = self.roster {
            self.roster = FlockRoster::Active([
                Sparrow::inhabiting(seeds[0], 0, listener, world),
                Sparrow::inhabiting(seeds[1], 1, listener, world),
                Sparrow::inhabiting(seeds[2], 2, listener, world),
                Sparrow::inhabiting(seeds[3], 3, listener, world),
                Sparrow::inhabiting(seeds[4], 4, listener, world),
                Sparrow::inhabiting(seeds[5], 5, listener, world),
                Sparrow::inhabiting(seeds[6], 6, listener, world),
                Sparrow::inhabiting(seeds[7], 7, listener, world),
                Sparrow::inhabiting(seeds[8], 8, listener, world),
            ]);
        }
        match &mut self.roster {
            FlockRoster::Active(birds) => birds,
            FlockRoster::Seeds(_) => unreachable!(),
        }
    }

    /// Steps all sparrows in the flock and triggers a 3D spatialized birdsong chirp when a nearby sparrow vocalizes.
    pub fn step_spatial(&mut self, dt: f32, listener: Vec3, view_dir: Vec3, world: &dyn PlanetaryWorld) -> bool {
        if !world.woodland_habitat(listener) {
            return false;
        }
        self.chirp_cooldown = (self.chirp_cooldown - dt).max(0.0);
        let birds = self.ensure_active(listener, world);
        let mut vocalizer: Option<(usize, Vec3, f32)> = None;
        for (idx, bird) in birds.iter_mut().enumerate() {
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

    /// Emits 3D faceted geometry for sparrows within `cone`.
    pub fn emit_visible(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        if let FlockRoster::Active(ref birds) = self.roster {
            for bird in birds {
                if cone.reaches(bird.position(), 42.0) {
                    bird.emit(target);
                }
            }
        }
    }

    #[cfg(test)]
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        for bird in self.birds() {
            bird.emit(target);
        }
    }
}
