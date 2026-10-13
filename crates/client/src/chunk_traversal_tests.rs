use glam::Vec3;
use world::{PlanetaryWorld, VolumePlanet};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::flight::SurfaceFlight;

struct ForwardWalkControls;

impl KeyboardControls for ForwardWalkControls {
    fn register_key(&mut self, _code: winit::keyboard::KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(forward);
    }
}

struct JumpAndForwardControls;

impl KeyboardControls for JumpAndForwardControls {
    fn register_key(&mut self, _code: winit::keyboard::KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(forward);
        target.jump();
    }
}

#[test]
fn test_avatar_traversal_and_jump_on_volume_planet() {
    let side_blocks = 32u32;
    let world: Box<dyn PlanetaryWorld> = Box::new(VolumePlanet::of_size(side_blocks));

    // Spawn on +Y face above terrain
    let half_size = (side_blocks as f32) * 0.5;
    let spawn = world.constrain_to_surface(Vec3::new(0.0, half_size + 1.0, 0.0), 1.0);
    assert_eq!(spawn.y, half_size + 1.0);

    let flight = Box::new(SurfaceFlight::terrestrial(1.0));
    let mut avatar = PlanetaryAvatar::with_flight(spawn, flight);

    // Stride forward along surface
    let forward_controls = ForwardWalkControls;
    let dt = 1.0 / 60.0;
    for _ in 0..10 {
        avatar.step_motion(dt, &forward_controls, world.as_ref());
    }

    let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
    let cam_pos = view_proj.inverse().project_point3(Vec3::ZERO);
    assert!(cam_pos.is_finite());
    // Avatar eye is at surface foot height (17.0) + eye level (0.8) = 17.8
    assert!((cam_pos.y - (half_size + 1.0 + 0.8)).abs() < 1e-4);

    // Execute jump
    let jump_controls = JumpAndForwardControls;
    avatar.step_motion(dt, &jump_controls, world.as_ref());

    let jumped_proj = avatar.view_projection_matrix(16.0 / 9.0);
    let jumped_cam = jumped_proj.inverse().project_point3(Vec3::ZERO);
    assert!(jumped_cam.y > half_size + 1.8, "Jump must elevate avatar: {}", jumped_cam.y);
}

#[test]
fn test_avatar_crosses_edge_on_volume_planet() {
    let side_blocks = 32u32;
    let world: Box<dyn PlanetaryWorld> = Box::new(VolumePlanet::of_size(side_blocks));
    let start_pos = Vec3::new(15.8, 17.0, 0.0);
    let mut avatar = PlanetaryAvatar::with_flight(start_pos, Box::new(SurfaceFlight::terrestrial(1.0)));
    let controls = ForwardWalkControls;
    avatar.rotate_view(std::f32::consts::FRAC_PI_2 / 0.003, 0.0);
    let dt = 1.0 / 60.0;
    for _ in 0..60 {
        avatar.step_motion(dt, &controls, world.as_ref());
    }
    let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
    let cam = view_proj.inverse().project_point3(Vec3::ZERO);
    assert!(cam.x > 16.5, "Should have crossed onto +X face, but cam was at {:?}", cam);
    assert!(cam.y < 16.0, "Should have rotated onto +X face and descended, but cam was at {:?}", cam);
}

struct SneakForwardControls;

impl KeyboardControls for SneakForwardControls {
    fn register_key(&mut self, _code: winit::keyboard::KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.sneak();
        target.travel(forward);
    }
}

#[test]
fn test_sneak_at_edge_on_volume_planet() {
    let side_blocks = 32u32;
    let world: Box<dyn PlanetaryWorld> = Box::new(VolumePlanet::of_size(side_blocks));
    let start_pos = Vec3::new(15.95, 17.0, 0.0);
    let mut avatar = PlanetaryAvatar::with_flight(start_pos, Box::new(SurfaceFlight::terrestrial(1.0)));
    let controls = SneakForwardControls;
    avatar.rotate_view(std::f32::consts::FRAC_PI_2 / 0.003, 0.0);
    let dt = 1.0 / 60.0;
    for _ in 0..60 {
        avatar.step_motion(dt, &controls, world.as_ref());
    }
    let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
    let cam = view_proj.inverse().project_point3(Vec3::ZERO);
    // Avatar position is stopped at x = 15.95; cam.x contains eye offset
    assert!(cam.x < 16.15, "Sneak must prevent avatar from crossing edge: cam.x = {}", cam.x);
    assert!(cam.y > 17.0, "Sneak must maintain avatar on top face: cam.y = {}", cam.y);
}


