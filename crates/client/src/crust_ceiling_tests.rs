use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, MeshTarget, PlanetaryWorld, Quad};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyMotion, KeyboardControls};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::gait::WalkGait;

struct SpanCapture { min_y: f32, max_y: f32, min_z: f32, max_z: f32 }

impl SpanCapture {
    fn empty() -> Self { Self { min_y: f32::MAX, max_y: f32::MIN, min_z: f32::MAX, max_z: f32::MIN } }
}

impl MeshTarget for SpanCapture {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, v1, v2, v3, _, _| {
            for v in [v0, v1, v2, v3] { (self.min_y, self.max_y, self.min_z, self.max_z) = (self.min_y.min(v.y), self.max_y.max(v.y), self.min_z.min(v.z), self.max_z.max(v.z)); }
        });
    }
}

#[test]
fn test_jumping_beneath_the_round_crust_never_peeks_through_the_vault_ceiling() {
    let world = BiomedCubeWorld::beveled(64, 1.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let mut pos = Vec3::new(0.25, 23.4, 0.25);
    assert!(world.weightless_at(pos) && !world.lethal_at(pos), "Start floating inside the hollow world beneath the 4-layer crust");
    flight.jump();
    for frame in 0..240 {
        pos = flight.advance(1.0 / 60.0, pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);
        assert!(pos.length() <= 24.0 - 0.25, "Frame {frame}: eye must stay beneath the radius-24 crust vault, got {pos:?}");
    }
    assert!(pos.y > 23.6, "Jump must carry the avatar up against the vault ceiling, got {pos:?}");

    let (mut avatar, mut controls) = (PlanetaryAvatar::at(Vec3::new(2.25, 23.0, 2.25)), KeyMotion::idle());
    controls.register_key(KeyCode::Space, true); controls.register_key(KeyCode::KeyW, true);
    for frame in 0..180 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
        let eye = avatar.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
        assert!(eye.length() <= 23.82, "Frame {frame}: camera eye must never poke through the beveled crust ceiling, got {eye:?} (r={})", eye.length());
    }
    let mut span = SpanCapture::empty();
    avatar.emit_appearance(&mut span);
    assert!(span.max_z - span.min_z > (span.max_y - span.min_y) * 1.4, "Weightless avatar must float horizontally (z_span={} vs y_span={})", span.max_z - span.min_z, span.max_y - span.min_y);
}

#[test]
fn test_repeated_head_bumps_against_crust_never_catapult_avatar_to_surface() {
    let mut world = BiomedCubeWorld::beveled(64, 1.0);
    for _ in 0..16 {
        let floor_y = world.constrain_to_surface(Vec3::new(0.25, 33.0, 0.25), 0.0).y;
        assert!(world.excavate_surface(Vec3::new(0.25, floor_y + 1.0, 0.25), Vec3::NEG_Y));
    }
    let (mut avatar, mut controls) = (PlanetaryAvatar::at(Vec3::new(1.25, 23.3, 1.25)), KeyMotion::idle());
    for pressed in [true, false, true] { controls.register_key(KeyCode::KeyW, pressed); }
    let (mut walk_floater, mut walk_ctrl) = (PlanetaryAvatar::at(Vec3::new(1.25, 23.3, 1.25)), KeyMotion::idle());
    walk_ctrl.register_key(KeyCode::KeyW, true);
    avatar.step_motion(1.0 / 60.0, &controls, &world); walk_floater.step_motion(1.0 / 60.0, &walk_ctrl, &world);
    let sprint_eye = avatar.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    let walk_eye = walk_floater.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    assert!((sprint_eye - walk_eye).length() < 1e-5, "Fast floating (sprinting) beneath the crust must not be possible: {sprint_eye:?} vs {walk_eye:?}");
    for frame in 0..360 {
        controls.register_key(KeyCode::Space, frame % 4 < 2);
        avatar.step_motion(1.0 / 60.0, &controls, &world);
        let eye = avatar.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
        assert!(eye.length() <= 23.82 && world.weightless_at(eye), "Frame {frame}: repeated head bumps must never catapult avatar to outer surface, got {eye:?}");
    }
}

#[test]
fn test_digging_upward_from_beneath_crust_removes_block_breaches_exit_to_surface_and_magma_core_kills() {
    use crate::attire::{HeldRelic, YouthAttire};
    use crate::cardiac::CardiacPulse;
    let mut world = BiomedCubeWorld::beveled(64, 1.0);
    let mut avatar = PlanetaryAvatar::at(Vec3::new(1.25, 22.2, 1.25));
    let start_fov_scale = avatar.view_projection_matrix(1.0).x_axis.x;
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::ExcavatedEarth));
    avatar.rotate_view(0.0, -2000.0);
    assert!(!avatar.use_right_hand(&mut world));
    assert!(avatar.release_hand(&mut world), "Short-click (press + release) while floating must place a block under the crust");
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::ExcavatedEarth));
    assert!(!avatar.use_right_hand(&mut world));
    assert!(avatar.use_right_hand(&mut world), "Digging under the crust must remove the placed block");
    assert!(!avatar.use_right_hand(&mut world));
    assert!(avatar.use_right_hand(&mut world), "Digging under the crust must remove one ceiling block");
    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::Space, true);
    for _ in 0..90 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let first_vp = avatar.view_projection_matrix(1.0);
    let first_eye = first_vp.inverse().project_point3(Vec3::ZERO);
    assert!(first_eye.y > 24.0 && first_eye.y < 24.3 && world.weightless_at(first_eye), "Single upward dig must only open one block (0.5 m) instead of a full shaft to the surface, got {first_eye:?}");

    let mut strained_scale = first_vp.x_axis.x;
    for step in 0..3 {
        assert!(!avatar.use_right_hand(&mut world));
        assert!(avatar.use_right_hand(&mut world));
        for _ in 0..30 {
            avatar.step_motion(1.0 / 60.0, &controls, &world);
        }
        if step == 1 { strained_scale = avatar.view_projection_matrix(1.0).x_axis.x; }
    }
    let crust_vp = avatar.view_projection_matrix(1.0);
    let crust_eye = crust_vp.inverse().project_point3(Vec3::ZERO);
    assert!(strained_scale > first_vp.x_axis.x && first_vp.x_axis.x > start_fov_scale, "Staying near the core must progressively narrow the FOV ({} -> {} -> {})", start_fov_scale, first_vp.x_axis.x, strained_scale);
    assert!(!world.weightless_at(crust_eye), "Weightlessness must end once the 4-layer crust is dug upward, got {crust_eye:?}");

    for _ in 0..25 {
        assert!(!avatar.use_right_hand(&mut world));
        if !avatar.use_right_hand(&mut world) { break; }
        for _ in 0..24 {
            avatar.step_motion(1.0 / 60.0, &controls, &world);
        }
    }
    for _ in 0..240 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let exit_vp = avatar.view_projection_matrix(1.0);
    let exit_eye = exit_vp.inverse().project_point3(Vec3::ZERO);
    assert!(exit_eye.y >= 32.5 && !world.weightless_at(exit_eye) && exit_vp.x_axis.x < strained_scale, "Escaping behind the crust must recover FOV and reach surface, got {exit_eye:?}");

    while world.constrain_to_surface(Vec3::new(3.25, 33.0, 0.25), 0.0).y > 20.0 {
        let fy = world.constrain_to_surface(Vec3::new(3.25, 33.0, 0.25), 0.0).y;
        assert!(world.excavate_surface(Vec3::new(3.25, fy + 1.0, 0.25), Vec3::NEG_Y));
    }
    while world.constrain_to_surface(Vec3::new(3.25, 33.0, 0.75), 0.0).y > 24.6 {
        let fy = world.constrain_to_surface(Vec3::new(3.25, 33.0, 0.75), 0.0).y;
        assert!(world.excavate_surface(Vec3::new(3.25, fy + 1.0, 0.75), Vec3::NEG_Y));
    }
    let mut breach_miner = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(3.25, 33.0, 0.45), 1.0));
    breach_miner.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::EmptyPalm));
    breach_miner.rotate_view(1047.0, 600.0);
    let mut press_wall = KeyMotion::idle();
    press_wall.register_key(KeyCode::KeyS, true);
    press_wall.register_key(KeyCode::ShiftLeft, true);
    assert!(!breach_miner.use_right_hand(&mut world) && breach_miner.use_right_hand(&mut world), "Must excavate last block before core");
    for _ in 0..90 { breach_miner.step_motion(1.0 / 60.0, &press_wall, &world); }
    let miner_eye = breach_miner.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    assert!(miner_eye.y > 22.5 && miner_eye.y < 23.8 && world.weightless_at(miner_eye) && !world.lethal_at(miner_eye), "Mining the last block before the core must leave the avatar floating safely beneath the crust instead of falling into the magma core, got {miner_eye:?}");

    let mut lingering = PlanetaryAvatar::at(Vec3::new(0.25, 21.5, 0.25));
    for _ in 0..118 { lingering.step_motion(1.0, &KeyMotion::idle(), &world); }
    let vp_before_turn = lingering.view_projection_matrix(1.0);
    lingering.rotate_view(120.0, 0.0);
    assert_ne!(lingering.view_projection_matrix(1.0), vp_before_turn, "Avatar must still be alive and responsive before 2 minutes (120s) elapse");
    for _ in 0..3 { lingering.step_motion(1.0, &KeyMotion::idle(), &world); }
    let vp_blackout = lingering.view_projection_matrix(1.0);
    lingering.rotate_view(120.0, 0.0);
    let mut linger_hud = SpanCapture::empty();
    lingering.emit_reticle(&mut linger_hud);
    assert!(lingering.view_projection_matrix(1.0) == vp_blackout && linger_hud.max_y - linger_hud.min_y > 1.8 && vp_blackout.y_axis.y > start_fov_scale * 1.4, "Lingering past 2 minutes (120s) near the core must narrow FOV into blackout");
    for _ in 0..2 { lingering.step_motion(1.0, &KeyMotion::idle(), &world); }
    let linger_awoken = lingering.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    assert!(linger_awoken.y >= 32.5 && !world.weightless_at(linger_awoken), "After full blackout, lingering avatar must wake up at surface spawn, got {linger_awoken:?}");

    let mut doomed = PlanetaryAvatar::at(Vec3::new(0.25, 16.8, 0.25));
    let initial_fov_scale = doomed.view_projection_matrix(1.0).y_axis.y;
    for _ in 0..60 { doomed.step_motion(1.0 / 60.0, &controls, &world); }
    let dead_vp = doomed.view_projection_matrix(1.0);
    let dead_eye = dead_vp.inverse().project_point3(Vec3::ZERO);
    assert!(dead_eye.y < 18.0 && world.lethal_at(Vec3::new(0.25, 16.8, 0.25)) && dead_vp.y_axis.y > initial_fov_scale * 1.25, "Avatar at magma core must enter shrinking tunnel-vision blackout in place, got {dead_eye:?}");
    let mut hud = SpanCapture::empty();
    doomed.emit_reticle(&mut hud);
    assert!(hud.max_y - hud.min_y > 1.8, "Blackout posture must emit full-screen transparent-gradient vignette");
    for _ in 0..120 { doomed.step_motion(1.0 / 60.0, &controls, &world); }
    let awoken_eye = doomed.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    assert!(awoken_eye.y >= 32.5 && !world.lethal_at(awoken_eye) && !world.weightless_at(awoken_eye), "Once completely black before their eyes, avatar must wake up at surface spawn, got {awoken_eye:?}");
    let wav = CardiacPulse::studio().riff_wave();
    assert!(wav.starts_with(b"RIFF") && wav.len() > 8000, "CardiacPulse must synthesize booming RIFF WAVE heartbeat audio");
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
