use glam::Vec3;
use world::PlanetaryWorld;
use crate::threshold::{StepThreshold, SurfaceClearance, SurfaceStep};

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
    half_width: f32, half_depth: f32, height: f32, facing: Vec3,
}

impl BodyEnvelope {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_dimensions(half_width: f32, half_depth: f32, height: f32, facing: Vec3) -> Self {
        Self { half_width, half_depth, height, facing }
    }
    pub fn youth() -> Self { Self::for_physique(crate::attire::YouthPhysique::SlenderSquire) }

    /// Secondary constructor deriving width (Breite), depth (Tiefe), and height (Höhe) from the character's YouthPhysique.
    pub fn for_physique(physique: crate::attire::YouthPhysique) -> Self {
        Self::with_dimensions((physique.upper_body_span() * 0.5).clamp(0.18, 0.55), (physique.body_depth() * 0.5).clamp(0.12, 0.34), physique.stature_height().clamp(1.15, 2.48), Vec3::ZERO)
    }

    pub fn with_crouch_and_facing(self, crouch_drop: f32, facing: Vec3) -> Self {
        Self::with_dimensions(self.half_width, self.half_depth, (self.height - crouch_drop.max(0.0)).max(0.95), facing)
    }

    pub fn confine_beneath_crust(&self, point: Vec3, origin: Vec3, up: Vec3, world: &dyn PlanetaryWorld) -> Vec3 {
        let head_off = if world.weightless_at(point) || world.weightless_at(origin) {
            Vec3::ZERO
        } else {
            up * (self.height - 0.95).clamp(0.36, 0.80)
        };
        world.confine_beneath_crust(point + head_off) - head_off
    }

    pub fn constrain_with_height(&self, point: Vec3, eye: f32, world: &dyn PlanetaryWorld) -> Vec3 {
        let (base_surf, face_up) = (world.constrain_to_surface(point, eye), world::ClimateZone::dominant(point).normal());
        let delta = base_surf - point;
        if (delta - face_up * delta.dot(face_up)).length_squared() > 1e-4 { return base_surf; }
        let span = if (self.height - 1.0).abs() < 2e-3 { 1.003 } else { self.height };
        world.constrain_to_surface(point + face_up * (span - eye), span) - face_up * (span - eye)
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

    pub fn overhead_obstructed(&self, position: Vec3, up: Vec3, eye: f32, world: &dyn PlanetaryWorld) -> bool {
        let crouched = Self::with_dimensions(self.half_width, self.half_depth, (self.height - 0.44).max(0.95), self.facing);
        let base_floor = crouched.constrain_with_height(position, eye, world);
        let f_tan = (self.facing - up * self.facing.dot(up)).normalize_or_zero();
        let fwd = if f_tan.length_squared() > 1e-4 { f_tan } else if up.y.abs() < 0.9 { up.cross(Vec3::Y).normalize() } else { Vec3::Z };
        let rgt = fwd.cross(up).normalize_or_zero();
        for offset in [Vec3::ZERO, fwd * (self.half_depth * 0.7), -fwd * (self.half_depth * 0.7), rgt * (self.half_width * 0.7), -rgt * (self.half_width * 0.7)] {
            let pt = position + offset;
            let low = crouched.constrain_with_height(pt, eye, world);
            let high = self.constrain_with_height(pt, eye, world);
            if (low - base_floor).dot(up).abs() <= 0.35 && (high - low).dot(up) > 0.35 {
                return true;
            }
        }
        false
    }

    fn wall_rise(base: Vec3, dir: Vec3, dist: f32, up: Vec3, world: &dyn PlanetaryWorld) -> Option<f32> {
        let surf = world.constrain_to_surface(base + dir * dist, 0.0);
        let rise = (surf - base).dot(up);
        if up.dot(world.surface_up_at(surf)) >= 0.7 && rise >= 1.25 { Some(rise) } else { None }
    }

    fn wall_distance(base: Vec3, dir: Vec3, up: Vec3, world: &dyn PlanetaryWorld) -> (f32, f32) {
        for step in 1..=24 {
            let d = (step as f32) * 0.05;
            if let Some(rise) = Self::wall_rise(base, dir, d, up, world) { return (d, rise); }
        }
        (2.0, 0.0)
    }

    fn axis_wedged(&self, base: Vec3, axis: Vec3, up: Vec3, world: &dyn PlanetaryWorld) -> bool {
        let ((d_pos, r_pos), (d_neg, r_neg)) = (Self::wall_distance(base, axis, up, world), Self::wall_distance(base, -axis, up, world));
        d_pos <= 1.2 && d_neg <= 1.2 && (d_pos + d_neg - 0.05) < self.half_width * 2.0 && ((r_pos - r_neg).abs() < 1.2 || r_pos.min(r_neg) > 2.2)
    }

    #[allow(clippy::too_many_arguments)]
    fn post_obstructed(&self, pt: Vec3, floor: Vec3, orig_floor: Vec3, up: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> bool {
        let s = self.constrain_with_height(pt, eye, world);
        up.dot(world.surface_up_at(s)) >= 0.7 && (s - floor).dot(up) > 0.50 && (s - orig_floor).dot(up) > 0.35 && !SurfaceStep::new(floor, s, up).walkable_with(limit)
    }
}

impl PhysicalEnvelope for BodyEnvelope {
    fn unobstructed_ground(&self, candidate: Vec3, surface_pos: Vec3, step: Vec3, up: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> bool {
        let tangent = step - up * step.dot(up);
        if tangent.length_squared() <= 1e-6 { return true; }
        let dir = tangent.normalize();
        let right = dir.cross(up).normalize_or_zero();
        let (raw_depth, raw_lat) = self.oriented_extents(dir, right, up);
        let origin = candidate - step;
        let sub = (world.constrain_to_surface(origin, 0.0) - origin).dot(up) > 2.2;
        let lateral_span = if sub { raw_lat.min(0.15) } else { raw_lat };
        let orig_center = self.constrain_with_height(origin, eye, world);
        let back_surf = self.constrain_with_height(origin - dir * 0.25, eye, world);
        let dropping = (orig_center - surface_pos).dot(up) > 0.1;
        let ref_floor = if dropping { orig_center } else { surface_pos };
        let chest_reach = if sub { raw_depth.min(0.15) } else if dropping || (back_surf - surface_pos).dot(up) > 0.45 { 0.18_f32.min(raw_depth) } else { raw_depth };
        let (ahead, front) = (candidate + dir * (chest_reach * 0.5), candidate + dir * chest_reach);
        let (l_off, r_off) = (-right * lateral_span, right * lateral_span);
        let l_hit = self.post_obstructed(ahead + l_off, ref_floor, orig_center, up, eye, limit, world) || self.post_obstructed(front + l_off, ref_floor, orig_center, up, eye, limit, world);
        let r_hit = self.post_obstructed(ahead + r_off, ref_floor, orig_center, up, eye, limit, world) || self.post_obstructed(front + r_off, ref_floor, orig_center, up, eye, limit, world);
        let orig_l = (self.constrain_with_height(origin + l_off, eye, world) - ref_floor).dot(up) > 0.50;
        let orig_r = (self.constrain_with_height(origin + r_off, eye, world) - ref_floor).dot(up) > 0.50;
        if (l_hit && r_hit)
            || (l_hit && (!orig_l || self.post_obstructed(ahead + right * (lateral_span * 1.35), ref_floor, orig_center, up, eye, limit, world)))
            || (r_hit && (!orig_r || self.post_obstructed(ahead - right * (lateral_span * 1.35), ref_floor, orig_center, up, eye, limit, world)))
        {
            return false;
        }
        let mid_surface = self.constrain_with_height(ahead, eye, world);
        let probe_surface = self.constrain_with_height(front, eye, world);
        let cand_back = self.constrain_with_height(candidate - dir * raw_depth, eye, world);
        if (probe_surface - ref_floor).dot(up) > 0.50 && (cand_back - ref_floor).dot(up) > 0.50 {
            return false;
        }
        if up.dot(world.surface_up_at(probe_surface)) < 0.7 { return true; }
        let mid_ref = if dropping && (ref_floor - mid_surface).dot(up) > 0.0 { ref_floor } else { mid_surface };
        SurfaceStep::new(ref_floor, mid_surface, up).walkable_with(limit)
            && SurfaceStep::new(mid_ref, probe_surface, up).walkable_with(limit)
            && (probe_surface - ref_floor).dot(up) < self.height
    }

    fn unobstructed_aerial(&self, origin: Vec3, candidate: Vec3, tangent: Vec3, up: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> bool {
        if tangent.length_squared() <= 1e-6 { return true; }
        let candidate_surf = self.constrain_with_height(candidate, eye, world);
        let base = if (candidate_surf - candidate).dot(up) > 0.0 { candidate_surf } else { candidate };
        let dir = tangent.normalize();
        let right = dir.cross(up).normalize_or_zero();
        let (raw_depth, raw_lat) = self.oriented_extents(dir, right, up);
        let sub = (world.constrain_to_surface(origin, 0.0) - origin).dot(up) > 2.2;
        let lateral_span = if sub { raw_lat.min(0.15) } else { raw_lat };
        let orig_center = self.constrain_with_height(origin, eye, world);
        let back_surf = self.constrain_with_height(origin - dir * 0.25, eye, world);
        let chest_reach = if sub { raw_depth.min(0.15) } else if (orig_center - candidate_surf).dot(up) > 0.1 || (back_surf - candidate_surf).dot(up) > 0.45 { 0.18_f32.min(raw_depth) } else { raw_depth };
        let (ahead, probe) = (base + dir * (chest_reach * 0.5), base + dir * chest_reach);
        let (l_off, r_off) = (-right * lateral_span, right * lateral_span);
        let l_hit = self.post_obstructed(ahead + l_off, candidate, orig_center, up, eye, limit, world);
        let r_hit = self.post_obstructed(ahead + r_off, candidate, orig_center, up, eye, limit, world);
        let orig_l = (self.constrain_with_height(origin + l_off, eye, world) - origin).dot(up) > 0.45;
        let orig_r = (self.constrain_with_height(origin + r_off, eye, world) - origin).dot(up) > 0.45;
        if (l_hit && r_hit && !(orig_l && orig_r)) || (l_hit && !orig_l) || (r_hit && !orig_r) {
            return false;
        }
        let mid_surface = self.constrain_with_height(ahead, eye, world);
        let probe_surface = self.constrain_with_height(probe, eye, world);
        let in_deep_shaft = (origin - orig_center).dot(up) >= 2.5;
        let over_pit = (origin - orig_center).dot(up) > 0.02 && !in_deep_shaft;
        let step_ref = if in_deep_shaft { origin } else { orig_center };
        if (candidate - origin).dot(up) <= 0.0
            && (((origin - probe_surface).dot(up) < if over_pit { 0.52 } else { -1e-4 } && !SurfaceStep::new(step_ref, probe_surface, up).walkable_with(limit))
                || (over_pit && (origin - mid_surface).dot(up) < 0.52 && !SurfaceStep::new(step_ref, mid_surface, up).walkable_with(limit)))
        {
            return false;
        }
        let clearance = SurfaceClearance::between(base, probe, probe_surface, up, world);
        if !clearance.cliff_impact(limit) { return true; }
        SurfaceStep::new(candidate_surf, mid_surface, up).walkable_with(limit)
            && SurfaceStep::new(mid_surface, probe_surface, up).walkable_with(limit)
    }

    fn wedged_in_shaft(&self, position: Vec3, up: Vec3, eye: f32, world: &dyn PlanetaryWorld) -> bool {
        if world.weightless_at(position) || (world.constrain_to_surface(position, 0.0) - position).dot(up) > 0.5 { return false; }
        let foot_pos = self.constrain_with_height(position, eye, world) - up * eye;
        let base = if (position - foot_pos).dot(up) < eye + 1.8 { foot_pos } else { position - up * eye };
        let ref_vec = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let axis_a = up.cross(ref_vec).normalize();
        let axis_b = up.cross(axis_a).normalize();
        Self::axis_wedged(self, base, axis_a, up, world) || Self::axis_wedged(self, base, axis_b, up, world)
    }
}
