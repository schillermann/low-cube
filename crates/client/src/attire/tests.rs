use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{
    ClimateZone, CubePlanet, CuboidShape, FacetedReliefFace, MeshTarget, NullFace, PlanetaryWorld, Quad,
};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};
use super::{
    AttireProfile, BodyPose, FileAttire, HeldRelic, SightMode, StrideRhythm, Wardrobe, YouthAttire,
    YouthPhysique,
};

struct QuadLedger { quads: Vec<Quad> }

impl QuadLedger {
    fn empty() -> Self { Self { quads: Vec::new() } }
    fn populated(&self) -> bool { !self.quads.is_empty() }
    fn exceeds(&self, other: &Self) -> bool { self.quads.len() > other.quads.len() }
    fn visible_in_frustum(&self, vp: glam::Mat4) -> usize {
        self.quads.iter().filter(|q| {
            q.transfer(|v0, _, _, _, _, _| {
                let clip = vp * v0.extend(1.0);
                clip.w > 0.03 && (clip.x / clip.w).abs() < 1.0 && (clip.y / clip.w).abs() < 1.0 && (clip.z / clip.w).abs() <= 1.0
            })
        }).count()
    }
}

impl MeshTarget for QuadLedger {
    fn push_quad(&mut self, quad: Quad) { self.quads.push(quad); }
}

fn world_with_cube_at(u: i32, v: i32) -> CubePlanet {
    let face = Box::new(
        FacetedReliefFace::in_zone(ClimateZone::Temperate, 8.0, 16)
            .with_molded_block(u, v, 0.0, Box::new(CuboidShape::full_cube())),
    );
    CubePlanet::new(vec![face], Box::new(NullFace))
}

struct WalkEastControls;

impl KeyboardControls for WalkEastControls {
    fn register_key(&mut self, _code: KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, _forward: Vec3, right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(right);
    }
}

#[test]
fn test_body_envelope_prevents_camera_from_clipping_into_block_wall() {
    let world = world_with_cube_at(1, 0);
    let start = world.constrain_to_surface(Vec3::new(0.0, 5.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(start);
    let controls = WalkEastControls;
    for _ in 0..60 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
    let cam_pos = view_proj.inverse().project_point3(Vec3::ZERO);
    let wall_clearance = 0.5 - cam_pos.x;
    assert!(wall_clearance >= 0.15, "Avatar camera must maintain safe body clearance, got {}", wall_clearance);
}

#[test]
fn test_first_person_omits_head_while_multiplayer_and_mirror_include_crown() {
    let pose = BodyPose::new(
        Vec3::new(0.0, 9.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, 0.8,
        StrideRhythm::new(0.0, 1.0), YouthPhysique::SlenderSquire,
    );
    let attire = YouthAttire::squire();
    let mut fp_ledger = QuadLedger::empty();
    attire.emit(&pose, SightMode::FirstPersonSight, &mut fp_ledger);
    let mut mp_ledger = QuadLedger::empty();
    attire.emit_peer(&pose, &mut mp_ledger);
    assert!(fp_ledger.populated(), "First-person view must emit torso, legs, boots, and hands");
    assert!(mp_ledger.exceeds(&fp_ledger), "Multiplayer and Looking-Glass view must emit additional crown quads");
}

#[test]
fn test_hands_raised_only_when_holding_item_and_body_visible_when_looking_down() {
    let start = Vec3::new(0.0, 9.0, 0.0);
    let mut avatar = PlanetaryAvatar::at(start);
    let mut empty_ledger = QuadLedger::empty();
    avatar.emit_appearance(&mut empty_ledger);
    let vp_level = avatar.view_projection_matrix(16.0 / 9.0);
    let visible_empty_level = empty_ledger.visible_in_frustum(vp_level);
    assert_eq!(visible_empty_level, 0, "Empty hands must hang at the sides and not obstruct level forward view");

    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::WoodmanAxe, HeldRelic::EmptyPalm));
    let mut held_ledger = QuadLedger::empty();
    avatar.emit_appearance(&mut held_ledger);
    let visible_held_level = held_ledger.visible_in_frustum(vp_level);
    assert!(visible_held_level > 0, "Holding an item must raise the hand into first-person level view");

    avatar.rotate_view(0.0, 350.0);
    let vp_down = avatar.view_projection_matrix(16.0 / 9.0);
    let mut down_ledger = QuadLedger::empty();
    avatar.emit_appearance(&mut down_ledger);
    let visible_down = down_ledger.visible_in_frustum(vp_down);
    assert!(visible_down > visible_held_level, "Looking down must bring torso, belt, legs, and boots into the camera view frustum");
}

#[test]
fn test_wardrobe_customization_keys_and_file_persistence() {
    let temp_path = std::env::temp_dir().join("test_medieval_character_profile.toml");
    let store = FileAttire::new(temp_path.to_str().unwrap().to_string());
    let squire = YouthAttire::squire();
    let initial_bytes = store.encode_multiplayer(&squire);
    let customized = squire
        .with_next_tunic().with_next_vambrace().with_next_girdle().with_next_footwear()
        .with_next_crown().with_next_relic().with_next_pigment().with_next_complexion();
    store.save(&customized);
    let loaded = store.load_or_squire();
    std::fs::remove_file(&temp_path).expect("Failed to clean up temporary profile");
    assert_eq!(customized, loaded);
    assert_ne!(initial_bytes, store.encode_multiplayer(&loaded));
}

#[test]
fn test_foraging_deadwood_and_flint_into_hand_or_satchel_crafts_axe_in_crafting_book() {
    let mut world = world::BiomedCubeWorld::beveled(32, 1.0);
    let mut probe_world = world::BiomedCubeWorld::beveled(32, 1.0);
    assert_eq!(probe_world.forage_surface(Vec3::new(-1.25, 17.0, -0.55), Vec3::new(0.0, -0.8, -0.6)), world::SurfaceFind::EmptyHanded);
    let (mut bough_spot, mut flint_spot) = (Vec3::ZERO, Vec3::ZERO);
    for cu in (-24..=24).step_by(6) {
        for cv in (-24..=24).step_by(6) {
            for (du, dv) in [(0, 0), (2, 0), (-2, 0), (0, 2), (0, -2)] {
                let (u, v) = (((cu + du) as f32) * 0.5 + 0.25, ((cv + dv) as f32) * 0.5 + 0.25);
                let g = probe_world.constrain_to_surface(Vec3::new(u, 22.0, v), 0.0);
                let find = probe_world.forage_surface(g + Vec3::new(0.0, 1.2, 0.7), Vec3::new(0.0, -1.15, -0.70).normalize());
                if find == world::SurfaceFind::DeadwoodBough && bough_spot == Vec3::ZERO { bough_spot = Vec3::new(u, 22.0, v + 0.70); }
                if find == world::SurfaceFind::FlintStone && flint_spot == Vec3::ZERO { flint_spot = Vec3::new(u, 22.0, v + 0.70); }
            }
        }
    }
    assert!(bough_spot.length() > 4.0 && flint_spot.length() > 2.5, "Deadwood must spawn near trees and Flint near rocky slopes");
    let mut avatar = PlanetaryAvatar::at(world.constrain_to_surface(bough_spot, 1.0));
    avatar.rotate_view(0.0, 410.0);
    let (mut empty_mesh, mut deadwood_mesh, mut stowed_mesh, mut book_mesh, mut axe_mesh) = (
        QuadLedger::empty(), QuadLedger::empty(), QuadLedger::empty(), QuadLedger::empty(), QuadLedger::empty(),
    );
    avatar.emit_appearance(&mut empty_mesh);
    assert!(avatar.use_right_hand(&mut world) && { avatar.emit_appearance(&mut deadwood_mesh); deadwood_mesh.exceeds(&empty_mesh) });
    assert!(avatar.dispatch_wardrobe_key(KeyCode::KeyE) && { avatar.emit_appearance(&mut stowed_mesh); !stowed_mesh.exceeds(&empty_mesh) });
    let mut avatar_flint = PlanetaryAvatar::at(world.constrain_to_surface(flint_spot, 1.0));
    avatar_flint.rotate_view(0.0, 410.0);
    assert!(avatar_flint.use_left_hand(&mut world));
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::FlintStone));
    assert!(avatar.dispatch_wardrobe_key(KeyCode::KeyC) && { avatar.emit_appearance(&mut book_mesh); book_mesh.exceeds(&deadwood_mesh) });
    assert!(!avatar.use_right_hand(&mut world) && avatar.dispatch_wardrobe_key(KeyCode::KeyC));
    avatar.emit_appearance(&mut axe_mesh);
    assert!(axe_mesh.exceeds(&deadwood_mesh), "Crafting WoodmanAxe from stowed Deadwood + held Flint must equip the axe in hand");
    let (satchel, attire) = super::BeltSatchel::empty().collect_felled_timber(1.0, YouthAttire::squire().with_relics(HeldRelic::WoodmanAxe, HeldRelic::EmptyPalm));
    assert_eq!(attire.held_pair(), (HeldRelic::WoodmanAxe, HeldRelic::TimberBlock));
    assert_eq!(satchel.resource_tally(HeldRelic::TimberBlock, attire), 3, "Felling a tree must yield 3 WOOD BLOCKs into hand + satchel");
    let (_, satchel_planks, attire_planks, crafted) = super::ArtisanTome::new(1, true).craft_selected(-1.0, satchel, attire);
    assert!(crafted && attire_planks.held_pair() == (HeldRelic::WoodmanAxe, HeldRelic::PlankBlock));
    assert_eq!(satchel_planks.resource_tally(HeldRelic::TimberBlock, attire_planks), 2);
    assert_eq!(satchel_planks.resource_tally(HeldRelic::PlankBlock, attire_planks), 4, "Crafting WOOD PLANKS from 1 WOOD BLOCK must yield 4 planks");
    let (aim_eye, aim_dir) = (Vec3::new(0.25, 16.92, 0.25), Vec3::new(0.0, -0.75, -0.75).normalize());
    assert!(world.place_molded_surface_block(aim_eye, aim_dir, Vec3::new(0.25, 16.0, 0.25), world::BlockMold::cube().in_medium(world::BlockMedium::PlanedPlanks)));
    assert_eq!(world.harvest_surface(aim_eye, aim_dir).map(HeldRelic::from_medium), Some(HeldRelic::PlankBlock), "Excavating a placed WOOD PLANKS block must yield WOOD PLANKS back into the hand");
}

#[test]
fn test_spade_and_dirt_block_replaces_held_block_when_excavating_different_medium() {
    let dual_tools = YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::WoodmanAxe);
    let (satchel, next_attire) = super::BeltSatchel::empty().collect_excavated_block(1.0, HeldRelic::ExcavatedEarth, dual_tools);
    assert_eq!(next_attire.held_pair(), (HeldRelic::IronSpade, HeldRelic::WoodmanAxe));
    assert_eq!(satchel.resource_tally(HeldRelic::ExcavatedEarth, next_attire), 1);

    let spade_and_dirt = YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::ExcavatedEarth);
    let satchel = super::BeltSatchel::empty().reconcile_attire(spade_and_dirt);
    let crust_harvested = HeldRelic::from_medium(world::BlockMedium::CrustRock);
    assert_eq!(crust_harvested, HeldRelic::CrustBlock, "Excavating planetary crust rock must yield CrustBlock");
    let (satchel_after_crust, attire_after_crust) = satchel.collect_excavated_block(-1.0, crust_harvested, spade_and_dirt);
    assert_eq!(attire_after_crust.held_pair(), (HeldRelic::IronSpade, HeldRelic::CrustBlock));
    assert_eq!(satchel_after_crust.resource_tally(HeldRelic::ExcavatedEarth, attire_after_crust), 1);
    assert_eq!(satchel_after_crust.resource_tally(HeldRelic::CrustBlock, attire_after_crust), 1);
    let (satchel_same, attire_same) = satchel_after_crust.collect_excavated_block(-1.0, HeldRelic::CrustBlock, attire_after_crust);
    assert_eq!(attire_same.held_pair(), (HeldRelic::IronSpade, HeldRelic::CrustBlock));
    assert_eq!(satchel_same.resource_tally(HeldRelic::CrustBlock, attire_same), 2);
}

#[test]
fn test_field_camera_rotation_shutter_sound_and_server_satchel_provisioning() {
    use crate::attire::PhotoShutter;
    use crate::cardiac::{AcousticWave, ShutterClick};
    use crate::controls::InputTrigger;
    use crate::photo_plate::PhotoPlate;
    use crate::server_satchel::{ArgumentSatchel, FileSatchel, ParsedSatchel, ServerSatchel, StartingSatchel, TestKitSatchel};

    let test_satchel = TestKitSatchel::new().provision(super::BeltSatchel::empty());
    let squire = YouthAttire::squire();
    assert_eq!(test_satchel.resource_tally(HeldRelic::FieldCamera, squire), 1);

    let custom = ParsedSatchel::parse("field_camera:2, woodman_axe, car_wheel:4", "test").provision(super::BeltSatchel::empty());
    assert_eq!(custom.resource_tally(HeldRelic::FieldCamera, squire), 2);
    assert_eq!(custom.resource_tally(HeldRelic::WoodmanAxe, squire), 1);
    assert_eq!(custom.resource_tally(HeldRelic::CarWheel, squire), 4);
    assert_eq!(custom.resource_tally(HeldRelic::RocketEngine, squire), 0);

    let args = vec!["client".to_string(), "--inventory".to_string(), "camera,iron_spade:2".to_string()];
    let cli_satchel = ArgumentSatchel::from_arguments(&args, Box::new(FileSatchel::server(Box::new(TestKitSatchel::new()))))
        .provision(super::BeltSatchel::empty());
    assert_eq!(cli_satchel.resource_tally(HeldRelic::FieldCamera, squire), 1);
    assert_eq!(cli_satchel.resource_tally(HeldRelic::IronSpade, squire), 2);

    let shutter_sound = ShutterClick::mechanical();
    let open_peak = (1..15).map(|ms| shutter_sound.amplitude_at((ms as f32) * 0.001).abs()).fold(0.0f32, f32::max);
    let close_peak = (64..85).map(|ms| shutter_sound.amplitude_at((ms as f32) * 0.001).abs()).fold(0.0f32, f32::max);
    assert!(open_peak > 0.25 && close_peak > 0.20 && shutter_sound.riff_wave().starts_with(b"RIFF"));

    let mut world = world::BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn).with_starting_satchel(&ServerSatchel::assembled());
    avatar.apply_attire(squire.with_relics(HeldRelic::FieldCamera, HeldRelic::EmptyPalm));

    // 0. Camera pointing forward -> WheelClick starts 6s countdown (numbers shrink large->small) + Polaroid overlay
    let mut base_reticle = QuadLedger::empty();
    avatar.emit_reticle(&mut base_reticle);
    let mut mesh_fwd = QuadLedger::empty();
    avatar.emit_appearance(&mut mesh_fwd);
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Idle);
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Idle);
    let (mut cd_large, mut cd_small) = (QuadLedger::empty(), QuadLedger::empty());
    avatar.emit_reticle(&mut cd_large);
    avatar.step_motion(0.8, &crate::controls::KeyMotion::idle(), &world);
    avatar.emit_reticle(&mut cd_small);
    let span_y = |l: &QuadLedger| l.quads.iter().skip(base_reticle.quads.len()).map(|q| q.transfer(|v0, _, v2, _, _, _| v0.y.abs().max(v2.y.abs()))).fold(0.0f32, f32::max);
    assert!(cd_large.quads.len() > base_reticle.quads.len() && span_y(&cd_large) > span_y(&cd_small) + 0.04);
    avatar.step_motion(5.3, &crate::controls::KeyMotion::idle(), &world);
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

    // 1. Rotate camera diagonally (schräg) and sideways (seitlich) -> angled and side photos
    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    let mut mesh_diag = QuadLedger::empty();
    avatar.emit_appearance(&mut mesh_diag);
    let v0_list = |l: &QuadLedger| -> Vec<Vec3> { l.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect() };
    assert_ne!(v0_list(&mesh_fwd), v0_list(&mesh_diag), "Rotating camera must visibly turn the camera in the hand");
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    avatar.step_motion(6.1, &crate::controls::KeyMotion::idle(), &world);
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::AngledSight));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    avatar.step_motion(6.1, &crate::controls::KeyMotion::idle(), &world);
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::ProfileSight));
    let tilt_shot_1 = avatar.polaroid_tilt();
    assert_ne!(tilt_shot_0, tilt_shot_1, "Consecutive Polaroids must appear at different tilt rotations");
    let mut side_avatar = QuadLedger::empty();
    avatar.emit_appearance_for_sight(SightMode::ProfileSight, &mut side_avatar);
    assert!(side_avatar.visible_in_frustum(avatar.view_projection_for_sight(16.0 / 9.0, SightMode::ProfileSight)) > 20);

    // 2. Rotate camera directly toward player -> front photo + place camera on block & trigger countdown
    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    avatar.step_motion(6.1, &crate::controls::KeyMotion::idle(), &world);
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::LookingGlassSight));
    let vp_front_level = avatar.view_projection_for_sight(16.0 / 9.0, SightMode::LookingGlassSight);
    avatar.rotate_degrees(0.0, -35.0);
    let vp_front_high = avatar.view_projection_for_sight(16.0 / 9.0, SightMode::LookingGlassSight);
    avatar.rotate_degrees(0.0, 70.0);
    let vp_front_low = avatar.view_projection_for_sight(16.0 / 9.0, SightMode::LookingGlassSight);
    let cam_y_high = vp_front_high.inverse().project_point3(Vec3::ZERO).y;
    let cam_y_level = vp_front_level.inverse().project_point3(Vec3::ZERO).y;
    let cam_y_low = vp_front_low.inverse().project_point3(Vec3::ZERO).y;
    assert!(cam_y_high > cam_y_level + 0.25 && cam_y_level > cam_y_low + 0.25, "Camera angle and elevation must track held camera height");
    let mut high_selfie = QuadLedger::empty();
    avatar.emit_appearance_for_sight(SightMode::LookingGlassSight, &mut high_selfie);
    let max_ndc_y = high_selfie.quads.iter().map(|q| vp_front_high.project_point3(q.transfer(|v0, _, _, _, _, _| v0)).y).fold(-1.0f32, f32::max);
    assert!( max_ndc_y < 0.92, "High selfie camera angle must keep the player's head inside the top of the frame (got max_ndc_y={max_ndc_y})");
    avatar.rotate_degrees(0.0, 23.0);
    assert!(!avatar.use_right_hand(&mut world) && avatar.release_hand(&mut world));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    avatar.step_motion(6.1, &crate::controls::KeyMotion::idle(), &world);
    assert_eq!(avatar.take_photo_shutter(), PhotoShutter::Triggered(SightMode::LookingGlassSight));

    let temp_dir = std::env::temp_dir().join("lowcube_test_screenshots");
    let path = PhotoPlate::new(4, 3, vec![180u8; 4 * 3 * 4]).develop_into(temp_dir.to_str().unwrap(), "front");
    let bytes = std::fs::read(&path).expect("PNG screenshot must exist on disk");
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n") && bytes.ends_with(b"IEND\xaeB`\x82"));
    let _ = std::fs::remove_dir_all(&temp_dir);
}


