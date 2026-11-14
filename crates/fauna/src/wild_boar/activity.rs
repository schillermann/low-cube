use std::f32::consts::TAU;
use glam::Vec3;
use world::{ClimateZone, MeshTarget, PlanetaryWorld};
use crate::roe_deer::TemperateGlade;
use super::mesh::BoarFigure;

const MAX_BOAR_HEALTH: u8 = 8;
const BOAR_HIT_RADIUS: f32 = 0.56;

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

    /// Secondary constructor seeding a Temperate wild boar prior to initial terrain anchoring.
    pub const fn seeded(seed: u32) -> Self {
        Self::new(
            BoarActivity::Rooting { pos: Vec3::new(0.0, 200.0, 0.0), heading: Vec3::X, timer: 2.5, phase: 0.0 },
            Vec3::Y,
            seed,
            (MAX_BOAR_HEALTH, 0),
        )
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

    pub const fn rooting(&self) -> bool {
        matches!(self.activity, BoarActivity::Rooting { .. })
    }

    pub const fn bristled(&self) -> bool {
        matches!(self.activity, BoarActivity::BristledWarning { .. })
    }

    pub const fn charging(&self) -> bool {
        matches!(self.activity, BoarActivity::TuskCharge { .. })
    }

    pub const fn vitality_at(&self, expected: u8) -> bool {
        self.vitality.0 == expected
    }

    fn planar_dist_sq(a: Vec3, b: Vec3) -> f32 {
        if (a.y - b.y).abs() > 6.5 {
            1e9
        } else {
            (a.x - b.x) * (a.x - b.x) + (a.z - b.z) * (a.z - b.z)
        }
    }

    pub fn anchor_in_temperate(&mut self, index: usize, listener: Vec3, world: &dyn PlanetaryWorld) {
        self.seed = Self::next_hash(self.seed.wrapping_add((index as u32).wrapping_mul(4211)));
        let glade = TemperateGlade::around(listener, world);
        let pos = glade.oak_rooting_spot(self.seed, 9.5 + (index as f32) * 2.2, 6.0, world);
        let (sa, ca) = (((self.seed % 360) as f32) * (TAU / 360.0)).sin_cos();
        self.up = world.surface_up_at(pos);
        self.vitality = (MAX_BOAR_HEALTH, 0);
        self.activity = BoarActivity::Rooting {
            pos,
            heading: Vec3::new(ca, 0.0, sa),
            timer: 2.2 + ((self.seed % 25) as f32) * 0.12,
            phase: ((index as f32) * 1.3) % TAU,
        };
    }

    pub fn intercept_ray(&self, eye: Vec3, dir: Vec3, max_dist: f32) -> Option<f32> {
        if !self.alive() {
            return None;
        }
        let ray = dir.normalize_or_zero();
        let center = self.position() + self.up * 0.48;
        let oc = eye - center;
        let b = oc.dot(ray);
        let c = oc.length_squared() - BOAR_HIT_RADIUS * BOAR_HIT_RADIUS;
        let disc = b * b - c;
        if disc < 0.0 {
            return None;
        }
        let t = (-b - disc.sqrt()).max(0.0);
        (t <= max_dist && (oc + ray * t).length_squared() <= (BOAR_HIT_RADIUS + 0.02).powi(2)).then_some(t)
    }

    pub fn alert_to_threat(&mut self, wound_spot: Vec3, attacker: Vec3) {
        let pos = self.position();
        if matches!(self.activity, BoarActivity::Rooting { .. } | BoarActivity::Trotting { .. })
            && Self::planar_dist_sq(pos, wound_spot) <= 9.0 * 9.0
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

    fn advance_toward(pos: Vec3, target: Vec3, speed: f32, dt: f32, charging: bool, world: &dyn PlanetaryWorld) -> (Vec3, Vec3, bool) {
        let flat = Vec3::new(target.x - pos.x, 0.0, target.z - pos.z);
        let dist = flat.length();
        if dist <= 0.30 {
            return (pos, Vec3::X, true);
        }
        let dir = flat / dist;
        let step = (speed * dt).min(dist);
        let max_climb = if charging { 1.65 } else { 1.25 };
        for steer in [0.0_f32, 0.68, -0.68] {
            let (s, c) = steer.sin_cos();
            let try_dir = Vec3::new(dir.x * c - dir.z * s, 0.0, dir.x * s + dir.z * c);
            let raw = pos + try_dir * step;
            let next = world.constrain_to_surface(Vec3::new(raw.x, pos.y + 2.5, raw.z), 0.0);
            let valid_zone = if charging {
                ClimateZone::dominant(next) == ClimateZone::Temperate && world.surface_up_at(next).y > 0.72
            } else {
                TemperateGlade::temperate_surface(next, world)
            };
            if (next.y - pos.y).abs() <= max_climb && valid_zone {
                return (next, try_dir, dist - step <= 0.30);
            }
            if !charging {
                break;
            }
        }
        (pos, dir, true)
    }

    /// Steps wild boar behavior, returning `(snorted, gore_impulse)`.
    pub fn step(&mut self, dt: f32, listener: Vec3, crouching: bool, world: &dyn PlanetaryWorld) -> (bool, Option<Vec3>) {
        if self.vitality.1 > 0 {
            self.vitality.1 -= 1;
        }
        let warn_r = if crouching { 5.2_f32 } else { 9.0_f32 };
        let charge_r = if crouching { 3.2_f32 } else { 5.4_f32 };
        match self.activity {
            BoarActivity::Fallen { pos, heading, timer } => {
                if timer - dt <= 0.0 {
                    self.anchor_in_temperate(0, listener, world);
                } else {
                    self.activity = BoarActivity::Fallen { pos, heading, timer: timer - dt };
                }
                (false, None)
            }
            BoarActivity::Rooting { pos, heading, timer, phase } => {
                self.up = world.surface_up_at(pos);
                let dist_sq = Self::planar_dist_sq(pos, listener);
                if dist_sq < charge_r * charge_r {
                    self.activity = BoarActivity::TuskCharge { pos, target: listener, phase: 0.0, timer: 3.0 };
                    return (true, None);
                }
                if dist_sq < warn_r * warn_r {
                    let face = Vec3::new(listener.x - pos.x, 0.0, listener.z - pos.z).normalize_or_zero();
                    self.activity = BoarActivity::BristledWarning { pos, heading: face, timer: 1.25, phase: 0.0 };
                    return (true, None);
                }
                if timer - dt <= 0.0 {
                    self.seed = Self::next_hash(self.seed);
                    let next_spot = TemperateGlade::new(pos).oak_rooting_spot(self.seed, 3.2, 5.5, world);
                    self.activity = BoarActivity::Trotting { pos, target: next_spot, phase: 0.0, cooldown: 0.0 };
                } else {
                    self.activity = BoarActivity::Rooting { pos, heading, timer: timer - dt, phase: (phase + dt * 4.5) % TAU };
                }
                (false, None)
            }
            BoarActivity::BristledWarning { pos, heading, timer, phase } => {
                self.up = world.surface_up_at(pos);
                let dist_sq = Self::planar_dist_sq(pos, listener);
                let face = Vec3::new(listener.x - pos.x, 0.0, listener.z - pos.z).normalize_or_zero();
                let next_h = if face.length_squared() > 0.2 { face } else { heading };
                if dist_sq > (warn_r + 2.6) * (warn_r + 2.6) {
                    self.activity = BoarActivity::Rooting { pos, heading: next_h, timer: 3.0, phase: 0.0 };
                    return (false, None);
                }
                if dist_sq < charge_r * charge_r || timer - dt <= 0.0 {
                    self.activity = BoarActivity::TuskCharge { pos, target: listener, phase: 0.0, timer: 3.0 };
                    return (true, None);
                }
                self.activity = BoarActivity::BristledWarning { pos, heading: next_h, timer: timer - dt, phase: (phase + dt * 7.0) % TAU };
                (false, None)
            }
            BoarActivity::Trotting { pos, target, phase, cooldown } => {
                let next_cd = (cooldown - dt).max(0.0);
                let dist_sq = Self::planar_dist_sq(pos, listener);
                if next_cd <= 0.0 && dist_sq < warn_r * warn_r {
                    let face = Vec3::new(listener.x - pos.x, 0.0, listener.z - pos.z).normalize_or_zero();
                    self.activity = BoarActivity::BristledWarning { pos, heading: face, timer: 1.0, phase: 0.0 };
                    return (true, None);
                }
                let (next_pos, dir, arrived) = Self::advance_toward(pos, target, 2.95, dt, false, world);
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
                let (next_pos, dir, _) = Self::advance_toward(pos, aim, 8.0, dt, true, world);
                self.up = world.surface_up_at(next_pos);
                let dy = listener.y - next_pos.y;
                if Self::planar_dist_sq(next_pos, listener) <= 1.28 * 1.28 && (-0.95..=2.25).contains(&dy) {
                    self.seed = Self::next_hash(self.seed);
                    let retreat = TemperateGlade::new(next_pos).escape_from(listener, self.seed, world);
                    self.activity = BoarActivity::Trotting { pos: next_pos, target: retreat, phase: 0.0, cooldown: 1.3 };
                    let shove = if dir.length_squared() > 0.2 { dir * 1.65 } else { Vec3::X * 1.65 };
                    return (false, Some(shove));
                }
                if timer - dt <= 0.0 {
                    self.activity = if Self::planar_dist_sq(next_pos, listener) < warn_r * warn_r {
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
        let flash = (self.vitality.1 as f32) / 14.0;
        let (pos, fwd, pitch, stride, bristle) = match self.activity {
            BoarActivity::Fallen { .. } => return,
            BoarActivity::Rooting { pos, heading, phase, .. } => (pos, heading, -0.44 + 0.12 * phase.sin(), 0.0, 0.15),
            BoarActivity::BristledWarning { pos, heading, phase, .. } => (pos, heading, 0.14 + 0.06 * phase.sin(), 0.0, 1.0),
            BoarActivity::Trotting { pos, target, phase, .. } => (pos, target - pos, -0.08, phase, 0.35),
            BoarActivity::TuskCharge { pos, target, phase, .. } => (pos, target - pos, -0.26 + 0.10 * phase.sin(), phase, 1.0),
        };
        BoarFigure::aligned(pos, self.up, fwd, pitch, stride, bristle, flash).emit(target);
    }
}
