use glam::Vec3;
use world::PlanetaryWorld;
use crate::attire::{BeltSatchel, HeldRelic, PhotoShutter, SightMode, YouthAttire};
use crate::avatar::{Avatar, PlanetaryAvatar};
use acoustics::{AcousticWave, ShutterClick};
use crate::controls::{InputTrigger, KeyMotion, KeyboardControls};
use crate::photo_plate::PhotoPlate;
use crate::server_satchel::{ArgumentSatchel, FileSatchel, ParsedSatchel, ServerSatchel, StartingSatchel, TestKitSatchel};
use super::QuadLedger;

#[test]
fn test_server_satchel_parses_cli_and_custom_inventory_specs() {
    let squire = YouthAttire::squire();
    let test_satchel = TestKitSatchel::new().provision(BeltSatchel::empty());
    assert_eq!(test_satchel.resource_tally(HeldRelic::FieldCamera, squire), 1);

    let custom = ParsedSatchel::parse("field_camera:2, woodman_axe, car_wheel:4", "test").provision(BeltSatchel::empty());
    assert_eq!(custom.resource_tally(HeldRelic::FieldCamera, squire), 2);
    assert_eq!(custom.resource_tally(HeldRelic::WoodmanAxe, squire), 1);
    assert_eq!(custom.resource_tally(HeldRelic::CarWheel, squire), 4);
    assert_eq!(custom.resource_tally(HeldRelic::RocketEngine, squire), 0);

    let args = vec!["client".to_string(), "--inventory".to_string(), "camera,iron_spade:2".to_string()];
    let cli_satchel = ArgumentSatchel::from_arguments(&args, Box::new(FileSatchel::server(Box::new(TestKitSatchel::new()))))
        .provision(BeltSatchel::empty());
    assert_eq!(cli_satchel.resource_tally(HeldRelic::FieldCamera, squire), 1);
    assert_eq!(cli_satchel.resource_tally(HeldRelic::IronSpade, squire), 2);
}

#[test]
fn test_handheld_camera_rotates_through_four_angles_and_snaps_immediately_without_countdown() {
    let shutter_sound = ShutterClick::mechanical();
    let open_peak = (1..15).map(|ms| shutter_sound.amplitude_at((ms as f32) * 0.001).abs()).fold(0.0f32, f32::max);
    let close_peak = (64..85).map(|ms| shutter_sound.amplitude_at((ms as f32) * 0.001).abs()).fold(0.0f32, f32::max);
    assert!(open_peak > 0.25 && close_peak > 0.20 && shutter_sound.riff_wave().starts_with(b"RIFF"));

    let world = world::BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn).with_starting_satchel(&ServerSatchel::assembled());
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::FieldCamera, HeldRelic::EmptyPalm));

    let mut base_reticle = QuadLedger::empty();
    avatar.emit_reticle(&mut base_reticle);
    let mut mesh_fwd = QuadLedger::empty();
    avatar.emit_appearance(&mut mesh_fwd);
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::FirstPersonSight));
    assert_eq!(SightMode::FirstPersonSight.photo_label(), "landscape");
    let tilt_shot_0 = avatar.polaroid_tilt();
    assert!(tilt_shot_0.abs() > 0.05, "Polaroid must be tilted at a diagonal angle");
    let mut polaroid_reticle = QuadLedger::empty();
    avatar.emit_reticle(&mut polaroid_reticle);
    assert!(polaroid_reticle.quads.len() > base_reticle.quads.len() + 300, "Polaroid overlay must appear centered on screen");
    let mut landscape_avatar = QuadLedger::empty();
    avatar.emit_appearance_for_sight(SightMode::FirstPersonSight, &mut landscape_avatar);
    assert!(!landscape_avatar.populated(), "Landscape photo must not be obstructed by the player's own body");

    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    let mut mesh_diag_fwd = QuadLedger::empty();
    avatar.emit_appearance(&mut mesh_diag_fwd);
    let v0_list = |l: &QuadLedger| -> Vec<Vec3> { l.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect() };
    assert_ne!(v0_list(&mesh_fwd), v0_list(&mesh_diag_fwd), "Rotating camera must visibly turn the camera to schräg vorne");
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::ForwardAngledSight));

    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    let mut mesh_side = QuadLedger::empty();
    avatar.emit_appearance(&mut mesh_side);
    assert_ne!(v0_list(&mesh_diag_fwd), v0_list(&mesh_side), "Rotating camera must visibly turn the camera to seitlich");
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::ProfileSight));
    assert_ne!(tilt_shot_0, avatar.polaroid_tilt(), "Consecutive Polaroids must appear at different tilt rotations");

    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    let mut mesh_diag_back = QuadLedger::empty();
    avatar.emit_appearance(&mut mesh_diag_back);
    assert_ne!(v0_list(&mesh_side), v0_list(&mesh_diag_back), "Rotating camera must visibly turn the camera to schräg hinten");
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::AngledSight));

    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::LookingGlassSight));
}

#[test]
fn test_placed_camera_runs_six_second_white_self_timer_countdown() {
    let mut world = world::BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn);
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::FieldCamera, HeldRelic::EmptyPalm));

    let mut base_reticle = QuadLedger::empty();
    avatar.emit_reticle(&mut base_reticle);
    avatar.rotate_degrees(0.0, 58.0);
    assert!(!avatar.use_right_hand(&mut world) && avatar.release_hand(&mut world));

    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Idle, "Placed camera must use 6s self-timer countdown");
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
    assert_eq!(cd_large.quads.last().unwrap().transfer(|_, _, _, _, _, col| col), Vec3::ONE, "Countdown digits must be white");
    avatar.step_motion(5.3, &KeyMotion::idle(), &world);
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::LookingGlassSight));
}

#[test]
fn test_selfie_camera_tracks_held_elevation_and_develops_png_plate() {
    let world = world::BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn);
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::FieldCamera, HeldRelic::EmptyPalm));

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
    assert!(max_ndc_y < 0.92, "High selfie camera angle must keep the player's head inside the top of the frame");

    let temp_dir = std::env::temp_dir().join("lowcube_test_screenshots");
    let path = PhotoPlate::new(4, 3, vec![180u8; 4 * 3 * 4]).develop_into(temp_dir.to_str().unwrap(), "front");
    let bytes = std::fs::read(&path).expect("PNG screenshot must exist on disk");
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n") && bytes.ends_with(b"IEND\xaeB`\x82"));
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_camera_on_three_dirt_blocks_photographs_from_tripod_lens_and_picks_up_in_one_click() {
    let mut world = world::BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn)
        .with_starting_satchel(&ParsedSatchel::parse("excavated_earth:3", "test"));
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::FieldCamera));
    avatar.rotate_degrees(0.0, 58.0);
    for pitch_up in [-12.0, -16.0, -19.0] {
        assert!(!avatar.use_right_hand(&mut world) && avatar.release_hand(&mut world));
        avatar.rotate_degrees(0.0, pitch_up);
    }
    assert!(!avatar.use_left_hand(&mut world) && avatar.release_hand(&mut world), "Placing FieldCamera on top of 3 dirt blocks must succeed");
    let (tripod_hub, _, _) = world.placed_camera_tripod().expect("World must report placed camera tripod hub");
    assert!(
        tripod_hub.y > spawn.y + 0.7,
        "Camera tripod hub ({tripod_hub:?}) must sit atop the 3 stacked dirt blocks above spawn ({spawn:?})"
    );

    assert!(
        !avatar.adjust_aimed_rig(InputTrigger::WheelClick, &mut world),
        "Middle-clicking (WheelClick) while camera is placed must NOT rotate the camera via adjust_aimed_rig"
    );
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
    assert!(
        (shot_eye - tripod_hub).length() < 0.35,
        "Placed camera photo must be captured from the tripod lens on the 3 dirt blocks ({tripod_hub:?}), got {shot_eye:?}"
    );
    let live_eye = avatar.view_projection_matrix(16.0 / 9.0).inverse().project_point3(Vec3::new(0.0, 0.0, -1.0));
    assert!((live_eye - shot_eye).length() > 1.5, "Live player view must remain at the player's own eyes while walking away");

    let mut pickup_avatar = PlanetaryAvatar::at(spawn);
    pickup_avatar.rotate_degrees(0.0, -6.0);
    assert!(
        pickup_avatar.use_right_hand(&mut world),
        "Single click on placed camera must pick it up immediately as an item without digging stages"
    );
    assert!(world.placed_camera_tripod().is_none(), "Picked up camera must be removed from the world immediately");
    assert!(pickup_avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(pickup_avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::FirstPersonSight));
}
