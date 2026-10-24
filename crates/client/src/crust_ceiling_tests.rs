use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, MeshTarget, PlanetaryWorld, Quad};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyMotion, KeyboardControls};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::gait::WalkGait;

struct SpanCapture {
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
}

impl SpanCapture {
    fn empty() -> Self {
        Self { min_y: f32::MAX, max_y: f32::MIN, min_z: f32::MAX, max_z: f32::MIN }
    }
}

impl MeshTarget for SpanCapture {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, v1, v2, v3, _, _| {
            for v in [v0, v1, v2, v3] {
                self.min_y = self.min_y.min(v.y);
                self.max_y = self.max_y.max(v.y);
                self.min_z = self.min_z.min(v.z);
                self.max_z = self.max_z.max(v.z);
            }
        });
    }
}

#[test]
fn test_jumping_beneath_the_round_crust_never_peeks_through_the_vault_ceiling() {
    let world = BiomedCubeWorld::beveled(64, 1.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let mut pos = Vec3::new(0.25, 13.4, 0.25);
    assert!(world.weightless_at(pos) && !world.lethal_at(pos), "Start floating inside the halved hollow world beneath the 4-layer crust");
    flight.jump();
    for frame in 0..240 {
        pos = flight.advance(1.0 / 60.0, pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);
        assert!(pos.length() <= 14.0 - 0.25, "Frame {frame}: eye must stay beneath the radius-14 halved crust vault, got {pos:?}");
    }
    assert!(pos.y > 13.6, "Jump must carry the avatar up against the vault ceiling, got {pos:?}");

    let mut avatar = PlanetaryAvatar::at(Vec3::new(2.25, 13.0, 2.25));
    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::Space, true);
    controls.register_key(KeyCode::KeyW, true);
    for frame in 0..180 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
        let eye = avatar.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
        assert!(eye.length() <= 13.82, "Frame {frame}: camera eye must never poke through the beveled crust ceiling, got {eye:?} (r={})", eye.length());
    }
    let mut span = SpanCapture::empty();
    avatar.emit_appearance(&mut span);
    assert!(span.max_z - span.min_z > (span.max_y - span.min_y) * 1.4, "Weightless avatar must float horizontally (z_span={} vs y_span={})", span.max_z - span.min_z, span.max_y - span.min_y);
}

#[test]
fn test_repeated_head_bumps_against_crust_never_catapult_avatar_to_surface() {
    let mut world = BiomedCubeWorld::beveled(64, 1.0);
    for _ in 0..37 {
        let floor_y = world.constrain_to_surface(Vec3::new(0.25, 33.0, 0.25), 0.0).y;
        assert!(world.excavate_surface(Vec3::new(0.25, floor_y + 1.0, 0.25), Vec3::NEG_Y));
    }
    let mut avatar = PlanetaryAvatar::at(Vec3::new(1.25, 13.3, 1.25));
    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::KeyW, true);
    for frame in 0..360 {
        controls.register_key(KeyCode::Space, frame % 4 < 2);
        avatar.step_motion(1.0 / 60.0, &controls, &world);
        let eye = avatar.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
        assert!(eye.length() <= 13.82 && world.weightless_at(eye), "Frame {frame}: repeated head bumps must never catapult avatar to outer surface, got {eye:?}");
    }
}

#[test]
fn test_digging_upward_from_beneath_crust_removes_block_breaches_exit_to_surface_and_magma_core_kills() {
    use crate::attire::{HeldRelic, YouthAttire};
    let mut world = BiomedCubeWorld::beveled(64, 1.0);
    let mut avatar = PlanetaryAvatar::at(Vec3::new(1.25, 12.2, 1.25));
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::ExcavatedEarth));
    avatar.rotate_view(0.0, -2000.0);
    assert!(!avatar.use_right_hand(&mut world));
    assert!(avatar.use_right_hand(&mut world), "Digging under the crust must remove the ceiling block and breach to the surface");
    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::Space, true);
    for _ in 0..480 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let exit_eye = avatar.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    assert!(exit_eye.y >= 32.5 && !world.weightless_at(exit_eye), "Avatar must dig upward through crust and emerge onto outer surface, got {exit_eye:?}");

    let mut doomed = PlanetaryAvatar::at(Vec3::new(0.25, 8.8, 0.25));
    doomed.step_motion(1.0 / 60.0, &controls, &world);
    let dead_eye = doomed.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    assert!(dead_eye.y < 10.0 && world.lethal_at(Vec3::new(0.25, 8.8, 0.25)), "Avatar near magma core must die in place without surface teleport, got {dead_eye:?}");
    let mut hud = SpanCapture::empty();
    doomed.emit_reticle(&mut hud);
    assert!(hud.max_y - hud.min_y > 0.20, "Perished avatar must emit YOU DIED banner instead of small crosshair");
}

#[test]
fn test_neck_remains_anchored_to_torso_when_head_tilts_back_and_server_scale_validates() {
    use glam::Quat;
    use crate::attire::{BodyPose, StrideRhythm, YouthPhysique};
    use crate::server_scale::{ArgumentScale, FileScale, MoonletScale, PlanetaryScale, ScaledCube};

    let upright = BodyPose::with_hover_pose(Vec3::ZERO, Quat::IDENTITY, Quat::IDENTITY, 0.8, 0.0, 0.0, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let hovered = upright.with_hover(1.0);
    assert_eq!(hovered.neck_frame().point(0.0, 0.0, 0.0), hovered.torso_frame().point(0.0, hovered.torso_span(0.88), -0.056), "Neck frame must remain anchored to torso");
    assert_ne!(hovered.crown_frame().point(0.0, 0.15, 0.0), hovered.neck_frame().point(0.0, 0.15, 0.0), "Head must tilt back around the neck top in hover");

    assert_eq!(MoonletScale::new().multiple(), 1);
    assert_eq!(MoonletScale::new().blocks(), 128);
    assert_eq!(ScaledCube::parse("2", "test").blocks(), 256);
    let args = vec!["low-cube".to_string(), "--size=3".to_string()];
    assert_eq!(ArgumentScale::from_arguments(&args, Box::new(MoonletScale::new())).blocks(), 384);
    let tmp = std::env::temp_dir().join("test_server_scale.toml");
    std::fs::write(&tmp, "[world]\nsize = 4\n").unwrap();
    assert_eq!(FileScale::from_path(tmp.to_str().unwrap(), Box::new(MoonletScale::new())).blocks(), 512);
    let _ = std::fs::remove_file(tmp);
}

#[test]
#[should_panic(expected = "minimum allowed setting is 1")]
fn test_server_scale_rejects_factor_below_one() {
    crate::server_scale::ScaledCube::parse("0", "--size");
}
