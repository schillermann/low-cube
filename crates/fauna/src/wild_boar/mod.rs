mod activity;
mod kinematics;
mod mesh;
mod remote;

use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, SightCone, SounderPresence, SurfaceFind, WorldEarthworks};
use acoustics::{AcousticWave, BoarSnort};
use self::activity::WildBoar;
pub use self::activity::{BoarActivity, WildBoar as TestWildBoar};
pub use self::remote::RemoteSounder;

/// Polymorphic wild-boar roster avoiding `anchored: bool` flags and half-born sky entities.
#[derive(Clone, Copy, Debug)]
enum SounderRoster {
    Seeds([u32; 3]),
    Active([WildBoar; 3]),
}

/// Small woodland wild-boar sounder ("Rotte") inhabiting `ClimateZone::Temperate` oak woods.
#[derive(Clone, Copy, Debug)]
pub struct BoarSounder {
    roster: SounderRoster,
    snort_cooldown: f32,
    grunt_turn: u8,
}

impl BoarSounder {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn from_roster(roster: SounderRoster, snort_cooldown: f32, grunt_turn: u8) -> Self {
        Self { roster, snort_cooldown, grunt_turn }
    }

    /// Secondary constructor wrapping three already-grounded Temperate wild boars.
    pub const fn active(boars: [WildBoar; 3]) -> Self {
        Self::from_roster(SounderRoster::Active(boars), 0.0, 0)
    }

    /// Secondary constructor storing deterministic Temperate seeds prior to first world contact.
    pub const fn seeded() -> Self {
        Self::from_roster(SounderRoster::Seeds([5107, 6229, 7351]), 0.0, 0)
    }

    /// Secondary constructor birthing all three wild boars directly onto `world`.
    pub fn inhabiting(listener: Vec3, world: &dyn PlanetaryWorld) -> Self {
        Self::active([
            WildBoar::inhabiting(5107, 0, listener, world),
            WildBoar::inhabiting(6229, 1, listener, world),
            WildBoar::inhabiting(7351, 2, listener, world),
        ])
    }

    pub const fn boars(&self) -> &[WildBoar] {
        match &self.roster {
            SounderRoster::Seeds(_) => &[],
            SounderRoster::Active(boars) => boars,
        }
    }

    pub fn presence(&self) -> SounderPresence {
        match &self.roster {
            SounderRoster::Seeds(_) => SounderPresence::Unseen,
            SounderRoster::Active(boars) => SounderPresence::Sighted([
                boars[0].presence(),
                boars[1].presence(),
                boars[2].presence(),
            ]),
        }
    }

    fn ensure_active(&mut self, hub_hint: Vec3, world: &dyn PlanetaryWorld) -> &mut [WildBoar; 3] {
        if let SounderRoster::Seeds(seeds) = self.roster {
            self.roster = SounderRoster::Active([
                WildBoar::inhabiting(seeds[0], 0, hub_hint, world),
                WildBoar::inhabiting(seeds[1], 1, hub_hint, world),
                WildBoar::inhabiting(seeds[2], 2, hub_hint, world),
            ]);
        }
        match &mut self.roster {
            SounderRoster::Active(boars) => boars,
            SounderRoster::Seeds(_) => unreachable!(),
        }
    }

    /// Steps the sounder against all active human `observers`, returning `(vocal_boar, gored_slot_and_shove)`.
    pub fn step_cohort(
        &mut self,
        dt: f32,
        observers: &[(u32, Vec3, bool)],
        world: &dyn PlanetaryWorld,
    ) -> (Option<(usize, Vec3)>, Option<(u32, Vec3)>) {
        self.snort_cooldown = (self.snort_cooldown - dt).max(0.0);
        if matches!(self.roster, SounderRoster::Seeds(_)) {
            let Some(&(_, first_pos, _)) = observers.first() else {
                return (None, None);
            };
            let _ = self.ensure_active(first_pos, world);
        }
        let SounderRoster::Active(ref mut boars) = self.roster else {
            return (None, None);
        };
        let mut vocal: Option<(usize, Vec3)> = None;
        let mut gore_hit: Option<(u32, Vec3)> = None;
        for (idx, boar) in boars.iter_mut().enumerate() {
            let (snorted, shoved) = boar.step_cohort(idx, dt, observers, world);
            if gore_hit.is_none() {
                gore_hit = shoved;
            }
            if snorted && vocal.is_none() {
                vocal = Some((idx, boar.position()));
            }
        }
        (vocal, gore_hit)
    }

    pub fn play_snort_near(&mut self, vocal: (usize, Vec3), listener: Vec3, view_dir: Vec3, up: Vec3) {
        if self.snort_cooldown <= 0.0 && (vocal.1 - listener).length_squared() <= 38.0 * 38.0 {
            let seed = (vocal.0 as u32).wrapping_mul(2654435761).wrapping_add(u32::from(self.grunt_turn));
            BoarSnort::spatial(vocal.1, listener, view_dir, up, seed).thump();
            self.grunt_turn = self.grunt_turn.wrapping_add(1);
            self.snort_cooldown = 1.45 + ((seed % 12) as f32) * 0.08;
        }
    }

    /// Steps all wild boars for a local solitary `listener`, emitting spatial snorts and returning any tusk knockback.
    pub fn step_spatial(
        &mut self,
        dt: f32,
        listener: Vec3,
        view_dir: Vec3,
        crouching: bool,
        world: &dyn PlanetaryWorld,
    ) -> Option<Vec3> {
        let (vocal, gore_hit) = self.step_cohort(dt, &[(1, listener, crouching)], world);
        if let Some(v) = vocal {
            self.play_snort_near(v, listener, view_dir, world.surface_up_at(listener));
        }
        gore_hit.map(|(_, shove)| shove)
    }

    /// Evaluates an authoritative melee strike against `world`, returning `(hit_boar, lethal_meat_cast_rays)`.
    pub fn evaluate_strike(
        &mut self,
        eye: Vec3,
        view_dir: Vec3,
        power: u8,
        max_reach: f32,
        play_audio: bool,
        world: &dyn PlanetaryWorld,
    ) -> (bool, Option<([Vec3; 2], Vec3)>) {
        let SounderRoster::Active(ref mut boars) = self.roster else {
            return (false, None);
        };
        let mut closest: Option<(usize, f32)> = None;
        for (idx, boar) in boars.iter().enumerate() {
            if let Some(dist) = boar.intercept_ray(eye, view_dir, max_reach) {
                if closest.is_none_or(|(_, best)| dist < best) {
                    closest = Some((idx, dist));
                }
            }
        }
        let Some((idx, _)) = closest else {
            return (false, None);
        };
        let boar_pos = boars[idx].position();
        let up = world.surface_up_at(boar_pos);
        if play_audio {
            let seed = (idx as u32).wrapping_add(u32::from(self.grunt_turn)).wrapping_mul(747_796_405);
            BoarSnort::spatial(boar_pos, eye, view_dir, up, seed).thump();
        }
        self.grunt_turn = self.grunt_turn.wrapping_add(1);
        for (mate_idx, mate) in boars.iter_mut().enumerate() {
            if mate_idx != idx {
                mate.alert_to_threat(boar_pos, eye, world);
            }
        }
        if !boars[idx].receive_strike(power, eye, world) {
            return (true, None);
        }
        let tangent = view_dir.cross(up).normalize_or_zero();
        let side = if tangent.length_squared() > 0.2 { tangent } else { Vec3::X };
        let origins = [
            boar_pos + up * 1.15 + side * 0.28,
            boar_pos + up * 1.15 - side * 0.28,
        ];
        (true, Some((origins, -up)))
    }

    pub fn cast_meat_drops(drops: ([Vec3; 2], Vec3), earthworks: &mut dyn WorldEarthworks) -> bool {
        let drop_a = earthworks.cast_surface_find(drops.0[0], drops.1, SurfaceFind::BoarMeat);
        let drop_b = earthworks.cast_surface_find(drops.0[1], drops.1, SurfaceFind::BoarMeat);
        drop_a || drop_b
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
        let (hit, drops) = self.evaluate_strike(eye, view_dir, power, max_reach, true, world);
        let dropped = drops.is_some_and(|d| Self::cast_meat_drops(d, world));
        (hit, dropped)
    }

    /// Emits 3D faceted geometry for living wild boars within `cone`.
    pub fn emit_visible(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        for boar in self.boars() {
            if boar.alive() && cone.reaches(boar.position(), 58.0) {
                boar.emit(target);
            }
        }
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        for boar in self.boars() {
            boar.emit(target);
        }
    }
}
