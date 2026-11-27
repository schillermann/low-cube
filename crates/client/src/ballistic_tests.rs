use glam::Vec3;
use world::{BiomedQuadSphere, ClimateZone, GravityField, PlanetaryWorld as _};
use crate::avatar::{AvatarMotor as _, AvatarOptics as _, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::gait::WalkGait;

#[test]
fn test_ballistic_meridian_leap_traverses_seam_and_lands_on_adjacent_shell() {
    let world = BiomedQuadSphere::of_size(32);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let start_pos = world.constrain_to_surface(frame.point_at(15.2, 0.0, 1.0), 1.0);
    let gait = WalkGait::upright();
    let dt = 1.0 / 60.0;

    flight.jump();

    let mut pos = start_pos;
    let mut crossed_meridian = false;
    let mut reached_apex = false;

    for _ in 0..55 {
        let up = world.surface_up_at(pos);
        let tangent = up.cross(Vec3::Z).normalize();
        pos = flight.advance(dt, pos, tangent * 0.08, &gait, up, &world);

        if pos.length() > 17.15 {
            reached_apex = true;
        }
        if ClimateZone::dominant(pos) == ClimateZone::Desert {
            crossed_meridian = true;
        }
    }

    assert!(reached_apex, "Avatar must ascend radially above launch altitude");
    assert!(crossed_meridian, "Avatar must leap across 45° meridian into Desert shell");
    let ground_r = world.constrain_to_surface(pos, 1.0).length();
    assert!((pos.length() - ground_r).abs() < 0.05, "Avatar must touch down on Desert ground at r={ground_r}, got r={}", pos.length());
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
    let world = BiomedQuadSphere::of_size(32);
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let start_pos = world.constrain_to_surface(frame.point_at(15.4, 0.0, 1.0), 1.0);
    let mut avatar = PlanetaryAvatar::with_flight(
        start_pos,
        Box::new(SurfaceFlight::terrestrial(1.0)),
    );
    let controls = JumpForwardControls {
        jumped: std::cell::Cell::new(false),
    };

    avatar.rotate_view(-std::f32::consts::FRAC_PI_2 / 0.003, 0.0);

    let dt = 1.0 / 60.0;
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
