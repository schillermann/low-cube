use glam::{Quat, Vec3};
use world::{ClimateZone, PlanetaryWorld, VehicleAssembly};
use super::road::{BumperRoad, GradedRoad, RoadProbe, SeamRoad, SlidingRoad, SURFACE_PROBE_LIFT};

const CHASSIS_CLIMB_LIMIT: f32 = 0.62;
const BUMPER_LEAD_REACH: f32 = 0.42;

/// Encapsulated surface-normal ground contact probe on a specific planetary cube face.
#[derive(Clone, Copy, Debug)]
pub(super) struct FaceContact {
    point: Vec3,
}

impl FaceContact {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(point: Vec3) -> Self {
        Self { point }
    }

    pub(super) fn ground(self, clearance: f32, world: &dyn PlanetaryWorld) -> Vec3 {
        let face_up = ClimateZone::dominant(self.point).normal();
        let lift = (self.point.abs().max_element() - self.point.dot(face_up)).max(0.0) + SURFACE_PROBE_LIFT;
        world.constrain_to_surface(self.point + face_up * lift, clearance)
    }

    pub(super) fn seat_intact(self, world: &dyn PlanetaryWorld) -> bool {
        let face_up = ClimateZone::dominant(self.point).normal();
        (self.ground(0.9, world) - self.point).dot(face_up) > 1.15
    }

    pub(super) fn parked_seat_origin(self, world: &dyn PlanetaryWorld) -> Vec3 {
        let zone = ClimateZone::dominant(self.point);
        let face_up = zone.normal();
        let frame = zone.surface_frame(0.0);
        let (u0, v0) = frame.decompose(self.point);
        let (cu, cv) = (u0.div_euclid(0.5) as i32, v0.div_euclid(0.5) as i32);
        let center = self.point
            + frame.tangent() * ((cu as f32) * 0.5 + 0.25 - u0)
            + frame.bitangent() * ((cv as f32) * 0.5 + 0.25 - v0);
        let mut best = center;
        let mut best_h = (Self::new(center).ground(0.9, world) - center).dot(face_up);
        if best_h > 1.15 {
            return best;
        }
        for du in -2..=2 {
            for dv in -2..=2 {
                let cand = center + frame.tangent() * ((du as f32) * 0.5) + frame.bitangent() * ((dv as f32) * 0.5);
                if ClimateZone::dominant(cand) == zone {
                    let h = (Self::new(cand).ground(0.9, world) - cand).dot(face_up);
                    if h > best_h + 0.05 {
                        best_h = h;
                        best = cand;
                    }
                }
            }
        }
        best
    }
}

/// Wheeled chassis surface contact composing the `SlidingRoad(BumperRoad(GradedRoad(SeamRoad)))` decorator chain.
#[derive(Clone, Copy, Debug)]
pub(super) struct WheelTrack {
    origin: Vec3,
    normal: Vec3,
    heading: Vec3,
    climb_limit: f32,
}

impl WheelTrack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: Vec3, normal: Vec3, heading: Vec3, climb_limit: f32) -> Self {
        Self { origin, normal, heading, climb_limit }
    }

    pub(super) const fn from_assembly(assembly: &VehicleAssembly) -> Self {
        Self::new(assembly.origin(), assembly.normal(), assembly.heading(), CHASSIS_CLIMB_LIMIT)
    }

    pub(super) fn advance(self, displacement: f32, dt: f32, world: &dyn PlanetaryWorld) -> (Vec3, Vec3, bool) {
        let src_zone = ClimateZone::dominant(self.origin);
        let src_normal = src_zone.normal();
        let prev_ground = FaceContact::new(self.origin).ground(0.0, world);
        let (next_ground, blocked) = if displacement.abs() <= 1e-6 {
            (prev_ground, false)
        } else {
            let cur_up = self.normal.normalize_or_zero();
            let unfolded = Quat::from_rotation_arc(cur_up, src_normal) * self.heading;
            let planar_dir = (unfolded - src_normal * unfolded.dot(src_normal)).normalize_or_zero();
            let road = SlidingRoad::new(
                BumperRoad::new(
                    GradedRoad::new(SeamRoad::new(self.origin, world), src_zone, prev_ground, self.climb_limit),
                    (self.origin, src_zone),
                    BUMPER_LEAD_REACH,
                ),
                src_normal,
            );
            match road.destination(planar_dir * displacement) {
                Some(dest) => (dest, false),
                None => (prev_ground, true),
            }
        };
        let target_up = world.surface_up_at(next_ground);
        let cur_up = self.normal.normalize_or_zero();
        let next_up = if cur_up.dot(target_up) >= 0.96 || cur_up.dot(target_up) <= -0.95 {
            target_up
        } else {
            let arc = Quat::from_rotation_arc(cur_up, target_up);
            (Quat::IDENTITY.slerp(arc, (20.0 * dt).clamp(0.0, 1.0)) * cur_up).normalize_or_zero()
        };
        (next_ground, next_up, blocked)
    }
}
