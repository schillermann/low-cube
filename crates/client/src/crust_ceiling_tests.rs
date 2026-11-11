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
    let mut world = BiomedCubeWorld::beveled(64, 1.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    assert!(world.excavate_surface(Vec3::new(0.25, 29.0, 0.25), Vec3::new(1.0, 0.2, 0.0).normalize()));
    assert!(world.excavate_surface(Vec3::new(0.25, 29.5, 0.25), Vec3::new(1.0, 0.2, 0.0).normalize()));
    let mut pos = Vec3::new(0.65, 29.5, 0.25);
    assert!(!world.lethal_at(pos), "Subterranean niche has standard planetary gravity");
    flight.jump();
    for frame in 0..120 {
        pos = flight.advance(1.0 / 60.0, pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);
        assert!(pos.y <= 30.0, "Frame {frame}: jumping inside subterranean niche must stay beneath the overhang ceiling, got {pos:?}");
    }
}

#[test]
fn test_repeated_head_bumps_against_crust_never_catapult_avatar_to_surface() {
    let mut world = BiomedCubeWorld::beveled(64, 1.0);
    for _ in 0..8 {
        let floor_y = world.constrain_to_surface(Vec3::new(1.25, 33.0, 1.25), 0.0).y;
        assert!(world.excavate_surface(Vec3::new(1.25, floor_y + 1.0, 1.25), Vec3::NEG_Y));
    }
    assert!(world.excavate_surface(Vec3::new(1.25, 29.0, 1.25), Vec3::new(1.0, 0.1, 0.0).normalize()));
    assert!(world.excavate_surface(Vec3::new(1.25, 29.5, 1.25), Vec3::new(1.0, 0.1, 0.0).normalize()));
    let (mut avatar, mut controls) = (PlanetaryAvatar::at(Vec3::new(1.65, 29.5, 1.25)), KeyMotion::idle());
    for frame in 0..240 {
        controls.register_key(KeyCode::Space, frame % 4 < 2);
        avatar.step_motion(1.0 / 60.0, &controls, &world);
        let eye = avatar.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
        assert!(eye.y < 30.0, "Frame {frame}: repeated head bumps under overhang must never catapult avatar to outer surface, got {eye:?}");
    }
}

#[test]
fn test_digging_down_through_crust_stops_at_indestructible_lethal_core_floor() {
    use crate::cardiac::{AcousticWave, CardiacPulse, MonitoredPulse};
    let mut world = BiomedCubeWorld::beveled(64, 1.0);
    for _ in 0..40 {
        let fy = world.constrain_to_surface(Vec3::new(0.25, 35.0, 0.25), 0.0).y;
        if !world.excavate_surface(Vec3::new(0.25, fy + 1.0, 0.25), Vec3::NEG_Y) {
            break;
        }
    }
    let core_floor_y = world.constrain_to_surface(Vec3::new(0.25, 35.0, 0.25), 0.0).y;
    assert!((core_floor_y - 24.0).abs() < 1e-3, "Digging through all topsoil and 4 crust layers must stop on the indestructible core floor at y=24.0, got {core_floor_y}");
    assert!(!world.excavate_surface(Vec3::new(0.25, core_floor_y + 1.0, 0.25), Vec3::NEG_Y), "Indestructible core floor must never be excavated");
    assert!(world.lethal_at(Vec3::new(0.25, core_floor_y + 1.0, 0.25)), "Direct contact with the exposed core floor must be lethal");

    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::Space, true);
    let mut doomed = PlanetaryAvatar::at(Vec3::new(0.25, core_floor_y + 1.0, 0.25));
    let initial_fov_scale = doomed.view_projection_matrix(1.0).y_axis.y;
    for _ in 0..60 { doomed.step_motion(1.0 / 60.0, &controls, &world); }
    let dead_vp = doomed.view_projection_matrix(1.0);
    let dead_eye = dead_vp.inverse().project_point3(Vec3::ZERO);
    assert!(dead_eye.y < 26.5 && world.lethal_at(Vec3::new(0.25, core_floor_y + 1.0, 0.25)) && dead_vp.y_axis.y > initial_fov_scale * 1.25, "Avatar touching exposed core floor must enter shrinking tunnel-vision blackout, got {dead_eye:?}");
    let mut hud = SpanCapture::empty();
    doomed.emit_reticle(&mut hud);
    assert!(hud.max_y - hud.min_y > 1.8, "Blackout posture must emit full-screen transparent-gradient vignette");
    for _ in 0..120 { doomed.step_motion(1.0 / 60.0, &controls, &world); }
    let awoken_eye = doomed.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    assert!(awoken_eye.y >= 32.5 && !world.lethal_at(awoken_eye), "Once completely black before their eyes, avatar must wake up at surface spawn, got {awoken_eye:?}");
    let wav = CardiacPulse::studio().riff_wave();
    let monitored_wav = MonitoredPulse::clinical().riff_wave();
    assert!(wav.starts_with(b"RIFF") && wav.len() > 8000 && monitored_wav != wav, "MonitoredPulse decorator must overlay clinical beep onto CardiacPulse RIFF WAVE audio");
    let mut p = crate::posture::AvatarPosture::standing().perish();
    let mut beats = Vec::new();
    for step in 0..240 {
        let (next_p, beat) = p.advance_perish(0.01);
        if beat { beats.push(step); }
        p = next_p;
    }
    assert!(beats.len() >= 3 && (beats[2] - beats[1]) > (beats[1] - beats[0]), "Heartbeats during death animation must progressively slow down: {beats:?}");
}

#[test]
fn test_neck_remains_anchored_to_torso_when_head_tilts_back_and_server_scale_validates() {
    use glam::Quat;
    use crate::attire::{BodyPose, StrideRhythm, YouthPhysique};
    use crate::server_scale::{ArgumentScale, FileScale, MoonletScale, PlanetaryScale, ScaledCube};

    let tilted = BodyPose::new(
        Vec3::ZERO,
        Quat::IDENTITY,
        Quat::from_rotation_x(0.45),
        0.8,
        StrideRhythm::idle(),
        YouthPhysique::SlenderSquire,
    );
    assert_eq!(tilted.neck_frame().point(0.0, 0.0, 0.0), tilted.torso_frame().point(0.0, tilted.torso_span(0.88), -0.056), "Neck frame must remain anchored to torso");
    assert_ne!(tilted.crown_frame().point(0.0, 0.15, 0.0), tilted.neck_frame().point(0.0, 0.15, 0.0), "Head must tilt back around the neck top when pitching view");

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

#[test]
fn test_head_contact_beneath_subterranean_dirt_block_never_catapults_to_surface_or_strips_grass() {
    let mut world = BiomedCubeWorld::beveled(64, 1.0);
    for _ in 0..14 {
        let fy = world.constrain_to_surface(Vec3::new(1.25, 35.0, 1.25), 0.0).y;
        assert!(world.excavate_surface(Vec3::new(1.25, fy + 1.0, 1.25), Vec3::NEG_Y));
    }
    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::Space, true);
    assert!(world.excavate_surface(Vec3::new(1.25, 27.0, 1.25), Vec3::new(1.0, 0.5, 0.0).normalize()));
    let mut niche_climber = PlanetaryAvatar::at(Vec3::new(1.65, 27.25, 1.25));
    for _ in 0..90 {
        niche_climber.step_motion(1.0 / 60.0, &controls, &world);
        let eye = niche_climber.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
        assert!(eye.y < 27.5, "Moving upward with Space under an overhanging dirt block at 27.5 must never enter the 2 dirt blocks above 27.5, got {eye:?}");
    }

    let mut sink = SpanCapture::empty();
    let mut before_crater = world::CountedMeshTarget::new(&mut sink);
    world.emit_modified_face(&mut before_crater);
    let before_count = before_crater.count();
    assert!(world.loosen_surface(Vec3::new(1.25, 25.2, 1.25), Vec3::new(1.0, 0.8, 0.0).normalize(), 2).is_some());
    let mut after_crater = world::CountedMeshTarget::new(&mut sink);
    world.emit_modified_face(&mut after_crater);
    let wall_crater_count = after_crater.count();
    assert_ne!(wall_crater_count, before_count, "Diagonal upward digging from core shaft must update wall mesh with crater animation");
    assert!(world.loosen_surface(Vec3::new(1.25, 27.2, 1.25), Vec3::new(1.0, 0.8, 0.0).normalize(), 2).is_some());
    let mut after_overhang = world::CountedMeshTarget::new(&mut sink);
    world.emit_modified_face(&mut after_overhang);
    assert!(after_overhang.count() != before_count && after_overhang.count() != wall_crater_count, "Digging overhanging dirt block must update faces with 3D crater animation");

    let frame = world::ClimateZone::Temperate.surface_frame(32.0);
    let shape = world::MediumShape::new(Box::new(world::CuboidShape::new(0.5)), world::BlockMedium::CrustRock, true, false);
    let mut rock_span = SpanCapture::empty();
    use world::VoxelShape;
    shape.emit_terraced_shape(world::GridCell::new(0, 0), -7.0, &frame, Vec3::ONE, Vec3::ONE, true, &mut rock_span);
    assert!(rock_span.max_y < 25.6 && rock_span.min_y >= 24.9, "Subterranean CrustRock block at -7.0..6.5 (y=25.0..25.5) must emit all side flanks at its own layer, got y={}..{}", rock_span.min_y, rock_span.max_y);
}
