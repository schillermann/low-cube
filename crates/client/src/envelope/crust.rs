use glam::Vec3;
use world::PlanetaryWorld;
use super::{BodyEnvelope, FOOTPRINT_PROBE_RATIO, GROUND_SETTLE_OFFSET, MIN_CROUCH_HEIGHT, STEP_RISE_LIMIT};

const CROUCH_HEAD_DROP: f32 = 0.44;
const CROWN_REACH_MIN: f32 = 0.36;
const CROWN_REACH_MAX: f32 = 0.80;
const OVERHEAD_SCAN_WINDOW: f32 = 0.65;
const DEEP_SHAFT_SPAN: f32 = 2.50;
const SHAFT_WALL_RISE: f32 = 1.25;
const SHAFT_WALL_STEP: f32 = 0.05;
const SHAFT_MAX_SPAN: f32 = 1.20;
const SHAFT_DEEP_WALL: f32 = 2.20;
const RIM_PROBE_OFFSET: f32 = 0.50;
const RIM_STEP_TOLERANCE: f32 = 0.45;
const NICHE_FLOOR_CLEARANCE: f32 = 0.25;
const FLOOR_SNAP_MARGIN: f32 = -0.05;
const UPWARD_SHAFT_CEILING_RISE: f32 = 0.20;

/// Decorator over `BodyEnvelope` and `PlanetaryWorld` enforcing subterranean crust and shaft confinement.
#[derive(Clone, Copy)]
pub struct CrustEnvelope<'a> {
    envelope: &'a BodyEnvelope,
    world: &'a dyn PlanetaryWorld,
}

impl<'a> CrustEnvelope<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(envelope: &'a BodyEnvelope, world: &'a dyn PlanetaryWorld) -> Self {
        Self { envelope, world }
    }

    pub fn confine(&self, point: Vec3, origin: Vec3, up: Vec3) -> Vec3 {
        let floor_pt = self.envelope.constrain_with_height(point, 1.0, self.world);
        if self.world.weightless_at(point) || self.world.weightless_at(origin) || (floor_pt - point).dot(up) > NICHE_FLOOR_CLEARANCE {
            return self.world.confine_beneath_crust(point);
        }
        let (half_w, half_d, height, facing) = self.envelope.dimensions();
        let head_off = up * (height - 1.0).clamp(CROWN_REACH_MIN, CROWN_REACH_MAX);
        let mut confined = self.world.confine_beneath_crust(point + head_off) - head_off;
        let f_tan = (facing - up * facing.dot(up)).normalize_or_zero();
        if f_tan.length_squared() > 1e-4 {
            let rgt = f_tan.cross(up).normalize_or_zero();
            let (d_probe, w_probe) = (half_d * FOOTPRINT_PROBE_RATIO, half_w * FOOTPRINT_PROBE_RATIO);
            for off in [f_tan * d_probe, -f_tan * d_probe, rgt * w_probe, -rgt * w_probe] {
                let probe = self.world.confine_beneath_crust(point + off + head_off) - head_off - off;
                if (probe - point).dot(up) < -1e-4 && (probe - confined).dot(up) < 0.0 {
                    confined += up * (probe - confined).dot(up);
                }
            }
        }
        if (point - floor_pt).dot(up) >= FLOOR_SNAP_MARGIN && (confined - floor_pt).dot(up) < 0.0 {
            confined += up * (floor_pt - confined).dot(up);
        }
        confined
    }

    pub fn in_deep_shaft(&self, point: Vec3, up: Vec3, eye: f32) -> bool {
        (point - self.envelope.constrain_with_height(point, eye, self.world)).dot(up) > DEEP_SHAFT_SPAN
    }

    pub fn in_upward_shaft(&self, point: Vec3, up: Vec3, eye: f32) -> bool {
        self.in_deep_shaft(point, up, eye)
            && (self.world.constrain_to_surface(point, 0.0) - point).dot(up) > UPWARD_SHAFT_CEILING_RISE
    }

    pub fn catch_shaft_rim(&self, origin: Vec3, next_pos: Vec3, up: Vec3, eye: f32) -> Option<Vec3> {
        if (next_pos - origin).dot(up) <= 0.0
            || self.world.weightless_at(next_pos)
            || (next_pos - self.world.constrain_to_surface(next_pos, eye)).dot(up) <= DEEP_SHAFT_SPAN
        {
            return None;
        }
        let axis_a = if up.x.abs() < 0.5 { Vec3::X } else { Vec3::Z };
        let axis_b = up.cross(axis_a).normalize();
        for dir in [axis_a, -axis_a, axis_b, -axis_b] {
            let rim = self.world.constrain_to_surface(next_pos + dir * RIM_PROBE_OFFSET + up * RIM_PROBE_OFFSET, eye);
            if (next_pos - rim).dot(up).abs() <= RIM_STEP_TOLERANCE && !self.world.weightless_at(rim) {
                return Some(rim);
            }
        }
        None
    }

    pub fn overhead_obstructed(&self, position: Vec3, up: Vec3, eye: f32) -> bool {
        let (half_w, half_d, height, facing) = self.envelope.dimensions();
        let crouched = BodyEnvelope::with_dimensions(half_w, half_d, (height - CROUCH_HEAD_DROP).max(MIN_CROUCH_HEIGHT), facing);
        let settled = position - up * GROUND_SETTLE_OFFSET;
        let base_floor = crouched.constrain_with_height(settled, eye, self.world);
        if (position - base_floor).dot(up).abs() > OVERHEAD_SCAN_WINDOW {
            return false;
        }
        let f_tan = (facing - up * facing.dot(up)).normalize_or_zero();
        let fwd = if f_tan.length_squared() > 1e-4 {
            f_tan
        } else if up.y.abs() < 0.9 {
            up.cross(Vec3::Y).normalize()
        } else {
            Vec3::Z
        };
        let rgt = fwd.cross(up).normalize_or_zero();
        let (d_probe, w_probe) = (half_d * FOOTPRINT_PROBE_RATIO, half_w * FOOTPRINT_PROBE_RATIO);
        for offset in [Vec3::ZERO, fwd * d_probe, -fwd * d_probe, rgt * w_probe, -rgt * w_probe] {
            let pt = position + offset - up * GROUND_SETTLE_OFFSET;
            let low = crouched.constrain_with_height(pt, eye, self.world);
            let high = self.envelope.constrain_with_height(pt, eye, self.world);
            if (low - base_floor).dot(up).abs() <= STEP_RISE_LIMIT && (high - low).dot(up) > STEP_RISE_LIMIT {
                return true;
            }
        }
        false
    }

    fn wall_rise(&self, base: Vec3, dir: Vec3, dist: f32, up: Vec3) -> Option<f32> {
        let surf = self.world.constrain_to_surface(base + dir * dist, 0.0);
        let rise = (surf - base).dot(up);
        if up.dot(self.world.surface_up_at(surf)) >= 0.7 && rise >= SHAFT_WALL_RISE {
            Some(rise)
        } else {
            None
        }
    }

    fn wall_distance(&self, base: Vec3, dir: Vec3, up: Vec3) -> (f32, f32) {
        for coarse in (3..=24).step_by(3) {
            if let Some(coarse_rise) = self.wall_rise(base, dir, (coarse as f32) * SHAFT_WALL_STEP, up) {
                for fine in (coarse - 2)..coarse {
                    let d = (fine as f32) * SHAFT_WALL_STEP;
                    if let Some(rise) = self.wall_rise(base, dir, d, up) {
                        return (d, rise);
                    }
                }
                return ((coarse as f32) * SHAFT_WALL_STEP, coarse_rise);
            }
        }
        (2.0, 0.0)
    }

    pub(super) fn axis_wedged(&self, base: Vec3, axis: Vec3, up: Vec3) -> bool {
        let (half_w, _, _, _) = self.envelope.dimensions();
        let ((d_pos, r_pos), (d_neg, r_neg)) = (
            self.wall_distance(base, axis, up),
            self.wall_distance(base, -axis, up),
        );
        d_pos <= SHAFT_MAX_SPAN
            && d_neg <= SHAFT_MAX_SPAN
            && (d_pos + d_neg - SHAFT_WALL_STEP) < half_w * 2.0
            && ((r_pos - r_neg).abs() < SHAFT_MAX_SPAN || r_pos.min(r_neg) > SHAFT_DEEP_WALL)
    }
}
