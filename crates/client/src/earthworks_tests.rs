use glam::Vec3;
use world::{BiomedCubeWorld, MeshTarget, PlanetaryWorld, Quad};
use crate::attire::{HeldRelic, YouthAttire};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::KeyMotion;
use crate::troupe::TroupeAvatar;

struct SurfaceLedger { quads: Vec<Quad> }
impl SurfaceLedger {
    fn empty() -> Self { Self { quads: Vec::new() } }
    fn count(&self) -> usize { self.quads.len() }
}
impl MeshTarget for SurfaceLedger { fn push_quad(&mut self, quad: Quad) { self.quads.push(quad); } }

#[test]
fn test_digging_speed_one_hand_vs_two_hands_vs_spade_and_abort_on_release() {
    let (mut world_one_hand, mut world_two_hands, mut world_spade) = (BiomedCubeWorld::beveled(16, 1.0), BiomedCubeWorld::beveled(16, 1.0), BiomedCubeWorld::beveled(16, 1.0));
    let spawn = world_one_hand.constrain_to_surface(Vec3::new(0.0, 9.0, 0.0), 1.0);

    let (mut avatar_one, mut avatar_two, mut avatar_spade) = (PlanetaryAvatar::at(spawn), PlanetaryAvatar::at(spawn), PlanetaryAvatar::at(spawn));
    avatar_one.rotate_view(0.0, 380.0);
    avatar_one.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::EmptyPalm));
    for _ in 0..7 { assert!(!avatar_one.use_left_hand(&mut world_one_hand)); }
    assert!(avatar_one.use_left_hand(&mut world_one_hand), "Empty left hand must dig with bare hand (8 strikes) even when right hand holds IronSpade");

    avatar_two.rotate_view(0.0, 380.0);
    for _ in 0..4 { assert!(!avatar_two.use_right_hand(&mut world_two_hands)); let _ = avatar_two.release_hand(&mut world_two_hands); }
    assert!(!avatar_two.use_left_hand(&mut world_two_hands) && !avatar_two.use_right_hand(&mut world_two_hands));
    for _ in 0..2 { assert!(!avatar_two.use_right_hand(&mut world_two_hands)); }
    assert!(avatar_two.use_right_hand(&mut world_two_hands), "Two free hands digging with both mouse buttons held must excavate on the 4th strike");

    avatar_spade.rotate_view(0.0, 410.0);
    avatar_spade.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::EmptyPalm));
    assert!(!avatar_spade.use_right_hand(&mut world_spade), "IronSpade 1st strike loosens earth without excavating yet");
    assert!(avatar_spade.release_hand(&mut world_spade), "Releasing mouse button early must abort digging and clear fissures");
    assert!(!avatar_spade.use_right_hand(&mut world_spade), "After aborting on release, next strike starts fresh at strike 1");
    for _ in 0..40 { avatar_spade.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world_spade); let _ = avatar_spade.step_digging(&mut world_spade); }
    assert!(!avatar_spade.release_hand(&mut world_spade), "Releasing right button after excavating must not place the block");
    let target_probe = Vec3::new(0.0, 9.0, -0.75);
    let height_after_spade = world_spade.constrain_to_surface(target_probe, 0.0).y;
    assert!(!avatar_spade.use_left_hand(&mut world_spade), "Pressing left button while holding block must not immediately place it");
    for _ in 0..270 { avatar_spade.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world_spade); let _ = avatar_spade.step_digging(&mut world_spade); }
    assert!(!avatar_spade.release_hand(&mut world_spade), "Releasing left button after hold-to-dig must not place held block");
    assert!((height_after_spade - world_spade.constrain_to_surface(target_probe, 0.0).y - 0.5).abs() < 1e-4, "Holding mouse button while holding a block in hand must excavate the block in front rather than placing the held block");
    assert!(!avatar_spade.use_left_hand(&mut world_spade) && avatar_spade.release_hand(&mut world_spade), "Short click (press + release) with block in hand must place it");
}

#[test]
fn test_mined_block_placed_only_with_same_hand_and_terrain_and_placed_blocks_modifiable() {
    let (mut world, spawn, target_probe) = (BiomedCubeWorld::beveled(16, 1.0), Vec3::new(0.0, 16.5, 0.0), Vec3::new(0.0, 9.0, -0.75));
    let initial_height = world.constrain_to_surface(target_probe, 0.0).y;
    let mut avatar = PlanetaryAvatar::at(world.constrain_to_surface(spawn, 1.0));
    avatar.rotate_view(0.0, 410.0);
    let (mut empty_hands_mesh, mut empty_satchel_mesh, mut held_block_mesh, mut digging_with_block_mesh) = (SurfaceLedger::empty(), SurfaceLedger::empty(), SurfaceLedger::empty(), SurfaceLedger::empty());
    avatar.emit_appearance(&mut empty_hands_mesh);
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::Tab) && { avatar.emit_appearance(&mut empty_satchel_mesh); empty_satchel_mesh.count() > empty_hands_mesh.count() } && avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::Tab));

    assert!(!avatar.use_right_hand(&mut world) && !avatar.use_left_hand(&mut world));
    for _ in 0..2 { assert!(!avatar.use_left_hand(&mut world)); }
    assert!(avatar.use_left_hand(&mut world) && (initial_height - world.constrain_to_surface(target_probe, 0.0).y - 0.5).abs() < 1e-4);
    avatar.emit_appearance(&mut held_block_mesh);
    assert!(held_block_mesh.count() > empty_hands_mesh.count() && !avatar.use_right_hand(&mut world));
    let _ = avatar.release_hand(&mut world);
    assert!(!avatar.use_left_hand(&mut world) && { avatar.emit_appearance(&mut digging_with_block_mesh); digging_with_block_mesh.count() == empty_hands_mesh.count() } && avatar.release_hand(&mut world) && (world.constrain_to_surface(target_probe, 0.0).y - initial_height).abs() < 1e-4);

    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::EmptyPalm));
    assert!(!avatar.use_right_hand(&mut world) && avatar.use_right_hand(&mut world) && world.constrain_to_surface(target_probe, 0.0).y < initial_height);
    avatar.scroll_mold(1.0, 0.0);
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyR));
    let (mut wedge_fwd_mesh, mut wedge_right_mesh) = (SurfaceLedger::empty(), SurfaceLedger::empty());
    avatar.emit_appearance(&mut wedge_fwd_mesh);
    avatar.turn_mold_clockwise();
    avatar.emit_appearance(&mut wedge_right_mesh);
    let vertices = |ledger: &SurfaceLedger| -> Vec<Vec3> { ledger.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect() };
    assert_ne!(vertices(&wedge_fwd_mesh), vertices(&wedge_right_mesh), "Rotating molded block must visibly rotate block in hand");
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyC) && !avatar.use_left_hand(&mut world) && avatar.release_hand(&mut world));
    assert!(world.constrain_to_surface(Vec3::new(0.25, 9.0, -0.95), 0.0).y > world.constrain_to_surface(Vec3::new(0.25, 9.0, -0.55), 0.0).y + 0.3);
    assert!(!avatar.use_right_hand(&mut world) && avatar.use_right_hand(&mut world) && !avatar.use_left_hand(&mut world) && avatar.release_hand(&mut world));
    assert!(world.constrain_to_surface(Vec3::new(0.25, 9.0, -0.95), 0.0).y > world.constrain_to_surface(Vec3::new(0.25, 9.0, -0.55), 0.0).y + 0.3);
    assert!(!avatar.use_right_hand(&mut world) && avatar.use_right_hand(&mut world));
    for _ in 0..2 { avatar.scroll_mold(-1.0, 0.0); }
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyQ) && avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyE));
    let (mut closed_mesh, mut open_mesh) = (SurfaceLedger::empty(), SurfaceLedger::empty());
    avatar.emit_appearance(&mut closed_mesh);
    assert_eq!(closed_mesh.count(), empty_hands_mesh.count(), "Stowed blocks and tools must be invisible while the satchel is closed");
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::Tab) && { avatar.emit_appearance(&mut open_mesh); empty_satchel_mesh.count() > empty_hands_mesh.count() + 100 && open_mesh.count() > empty_satchel_mesh.count() });
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyE) && avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyQ));
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyD) && avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyQ) && !avatar.use_left_hand(&mut world) && avatar.release_hand(&mut world));

    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::EmptyPalm));
    assert!(!avatar.use_right_hand(&mut world) && avatar.release_hand(&mut world) && (world.constrain_to_surface(target_probe, 0.0).y - (initial_height + 0.5)).abs() < 1e-4);
    assert!(!avatar.use_left_hand(&mut world) && !avatar.use_right_hand(&mut world));
    for _ in 0..2 { assert!(!avatar.use_right_hand(&mut world)); }
    assert!(avatar.use_right_hand(&mut world) && !avatar.release_hand(&mut world) && (world.constrain_to_surface(target_probe, 0.0).y - initial_height).abs() < 1e-4);
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::QuarriedStone, HeldRelic::EmptyPalm));
    for _ in 0..2 { avatar.scroll_mold(1.0, 0.0); }
    avatar.turn_mold_clockwise();
    for _ in 0..7 { assert!(!avatar.use_left_hand(&mut world)); }
    assert!(avatar.use_left_hand(&mut world) && !avatar.release_hand(&mut world), "Second hand excavating earth inherits first hand's mold and rotation regardless of material");
    assert!(!avatar.use_left_hand(&mut world) && avatar.release_hand(&mut world) && world.constrain_to_surface(Vec3::new(0.45, 9.0, -0.75), 0.0).y > world.constrain_to_surface(Vec3::new(0.05, 9.0, -0.75), 0.0).y + 0.3);
    avatar.scroll_mold(-1.0, 0.0);
    assert!(!avatar.use_right_hand(&mut world) && avatar.release_hand(&mut world) && (world.constrain_to_surface(target_probe, 0.0).y - (initial_height + 0.25)).abs() < 1e-4);
}

#[test]
fn test_two_free_hands_swing_both_arms_and_troupe_avatar_excavates_and_places() {
    let mut world = BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 9.0, 0.0), 1.0);
    let controls = KeyMotion::idle();
    let mut avatar = TroupeAvatar::near_spawn(Box::new(PlanetaryAvatar::at(spawn)), spawn, &world, YouthAttire::squire());
    avatar.rotate_view(0.0, 380.0);

    let (mut idle_mesh, mut digging_mesh) = (SurfaceLedger::empty(), SurfaceLedger::empty());
    avatar.emit_appearance(&mut idle_mesh);
    assert!(!avatar.use_left_hand(&mut world) && !avatar.use_right_hand(&mut world));
    for _ in 0..6 { avatar.step_motion(1.0 / 60.0, &controls, &world); }
    avatar.emit_appearance(&mut digging_mesh);
    let first_v0 = |ledger: &SurfaceLedger| -> Vec<Vec3> { ledger.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect() };
    assert_ne!(first_v0(&idle_mesh), first_v0(&digging_mesh));
    for _ in 0..2 { assert!(!avatar.use_right_hand(&mut world)); }
    assert!(avatar.use_right_hand(&mut world) && !avatar.release_hand(&mut world));
    assert!(!avatar.use_right_hand(&mut world) && avatar.release_hand(&mut world));
}

#[test]
fn test_spade_in_left_hand_with_empty_right_hand_excavates_sloped_block_cleanly() {
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let mut initial_batch = engine::CpuMeshBatch::empty();
    world.emit_surface(&mut initial_batch);

    let (mut slope_u, mut slope_z, mut slope_mid_y) = (0.0f32, -4.25f32, 0.0f32);
    'search: for cu in -12..12 {
        for cv in -12..-5 {
            let (u, v) = ((cu as f32) * 0.5 + 0.25, (cv as f32) * 0.5 + 0.25);
            let sample_y = world.constrain_to_surface(Vec3::new(u, 18.0, v), 0.0).y - 16.0;
            if (0.10..0.40).contains(&sample_y.rem_euclid(0.5)) { (slope_u, slope_z, slope_mid_y) = (u, v, sample_y); break 'search; }
        }
    }
    assert!(slope_mid_y > 0.0, "Must locate a sloped ramp block on the Temperate face");
    let stand_pos = world.constrain_to_surface(Vec3::new(slope_u, 18.0, slope_z + 0.75), 1.0);
    let (controls, attire) = (KeyMotion::idle(), YouthAttire::squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::IronSpade));
    let mut avatar = TroupeAvatar::near_spawn(Box::new(PlanetaryAvatar::at(stand_pos)), stand_pos, &world, attire);
    avatar.apply_attire(attire);
    avatar.rotate_view(0.0, 380.0);
    assert!(!avatar.use_left_hand(&mut world) && avatar.use_left_hand(&mut world), "Spade in left hand with empty right hand must excavate the sloped block on 2nd strike");
    assert!(world.constrain_to_surface(Vec3::new(slope_u, 18.0, slope_z), 0.0).y - 16.0 < slope_mid_y);
    assert_eq!(world.face_count(), 6, "Cube planet must expose 6 independent face slices");
    let (mut full_planet_ledger, mut modified_face_ledger) = (SurfaceLedger::empty(), SurfaceLedger::empty());
    world.emit_surface(&mut full_planet_ledger);
    assert_eq!(world.emit_modified_face(&mut modified_face_ledger), 0, "Temperate (+Y) face must be identified as the modified face");
    assert!(modified_face_ledger.count() > 0 && modified_face_ledger.count() * 4 < full_planet_ledger.count());

    for _ in 0..10 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
        let mut frame_batch = SurfaceLedger::empty();
        avatar.emit_appearance(&mut frame_batch);
        assert!(frame_batch.count() > 0);
    }
    assert!(!avatar.use_right_hand(&mut world) && avatar.release_hand(&mut world), "Mined block in right hand must be placeable with right hand");
}

#[test]
fn test_crosshair_aim_excavates_within_three_blocks_and_rejects_beyond_three_blocks() {
    let mut world = BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 9.0, 0.0), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn);
    let mut idle_avatar = SurfaceLedger::empty();
    avatar.emit_appearance(&mut idle_avatar);

    // 1. Aim at a block 4 blocks ahead (z = -2.25, cell cv = -5 while eye is in cv = -1): must NOT excavate or emit debris
    avatar.rotate_view(0.0, 234.0);
    let far_probe = Vec3::new(0.0, 9.0, -2.25);
    let initial_far = world.constrain_to_surface(far_probe, 0.0).y;
    assert!(!avatar.use_left_hand(&mut world));
    let mut air_swing = SurfaceLedger::empty();
    avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    avatar.emit_appearance(&mut air_swing);
    assert_eq!(air_swing.count(), idle_avatar.count(), "Swinging in air must not emit soil clod debris");
    assert!((world.constrain_to_surface(far_probe, 0.0).y - initial_far).abs() < 1e-4);
    avatar.rotate_view(0.0, 41.0);
    let (closer_probe, aimed_probe) = (Vec3::new(0.0, 9.0, -1.25), Vec3::new(0.0, 9.0, -1.75));
    let (initial_closer, initial_aimed) = (world.constrain_to_surface(closer_probe, 0.0).y, world.constrain_to_surface(aimed_probe, 0.0).y);
    let (mut untouched_face, mut loosened_face, mut swing_mesh) = (SurfaceLedger::empty(), SurfaceLedger::empty(), SurfaceLedger::empty());
    world.emit_modified_face(&mut untouched_face);
    assert!(!avatar.use_right_hand(&mut world));
    assert!(!avatar.use_left_hand(&mut world));
    world.emit_modified_face(&mut loosened_face);
    assert!(loosened_face.count() > untouched_face.count(), "First digging strike must emit 3D crater on the aimed block");
    avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    avatar.emit_appearance(&mut swing_mesh);
    assert!(swing_mesh.count() > air_swing.count(), "Digging a block must emit soil clod debris above the block");
    for _ in 0..2 { assert!(!avatar.use_left_hand(&mut world)); }
    assert!(avatar.use_left_hand(&mut world), "Block within 3 blocks must be excavated when aimed at");
    assert!((world.constrain_to_surface(closer_probe, 0.0).y - initial_closer).abs() < 1e-4);
    assert!((initial_aimed - world.constrain_to_surface(aimed_probe, 0.0).y - 0.5).abs() < 1e-4);
}
