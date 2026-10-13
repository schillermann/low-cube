use glam::Vec3;
use world::{BiomedCubeWorld, GravityField, PlanetaryWorld};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::gait::{SneakGait, WalkGait};
use crate::threshold::{StepThreshold, SurfaceClearance};

struct ContinuousForwardControls;

impl KeyboardControls for ContinuousForwardControls {
    fn register_key(&mut self, _code: winit::keyboard::KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(forward);
    }
}

#[test]
fn test_seam_clearance_identifies_90_degree_fold() {
    let world = BiomedCubeWorld::beveled(32, 0.5);
    let origin = Vec3::new(15.95, 17.0, 0.0);
    let step = Vec3::new(0.1, 0.0, 0.0);
    let candidate = origin + step;
    let surface_pos = world.constrain_to_surface(candidate, 1.0);

    let clearance = SurfaceClearance::between(origin, candidate, surface_pos, Vec3::Y, &world);
    assert!(clearance.passable_with(true));
    assert!(!clearance.passable_with(false));
    assert!(clearance.floor_contact(StepThreshold::human_stride()));
    assert!(!clearance.cliff_impact(StepThreshold::human_stride()));
}

#[test]
fn test_continuous_ground_edge_traversal_without_pushback() {
    let world = BiomedCubeWorld::beveled(32, 0.5);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // Start on Top face (+Y, normal Vec3::Y) near the +X edge (edge is at x = 16.0)
    let mut pos = Vec3::new(15.9, 17.0, 0.0);
    let gait = WalkGait::upright();
    let dt = 1.0 / 60.0;
    let speed = 4.3; // m/s
    let stride_len = speed * dt;

    // Advance 10 steps forward across the edge onto the Right face (+X)
    for _ in 0..10 {
        let surface_up = world.surface_up_at(pos);
        // For the first step across edge, stride is along +X
        let stride = if pos.x < 16.0 {
            Vec3::new(stride_len, 0.0, 0.0)
        } else {
            // Once on Right face, forward motion is down along -Y
            Vec3::new(0.0, -stride_len, 0.0)
        };
        pos = flight.advance(dt, pos, stride, &gait, surface_up, &world);
    }

    // Pos must now be on Right face (x = 17.0, y < 16.0) without being snapped back
    assert!(pos.x > 16.5);
    assert!(pos.y < 16.0);
}

#[test]
fn test_avatar_step_motion_continuous_across_seam() {
    let world = BiomedCubeWorld::beveled(32, 0.5);
    let start_pos = Vec3::new(15.8, 17.0, 0.0);
    let mut avatar = PlanetaryAvatar::with_flight(start_pos, Box::new(SurfaceFlight::terrestrial(1.0)));
    let controls = ContinuousForwardControls;

    // Face towards +X (yaw = -90 deg)
    avatar.rotate_view(-std::f32::consts::FRAC_PI_2, 0.0);

    let dt = 1.0 / 60.0;
    // Step forward 60 frames (1 second of walking)
    for _ in 0..60 {
        avatar.step_motion(dt, &controls, &world);
    }

    // View matrix must be valid and finite (no NaN, no singularity)
    let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
    for col in 0..4 {
        for row in 0..4 {
            assert!(view_proj.col(col)[row].is_finite());
        }
    }
}

#[test]
fn test_sneak_at_seam_prevents_edge_crossing() {
    let world = BiomedCubeWorld::beveled(32, 0.5);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // At x = 15.95 on top face (+Y, normal Vec3::Y), 5cm away from edge at x = 16.0
    let start_pos = Vec3::new(15.95, 17.0, 0.0);
    let stride = Vec3::new(0.1, 0.0, 0.0);

    // Sneaking gait must stop at the planetary cube edge and prevent crossing onto +X face
    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &SneakGait::crouching(), Vec3::Y, &world);

    assert_eq!(next_pos.x, start_pos.x);
    assert_eq!(next_pos.y, start_pos.y);
    assert_eq!(next_pos.z, start_pos.z);
}

#[test]
fn test_sneak_at_seam_diagonal_slides_along_edge() {
    let world = BiomedCubeWorld::beveled(32, 0.5);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let start_pos = Vec3::new(15.95, 17.0, 0.0);
    // Moving diagonally: +X (towards seam) and +Z (along seam)
    let stride = Vec3::new(0.1, 0.0, 0.1);

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &SneakGait::crouching(), Vec3::Y, &world);

    // X motion over edge is arrested, Z motion parallel to edge proceeds safely
    assert_eq!(next_pos.x, start_pos.x);
    assert!(next_pos.z > start_pos.z);
    assert_eq!(next_pos.y, start_pos.y);
}
