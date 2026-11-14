use std::f32::consts::{PI, TAU};
use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld};
use super::habitat::TemperateGlade;
use super::mesh::RoeFigure;

const STANDING_ALERT_SQ: f32 = 12.5 * 12.5;
const STANDING_FLEE_SQ: f32 = 7.5 * 7.5;
const CROUCH_ALERT_SQ: f32 = 6.2 * 6.2;
const CROUCH_FLEE_SQ: f32 = 3.4 * 3.4;

/// Polymorphic behavioral state of a Temperate European roe deer.
#[derive(Clone, Copy, Debug)]
pub enum RoeActivity {
    Grazing {
        position: Vec3,
        yaw: f32,
        chew_phase: f32,
        remaining: f32,
    },
    AlertStance {
        position: Vec3,
        yaw: f32,
        scan_phase: f32,
        remaining: f32,
    },
    Trotting {
        position: Vec3,
        target: Vec3,
        stride_phase: f32,
        remaining: f32,
    },
    BoundingFlight {
        origin: Vec3,
        target: Vec3,
        progress: f32,
        stride_phase: f32,
    },
}

/// Autonomous woodland roe deer native to `ClimateZone::Temperate` meadows and oak glades.
#[derive(Clone, Copy, Debug)]
pub struct RoeDeer {
    activity: RoeActivity,
    up: Vec3,
    seed: u32,
    antlered: bool,
}

impl RoeDeer {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(activity: RoeActivity, up: Vec3, seed: u32, antlered: bool) -> Self {
        Self { activity, up, seed, antlered }
    }

    /// Secondary constructor initializing an unanchored roe deer with deterministic seed and antler trait.
    pub const fn seeded(seed: u32, antlered: bool) -> Self {
        let calm = RoeActivity::Grazing {
            position: Vec3::Y,
            yaw: 0.0,
            chew_phase: 0.0,
            remaining: 2.0,
        };
        Self::new(calm, Vec3::Y, seed, antlered)
    }

    #[cfg(test)]
    pub const fn grazing(&self) -> bool {
        matches!(self.activity, RoeActivity::Grazing { .. })
    }

    #[cfg(test)]
    pub const fn alert(&self) -> bool {
        matches!(self.activity, RoeActivity::AlertStance { .. })
    }

    #[cfg(test)]
    pub const fn trotting(&self) -> bool {
        matches!(self.activity, RoeActivity::Trotting { .. })
    }

    #[cfg(test)]
    pub const fn bounding(&self) -> bool {
        matches!(self.activity, RoeActivity::BoundingFlight { .. })
    }

    #[cfg(test)]
    pub const fn antlered(&self) -> bool {
        self.antlered
    }

    pub fn position(&self) -> Vec3 {
        match self.activity {
            RoeActivity::Grazing { position, .. }
            | RoeActivity::AlertStance { position, .. }
            | RoeActivity::Trotting { position, .. } => position,
            RoeActivity::BoundingFlight { origin, target, progress, .. } => {
                let p = progress.clamp(0.0, 1.0);
                let leap = (p * 3.0 * PI).sin().abs() * 0.74;
                origin.lerp(target, p) + self.up * leap
            }
        }
    }

    const fn next_hash(seed: u32) -> u32 {
        seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453)
    }

    fn proximity_threat(deer_pos: Vec3, human_pos: Vec3, crouching: bool) -> (bool, bool) {
        let delta = deer_pos - human_pos;
        if delta.y.abs() > 5.0 {
            return (false, false);
        }
        let planar_sq = delta.x * delta.x + delta.z * delta.z;
        let (alert_sq, flee_sq) = if crouching {
            (CROUCH_ALERT_SQ, CROUCH_FLEE_SQ)
        } else {
            (STANDING_ALERT_SQ, STANDING_FLEE_SQ)
        };
        (planar_sq < alert_sq, planar_sq < flee_sq)
    }

    fn yaw_toward(from: Vec3, toward: Vec3) -> f32 {
        (toward.z - from.z).atan2(toward.x - from.x)
    }

    /// Anchors this roe deer on `ClimateZone::Temperate` in a peaceful herd formation.
    pub fn anchor_in_temperate(&mut self, slot: usize, listener: Vec3, world: &dyn PlanetaryWorld) {
        let glade = TemperateGlade::around(listener, world);
        self.seed = Self::next_hash(self.seed ^ (slot as u32).wrapping_mul(2_654_435_761));
        let position = glade.meadow_spot(self.seed, 9.5, 9.0, world);
        self.up = world.surface_up_at(position);
        let yaw = ((self.seed % 628) as f32) * 0.01;
        let phase = (((self.seed >> 8) % 628) as f32) * 0.01;
        let remaining = 3.8 + (((self.seed >> 16) % 45) as f32) * 0.10;
        self.activity = match slot % 4 {
            1 => RoeActivity::AlertStance { position, yaw, scan_phase: phase, remaining },
            3 => {
                let next_seed = Self::next_hash(self.seed);
                let target = TemperateGlade::new(position).meadow_spot(next_seed, 5.5, 5.5, world);
                RoeActivity::Trotting { position, target, stride_phase: phase, remaining: 6.0 }
            }
            _ => RoeActivity::Grazing { position, yaw, chew_phase: phase, remaining },
        };
    }

    /// Advances grazing, alert watching, calm trotting, or bounding flight; returns `true` when bounding away.
    pub fn step(&mut self, dt: f32, listener: Vec3, crouching: bool, world: &dyn PlanetaryWorld) -> bool {
        let cur_pos = self.position();
        self.up = world.surface_up_at(cur_pos);
        let (cautious, startled) = Self::proximity_threat(cur_pos, listener, crouching);
        match self.activity {
            RoeActivity::Grazing { position, yaw, chew_phase, remaining } => {
                if startled {
                    self.seed = Self::next_hash(self.seed);
                    let target = TemperateGlade::new(position).escape_from(listener, self.seed, world);
                    self.activity = RoeActivity::BoundingFlight { origin: position, target, progress: 0.0, stride_phase: 0.0 };
                    return true;
                }
                if cautious {
                    let look_yaw = Self::yaw_toward(position, listener);
                    self.activity = RoeActivity::AlertStance { position, yaw: look_yaw, scan_phase: 0.0, remaining: 2.8 };
                    return false;
                }
                let next_rem = remaining - dt;
                if next_rem <= 0.0 {
                    self.seed = Self::next_hash(self.seed);
                    if (self.seed & 1) == 0 {
                        let target = TemperateGlade::around(listener, world).meadow_spot(self.seed, 9.5, 11.0, world);
                        self.activity = RoeActivity::Trotting { position, target, stride_phase: 0.0, remaining: 8.5 };
                    } else {
                        self.activity = RoeActivity::AlertStance { position, yaw, scan_phase: 0.0, remaining: 2.6 };
                    }
                    return false;
                }
                self.activity = RoeActivity::Grazing {
                    position,
                    yaw,
                    chew_phase: (chew_phase + dt * 5.4) % TAU,
                    remaining: next_rem,
                };
                false
            }
            RoeActivity::AlertStance { position, yaw, scan_phase, remaining } => {
                if startled {
                    self.seed = Self::next_hash(self.seed);
                    let target = TemperateGlade::new(position).escape_from(listener, self.seed, world);
                    self.activity = RoeActivity::BoundingFlight { origin: position, target, progress: 0.0, stride_phase: 0.0 };
                    return true;
                }
                let next_yaw = if cautious { Self::yaw_toward(position, listener) } else { yaw };
                let next_rem = if cautious { remaining.max(1.4) - dt * 0.35 } else { remaining - dt };
                if next_rem <= 0.0 {
                    self.seed = Self::next_hash(self.seed);
                    let graze_dur = 4.2 + ((self.seed % 40) as f32) * 0.10;
                    self.activity = RoeActivity::Grazing { position, yaw: next_yaw, chew_phase: 0.0, remaining: graze_dur };
                    return false;
                }
                self.activity = RoeActivity::AlertStance {
                    position,
                    yaw: next_yaw,
                    scan_phase: (scan_phase + dt * 2.6) % TAU,
                    remaining: next_rem,
                };
                false
            }
            RoeActivity::Trotting { position, target, stride_phase, remaining } => {
                if startled {
                    self.seed = Self::next_hash(self.seed);
                    let escape = TemperateGlade::new(position).escape_from(listener, self.seed, world);
                    self.activity = RoeActivity::BoundingFlight { origin: position, target: escape, progress: 0.0, stride_phase };
                    return true;
                }
                let planar = Vec3::new(target.x - position.x, 0.0, target.z - position.z);
                let dist = planar.length();
                let next_rem = remaining - dt;
                if dist < 0.40 || next_rem <= 0.0 {
                    let yaw = if dist > 1e-3 { planar.z.atan2(planar.x) } else { 0.0 };
                    self.activity = RoeActivity::Grazing { position, yaw, chew_phase: 0.0, remaining: 4.5 };
                    return false;
                }
                let step_vec = (planar / dist) * (2.35 * dt).min(dist);
                let candidate = world.constrain_to_surface(position + step_vec + Vec3::Y * 4.0, 0.0);
                let next_pos = if TemperateGlade::temperate_surface(candidate, world) { candidate } else { position };
                self.activity = RoeActivity::Trotting {
                    position: next_pos,
                    target,
                    stride_phase: (stride_phase + dt * 9.2) % TAU,
                    remaining: next_rem,
                };
                false
            }
            RoeActivity::BoundingFlight { origin, target, progress, stride_phase } => {
                let span = (target - origin).length().max(4.0);
                let next_prog = progress + dt * (7.8 / span);
                if next_prog < 1.0 {
                    self.activity = RoeActivity::BoundingFlight {
                        origin,
                        target,
                        progress: next_prog,
                        stride_phase: (stride_phase + dt * 16.0) % TAU,
                    };
                    return false;
                }
                let landed = world.constrain_to_surface(target + Vec3::Y * 4.0, 0.0);
                let watch_yaw = Self::yaw_toward(landed, listener);
                self.activity = RoeActivity::AlertStance { position: landed, yaw: watch_yaw, scan_phase: 0.0, remaining: 3.4 };
                false
            }
        }
    }

    /// Emits this roe deer's 3D sculpted mesh in its active posture.
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        match self.activity {
            RoeActivity::Grazing { position, yaw, chew_phase, .. } => {
                let (sy, cy) = (yaw + (chew_phase * 0.5).sin() * 0.08).sin_cos();
                let neck_dip = 1.56 + chew_phase.sin() * 0.12;
                RoeFigure::aligned(position, self.up, Vec3::new(cy, 0.0, sy), neck_dip, 0.0, 0.0, self.antlered).emit(target);
            }
            RoeActivity::AlertStance { position, yaw, scan_phase, .. } => {
                let (sy, cy) = (yaw + scan_phase.sin() * 0.16).sin_cos();
                RoeFigure::aligned(position, self.up, Vec3::new(cy, 0.0, sy), -0.14, 0.0, 0.0, self.antlered).emit(target);
            }
            RoeActivity::Trotting { position, target: dest, stride_phase, .. } => {
                let fwd = Vec3::new(dest.x - position.x, 0.0, dest.z - position.z);
                let bob_pitch = 0.08 + (stride_phase * 2.0).sin() * 0.06;
                RoeFigure::aligned(position, self.up, fwd, bob_pitch, stride_phase, 0.0, self.antlered).emit(target);
            }
            RoeActivity::BoundingFlight { origin, target: dest, progress, stride_phase } => {
                let fwd = Vec3::new(dest.x - origin.x, 0.0, dest.z - origin.z);
                let lift = (progress.clamp(0.0, 1.0) * 3.0 * PI).sin().abs() * 0.74;
                RoeFigure::aligned(self.position(), self.up, fwd, -0.10, stride_phase, lift, self.antlered).emit(target);
            }
        }
    }
}
