use glam::Vec3;
use world::PlanetaryWorld;
use crate::attire::{BeltSatchel, EvaAttire, HeldRelic, PhotoShutter, SightMode};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarOptics as _, AvatarPresence as _, PhotoPort as _,
    PlanetaryAvatar,
};
use acoustics::{AcousticWave, ShutterClick};
use crate::controls::{InputTrigger, KeyMotion, KeyboardControls};
use crate::photo_plate::PhotoPlate;
use crate::server_satchel::{ArgumentSatchel, EvaKitSatchel, FileSatchel, ParsedSatchel, ServerSatchel, StartingSatchel};
use super::QuadLedger;

#[test]
fn test_server_satchel_parses_cli_and_custom_inventory_specs() {
    let cadet = EvaAttire::astronaut();
    let test_satchel = EvaKitSatchel::new().provision(BeltSatchel::empty());
    assert_eq!(test_satchel.resource_tally(HeldRelic::CantileverCrane, cadet), 1);
    assert_eq!(test_satchel.resource_tally(HeldRelic::FieldCamera, cadet), 1);
    assert_eq!(test_satchel.resource_tally(HeldRelic::VehicleBlueprint, cadet), 1);
    assert_eq!(test_satchel.resource_tally(HeldRelic::RocketCamera, cadet), 4);
    assert!(HeldRelic::EmptyPalm.parse_token("cross_axle").is_none());
    assert!(HeldRelic::EmptyPalm.parse_token("knightly_sword").is_none());

    let custom = ParsedSatchel::parse("field_camera:2, rig_wrench, car_wheel:4", "test").provision(BeltSatchel::empty());
    assert_eq!(custom.resource_tally(HeldRelic::FieldCamera, cadet), 2);
    assert_eq!(custom.resource_tally(HeldRelic::RigWrench, cadet), 1);
    assert_eq!(custom.resource_tally(HeldRelic::CarWheel, cadet), 4);
    assert_eq!(custom.resource_tally(HeldRelic::FlightConsole, cadet), 0);

    let args = vec!["client".to_string(), "--inventory".to_string(), "camera,suit_radio:1".to_string()];
    let cli_satchel = ArgumentSatchel::from_arguments(&args, Box::new(FileSatchel::server(Box::new(EvaKitSatchel::new()))))
        .provision(BeltSatchel::empty());
    assert_eq!(cli_satchel.resource_tally(HeldRelic::FieldCamera, cadet), 1);
    assert_eq!(cli_satchel.resource_tally(HeldRelic::SuitRadio, cadet), 1);

    let root_toml = format!("{}/../../server.toml", env!("CARGO_MANIFEST_DIR"));
    let root_satchel = FileSatchel::from_path(&root_toml, Box::new(EvaKitSatchel::new()))
        .provision(BeltSatchel::empty());
    assert_eq!(root_satchel.resource_tally(HeldRelic::CantileverCrane, cadet), 1);
    assert_eq!(root_satchel.resource_tally(HeldRelic::FieldCamera, cadet), 1);
    assert_eq!(root_satchel.resource_tally(HeldRelic::VehicleBlueprint, cadet), 1);
    assert_eq!(root_satchel.resource_tally(HeldRelic::RocketCamera, cadet), 4);
    assert_eq!(root_satchel.resource_tally(HeldRelic::FlightConsole, cadet), 1);
    assert_eq!(root_satchel.resource_tally(HeldRelic::RocketNozzle, cadet), 6);

    for code in 1..(HeldRelic::COUNT as u8) {
        let relic = HeldRelic::from_code(code);
        if relic.placeable_block() {
            assert_eq!(world::RelicCatalog::from_medium(relic.block_medium()), code);
        } else {
            assert_eq!(world::RelicCatalog::from_find(relic.surface_find()), code);
        }
    }

    let cheated = ParsedSatchel::parse("fusion_torch:99, gold_ore:99", "cheat");
    let server_kit = world::SatchelManifest::parse_server_kit("rig_wrench:1, suit_radio:1", "server");
    let (synced_satchel, synced_attire) = BeltSatchel::from_survivor_manifest(
        server_kit,
        cadet.with_next_shell().with_relics(HeldRelic::FusionTorch, HeldRelic::GoldOre),
    );
    let _ = cheated;
    assert_eq!(synced_satchel.resource_tally(HeldRelic::FusionTorch, synced_attire), 0);
    assert_eq!(synced_satchel.resource_tally(HeldRelic::GoldOre, synced_attire), 0);
    assert_eq!(synced_satchel.resource_tally(HeldRelic::RigWrench, synced_attire), 1);
    assert_eq!(synced_satchel.resource_tally(HeldRelic::SuitRadio, synced_attire), 1);
    assert_eq!(synced_satchel.resource_tally(HeldRelic::CantileverCrane, synced_attire), 1);
}

#[test]
fn test_handheld_camera_rotates_through_four_angles_and_snaps_immediately_without_countdown() {
    let shutter_sound = ShutterClick::mechanical();
    let open_peak = (1..15).map(|ms| shutter_sound.amplitude_at((ms as f32) * 0.001).abs()).fold(0.0f32, f32::max);
    let close_peak = (64..85).map(|ms| shutter_sound.amplitude_at((ms as f32) * 0.001).abs()).fold(0.0f32, f32::max);
    assert!(open_peak > 0.25 && close_peak > 0.20 && shutter_sound.riff_wave().starts_with(b"RIFF"));

    let world = world::BiomedQuadSphere::of_size(16);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 8.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn).with_starting_satchel(&ServerSatchel::assembled());
    avatar.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::FieldCamera, HeldRelic::EmptyPalm));

    let mut base_reticle = QuadLedger::empty();
    avatar.emit_reticle(&mut base_reticle);
    let mut mesh_fwd = QuadLedger::empty();
    avatar.emit_appearance(&mut mesh_fwd);
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::FirstPersonSight));
    assert_eq!(SightMode::FirstPersonSight.photo_label(), "landscape");
    let tilt_shot_0 = avatar.polaroid_tilt();
    assert!(tilt_shot_0.abs() > 0.05);
    let mut polaroid_reticle = QuadLedger::empty();
    avatar.emit_reticle(&mut polaroid_reticle);
    assert!(polaroid_reticle.quads.len() > base_reticle.quads.len() + 300);
    let mut landscape_avatar = QuadLedger::empty();
    avatar.emit_appearance_for_sight(SightMode::FirstPersonSight, &mut landscape_avatar);
    assert!(!landscape_avatar.populated());

    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    let mut mesh_diag_fwd = QuadLedger::empty();
    avatar.emit_appearance(&mut mesh_diag_fwd);
    let v0_list = |l: &QuadLedger| -> Vec<Vec3> { l.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect() };
    assert_ne!(v0_list(&mesh_fwd), v0_list(&mesh_diag_fwd));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::ForwardAngledSight));

    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    let mut mesh_side = QuadLedger::empty();
    avatar.emit_appearance(&mut mesh_side);
    assert_ne!(v0_list(&mesh_diag_fwd), v0_list(&mesh_side));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::ProfileSight));
    assert_ne!(tilt_shot_0, avatar.polaroid_tilt());

    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    let mut mesh_diag_back = QuadLedger::empty();
    avatar.emit_appearance(&mut mesh_diag_back);
    assert_ne!(v0_list(&mesh_side), v0_list(&mesh_diag_back));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::AngledSight));

    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::LookingGlassSight));
}

#[test]
fn test_placed_camera_runs_six_second_white_self_timer_countdown() {
    let mut world = world::BiomedQuadSphere::of_size(16);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 8.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn);
    avatar.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::FieldCamera, HeldRelic::EmptyPalm));

    let mut base_reticle = QuadLedger::empty();
    avatar.emit_reticle(&mut base_reticle);
    avatar.rotate_degrees(0.0, 58.0);
    assert!(!avatar.use_right_hand(&mut world) && avatar.release_hand(&mut world));

    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Idle);
    let (mut cd_large, mut cd_small) = (QuadLedger::empty(), QuadLedger::empty());
    avatar.emit_reticle(&mut cd_large);
    avatar.step_motion(0.8, &KeyMotion::idle(), &world);
    avatar.emit_reticle(&mut cd_small);
    let span_y = |l: &QuadLedger| {
        l.quads
            .iter()
            .skip(base_reticle.quads.len())
            .map(|q| q.transfer(|v0, _, v2, _, _, _| v0.y.abs().max(v2.y.abs())))
            .fold(0.0f32, f32::max)
    };
    assert!(cd_large.quads.len() > base_reticle.quads.len() && span_y(&cd_large) > span_y(&cd_small) + 0.04);
    assert_eq!(cd_large.quads.last().unwrap().transfer(|_, _, _, _, _, col| col), Vec3::ONE);
    avatar.step_motion(5.3, &KeyMotion::idle(), &world);
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::LookingGlassSight));
}

#[test]
fn test_selfie_camera_tracks_held_elevation_and_develops_png_plate() {
    let world = world::BiomedQuadSphere::of_size(16);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 8.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn);
    avatar.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::FieldCamera, HeldRelic::EmptyPalm));

    let vp_front_level = avatar.view_projection_for_sight(16.0 / 9.0, SightMode::LookingGlassSight);
    avatar.rotate_degrees(0.0, -35.0);
    let vp_front_high = avatar.view_projection_for_sight(16.0 / 9.0, SightMode::LookingGlassSight);
    avatar.rotate_degrees(0.0, 70.0);
    let vp_front_low = avatar.view_projection_for_sight(16.0 / 9.0, SightMode::LookingGlassSight);
    let cam_y_high = vp_front_high.inverse().project_point3(Vec3::ZERO).y;
    let cam_y_level = vp_front_level.inverse().project_point3(Vec3::ZERO).y;
    let cam_y_low = vp_front_low.inverse().project_point3(Vec3::ZERO).y;
    assert!(cam_y_high > cam_y_level + 0.25 && cam_y_level > cam_y_low + 0.25);

    let mut high_selfie = QuadLedger::empty();
    avatar.emit_appearance_for_sight(SightMode::LookingGlassSight, &mut high_selfie);
    let max_ndc_y = high_selfie
        .quads
        .iter()
        .map(|q| vp_front_high.project_point3(q.transfer(|v0, _, _, _, _, _| v0)).y)
        .fold(-1.0f32, f32::max);
    assert!(max_ndc_y < 0.96, "High selfie camera angle must keep the crew helmet inside the top of the frame");

    let temp_dir = std::env::temp_dir().join("lowcube_test_screenshots");
    let path = PhotoPlate::new(4, 3, vec![180u8; 4 * 3 * 4]).develop_into(temp_dir.to_str().unwrap(), "front");
    let bytes = std::fs::read(&path).expect("PNG screenshot must exist on disk");
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n") && bytes.ends_with(b"IEND\xaeB`\x82"));
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_camera_on_three_dirt_blocks_photographs_from_tripod_lens_and_picks_up_in_one_click() {
    let mut world = world::BiomedQuadSphere::of_size(32);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn)
        .with_starting_satchel(&ParsedSatchel::parse("excavated_earth:3", "test"));
    avatar.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::FieldCamera));
    avatar.rotate_degrees(0.0, 58.0);
    for pitch_up in [-12.0, -16.0, -18.0] {
        assert!(!avatar.use_right_hand(&mut world) && avatar.release_hand(&mut world));
        avatar.rotate_degrees(0.0, pitch_up);
    }
    assert!(!avatar.use_left_hand(&mut world) && avatar.release_hand(&mut world));
    let (tripod_hub, _, _) = world.placed_camera_tripod().expect("World must report placed camera tripod hub");
    assert!(tripod_hub.length() > spawn.length() + 0.7);

    assert!(!avatar.adjust_aimed_rig(InputTrigger::WheelClick, &mut world));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    let mut walked = KeyMotion::idle();
    walked.register_key(winit::keyboard::KeyCode::KeyS, true);
    walked.register_key(winit::keyboard::KeyCode::KeyD, true);
    for _ in 0..20 {
        avatar.step_motion(0.05, &walked, &world);
    }
    avatar.rotate_degrees(45.0, 0.0);
    avatar.step_motion(5.1, &KeyMotion::idle(), &world);
    let PhotoShutter::Triggered(sight) = avatar.take_photo_shutter() else {
        panic!("Placed camera self-timer must trigger after 6 seconds");
    };
    let shot_vp = avatar.view_projection_for_sight(16.0 / 9.0, sight);
    let shot_eye = shot_vp.inverse().project_point3(Vec3::new(0.0, 0.0, -1.0));
    assert!((shot_eye - tripod_hub).length() < 0.35);
    let live_eye = avatar.view_projection_matrix(16.0 / 9.0).inverse().project_point3(Vec3::new(0.0, 0.0, -1.0));
    assert!((live_eye - shot_eye).length() > 1.5);

    let mut pickup_avatar = PlanetaryAvatar::at(spawn);
    pickup_avatar.rotate_degrees(0.0, -6.0);
    assert!(pickup_avatar.use_right_hand(&mut world));
    assert!(world.placed_camera_tripod().is_none());
    assert!(pickup_avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(pickup_avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::FirstPersonSight));
}

#[test]
fn test_multiplayer_companion_handheld_camera_rotation_visible_to_peer() {
    use crate::black_box::PresenceProbe;
    use crate::companion::CompanionFigure;
    use world::CompanionPresence;

    let peer_vertices = |avatar: &PlanetaryAvatar| -> Vec<Vec3> {
        let mut probe = PresenceProbe::at_origin();
        avatar.report_presence(&mut probe);
        let mut wire = Vec::new();
        probe.presence().inscribe(&mut wire);
        let (decoded, used) = CompanionPresence::decode(&wire).expect("decode companion presence");
        assert_eq!(used, wire.len());
        let mut ledger = QuadLedger::empty();
        CompanionFigure::new(decoded).emit(&mut ledger);
        ledger.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect()
    };

    let world = world::BiomedQuadSphere::barren(16);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let mut companion = PlanetaryAvatar::at(spawn);

    for hands in [
        (HeldRelic::FieldCamera, HeldRelic::EmptyPalm),
        (HeldRelic::EmptyPalm, HeldRelic::FieldCamera),
    ] {
        companion.apply_attire(EvaAttire::astronaut().with_relics(hands.0, hands.1));
        let v_fwd = peer_vertices(&companion);
        assert!(companion.dispatch_trigger(InputTrigger::WheelDown));
        let v_diag_fwd = peer_vertices(&companion);
        assert_ne!(v_fwd, v_diag_fwd);

        assert!(companion.dispatch_trigger(InputTrigger::WheelDown));
        let v_side = peer_vertices(&companion);
        assert_ne!(v_diag_fwd, v_side);

        assert!(companion.dispatch_trigger(InputTrigger::WheelDown));
        let v_diag_back = peer_vertices(&companion);
        assert_ne!(v_side, v_diag_back);

        assert!(companion.dispatch_trigger(InputTrigger::WheelDown));
        let v_selfie = peer_vertices(&companion);
        assert_ne!(v_diag_back, v_selfie);
    }
}
