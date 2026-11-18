use glam::Vec3;
use world::{ClimateZone, PlanetaryWorld};

pub(super) const SURFACE_PROBE_LIFT: f32 = 1.15;

/// Focused interface for wheeled ground-step resolution across planar faces, slopes, walls, and 90° cube seams.
pub(super) trait RoadProbe {
    fn destination(&self, step: Vec3) -> Option<Vec3>;
}

/// Base `RoadProbe` projecting a tangential step onto its source face and folding 90° cube-seam crossings onto the adjacent face.
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
        let src_zone = ClimateZone::dominant(self.origin);
        let src_normal = src_zone.normal();
        let candidate = self.origin + step;
        let lift = (candidate.abs().max_element() - candidate.dot(src_normal)).max(0.0) + SURFACE_PROBE_LIFT;
        let folded = self.world.constrain_to_surface(candidate + src_normal * lift, 0.0);
        let delta = folded - candidate;
        let tan_shift = delta - src_normal * delta.dot(src_normal);
        let crossed_seam = tan_shift.length_squared() > 1e-5 || ClimateZone::dominant(folded) != src_zone;
        if !crossed_seam {
            return Some(folded);
        }
        let dst_zone = if ClimateZone::dominant(folded) != src_zone {
            ClimateZone::dominant(folded)
        } else {
            ClimateZone::dominant(folded + step.normalize_or_zero())
        };
        let dst_normal = dst_zone.normal();
        let dst_lift = (folded.abs().max_element() - folded.dot(dst_normal)).max(0.0) + SURFACE_PROBE_LIFT;
        let dst_ground = self.world.constrain_to_surface(folded + dst_normal * dst_lift, 0.0);
        ((dst_ground - folded).dot(dst_normal) <= 0.62).then_some(dst_ground)
    }
}

/// `RoadProbe` decorator rejecting vertical cliff rises above `climb_limit` on the source face while passing 90° seam folds.
pub(super) struct GradedRoad<P> {
    origin: P,
    src_zone: ClimateZone,
    floor: Vec3,
    climb_limit: f32,
}

impl<P> GradedRoad<P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: P, src_zone: ClimateZone, floor: Vec3, climb_limit: f32) -> Self {
        Self { origin, src_zone, floor, climb_limit }
    }
}

impl<P: RoadProbe> RoadProbe for GradedRoad<P> {
    fn destination(&self, step: Vec3) -> Option<Vec3> {
        let dest = self.origin.destination(step)?;
        if ClimateZone::dominant(dest) != self.src_zone {
            return Some(dest);
        }
        ((dest - self.floor).dot(self.src_zone.normal()) <= self.climb_limit).then_some(dest)
    }
}

/// `RoadProbe` decorator ensuring the vehicle's leading front bumper clears same-face walls before advancing.
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
        let frame = src_zone.surface_frame(0.0);
        let half = start.dot(src_zone.normal()).abs();
        let (bu, bv) = frame.decompose(cand_bumper);
        if bu.abs() >= half - 0.05 || bv.abs() >= half - 0.05 {
            return Some(dest);
        }
        let bumper_dest = self.origin.destination(bumper_step)?;
        let _ = bumper_dest;
        Some(dest)
    }
}

/// `RoadProbe` decorator sliding tangentially along cardinal axes when an angled wall blocks direct motion.
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
        let n = self.face_up.abs();
        let (a, b) = if n.y >= n.x && n.y >= n.z {
            (Vec3::new(step.x, 0.0, 0.0), Vec3::new(0.0, 0.0, step.z))
        } else if n.x >= n.y && n.x >= n.z {
            (Vec3::new(0.0, step.y, 0.0), Vec3::new(0.0, 0.0, step.z))
        } else {
            (Vec3::new(step.x, 0.0, 0.0), Vec3::new(0.0, step.y, 0.0))
        };
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
