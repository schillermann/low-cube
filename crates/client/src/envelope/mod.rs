mod crust;

use glam::Vec3;
use world::PlanetaryWorld;
use crate::threshold::{StepThreshold, SurfaceClearance, SurfaceStep};

pub(super) const MIN_CROUCH_HEIGHT: f32 = 0.95;
pub(super) const FOOTPRINT_PROBE_RATIO: f32 = 0.70;
pub(super) const GROUND_SETTLE_OFFSET: f32 = 0.45;
pub(super) const STEP_RISE_LIMIT: f32 = 0.35;
const POST_WALL_RISE: f32 = 0.50;
const NARROW_CHEST_REACH: f32 = 0.18;
const HEEL_STEP_OFFSET: f32 = 0.25;
const DROP_LEDGE_MARGIN: f32 = 0.10;
const FLANK_DOORPOST_RATIO: f32 = 1.35;
const PIT_RIM_DROP: f32 = 0.52;
const DEEP_SHAFT_SPAN: f32 = 2.50;
const SHAFT_FOOT_SNAP: f32 = 1.80;

/// Interface for physical avatar collision volumes preventing wall penetration and camera clipping.
pub trait PhysicalEnvelope {
    #[allow(clippy::too_many_arguments)]
    fn unobstructed_ground(&self, candidate: Vec3, surface_pos: Vec3, step: Vec3, up: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn unobstructed_aerial(&self, origin: Vec3, candidate: Vec3, tangent: Vec3, up: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> bool;

    fn wedged_in_shaft(&self, position: Vec3, up: Vec3, eye: f32, world: &dyn PlanetaryWorld) -> bool;
}

/// Physical avatar body volume preventing wall penetration and camera clipping.
#[derive(Clone, Copy, Debug)]
pub struct BodyEnvelope {
    half_width: f32,
    half_depth: f32,
    height: f32,
    facing: Vec3,
}

impl BodyEnvelope {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_dimensions(half_width: f32, half_depth: f32, height: f32, facing: Vec3) -> Self {
        Self { half_width, half_depth, height, facing }
    }

    pub fn youth() -> Self {
        Self::for_physique(crate::attire::YouthPhysique::SlenderSquire)
    }

    /// Secondary constructor deriving width (Breite), depth (Tiefe), and height (Höhe) from the character's YouthPhysique.
    pub fn for_physique(physique: crate::attire::YouthPhysique) -> Self {
        Self::with_dimensions(
            (physique.upper_body_span() * 0.5).clamp(0.18, 0.55),
            (physique.body_depth() * 0.5).clamp(0.12, 0.34),
            physique.stature_height().clamp(1.15, 2.48),
            Vec3::ZERO,
        )
    }

    pub fn with_crouch_and_facing(self, crouch_drop: f32, facing: Vec3) -> Self {
        Self::with_dimensions(
            self.half_width,
            self.half_depth,
            (self.height - crouch_drop.max(0.0)).max(MIN_CROUCH_HEIGHT),
            facing,
        )
    }

    fn cardinal_up(up: Vec3) -> Vec3 {
        let ax = up.abs();
        if ax.x >= ax.y && ax.x >= ax.z {
            Vec3::new(up.x.signum(), 0.0, 0.0)
        } else if ax.y >= ax.x && ax.y >= ax.z {
            Vec3::new(0.0, up.y.signum(), 0.0)
        } else {
            Vec3::new(0.0, 0.0, up.z.signum())
        }
    }

    pub fn constrain_with_height(&self, point: Vec3, eye: f32, world: &dyn PlanetaryWorld) -> Vec3 {
        let (base_surf, face_up) = (world.constrain_to_surface(point, eye), Self::cardinal_up(world.surface_up_at(point)));
        let delta = base_surf - point;
        if (delta - face_up * delta.dot(face_up)).length_squared() > 1e-4 {
            return base_surf;
        }
        world.constrain_to_surface(point + face_up * (self.height - eye), self.height) - face_up * (self.height - eye)
    }

    fn oriented_extents(&self, dir: Vec3, right: Vec3, up: Vec3) -> (f32, f32) {
        let f_tan = (self.facing - up * self.facing.dot(up)).normalize_or_zero();
        if f_tan.length_squared() <= 1e-4 {
            (self.half_depth, self.half_width)
        } else {
            let (c, s) = (dir.dot(f_tan).abs(), right.dot(f_tan).abs());
            (self.half_depth * c + self.half_width * s, self.half_width * c + self.half_depth * s)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn post_obstructed(&self, pt: Vec3, floor: Vec3, orig_floor: Vec3, up: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> bool {
        let s = self.constrain_with_height(pt, eye, world);
        up.dot(world.surface_up_at(s)) >= 0.7
            && (s - floor).dot(up) > POST_WALL_RISE
            && (s - orig_floor).dot(up) > STEP_RISE_LIMIT
            && !SurfaceStep::new(floor, s, up).walkable_with(limit)
    }

    #[allow(clippy::too_many_arguments)]
    fn chest_reach(&self, origin: Vec3, dir: Vec3, orig_center: Vec3, floor: Vec3, raw_depth: f32, up: Vec3, eye: f32, world: &dyn PlanetaryWorld) -> f32 {
        let back_surf = self.constrain_with_height(origin - dir * HEEL_STEP_OFFSET, eye, world);
        if (orig_center - floor).dot(up) > DROP_LEDGE_MARGIN || (back_surf - floor).dot(up) > GROUND_SETTLE_OFFSET {
            NARROW_CHEST_REACH.min(raw_depth)
        } else {
            raw_depth
        }
    }
}

impl PhysicalEnvelope for BodyEnvelope {
    fn unobstructed_ground(&self, candidate: Vec3, surface_pos: Vec3, step: Vec3, up: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> bool {
        let tangent = step - up * step.dot(up);
        if tangent.length_squared() <= 1e-6 {
            return true;
        }
        let dir = tangent.normalize();
        let right = dir.cross(up).normalize_or_zero();
        let (raw_depth, lateral_span) = self.oriented_extents(dir, right, up);
        let origin = candidate - step;
        let orig_center = self.constrain_with_height(origin, eye, world);
        let dropping = (orig_center - surface_pos).dot(up) > DROP_LEDGE_MARGIN;
        let ref_floor = if dropping { orig_center } else { surface_pos };
        let chest_reach = self.chest_reach(origin, dir, orig_center, surface_pos, raw_depth, up, eye, world);
        let (ahead, front) = (candidate + dir * (chest_reach * 0.5), candidate + dir * chest_reach);
        let (l_off, r_off) = (-right * lateral_span, right * lateral_span);
        let l_hit = self.post_obstructed(ahead + l_off, ref_floor, orig_center, up, eye, limit, world)
            || self.post_obstructed(front + l_off, ref_floor, orig_center, up, eye, limit, world);
        let r_hit = self.post_obstructed(ahead + r_off, ref_floor, orig_center, up, eye, limit, world)
            || self.post_obstructed(front + r_off, ref_floor, orig_center, up, eye, limit, world);
        let orig_l = (self.constrain_with_height(origin + l_off, eye, world) - ref_floor).dot(up) > POST_WALL_RISE;
        let orig_r = (self.constrain_with_height(origin + r_off, eye, world) - ref_floor).dot(up) > POST_WALL_RISE;
        if (l_hit && r_hit)
            || (l_hit && (!orig_l || self.post_obstructed(ahead + right * (lateral_span * FLANK_DOORPOST_RATIO), ref_floor, orig_center, up, eye, limit, world)))
            || (r_hit && (!orig_r || self.post_obstructed(ahead - right * (lateral_span * FLANK_DOORPOST_RATIO), ref_floor, orig_center, up, eye, limit, world)))
        {
            return false;
        }
        let mid_surface = self.constrain_with_height(ahead, eye, world);
        let probe_surface = self.constrain_with_height(front, eye, world);
        let cand_back = self.constrain_with_height(candidate - dir * raw_depth, eye, world);
        if (probe_surface - ref_floor).dot(up) > POST_WALL_RISE && (cand_back - ref_floor).dot(up) > POST_WALL_RISE {
            return false;
        }
        if up.dot(world.surface_up_at(probe_surface)) < 0.7 {
            return true;
        }
        let mid_ref = if dropping && (ref_floor - mid_surface).dot(up) > 0.0 { ref_floor } else { mid_surface };
        SurfaceStep::new(ref_floor, mid_surface, up).walkable_with(limit)
            && SurfaceStep::new(mid_ref, probe_surface, up).walkable_with(limit)
            && (probe_surface - ref_floor).dot(up) < self.height
    }

    fn unobstructed_aerial(&self, origin: Vec3, candidate: Vec3, tangent: Vec3, up: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> bool {
        if tangent.length_squared() <= 1e-6 {
            return true;
        }
        let candidate_surf = self.constrain_with_height(candidate, eye, world);
        let orig_center = self.constrain_with_height(origin, eye, world);
        let on_slope = (origin - orig_center).dot(up) >= -0.01 && SurfaceStep::new(orig_center, candidate_surf, up).walkable_with(limit);
        let base = if on_slope && (candidate_surf - candidate).dot(up) > 0.0 { candidate_surf } else { candidate };
        let dir = tangent.normalize();
        let right = dir.cross(up).normalize_or_zero();
        let (raw_depth, lateral_span) = self.oriented_extents(dir, right, up);
        let chest_reach = self.chest_reach(origin, dir, orig_center, candidate_surf, raw_depth, up, eye, world);
        let (ahead, probe) = (base + dir * (chest_reach * 0.5), base + dir * chest_reach);
        let (l_off, r_off) = (-right * lateral_span, right * lateral_span);
        let l_hit = self.post_obstructed(ahead + l_off, candidate, orig_center, up, eye, limit, world);
        let r_hit = self.post_obstructed(ahead + r_off, candidate, orig_center, up, eye, limit, world);
        let orig_l = (self.constrain_with_height(origin + l_off, eye, world) - origin).dot(up) > GROUND_SETTLE_OFFSET;
        let orig_r = (self.constrain_with_height(origin + r_off, eye, world) - origin).dot(up) > GROUND_SETTLE_OFFSET;
        if (l_hit && r_hit && !(orig_l && orig_r)) || (l_hit && !orig_l) || (r_hit && !orig_r) {
            return false;
        }
        let mid_surface = self.constrain_with_height(ahead, eye, world);
        let probe_surface = self.constrain_with_height(probe, eye, world);
        let in_deep_shaft = (origin - orig_center).dot(up) >= DEEP_SHAFT_SPAN;
        let over_pit = (origin - orig_center).dot(up) > 0.02 && !in_deep_shaft;
        let step_ref = if in_deep_shaft { origin } else { orig_center };
        if (candidate - origin).dot(up) <= 0.0
            && (((origin - probe_surface).dot(up) < if over_pit { PIT_RIM_DROP } else { -1e-4 }
                && !SurfaceStep::new(step_ref, probe_surface, up).walkable_with(limit))
                || (over_pit
                    && (origin - mid_surface).dot(up) < PIT_RIM_DROP
                    && !SurfaceStep::new(step_ref, mid_surface, up).walkable_with(limit)))
        {
            return false;
        }
        let clearance = SurfaceClearance::between(base, probe, probe_surface, up, world);
        if !clearance.cliff_impact(limit) {
            return true;
        }
        SurfaceStep::new(candidate_surf, mid_surface, up).walkable_with(limit)
            && SurfaceStep::new(mid_surface, probe_surface, up).walkable_with(limit)
    }

    fn wedged_in_shaft(&self, position: Vec3, up: Vec3, eye: f32, world: &dyn PlanetaryWorld) -> bool {
        if world.weightless_at(position) || (world.constrain_to_surface(position, 0.0) - position).dot(up) > POST_WALL_RISE {
            return false;
        }
        let foot_pos = self.constrain_with_height(position, eye, world) - up * eye;
        let base = if (position - foot_pos).dot(up) < eye + SHAFT_FOOT_SNAP { foot_pos } else { position - up * eye };
        let ref_vec = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let axis_a = up.cross(ref_vec).normalize();
        let axis_b = up.cross(axis_a).normalize();
        self.axis_wedged(base, axis_a, up, world) || self.axis_wedged(base, axis_b, up, world)
    }
}
