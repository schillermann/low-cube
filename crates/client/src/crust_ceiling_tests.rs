use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BiomedQuadSphere, MeshTarget, PlanetaryWorld, Quad, TerrainMesh as _, WorldEarthworks as _};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarOptics as _, AvatarPresence as _, PlanetaryAvatar,
};
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
    let mut world = BiomedQuadSphere::of_size(64);
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
    let mut world = BiomedQuadSphere::of_size(64);
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
    use acoustics::{AcousticWave, CardiacPulse, MonitoredPulse};
    let mut world = BiomedQuadSphere::of_size(64);
    for _ in 0..40 {
        let floor = world.constrain_to_surface(Vec3::new(0.25, 32.0, 0.25), 0.0);
        if !world.excavate_surface(floor + floor.normalize() * 1.0, -floor.normalize()) {
            break;
        }
    }
    let core_floor = world.constrain_to_surface(Vec3::new(0.25, 32.0, 0.25), 0.0);
    let core_floor_r = core_floor.length();
    assert!((core_floor_r - 24.0).abs() < 1e-3, "Digging through all topsoil and 4 crust layers must stop on the indestructible core floor at r=24.0, got {core_floor_r}");
    assert!(!world.excavate_surface(core_floor + core_floor.normalize() * 1.0, -core_floor.normalize()), "Indestructible core floor must never be diggable");
    let contact = core_floor + core_floor.normalize() * 1.0;
    assert!(world.lethal_at(contact), "Direct contact with the exposed core floor must be lethal");

    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::Space, true);
    let mut doomed = PlanetaryAvatar::at(contact);
    let initial_fov_scale = doomed.view_projection_matrix(1.0).y_axis.y;
    for _ in 0..60 { doomed.step_motion(1.0 / 60.0, &controls, &world); }
    let dead_vp = doomed.view_projection_matrix(1.0);
    let dead_eye = dead_vp.inverse().project_point3(Vec3::ZERO);
    assert!(dead_eye.length() < 26.5 && world.lethal_at(contact) && dead_vp.y_axis.y > initial_fov_scale * 1.25, "Avatar touching exposed core floor must enter shrinking tunnel-vision blackout, got {dead_eye:?}");
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
    use crate::attire::{BodyPose, CrewPhysique, StrideRhythm};
    use crate::server_scale::{ArgumentScale, FileScale, MoonletScale, PlanetaryScale, ScaledCube};

    let tilted = BodyPose::new(
        Vec3::ZERO,
        Quat::IDENTITY,
        Quat::from_rotation_x(0.45),
        0.8,
        StrideRhythm::idle(),
        CrewPhysique::OrbitalCadet,
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
    let mut world = BiomedQuadSphere::of_size(64);
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
    let shape = world::MediumShape::new(world::CuboidShape::new(0.5), world::BlockMedium::CrustRock, true, false);
    let mut rock_span = SpanCapture::empty();
    use world::VoxelShape;
    shape.emit_terraced_shape(world::GridCell::new(0, 0), -7.0, &frame, Vec3::ONE, Vec3::ONE, true, &mut rock_span);
    assert!(rock_span.max_y < 25.6 && rock_span.min_y >= 24.9, "Subterranean CrustRock block at -7.0..6.5 (y=25.0..25.5) must emit all side flanks at its own layer, got y={}..{}", rock_span.min_y, rock_span.max_y);
}

#[test]
fn test_multiplayer_narrow_shaft_looking_down_clamps_crosshair_and_rejects_underfoot_block() {
    use sanctuary::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};
    use crate::attire::{EvaAttire, HeldRelic};

    let host_inner: Box<dyn PlanetaryWorld> = Box::new(BiomedQuadSphere::of_size(64));
    let host = SanctuaryHost::bind("127.0.0.1:0", host_inner, 64).expect("bind host");
    let addr = host.local_addr().expect("addr");
    let mut host_planet = HostedPlanet::new(host);
    let mut guest_planet = RemotePlanet::dial(JoinTarget::Direct(addr), 64, 1.0).expect("dial guest");
    for _ in 0..10 {
        let _ = guest_planet.step_surface(0.02);
        let _ = host_planet.step_surface(0.02);
    }
    assert!(guest_planet.connected());

    let rim_y = host_planet.constrain_to_surface(Vec3::new(0.25, 40.0, 0.25), 1.0).y;
    let wall_rim_y = host_planet.constrain_to_surface(Vec3::new(0.25, 40.0, -0.25), 0.0).y;
    for _ in 0..8 {
        let fy = host_planet.constrain_to_surface(Vec3::new(0.25, 40.0, 0.25), 0.0).y;
        assert!(host_planet.excavate_surface(Vec3::new(0.25, fy + 1.0, 0.25), Vec3::NEG_Y));
    }
    let bottom_y = host_planet.constrain_to_surface(Vec3::new(0.25, 40.0, 0.25), 0.0).y;
    assert!(host_planet.place_molded_surface_block(
        Vec3::new(0.25, bottom_y + 1.0, 0.25),
        Vec3::NEG_Y,
        Vec3::new(5.0, rim_y, 5.0),
        world::BlockMold::cube().in_medium(world::BlockMedium::TurfEarth),
    ));
    for _ in 0..12 {
        let _ = host_planet.step_surface(0.06);
        let _ = guest_planet.step_surface(0.06);
    }

    let shaft_pos = guest_planet.constrain_to_surface(Vec3::new(0.25, bottom_y + 1.5, 0.25), 1.0);
    assert!((shaft_pos.y - (bottom_y + 1.5)).abs() < 0.05);
    let mut avatar = PlanetaryAvatar::at(shaft_pos);
    for _ in 0..15 {
        avatar.step_motion(1.0 / 120.0, &KeyMotion::idle(), &guest_planet);
    }
    avatar.sync_world_presence(&mut guest_planet);
    for _ in 0..4 {
        let _ = guest_planet.step_surface(0.06);
        let _ = host_planet.step_surface(0.06);
    }

    // Dragging crosshair all the way down must stop well before pointing under the player's feet, and placing a block must be rejected!
    avatar.rotate_degrees(0.0, 88.0);
    let inv = avatar.view_projection_matrix(1.0).inverse();
    let eye_before = inv.project_point3(Vec3::ZERO);
    let fwd_down = (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - eye_before).normalize();
    assert!(fwd_down.y > -0.84, "Crosshair in narrow shaft must stop before reaching under the player's feet, got dir={fwd_down:?}");

    avatar.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::RigWrench));
    let _ = avatar.use_right_hand(&mut guest_planet);
    assert!(!avatar.release_hand(&mut guest_planet), "Looking down in 1-block shaft must never place a block beneath or inside the player");
    assert!(
        !guest_planet.place_molded_surface_block(
            Vec3::new(0.25, bottom_y + 1.8, 0.25),
            Vec3::NEG_Y,
            shaft_pos,
            world::BlockMold::cube(),
        ),
        "Direct downward block placement overlapping player stance must be rejected"
    );
    assert!(
        (guest_planet.constrain_to_surface(Vec3::new(0.25, 40.0, -0.25), 0.0).y - wall_rim_y).abs() < 1e-4,
        "Rejected shaft placement must never stack a phantom block onto the top shaft rim"
    );

    // Digging through the molded block in the 1-block shaft must stay on the shaft floor without falling to core or teleporting to rim!
    let fy = guest_planet.constrain_to_surface(Vec3::new(0.25, 40.0, 0.25), 0.0).y;
    assert!(guest_planet.excavate_surface(Vec3::new(0.25, fy + 0.8, 0.25), Vec3::NEG_Y));
    let mut jump_walk = KeyMotion::idle();
    jump_walk.register_key(KeyCode::Space, true);
    jump_walk.register_key(KeyCode::KeyW, true);
    for _ in 0..90 {
        avatar.step_motion(1.0 / 60.0, &jump_walk, &guest_planet);
        let eye = avatar.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO).y;
        assert!(
            eye > bottom_y + 0.5 && eye < rim_y,
            "Avatar in 1-block shaft after digging molded blocks must neither fall through solid floor nor teleport to rim (eye={eye}, bottom={bottom_y}, rim={rim_y})"
        );
    }
}

#[test]
fn test_releasing_hand_while_perished_halts_digging_and_preserves_full_respawn_pace() {
    use crate::attire::{EvaAttire, HeldRelic};

    let mut world = BiomedQuadSphere::of_size(64);
    for _ in 0..40 {
        let fy = world.constrain_to_surface(Vec3::new(0.25, 35.0, 0.25), 0.0).y;
        if !world.excavate_surface(Vec3::new(0.25, fy + 1.0, 0.25), Vec3::NEG_Y) {
            break;
        }
    }
    let core_y = world.constrain_to_surface(Vec3::new(0.25, 35.0, 0.25), 0.0).y;
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.25, core_y + 1.5, 0.25));
    avatar.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::RigWrench, HeldRelic::EmptyPalm));
    avatar.rotate_degrees(0.0, 82.0);
    let _ = avatar.use_right_hand(&mut world);

    // Touch lethal core while hand is held, then release hand during blackout!
    let mut avatar_core = PlanetaryAvatar::at(Vec3::new(0.25, core_y + 1.0, 0.25));
    avatar_core.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::RigWrench, HeldRelic::EmptyPalm));
    avatar_core.rotate_degrees(0.0, 82.0);
    avatar_core.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    let _ = avatar_core.release_hand(&mut world);
    for _ in 0..180 {
        avatar_core.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    }
    assert!(!avatar_core.step_digging(&mut world), "Respawned avatar must not ghost-dig after releasing mouse during blackout");
    let mut walk = KeyMotion::idle();
    walk.register_key(KeyCode::KeyW, true);
    let start = avatar_core.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    for _ in 0..30 {
        avatar_core.step_motion(1.0 / 60.0, &walk, &world);
    }
    let end = avatar_core.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    assert!((end - start).length() > 2.2, "Movement after respawn must run at full brisk walking speed, got {}", (end - start).length());
}
