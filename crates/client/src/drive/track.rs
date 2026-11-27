use glam::{Quat, Vec3};
use world::{ClimateZone, PlanetaryWorld, VehicleAssembly};
use super::road::{BumperRoad, GradedRoad, RoadProbe, SeamRoad, SlidingRoad, SURFACE_PROBE_LIFT};

const CHASSIS_CLIMB_LIMIT: f32 = 0.62;
const BUMPER_LEAD_REACH: f32 = 0.42;

/// Encapsulated radial surface-normal ground contact probe on a `QuadSphere` body (`1` field).
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
        let up = world.surface_up_at(self.point);
        world.constrain_to_surface(self.point + up * SURFACE_PROBE_LIFT, clearance)
    }

    pub(super) fn seat_intact(self, world: &dyn PlanetaryWorld) -> bool {
        let up = world.surface_up_at(self.point);
        if (self.point - self.ground(0.0, world)).dot(up) > 4.5 {
            return true;
        }
        (self.ground(0.9, world) - self.point).dot(up) > 1.15
    }
}

/// Wheeled chassis surface contact composing the `SlidingRoad(BumperRoad(GradedRoad(SeamRoad)))` decorator chain (`4` fields).
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
        let src_zone = ClimateZone::dominant(world.local_body_point(self.origin));
        let cur_up = world.surface_up_at(self.origin);
        let prev_ground = FaceContact::new(self.origin).ground(0.0, world);
        let (next_ground, blocked) = if displacement.abs() <= 1e-6 {
            (prev_ground, false)
        } else {
            let tangent_dir = (self.heading - cur_up * self.heading.dot(cur_up)).normalize_or_zero();
            let road = SlidingRoad::new(
                BumperRoad::new(
                    GradedRoad::new(SeamRoad::new(self.origin, world), world, prev_ground, self.climb_limit),
                    (self.origin, src_zone),
                    BUMPER_LEAD_REACH,
                ),
                cur_up,
            );
            match road.destination(tangent_dir * displacement) {
                Some(dest) => (dest, false),
                None => (prev_ground, true),
            }
        };
        let target_up = world.surface_up_at(next_ground);
        let prior_up = self.normal.normalize_or_zero();
        let next_up = if prior_up.dot(target_up) >= 0.96 || prior_up.dot(target_up) <= -0.95 {
            target_up
        } else {
            let arc = Quat::from_rotation_arc(prior_up, target_up);
            (Quat::IDENTITY.slerp(arc, (20.0 * dt).clamp(0.0, 1.0)) * prior_up).normalize_or_zero()
        };
        (next_ground, next_up, blocked)
    }
}
