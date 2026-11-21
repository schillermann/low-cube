mod activity;
mod habitat;
mod mesh;

use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, SightCone};
use acoustics::{AcousticWave, RoeBark, RoeHoofbeats};
use self::activity::RoeDeer;
pub(crate) use self::habitat::TemperateGlade;
pub use self::activity::{RoeActivity, RoeDeer as TestRoeDeer};

/// Polymorphic roe-deer herd state avoiding `anchored: bool` flags and half-born entities.
#[derive(Clone, Copy, Debug)]
enum HerdRoster {
    Seeds([(u32, bool); 4]),
    Active([RoeDeer; 4]),
}

/// Small woodland roe deer herd ("Sprung") inhabiting `ClimateZone::Temperate` oak glades and meadows.
#[derive(Clone, Copy, Debug)]
pub struct RoeHerd {
    roster: HerdRoster,
    sound_cooldown: f32,
    bark_turn: bool,
}

impl RoeHerd {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn from_roster(roster: HerdRoster, sound_cooldown: f32, bark_turn: bool) -> Self {
        Self { roster, sound_cooldown, bark_turn }
    }

    /// Secondary constructor storing deterministic roebuck and doe seeds prior to first world contact.
    pub const fn seeded() -> Self {
        Self::from_roster(
            HerdRoster::Seeds([(1301, true), (2417, false), (3529, true), (4639, false)]),
            0.0,
            false,
        )
    }

    #[cfg(test)]
    pub const fn deer(&self) -> &[RoeDeer] {
        match &self.roster {
            HerdRoster::Seeds(_) => &[],
            HerdRoster::Active(deer) => deer,
        }
    }

    #[cfg(test)]
    pub const fn bark_queued(&self) -> bool {
        self.bark_turn
    }

    fn ensure_active(&mut self, listener: Vec3, world: &dyn PlanetaryWorld) -> &mut [RoeDeer; 4] {
        if let HerdRoster::Seeds(seeds) = self.roster {
            self.roster = HerdRoster::Active([
                RoeDeer::inhabiting(seeds[0].0, seeds[0].1, 0, listener, world),
                RoeDeer::inhabiting(seeds[1].0, seeds[1].1, 1, listener, world),
                RoeDeer::inhabiting(seeds[2].0, seeds[2].1, 2, listener, world),
                RoeDeer::inhabiting(seeds[3].0, seeds[3].1, 3, listener, world),
            ]);
        }
        match &mut self.roster {
            HerdRoster::Active(deer) => deer,
            HerdRoster::Seeds(_) => unreachable!(),
        }
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
        self.sound_cooldown = (self.sound_cooldown - dt).max(0.0);
        let deer = self.ensure_active(listener, world);
        let mut startled: Option<(usize, Vec3, f32)> = None;
        for (idx, member) in deer.iter_mut().enumerate() {
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

    /// Emits 3D faceted geometry for roe deer within `cone`.
    pub fn emit_visible(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        if let HerdRoster::Active(ref deer) = self.roster {
            for member in deer {
                if cone.reaches(member.position(), 58.0) {
                    member.emit(target);
                }
            }
        }
    }

    #[cfg(test)]
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        for member in self.deer() {
            member.emit(target);
        }
    }
}
