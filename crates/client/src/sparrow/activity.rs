use std::f32::consts::{PI, TAU};
use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld};
use super::mesh::SparrowFigure;

const FEAR_DISTANCE_SQ: f32 = 4.5 * 4.5;

/// Polymorphic behavioral state of a woodland sparrow: flying, ground pecking, or tree perching.
#[derive(Clone, Copy, Debug)]
pub enum SparrowActivity {
    Flying {
        origin: Vec3,
        target: Vec3,
        progress: f32,
        wing_phase: f32,
    },
    GroundPecking {
        position: Vec3,
        yaw: f32,
        peck_phase: f32,
        remaining: f32,
    },
    TreePerched {
        position: Vec3,
        yaw: f32,
        perch_phase: f32,
        remaining: f32,
    },
}

/// Autonomous woodland house sparrow entity cycling through flight, ground foraging, and tree perching.
#[derive(Clone, Copy, Debug)]
pub struct Sparrow {
    activity: SparrowActivity,
    up: Vec3,
    seed: u32,
}

impl Sparrow {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(activity: SparrowActivity, up: Vec3, seed: u32) -> Self {
        Self { activity, up, seed }
    }

    /// Secondary constructor initializing an unanchored sparrow with a unique seed.
    pub const fn seeded(seed: u32) -> Self {
        let idle = SparrowActivity::GroundPecking {
            position: Vec3::Y,
            yaw: 0.0,
            peck_phase: 0.0,
            remaining: 1.0,
        };
        Self::new(idle, Vec3::Y, seed)
    }

    #[cfg(test)]
    pub const fn airborne(&self) -> bool {
        matches!(self.activity, SparrowActivity::Flying { .. })
    }

    #[cfg(test)]
    pub const fn ground_pecking(&self) -> bool {
        matches!(self.activity, SparrowActivity::GroundPecking { .. })
    }

    #[cfg(test)]
    pub const fn tree_perched(&self) -> bool {
        matches!(self.activity, SparrowActivity::TreePerched { .. })
    }

    pub fn position(&self) -> Vec3 {
        match self.activity {
            SparrowActivity::Flying { origin, target, progress, .. } => {
                let p = progress.clamp(0.0, 1.0);
                let span = (target - origin).length().max(2.0);
                let arc = (p * PI).sin() * (1.20 + span * 0.14).min(3.4);
                origin.lerp(target, p) + self.up * arc
            }
            SparrowActivity::GroundPecking { position, .. } | SparrowActivity::TreePerched { position, .. } => position,
        }
    }

    const fn next_hash(seed: u32) -> u32 {
        seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453)
    }

    fn tangent_frame(up: Vec3) -> (Vec3, Vec3) {
        let n_up = up.normalize_or_zero();
        let axis = if n_up.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let t1 = n_up.cross(axis).normalize_or_zero();
        (t1, n_up.cross(t1).normalize_or_zero())
    }

    fn frightened_by(bird_pos: Vec3, human_pos: Vec3, up: Vec3) -> bool {
        let delta = bird_pos - human_pos;
        if delta.length_squared() < FEAR_DISTANCE_SQ {
            return true;
        }
        let vertical = delta.dot(up).abs();
        let planar_sq = (delta - up * delta.dot(up)).length_squared();
        vertical < 3.8 && planar_sq < 3.6 * 3.6
    }

    fn flee_target_from(bird_pos: Vec3, human_pos: Vec3, up: Vec3, hash: u32, world: &dyn PlanetaryWorld) -> Vec3 {
        let (t1, _) = Self::tangent_frame(up);
        let raw_away = bird_pos - human_pos;
        let flat_away = (raw_away - up * raw_away.dot(up)).normalize_or_zero();
        let away = if flat_away.length_squared() > 0.25 { flat_away } else { t1 };
        let side = up.cross(away).normalize_or_zero();
        let spread = (((hash % 41) as i32 - 20) as f32) * 0.024;
        let (ss, cs) = spread.sin_cos();
        let flee_dir = (away * cs + side * ss).normalize_or_zero();
        let flee_dist = 9.5 + (((hash >> 8) % 50) as f32) * 0.10;
        let escape_point = bird_pos + flee_dir * flee_dist;
        if (hash & 1) == 0 {
            if let Some(perch) = world.arboreal_perch_near(escape_point, (hash as usize) % 16) {
                if (perch - human_pos).length_squared() > 7.5 * 7.5 && (perch - bird_pos).dot(flee_dir) > 2.0 {
                    return perch;
                }
            }
        }
        world.constrain_to_surface(escape_point + up * 6.0, 0.0)
    }

    fn ground_spot_near(center: Vec3, up: Vec3, hash: u32, world: &dyn PlanetaryWorld) -> Vec3 {
        let (t1, t2) = Self::tangent_frame(up);
        let (sa, ca) = (((hash % 360) as f32) * (TAU / 360.0)).sin_cos();
        let radius = 5.4 + (((hash >> 9) % 70) as f32) * 0.11;
        world.constrain_to_surface(center + (t1 * ca + t2 * sa) * radius + up * 6.0, 0.0)
    }

    fn tree_or_ground_target(center: Vec3, up: Vec3, hash: u32, prefer_tree: bool, world: &dyn PlanetaryWorld) -> Vec3 {
        if prefer_tree {
            if let Some(perch) = world.arboreal_perch_near(center, (hash as usize) % 16) {
                if !Self::frightened_by(perch, center, up) {
                    return perch;
                }
            }
        }
        Self::ground_spot_near(center, up, hash, world)
    }

    /// Anchors this sparrow into an initial role around `listener` (0: Flying, 1: GroundPecking, 2: TreePerched).
    pub fn anchor_around(&mut self, slot: usize, listener: Vec3, world: &dyn PlanetaryWorld) {
        self.up = world.surface_up_at(listener);
        self.seed = Self::next_hash(self.seed ^ (slot as u32).wrapping_mul(1_664_525));
        let yaw = ((self.seed % 628) as f32) * 0.01;
        let phase = (((self.seed >> 8) % 628) as f32) * 0.01;
        let remaining = 3.2 + (((self.seed >> 16) % 40) as f32) * 0.10;
        self.activity = match slot % 3 {
            0 => {
                let origin = Self::ground_spot_near(listener, self.up, self.seed, world) + self.up * 0.8;
                let next_seed = Self::next_hash(self.seed);
                let target = Self::tree_or_ground_target(listener, self.up, next_seed, (slot / 3) % 2 == 0, world);
                SparrowActivity::Flying { origin, target, progress: ((slot as f32) * 0.19) % 0.65, wing_phase: phase }
            }
            1 => SparrowActivity::GroundPecking {
                position: Self::ground_spot_near(listener, self.up, self.seed, world),
                yaw,
                peck_phase: phase,
                remaining,
            },
            _ => match world.arboreal_perch_near(listener, slot).filter(|&p| !Self::frightened_by(p, listener, self.up)) {
                Some(position) => SparrowActivity::TreePerched { position, yaw, perch_phase: phase, remaining },
                None => SparrowActivity::GroundPecking {
                    position: Self::ground_spot_near(listener, self.up, self.seed, world),
                    yaw,
                    peck_phase: phase,
                    remaining,
                },
            },
        };
    }

    /// Advances flight trajectory, ground pecking, or tree perching and returns `true` when vocalizing a chirp.
    pub fn step(&mut self, dt: f32, listener: Vec3, world: &dyn PlanetaryWorld) -> bool {
        let cur_pos = self.position();
        self.up = world.surface_up_at(cur_pos);
        match self.activity {
            SparrowActivity::Flying { origin, target, progress, wing_phase } => {
                if progress > 0.15 && Self::frightened_by(target, listener, self.up) {
                    self.seed = Self::next_hash(self.seed);
                    let escape = Self::flee_target_from(cur_pos, listener, self.up, self.seed, world);
                    self.activity = SparrowActivity::Flying { origin: cur_pos, target: escape, progress: 0.0, wing_phase };
                    return true;
                }
                let span = (target - origin).length().max(2.0);
                let next_prog = progress + dt * (5.2 / span);
                let next_wing = (wing_phase + dt * 22.0) % TAU;
                if next_prog < 1.0 {
                    self.activity = SparrowActivity::Flying { origin, target, progress: next_prog, wing_phase: next_wing };
                    return false;
                }
                self.seed = Self::next_hash(self.seed);
                let yaw = ((self.seed % 628) as f32) * 0.01;
                let remaining = 3.0 + (((self.seed >> 10) % 45) as f32) * 0.10;
                let ground = world.constrain_to_surface(target, 0.0);
                if world.canopy_drag(target) > 0.01 && (target - ground).dot(self.up) > 1.2 {
                    self.activity = SparrowActivity::TreePerched { position: target, yaw, perch_phase: 0.0, remaining };
                    true
                } else {
                    self.activity = SparrowActivity::GroundPecking { position: ground, yaw, peck_phase: 0.0, remaining };
                    false
                }
            }
            SparrowActivity::GroundPecking { position, yaw, peck_phase, remaining } => {
                let next_peck = (peck_phase + dt * 9.5) % TAU;
                let next_rem = remaining - dt;
                let startled = Self::frightened_by(position, listener, self.up);
                if startled || next_rem <= 0.0 || (position - listener).length_squared() > 38.0 * 38.0 {
                    self.seed = Self::next_hash(self.seed);
                    let target = if startled {
                        Self::flee_target_from(position, listener, self.up, self.seed, world)
                    } else {
                        Self::tree_or_ground_target(listener, self.up, self.seed, (self.seed & 1) == 0, world)
                    };
                    self.activity = SparrowActivity::Flying { origin: position + self.up * 0.08, target, progress: 0.0, wing_phase: 0.0 };
                    return startled;
                }
                let (t1, t2) = Self::tangent_frame(self.up);
                let (sy, cy) = yaw.sin_cos();
                let hop = (next_peck * 0.5).sin().max(0.0) * dt * 0.24;
                let next_pos = world.constrain_to_surface(position + (t1 * cy + t2 * sy) * hop + self.up * 0.5, 0.0);
                self.activity = SparrowActivity::GroundPecking { position: next_pos, yaw, peck_phase: next_peck, remaining: next_rem };
                false
            }
            SparrowActivity::TreePerched { position, yaw, perch_phase, remaining } => {
                let next_phase = (perch_phase + dt * 4.4) % TAU;
                let next_rem = remaining - dt;
                let startled = Self::frightened_by(position, listener, self.up);
                let tree_felled = world.canopy_drag(position) <= 0.01;
                if startled || tree_felled || next_rem <= 0.0 || (position - listener).length_squared() > 38.0 * 38.0 {
                    self.seed = Self::next_hash(self.seed);
                    let target = if startled {
                        Self::flee_target_from(position, listener, self.up, self.seed, world)
                    } else {
                        Self::tree_or_ground_target(listener, self.up, self.seed, !tree_felled && self.seed % 3 == 0, world)
                    };
                    self.activity = SparrowActivity::Flying { origin: position, target, progress: 0.0, wing_phase: 0.0 };
                    return true;
                }
                self.activity = SparrowActivity::TreePerched { position, yaw, perch_phase: next_phase, remaining: next_rem };
                perch_phase < PI && next_phase >= PI
            }
        }
    }

    /// Emits this sparrow's 3D sculpted mesh in its current flight, pecking, or perched posture.
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        let (t1, t2) = Self::tangent_frame(self.up);
        match self.activity {
            SparrowActivity::Flying { origin, target: dest, progress, wing_phase } => {
                let dir = (dest - origin).normalize_or_zero();
                SparrowFigure::aligned(self.position(), self.up, dir, -((0.5 - progress) * 0.42), wing_phase.sin() * 0.88, true).emit(target);
            }
            SparrowActivity::GroundPecking { position, yaw, peck_phase, .. } => {
                let (sy, cy) = (yaw + (peck_phase * 0.5).sin() * 0.18).sin_cos();
                let peck_dip = peck_phase.sin().max(0.0).powi(2) * 0.88;
                SparrowFigure::aligned(position, self.up, t1 * cy + t2 * sy, peck_dip, 0.0, false).emit(target);
            }
            SparrowActivity::TreePerched { position, yaw, perch_phase, .. } => {
                let (sy, cy) = (yaw + (perch_phase * 0.8).sin() * 0.36).sin_cos();
                SparrowFigure::aligned(position, self.up, t1 * cy + t2 * sy, (perch_phase * 2.0).sin() * 0.10, (perch_phase * 3.0).sin() * 0.25, false).emit(target);
            }
        }
    }
}
