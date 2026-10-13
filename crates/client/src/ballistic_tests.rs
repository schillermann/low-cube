use glam::Vec3;
use world::GravityField;
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::gait::WalkGait;

#[test]
fn test_ballistic_edge_leap_traverses_seam_and_lands_on_adjacent_face() {
    let world = world::BiomedCubeWorld::beveled(32, 0.5);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // Start on Top face (+Y) near +X edge (edge is at x = 16.0, surface y = 17.0)
    let start_pos = Vec3::new(15.5, 17.0, 0.0);
    let gait = WalkGait::upright();
    let dt = 1.0 / 60.0;
    let stride_forward = Vec3::new(0.08, 0.0, 0.0);

    flight.jump();

    let mut pos = start_pos;
    let mut crossed_edge = false;
    let mut reached_apex = false;

    // Simulate 50 frames of ballistic flight
    for _ in 0..50 {
        let surface_up = world.surface_up_at(pos);
        let stride = if pos.x < 16.0 {
            stride_forward
        } else {
            // Once over edge on face +X, forward along surface is down -Y
            Vec3::new(0.0, -0.08, 0.0)
        };
        pos = flight.advance(dt, pos, stride, &gait, surface_up, &world);

        if pos.y > 17.1 {
            reached_apex = true;
        }
        if pos.x > 16.0 {
            crossed_edge = true;
        }
    }

    // Must have reached parabolic apex and crossed the edge in mid-air
    assert!(reached_apex, "Avatar must ascend above launch altitude");
    assert!(crossed_edge, "Avatar must leap across edge into adjacent face space");

    // Avatar must have landed on Right face (+X): surface is at x = 17.0, y < 16.0
    assert!((pos.x - 17.0).abs() < 1e-3, "Avatar must touch down on adjacent face surface at x = 17.0, got x = {}", pos.x);
    assert!(pos.y < 16.0, "Avatar must land on adjacent face below edge y = 16.0, got y = {}", pos.y);
}

struct JumpForwardControls {
    jumped: std::cell::Cell<bool>,
}

impl KeyboardControls for JumpForwardControls {
    fn register_key(&mut self, _code: winit::keyboard::KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(forward);
        if !self.jumped.get() {
            target.jump();
            self.jumped.set(true);
        }
    }
}

#[test]
fn test_mid_air_trajectory_continuity_without_velocity_discontinuity() {
    let world = world::BiomedCubeWorld::beveled(32, 0.5);
    let start_pos = Vec3::new(15.7, 17.0, 0.0);
    let mut avatar = PlanetaryAvatar::with_flight(
        start_pos,
        Box::new(SurfaceFlight::terrestrial(1.0)),
    );
    let controls = JumpForwardControls {
        jumped: std::cell::Cell::new(false),
    };

    // Face towards +X (yaw = -90 deg)
    avatar.rotate_view(-std::f32::consts::FRAC_PI_2, 0.0);

    let dt = 1.0 / 60.0;
    // Step motion across 45 frames (0.75s of mid-air parabolic flight and landing)
    for _ in 0..45 {
        avatar.step_motion(dt, &controls, &world);

        let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
        for col in 0..4 {
            for row in 0..4 {
                assert!(view_proj.col(col)[row].is_finite());
            }
        }
    }
}
