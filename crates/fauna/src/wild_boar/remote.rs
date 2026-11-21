use glam::Vec3;
use world::{BoarStance, MeshTarget, PlanetaryWorld, SightCone, SounderPresence};
use acoustics::{AcousticWave, BoarSnort};
use super::mesh::BoarFigure;

/// Client-side mirror rendering and spatializing an authoritative `SounderPresence` without autonomous AI.
#[derive(Clone, Copy, Debug)]
pub struct RemoteSounder {
    prev_stances: [BoarStance; 3],
    hit_flash: [f32; 3],
    snort_cooldown: f32,
    grunt_turn: u8,
}

impl RemoteSounder {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        prev_stances: [BoarStance; 3],
        hit_flash: [f32; 3],
        snort_cooldown: f32,
        grunt_turn: u8,
    ) -> Self {
        Self { prev_stances, hit_flash, snort_cooldown, grunt_turn }
    }

    /// Secondary constructor starting in a calm, unprovoked observation state.
    pub const fn calm() -> Self {
        Self::new([BoarStance::Rooting; 3], [0.0; 3], 0.0, 0)
    }

    /// Observes stance transitions in `sounder` to emit spatialized warning or charge snorts near `listener`.
    pub fn observe(
        &mut self,
        dt: f32,
        sounder: SounderPresence,
        listener: Vec3,
        view_dir: Vec3,
        world: &dyn PlanetaryWorld,
    ) {
        self.snort_cooldown = (self.snort_cooldown - dt).max(0.0);
        for fl in &mut self.hit_flash {
            *fl = (*fl - dt * 4.2).max(0.0);
        }
        let SounderPresence::Sighted(boars) = sounder else {
            return;
        };
        let mut vocal: Option<(usize, Vec3, f32)> = None;
        for (idx, boar) in boars.into_iter().enumerate() {
            let next_stance = boar.stance();
            let prev = self.prev_stances[idx];
            self.prev_stances[idx] = next_stance;
            let newly_provoked = next_stance.vocal() && next_stance != prev;
            if newly_provoked {
                let pos = boar.position();
                let dist_sq = (pos - listener).length_squared();
                if dist_sq <= 38.0 * 38.0 && vocal.is_none_or(|(_, _, best)| dist_sq < best) {
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
                self.snort_cooldown = 1.15;
            }
        }
    }

    /// Tests whether a melee swing along `(eye, view_dir)` within `max_reach` intercepts any living remote boar.
    pub fn intercept_strike(
        &mut self,
        sounder: SounderPresence,
        eye: Vec3,
        view_dir: Vec3,
        max_reach: f32,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        let SounderPresence::Sighted(boars) = sounder else {
            return false;
        };
        let mut closest: Option<(usize, f32)> = None;
        for (idx, boar) in boars.into_iter().enumerate() {
            let up = world.surface_up_at(boar.position());
            if let Some(dist) = boar.intercept_ray(up, eye, view_dir, max_reach) {
                if closest.is_none_or(|(_, best)| dist < best) {
                    closest = Some((idx, dist));
                }
            }
        }
        let Some((idx, _)) = closest else {
            return false;
        };
        self.hit_flash[idx] = 1.0;
        let boar_pos = boars[idx].position();
        let up = world.surface_up_at(boar_pos);
        let seed = (idx as u32).wrapping_add(u32::from(self.grunt_turn)).wrapping_mul(747_796_405);
        BoarSnort::spatial(boar_pos, eye, view_dir, up, seed).thump();
        self.grunt_turn = self.grunt_turn.wrapping_add(1);
        true
    }

    /// Emits 3D faceted geometry for living authoritative boars inside `cone`.
    pub fn emit_visible(
        &self,
        sounder: SounderPresence,
        cone: SightCone,
        world: &dyn PlanetaryWorld,
        target: &mut dyn MeshTarget,
    ) {
        let SounderPresence::Sighted(boars) = sounder else {
            return;
        };
        for (idx, boar) in boars.into_iter().enumerate() {
            if boar.alive() && cone.reaches(boar.position(), 58.0) {
                let flashed = boar.with_flash(self.hit_flash[idx]);
                flashed.transfer(|pos, fwd, phase, flash, stance| {
                    let (pitch, stride, bristle) = match stance {
                        BoarStance::Fallen => return,
                        BoarStance::Rooting => (-0.44 + 0.12 * phase.sin(), 0.0, 0.15),
                        BoarStance::Bristled => (0.14 + 0.06 * phase.sin(), 0.0, 1.0),
                        BoarStance::Trotting => (-0.08, phase, 0.35),
                        BoarStance::Charging => (-0.26 + 0.10 * phase.sin(), phase, 1.0),
                    };
                    let up = world.surface_up_at(pos);
                    BoarFigure::aligned(pos, up, fwd, pitch, stride, bristle, flash).emit(target);
                });
            }
        }
    }
}
