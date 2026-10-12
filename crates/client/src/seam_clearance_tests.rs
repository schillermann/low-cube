use glam::Vec3;
use world::{BiomedCubeWorld, GravityField, PlanetaryWorld};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::gait::WalkGait;
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
    assert!(!clearance.precipice());
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
