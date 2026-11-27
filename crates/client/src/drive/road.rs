use glam::Vec3;
use world::{ClimateZone, PlanetaryWorld};

pub(super) const SURFACE_PROBE_LIFT: f32 = 1.15;

/// Focused interface for wheeled ground-step resolution across spherical `QuadSphere` terrain, slopes, and walls.
pub(super) trait RoadProbe {
    fn destination(&self, step: Vec3) -> Option<Vec3>;
}

/// Base `RoadProbe` projecting a tangential step radially onto the spherical `QuadSphere` crust.
pub(super) struct SeamRoad<'a> {
    origin: Vec3,
    world: &'a dyn PlanetaryWorld,
}

impl<'a> SeamRoad<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: Vec3, world: &'a dyn PlanetaryWorld) -> Self {
        Self { origin, world }
    }
}

impl RoadProbe for SeamRoad<'_> {
    fn destination(&self, step: Vec3) -> Option<Vec3> {
        if step.length_squared() <= 1e-8 {
            return None;
        }
        let candidate = self.origin + step;
        let up = self.world.surface_up_at(candidate);
        Some(self.world.constrain_to_surface(candidate + up * SURFACE_PROBE_LIFT, 0.0))
    }
}

/// `RoadProbe` decorator rejecting vertical cliff rises above `climb_limit` relative to local radial gravity (`4` fields).
pub(super) struct GradedRoad<'a, P> {
    origin: P,
    world: &'a dyn PlanetaryWorld,
    floor: Vec3,
    climb_limit: f32,
}

impl<'a, P> GradedRoad<'a, P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: P, world: &'a dyn PlanetaryWorld, floor: Vec3, climb_limit: f32) -> Self {
        Self { origin, world, floor, climb_limit }
    }
}

impl<P: RoadProbe> RoadProbe for GradedRoad<'_, P> {
    fn destination(&self, step: Vec3) -> Option<Vec3> {
        let dest = self.origin.destination(step)?;
        let u0 = self.world.surface_up_at(self.floor);
        let u1 = self.world.surface_up_at(dest);
        let rise = (dest - self.floor).dot(u0 + u1) / (1.0 + u0.dot(u1)).max(0.25);
        (rise <= self.climb_limit).then_some(dest)
    }
}

/// `RoadProbe` decorator ensuring the vehicle's leading front bumper clears walls before advancing.
pub(super) struct BumperRoad<P> {
    origin: P,
    anchor: (Vec3, ClimateZone),
    lead_reach: f32,
}

impl<P> BumperRoad<P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: P, anchor: (Vec3, ClimateZone), lead_reach: f32) -> Self {
        Self { origin, anchor, lead_reach }
    }
}

impl<P: RoadProbe> RoadProbe for BumperRoad<P> {
    fn destination(&self, step: Vec3) -> Option<Vec3> {
        let dest = self.origin.destination(step)?;
        let (start, src_zone) = self.anchor;
        if ClimateZone::dominant(dest) != src_zone {
            return Some(dest);
        }
        let bumper_step = step + step.normalize_or_zero() * self.lead_reach;
        let cand_bumper = start + bumper_step;
        let radius = start.length().max(4.0);
        let frame = src_zone.surface_frame(radius);
        let (bu, bv) = frame.decompose(cand_bumper);
        if bu.abs() >= radius - 0.05 || bv.abs() >= radius - 0.05 {
            return Some(dest);
        }
        let _ = self.origin.destination(bumper_step)?;
        Some(dest)
    }
}

/// `RoadProbe` decorator sliding tangentially along local basis axes when an angled wall blocks direct motion.
pub(super) struct SlidingRoad<P> {
    origin: P,
    face_up: Vec3,
}

impl<P> SlidingRoad<P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: P, face_up: Vec3) -> Self {
        Self { origin, face_up }
    }

    fn axial_pair(&self, step: Vec3) -> (Vec3, Vec3) {
        let ref_ax = if self.face_up.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
        let tang = ref_ax.cross(self.face_up).normalize_or_zero();
        let bitan = self.face_up.cross(tang).normalize_or_zero();
        let a = tang * step.dot(tang);
        let b = bitan * step.dot(bitan);
        if a.length_squared() >= b.length_squared() { (a, b) } else { (b, a) }
    }
}

impl<P: RoadProbe> RoadProbe for SlidingRoad<P> {
    fn destination(&self, step: Vec3) -> Option<Vec3> {
        if let Some(direct) = self.origin.destination(step) {
            return Some(direct);
        }
        let (first, second) = self.axial_pair(step);
        self.origin.destination(first).or_else(|| self.origin.destination(second))
    }
}
