use std::f32::consts::TAU;
use glam::Vec3;
use world::{BoarPresence, BoarStance, ClimateZone, MeshTarget, PlanetaryWorld};
use crate::roe_deer::TemperateGlade;
use super::kinematics::BoarTerrain;
use super::mesh::BoarFigure;

const MAX_BOAR_HEALTH: u8 = 8;

/// Polymorphic behavioral state of an individual European wild boar in `ClimateZone::Temperate`.
#[derive(Clone, Copy, Debug)]
pub enum BoarActivity {
    Rooting { pos: Vec3, heading: Vec3, timer: f32, phase: f32 },
    BristledWarning { pos: Vec3, heading: Vec3, timer: f32, phase: f32 },
    TuskCharge { pos: Vec3, target: Vec3, phase: f32, timer: f32 },
    Trotting { pos: Vec3, target: Vec3, phase: f32, cooldown: f32 },
    Fallen { pos: Vec3, heading: Vec3, timer: f32 },
}

/// Autonomous woodland wild boar (*Sus scrofa*) rooting under oaks, snorting in warning, and charging with its tusks.
#[derive(Clone, Copy, Debug)]
pub struct WildBoar {
    activity: BoarActivity,
    up: Vec3,
    seed: u32,
    vitality: (u8, u8),
}

impl WildBoar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(activity: BoarActivity, up: Vec3, seed: u32, vitality: (u8, u8)) -> Self {
        Self { activity, up, seed, vitality }
    }

    /// Secondary constructor birthing a wild boar directly onto Temperate oak terrain without intermediate sky state.
    pub fn inhabiting(seed: u32, index: usize, listener: Vec3, world: &dyn PlanetaryWorld) -> Self {
        let next_seed = Self::next_hash(seed.wrapping_add((index as u32).wrapping_mul(4211)));
        let glade = TemperateGlade::around(listener, world);
        let pos = glade.oak_rooting_spot(next_seed, 9.5 + (index as f32) * 2.2, 6.0, world);
        let (sa, ca) = (((next_seed % 360) as f32) * (TAU / 360.0)).sin_cos();
        let up = world.surface_up_at(pos);
        let activity = BoarActivity::Rooting {
            pos,
            heading: Vec3::new(ca, 0.0, sa),
            timer: 2.2 + ((next_seed % 25) as f32) * 0.12,
            phase: ((index as f32) * 1.3) % TAU,
        };
        Self::new(activity, up, next_seed, (MAX_BOAR_HEALTH, 0))
    }

    const fn next_hash(seed: u32) -> u32 {
        seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453)
    }

    pub const fn alive(&self) -> bool {
        !matches!(self.activity, BoarActivity::Fallen { .. })
    }

    pub const fn position(&self) -> Vec3 {
        match self.activity {
            BoarActivity::Rooting { pos, .. }
            | BoarActivity::BristledWarning { pos, .. }
            | BoarActivity::TuskCharge { pos, .. }
            | BoarActivity::Trotting { pos, .. }
            | BoarActivity::Fallen { pos, .. } => pos,
        }
    }

    pub const fn rooting(&self) -> bool { matches!(self.activity, BoarActivity::Rooting { .. }) }
    pub const fn bristled(&self) -> bool { matches!(self.activity, BoarActivity::BristledWarning { .. }) }
    pub const fn charging(&self) -> bool { matches!(self.activity, BoarActivity::TuskCharge { .. }) }
    pub const fn vitality_at(&self, expected: u8) -> bool { self.vitality.0 == expected }

    pub fn presence(&self) -> BoarPresence {
        let flash = (self.vitality.1 as f32) / 14.0;
        let (pos, heading, phase, stance) = match self.activity {
            BoarActivity::Rooting { pos, heading, phase, .. } => (pos, heading, phase, BoarStance::Rooting),
            BoarActivity::BristledWarning { pos, heading, phase, .. } => (pos, heading, phase, BoarStance::Bristled),
            BoarActivity::TuskCharge { pos, target, phase, .. } => (pos, target - pos, phase, BoarStance::Charging),
            BoarActivity::Trotting { pos, target, phase, .. } => (pos, target - pos, phase, BoarStance::Trotting),
            BoarActivity::Fallen { pos, heading, .. } => (pos, heading, 0.0, BoarStance::Fallen),
        };
        BoarPresence::new(pos, heading, (phase, flash), stance)
    }

    pub fn intercept_ray(&self, eye: Vec3, dir: Vec3, max_dist: f32) -> Option<f32> {
        self.presence().intercept_ray(self.up, eye, dir, max_dist)
    }

    pub fn alert_to_threat(&mut self, wound_spot: Vec3, attacker: Vec3, world: &dyn PlanetaryWorld) {
        let pos = self.position();
        let terrain = BoarTerrain::new(world);
        if matches!(self.activity, BoarActivity::Rooting { .. } | BoarActivity::Trotting { .. })
            && terrain.planar_dist_sq(pos, wound_spot) <= 9.0 * 9.0
        {
            let face = Vec3::new(attacker.x - pos.x, 0.0, attacker.z - pos.z).normalize_or_zero();
            self.activity = BoarActivity::BristledWarning { pos, heading: face, timer: 0.55, phase: 0.0 };
        }
    }

    pub fn receive_strike(&mut self, power: u8, attacker: Vec3, world: &dyn PlanetaryWorld) -> bool {
        if !self.alive() {
            return false;
        }
        let next_hp = self.vitality.0.saturating_sub(power.max(1));
        self.vitality = (next_hp, 14);
        let pos = self.position();
        if next_hp == 0 {
            let heading = (attacker - pos).normalize_or_zero();
            self.activity = BoarActivity::Fallen { pos, heading, timer: 38.0 };
            true
        } else {
            let target = world.constrain_to_surface(Vec3::new(attacker.x, attacker.y + 3.0, attacker.z), 0.0);
            self.activity = BoarActivity::TuskCharge { pos, target, phase: 0.0, timer: 3.4 };
            false
        }
    }

    fn relocate(&mut self, index: usize, hub_hint: Vec3, world: &dyn PlanetaryWorld) {
        *self = Self::inhabiting(self.seed, index, hub_hint, world);
    }

    /// Steps wild boar behavior against the closest human in `observers`, returning `(snorted, Some((slot, shove)))`.
    pub fn step_cohort(
        &mut self,
        index: usize,
        dt: f32,
        observers: &[(u32, Vec3, bool)],
        world: &dyn PlanetaryWorld,
    ) -> (bool, Option<(u32, Vec3)>) {
        let terrain = BoarTerrain::new(world);
        let pos = self.position();
        let mut closest: Option<(u32, Vec3, bool, f32)> = None;
        for &(slot, obs_pos, crouching) in observers {
            let d_sq = terrain.planar_dist_sq(pos, obs_pos);
            if closest.is_none_or(|(_, _, _, best)| d_sq < best) {
                closest = Some((slot, obs_pos, crouching, d_sq));
            }
        }
        if let Some((slot, listener, crouching, _)) = closest {
            let (snorted, shoved) = self.step_indexed(index, dt, listener, crouching, world);
            (snorted, shoved.map(|imp| (slot, imp)))
        } else {
            let distant = Vec3::new(0.0, -2048.0, 0.0);
            let _ = self.step_indexed(index, dt, distant, false, world);
            (false, None)
        }
    }

    /// Steps wild boar behavior against a single `listener`, returning `(snorted, gore_impulse)`.
    pub fn step(&mut self, dt: f32, listener: Vec3, crouching: bool, world: &dyn PlanetaryWorld) -> (bool, Option<Vec3>) {
        self.step_indexed(0, dt, listener, crouching, world)
    }

    fn step_indexed(&mut self, index: usize, dt: f32, listener: Vec3, crouching: bool, world: &dyn PlanetaryWorld) -> (bool, Option<Vec3>) {
        if self.vitality.1 > 0 {
            self.vitality.1 -= 1;
        }
        let terrain = BoarTerrain::new(world);
        let warn_r = if crouching { 5.2_f32 } else { 9.0_f32 };
        let charge_r = if crouching { 3.2_f32 } else { 5.4_f32 };
        match self.activity {
            BoarActivity::Fallen { pos, heading, timer } => {
                if timer - dt <= 0.0 {
                    self.relocate(index, listener, world);
                } else {
                    self.activity = BoarActivity::Fallen { pos, heading, timer: timer - dt };
                }
                (false, None)
            }
            BoarActivity::Rooting { pos, heading, timer, phase } => {
                let Some(grounded) = terrain.reconcile_ground(pos, self.up) else {
                    self.relocate(index, listener, world);
                    return (false, None);
                };
                self.up = world.surface_up_at(grounded);
                if terrain.threat_reachable(grounded, listener, charge_r) {
                    self.activity = BoarActivity::TuskCharge { pos: grounded, target: listener, phase: 0.0, timer: 3.0 };
                    return (true, None);
                }
                if terrain.threat_reachable(grounded, listener, warn_r) {
                    let face = Vec3::new(listener.x - grounded.x, 0.0, listener.z - grounded.z).normalize_or_zero();
                    self.activity = BoarActivity::BristledWarning { pos: grounded, heading: face, timer: 1.25, phase: 0.0 };
                    return (true, None);
                }
                if timer - dt <= 0.0 {
                    self.seed = Self::next_hash(self.seed);
                    let next_spot = TemperateGlade::new(grounded).oak_rooting_spot(self.seed, 3.2, 5.5, world);
                    self.activity = BoarActivity::Trotting { pos: grounded, target: next_spot, phase: 0.0, cooldown: 0.0 };
                } else {
                    self.activity = BoarActivity::Rooting { pos: grounded, heading, timer: timer - dt, phase: (phase + dt * 4.5) % TAU };
                }
                (false, None)
            }
            BoarActivity::BristledWarning { pos, heading, timer, phase } => {
                let Some(grounded) = terrain.reconcile_ground(pos, self.up) else {
                    self.relocate(index, listener, world);
                    return (false, None);
                };
                self.up = world.surface_up_at(grounded);
                let face = Vec3::new(listener.x - grounded.x, 0.0, listener.z - grounded.z).normalize_or_zero();
                let next_h = if face.length_squared() > 0.2 { face } else { heading };
                if !terrain.threat_reachable(grounded, listener, warn_r + 2.6) {
                    self.activity = BoarActivity::Rooting { pos: grounded, heading: next_h, timer: 3.0, phase: 0.0 };
                    return (false, None);
                }
                if terrain.threat_reachable(grounded, listener, charge_r) || timer - dt <= 0.0 {
                    self.activity = BoarActivity::TuskCharge { pos: grounded, target: listener, phase: 0.0, timer: 3.0 };
                    return (true, None);
                }
                self.activity = BoarActivity::BristledWarning { pos: grounded, heading: next_h, timer: timer - dt, phase: (phase + dt * 7.0) % TAU };
                (false, None)
            }
            BoarActivity::Trotting { pos, target, phase, cooldown } => {
                let next_cd = (cooldown - dt).max(0.0);
                if next_cd <= 0.0 && terrain.threat_reachable(pos, listener, warn_r) {
                    let face = Vec3::new(listener.x - pos.x, 0.0, listener.z - pos.z).normalize_or_zero();
                    self.activity = BoarActivity::BristledWarning { pos, heading: face, timer: 1.0, phase: 0.0 };
                    return (true, None);
                }
                let (next_pos, dir, arrived) = terrain.advance_toward(pos, target, 2.95, dt, false);
                self.up = world.surface_up_at(next_pos);
                if arrived {
                    self.seed = Self::next_hash(self.seed);
                    let wait = 2.6 + ((self.seed % 30) as f32) * 0.12;
                    self.activity = BoarActivity::Rooting { pos: next_pos, heading: dir, timer: wait, phase: 0.0 };
                } else {
                    self.activity = BoarActivity::Trotting { pos: next_pos, target, phase: (phase + dt * 11.5) % TAU, cooldown: next_cd };
                }
                (false, None)
            }
            BoarActivity::TuskCharge { pos, target, phase, timer } => {
                let aim = if ClimateZone::dominant(listener) == ClimateZone::Temperate { listener } else { target };
                let (next_pos, dir, _) = terrain.advance_toward(pos, aim, 8.0, dt, true);
                self.up = world.surface_up_at(next_pos);
                if terrain.tusk_reaches(next_pos, listener) {
                    self.seed = Self::next_hash(self.seed);
                    let retreat = TemperateGlade::new(next_pos).escape_from(listener, self.seed, world);
                    self.activity = BoarActivity::Trotting { pos: next_pos, target: retreat, phase: 0.0, cooldown: 1.3 };
                    let shove = if dir.length_squared() > 0.2 { dir * 1.65 } else { Vec3::X * 1.65 };
                    return (false, Some(shove));
                }
                if timer - dt <= 0.0 {
                    self.activity = if terrain.threat_reachable(next_pos, listener, warn_r) {
                        BoarActivity::BristledWarning { pos: next_pos, heading: dir, timer: 0.85, phase: 0.0 }
                    } else {
                        BoarActivity::Rooting { pos: next_pos, heading: dir, timer: 2.2, phase: 0.0 }
                    };
                } else {
                    self.activity = BoarActivity::TuskCharge { pos: next_pos, target: aim, phase: (phase + dt * 21.0) % TAU, timer: timer - dt };
                }
                (false, None)
            }
        }
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        self.presence().transfer(|pos, fwd, phase, flash, stance| {
            let (pitch, stride, bristle) = match stance {
                BoarStance::Fallen => return,
                BoarStance::Rooting => (-0.44 + 0.12 * phase.sin(), 0.0, 0.15),
                BoarStance::Bristled => (0.14 + 0.06 * phase.sin(), 0.0, 1.0),
                BoarStance::Trotting => (-0.08, phase, 0.35),
                BoarStance::Charging => (-0.26 + 0.10 * phase.sin(), phase, 1.0),
            };
            BoarFigure::aligned(pos, self.up, fwd, pitch, stride, bristle, flash).emit(target);
        });
    }
}
