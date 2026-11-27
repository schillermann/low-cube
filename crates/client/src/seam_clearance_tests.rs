use glam::Vec3;
use world::{BiomedQuadSphere, ClimateZone, GravityField, PlanetaryWorld};
use crate::avatar::{AvatarMotor as _, AvatarOptics as _, PlanetaryAvatar};
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
fn test_quad_sphere_meridian_clearance_is_smooth_planar_ground() {
    let world = BiomedQuadSphere::of_size(32);
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let origin = frame.point_at(15.95, 0.0, 1.0);
    let candidate = frame.point_at(16.05, 0.0, 1.0);
    let surface_pos = world.constrain_to_surface(candidate, 1.0);
    let surface_up = world.surface_up_at(origin);

    let clearance = SurfaceClearance::between(origin, candidate, surface_pos, surface_up, &world);
    assert!(clearance.passable_with(true));
    assert!(clearance.passable_with(false));
    assert!(clearance.floor_contact(StepThreshold::human_stride()));
    assert!(!clearance.cliff_impact(StepThreshold::human_stride()));
}

#[test]
fn test_continuous_ground_meridian_traversal_without_pushback() {
    let world = BiomedQuadSphere::of_size(32);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let mut pos = world.constrain_to_surface(frame.point_at(15.6, 0.0, 1.0), 1.0);
    let gait = WalkGait::upright();
    let dt = 1.0 / 60.0;
    let stride_len = 4.3 * dt;

    for _ in 0..25 {
        let up = world.surface_up_at(pos);
        let tangent = up.cross(Vec3::Z).normalize();
        pos = flight.advance(dt, pos, tangent * stride_len, &gait, up, &world);
    }

    assert_eq!(ClimateZone::dominant(pos), ClimateZone::Desert);
    assert!((pos.length() - 17.0).abs() < 0.05, "Avatar must stay at r=17.0 on QuadSphere, got {}", pos.length());
}

#[test]
fn test_avatar_step_motion_continuous_across_meridian() {
    let world = BiomedQuadSphere::of_size(32);
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let start_pos = world.constrain_to_surface(frame.point_at(15.5, 0.0, 1.0), 1.0);
    let mut avatar = PlanetaryAvatar::with_flight(start_pos, Box::new(SurfaceFlight::terrestrial(1.0)));
    let controls = ContinuousForwardControls;

    avatar.rotate_view(-std::f32::consts::FRAC_PI_2 / 0.003, 0.0);

    let dt = 1.0 / 60.0;
    for _ in 0..60 {
        avatar.step_motion(dt, &controls, &world);
    }

    let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
    for col in 0..4 {
        for row in 0..4 {
            assert!(view_proj.col(col)[row].is_finite());
        }
    }
}

#[test]
fn test_sneak_at_voxel_ledge_prevents_drop_while_meridian_remains_walkable() {
    use world::WorldEarthworks as _;
    let mut world = BiomedQuadSphere::barren(32);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let pillar_eye = frame.point_at(2.25, 0.25, 2.0);
    let av_far = frame.point_at(0.25, 0.25, 0.0);
    assert!(world.place_molded_surface_block(pillar_eye, -frame.up_at(pillar_eye), av_far, world::BlockMold::cube()));
    assert!(world.place_molded_surface_block(frame.point_at(2.25, 0.25, 2.5), -frame.up_at(pillar_eye), av_far, world::BlockMold::cube()));

    let start_pos = world.constrain_to_surface(frame.point_at(2.45, 0.25, 3.0), 1.0);
    let up = world.surface_up_at(start_pos);
    let east = (Vec3::X - up * Vec3::X.dot(up)).normalize();
    let next_pos = flight.advance(1.0 / 60.0, start_pos, east * 0.15, &SneakGait::crouching(), up, &world);
    assert!((next_pos - start_pos).length() < 1e-4, "Sneaking must arrest at a 1.0m voxel ledge on QuadSphere");
}

#[test]
fn test_grazing_sprint_jump_along_climate_meridian_never_falls_through_surface() {
    use winit::keyboard::KeyCode;
    use crate::controls::KeyMotion;

    let world = BiomedQuadSphere::barren(16);
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    for yaw_px in [-12.0f32, -35.0, -90.0, -240.0] {
        for start_u in [7.75f32, 7.80, 7.85, 7.92] {
            let start_pos = world.constrain_to_surface(frame.point_at(start_u, 0.25, 2.0), 1.0);
            let mut avatar = PlanetaryAvatar::at(start_pos);
            avatar.rotate_view(yaw_px, 0.0);
            let mut controls = KeyMotion::idle();
            controls.register_key(KeyCode::KeyW, true);
            controls.register_key(KeyCode::ShiftLeft, true);
            controls.register_key(KeyCode::Space, true);
            for tick in 0..100 {
                if tick == 1 {
                    controls.register_key(KeyCode::Space, false);
                }
                avatar.step_motion(1.0 / 60.0, &controls, &world);
                let eye = avatar.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
                assert!(
                    eye.length() >= 8.5,
                    "Grazing sprint-jump at yaw_px={yaw_px}, start_u={start_u}, tick={tick} must never sink into QuadSphere, got eye={eye:?} (r={})",
                    eye.length()
                );
            }
        }
    }
}
