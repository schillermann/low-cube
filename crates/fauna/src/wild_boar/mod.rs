mod activity;
mod mesh;

use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, SurfaceFind};
use acoustics::{AcousticWave, BoarSnort};
use crate::SightCone;
use self::activity::WildBoar;
pub use self::activity::{BoarActivity, WildBoar as TestWildBoar};

/// Small woodland wild-boar sounder ("Rotte") inhabiting `ClimateZone::Temperate` oak woods.
#[derive(Clone, Copy, Debug)]
pub struct BoarSounder {
    boars: [WildBoar; 3],
    snort_cooldown: f32,
    anchored: bool,
    grunt_turn: u8,
}

impl BoarSounder {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(boars: [WildBoar; 3], snort_cooldown: f32, anchored: bool, grunt_turn: u8) -> Self {
        Self {
            boars,
            snort_cooldown,
            anchored,
            grunt_turn,
        }
    }

    /// Secondary constructor seeding three Temperate wild boars ready to anchor under oak canopies.
    pub const fn seeded() -> Self {
        Self::new(
            [
                WildBoar::seeded(5107),
                WildBoar::seeded(6229),
                WildBoar::seeded(7351),
            ],
            0.0,
            false,
            0,
        )
    }

    pub const fn boars(&self) -> &[WildBoar; 3] {
        &self.boars
    }

    /// Steps all wild boars on `ClimateZone::Temperate`, emitting spatial snorts and returning any tusk-charge knockback.
    pub fn step_spatial(
        &mut self,
        dt: f32,
        listener: Vec3,
        view_dir: Vec3,
        crouching: bool,
        world: &dyn PlanetaryWorld,
    ) -> Option<Vec3> {
        if !self.anchored {
            for (idx, boar) in self.boars.iter_mut().enumerate() {
                boar.anchor_in_temperate(idx, listener, world);
            }
            self.anchored = true;
        }
        self.snort_cooldown = (self.snort_cooldown - dt).max(0.0);
        let mut vocal: Option<(usize, Vec3, f32)> = None;
        let mut gore_hit: Option<Vec3> = None;
        for (idx, boar) in self.boars.iter_mut().enumerate() {
            let (snorted, shoved) = boar.step(dt, listener, crouching, world);
            if gore_hit.is_none() {
                gore_hit = shoved;
            }
            if snorted {
                let pos = boar.position();
                let dist_sq = (pos - listener).length_squared();
                if dist_sq <= 38.0 * 38.0 && vocal.is_none_or(|(_, _, best_sq)| dist_sq < best_sq) {
                    vocal = Some((idx, pos, dist_sq));
                }
            }
        }
        if self.snort_cooldown <= 0.0 {
            if let Some((idx, boar_pos, _)) = vocal {
                let up = world.surface_up_at(listener);
                let seed = (idx as u32).wrapping_mul(2654435761).wrapping_add(u32::from(self.grunt_turn));
                BoarSnort::spatial(boar_pos, listener, view_dir, up, seed).thump();
                self.grunt_turn = self.grunt_turn.wrapping_add(1);
                self.snort_cooldown = 1.45 + ((seed % 12) as f32) * 0.08;
            }
        }
        gore_hit
    }

    /// Strikes the closest wild boar within `max_reach` along `(eye, view_dir)`, provoking the sounder and returning `(hit_boar, dropped_meat)`.
    pub fn strike_boar(
        &mut self,
        eye: Vec3,
        view_dir: Vec3,
        power: u8,
        max_reach: f32,
        world: &mut dyn PlanetaryWorld,
    ) -> (bool, bool) {
        if !self.anchored {
            return (false, false);
        }
        let mut closest: Option<(usize, f32)> = None;
        for (idx, boar) in self.boars.iter().enumerate() {
            if let Some(dist) = boar.intercept_ray(eye, view_dir, max_reach) {
                if closest.is_none_or(|(_, best)| dist < best) {
                    closest = Some((idx, dist));
                }
            }
        }
        let Some((idx, _)) = closest else {
            return (false, false);
        };
        let boar_pos = self.boars[idx].position();
        let up = world.surface_up_at(boar_pos);
        let seed = (idx as u32).wrapping_add(u32::from(self.grunt_turn)).wrapping_mul(747_796_405);
        BoarSnort::spatial(boar_pos, eye, view_dir, up, seed).thump();
        self.grunt_turn = self.grunt_turn.wrapping_add(1);
        for (mate_idx, mate) in self.boars.iter_mut().enumerate() {
            if mate_idx != idx {
                mate.alert_to_threat(boar_pos, eye);
            }
        }
        if !self.boars[idx].receive_strike(power, eye, world) {
            return (true, false);
        }
        let tangent = view_dir.cross(up).normalize_or_zero();
        let side = if tangent.length_squared() > 0.2 { tangent } else { Vec3::X };
        let drop_a = world.cast_surface_find(boar_pos + up * 1.15 + side * 0.28, -up, SurfaceFind::BoarMeat);
        let drop_b = world.cast_surface_find(boar_pos + up * 1.15 - side * 0.28, -up, SurfaceFind::BoarMeat);
        (true, drop_a || drop_b)
    }

    /// Emits 3D faceted geometry for living wild boars within `cone`.
    pub fn emit_visible(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        if self.anchored {
            for boar in &self.boars {
                if boar.alive() && cone.reaches(boar.position(), 58.0) {
                    boar.emit(target);
                }
            }
        }
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        if self.anchored {
            for boar in &self.boars {
                boar.emit(target);
            }
        }
    }
}
