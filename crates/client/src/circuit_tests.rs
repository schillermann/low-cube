use glam::Vec3;
use world::{BiomedCubeWorld, ClimateZone, GravityField};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::flight::SurfaceFlight;

struct ContinuousForwardControls;

impl KeyboardControls for ContinuousForwardControls {
    fn register_key(&mut self, _code: winit::keyboard::KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(forward);
    }
}

#[test]
fn test_avatar_completes_360_degree_planetary_circuit() {
    // 16-block planetary cube: half-size = 8.0, face side = 16.0m
    let world = BiomedCubeWorld::beveled(16, 0.5);
    let start_pos = Vec3::new(0.0, 9.0, 0.0); // Center of Temperate (+Y) face
    let mut avatar = PlanetaryAvatar::with_flight(start_pos, Box::new(SurfaceFlight::terrestrial(1.0)));
    let controls = ContinuousForwardControls;

    // Face towards -X (Arctic face): mouse sensitivity is 0.003, yaw = -dx * sensitivity
    avatar.rotate_view(-std::f32::consts::FRAC_PI_2 / 0.003, 0.0);

    let dt = 1.0 / 60.0;
    // Circuit length is 4 faces * 16.0m = 64.0m.
    // Traverses Temperate (+Y) -> Arctic (-X) -> Volcanic (-Y) -> Desert (+X) -> Temperate (+Y).
    let mut visited_arctic = false;
    let mut visited_volcanic = false;
    let mut visited_desert = false;
    let mut returned_to_temperate = false;

    for frame in 0..650 {
        avatar.step_motion(dt, &controls, &world);
        let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
        for col in 0..4 {
            for row in 0..4 {
                assert!(
                    view_proj.col(col)[row].is_finite(),
                    "View projection matrix element must be finite at frame {}",
                    frame
                );
            }
        }

        // Track sequential traversal through all 4 climate zones of the circuit
        let inv_view_proj = view_proj.inverse();
        let cam_pos = inv_view_proj.project_point3(Vec3::ZERO);
        let dominant = ClimateZone::dominant(cam_pos);

        match dominant {
            ClimateZone::Arctic => visited_arctic = true,
            ClimateZone::Volcanic => {
                if visited_arctic {
                    visited_volcanic = true;
                }
            }
            ClimateZone::Desert => {
                if visited_volcanic {
                    visited_desert = true;
                }
            }
            ClimateZone::Temperate => {
                if visited_desert {
                    returned_to_temperate = true;
                }
            }
            _ => {}
        }
    }

    assert!(visited_arctic, "Avatar must traverse onto Arctic (-X) face");
    assert!(visited_volcanic, "Avatar must traverse onto Volcanic (-Y) face");
    assert!(visited_desert, "Avatar must traverse onto Desert (+X) face");
    assert!(returned_to_temperate, "Avatar must complete 360° loop returning to Temperate (+Y) face");
}

#[test]
fn test_circuit_on_of_size_world() {
    let world = BiomedCubeWorld::of_size(32);
    let start_pos = Vec3::new(0.0, 17.0, 0.0);
    let mut avatar = PlanetaryAvatar::with_flight(start_pos, Box::new(SurfaceFlight::terrestrial(1.0)));
    let controls = ContinuousForwardControls;
    avatar.rotate_view(-std::f32::consts::FRAC_PI_2 / 0.003, 0.0);
    let mut visited_arctic = false;
    let mut visited_volcanic = false;
    let mut visited_desert = false;
    let mut returned_to_temperate = false;
    let dt = 1.0 / 60.0;

    for frame in 0..1300 {
        avatar.step_motion(dt, &controls, &world);
        let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
        let inv = view_proj.inverse();
        let cam_pos = inv.project_point3(Vec3::ZERO);

        for col in 0..4 {
            for row in 0..4 {
                assert!(view_proj.col(col)[row].is_finite());
            }
        }

        // Before reaching edge, view pitch is strictly level along the face
        if frame < 100 || (frame > 220 && frame < 400) {
            let fwd = (inv.project_point3(Vec3::new(0.0, 0.0, 1.0)) - cam_pos).normalize();
            let up = world.surface_up_at(cam_pos);
            let pitch_angle = fwd.dot(up).asin().to_degrees();
            assert!(
                pitch_angle.abs() < 0.1,
                "View pitch must remain level on face, got {:.2}° at frame {}",
                pitch_angle,
                frame
            );
        }

        let dominant = ClimateZone::dominant(cam_pos);
        match dominant {
            ClimateZone::Arctic => visited_arctic = true,
            ClimateZone::Volcanic => if visited_arctic { visited_volcanic = true; },
            ClimateZone::Desert => if visited_volcanic { visited_desert = true; },
            ClimateZone::Temperate => if visited_desert { returned_to_temperate = true; },
            _ => {}
        }
    }

    assert!(visited_arctic, "Avatar must traverse onto Arctic (-X) face");
    assert!(visited_volcanic, "Avatar must traverse onto Volcanic (-Y) face");
    assert!(visited_desert, "Avatar must traverse onto Desert (+X) face");
    assert!(returned_to_temperate, "Avatar must complete 360° circuit");
}
