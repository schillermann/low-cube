use glam::Vec3;
use world::{BiomedCubeWorld, MeshTarget, PlanetaryWorld, Quad};
use crate::attire::{HeldRelic, YouthAttire};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::KeyMotion;
use crate::troupe::TroupeAvatar;

struct SurfaceLedger {
    quads: Vec<Quad>,
}

impl SurfaceLedger {
    fn empty() -> Self {
        Self { quads: Vec::new() }
    }

    fn count(&self) -> usize {
        self.quads.len()
    }
}

impl MeshTarget for SurfaceLedger {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

#[test]
fn test_digging_speed_one_hand_vs_two_hands_vs_shovel() {
    let mut world_one_hand = BiomedCubeWorld::beveled(16, 1.0);
    let mut world_two_hands = BiomedCubeWorld::beveled(16, 1.0);
    let mut world_shovel = BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world_one_hand.constrain_to_surface(Vec3::new(0.0, 9.0, 0.0), 1.0);

    // 1. One free hand (right hand holds WoodmanAxe, left hand is EmptyPalm): requires 4 strikes
    let mut avatar_one = PlanetaryAvatar::at(spawn);
    avatar_one.apply_attire(YouthAttire::squire().with_relics(HeldRelic::WoodmanAxe, HeldRelic::EmptyPalm));
    assert!(!avatar_one.use_left_hand(&mut world_one_hand));
    assert!(!avatar_one.use_left_hand(&mut world_one_hand));
    assert!(!avatar_one.use_left_hand(&mut world_one_hand));
    assert!(avatar_one.use_left_hand(&mut world_one_hand), "Single free hand must excavate earth on the 4th strike");

    // 2. Both hands free (EmptyPalm in both): both hands dig together, requiring only 2 strikes
    let mut avatar_two = PlanetaryAvatar::at(spawn);
    assert!(!avatar_two.use_right_hand(&mut world_two_hands));
    assert!(avatar_two.use_right_hand(&mut world_two_hands), "Two free hands digging together must excavate on the 2nd strike");

    // 3. Shovel (IronShovel): fastest, excavating earth in a single strike
    let mut avatar_shovel = PlanetaryAvatar::at(spawn);
    avatar_shovel.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronShovel, HeldRelic::EmptyPalm));
    assert!(avatar_shovel.use_right_hand(&mut world_shovel), "IronShovel must excavate earth in 1 single strike");
}

#[test]
fn test_mined_block_placed_only_with_same_hand_and_terrain_and_placed_blocks_modifiable() {
    let mut world = BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 9.0, 0.0), 1.0);
    let target_probe = Vec3::new(0.0, 9.0, -1.0);
    let initial_height = world.constrain_to_surface(target_probe, 0.0).y;

    let mut avatar = PlanetaryAvatar::at(spawn);
    let mut empty_hands_mesh = SurfaceLedger::empty();
    avatar.emit_appearance(&mut empty_hands_mesh);

    // Dig with both hands free using left hand action (2 strikes -> block goes into left hand)
    assert!(!avatar.use_left_hand(&mut world));
    assert!(avatar.use_left_hand(&mut world));

    let excavated_height = world.constrain_to_surface(target_probe, 0.0).y;
    assert!((initial_height - excavated_height - 0.5).abs() < 1e-4, "Excavating terrain must lower target cell by 0.5m");

    let mut held_block_mesh = SurfaceLedger::empty();
    avatar.emit_appearance(&mut held_block_mesh);
    assert!(held_block_mesh.count() > empty_hands_mesh.count(), "Left hand must visibly hold the excavated earth block");

    // Right hand is still free (does not hold the block), so right hand action digs instead of placing
    assert!(!avatar.use_right_hand(&mut world), "Right hand is empty and must dig rather than place left hand's block");

    // Left hand holds the excavated block, so left hand action places the block back
    assert!(avatar.use_left_hand(&mut world), "Left hand holding the excavated block must place it back");
    let restored_height = world.constrain_to_surface(target_probe, 0.0).y;
    assert!((restored_height - initial_height).abs() < 1e-4);

    // Equip shovel in right hand, dig block into left hand, place back, then stack above ground
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronShovel, HeldRelic::EmptyPalm));
    assert!(avatar.use_right_hand(&mut world));
    assert!(world.constrain_to_surface(target_probe, 0.0).y < initial_height);
    assert!(avatar.use_left_hand(&mut world));

    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::EmptyPalm));
    assert!(avatar.use_right_hand(&mut world), "Right hand holding ExcavatedEarth must place a block above terrain");
    let stacked_height = world.constrain_to_surface(target_probe, 0.0).y;
    assert!((stacked_height - (initial_height + 0.5)).abs() < 1e-4);

    // Now excavate the player-placed block back down with two free hands
    assert!(!avatar.use_right_hand(&mut world));
    assert!(avatar.use_right_hand(&mut world), "Player-placed block must be breakable back down to original terrain");
    assert!((world.constrain_to_surface(target_probe, 0.0).y - initial_height).abs() < 1e-4);
}

#[test]
fn test_two_free_hands_swing_both_arms_and_troupe_avatar_excavates_and_places() {
    let mut world = BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 9.0, 0.0), 1.0);
    let controls = KeyMotion::idle();

    let mut avatar = TroupeAvatar::near_spawn(
        Box::new(PlanetaryAvatar::at(spawn)),
        spawn,
        &world,
        YouthAttire::squire(),
    );

    let mut idle_mesh = SurfaceLedger::empty();
    avatar.emit_appearance(&mut idle_mesh);

    assert!(!avatar.use_right_hand(&mut world));
    for _ in 0..6 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let mut digging_mesh = SurfaceLedger::empty();
    avatar.emit_appearance(&mut digging_mesh);
    let first_v0 = |ledger: &SurfaceLedger| -> Vec<Vec3> {
        ledger.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect()
    };
    assert_ne!(first_v0(&idle_mesh), first_v0(&digging_mesh));
    assert!(avatar.use_right_hand(&mut world));
    assert!(avatar.use_right_hand(&mut world));
}

#[test]
fn test_shovel_in_left_hand_with_empty_right_hand_excavates_sloped_block_cleanly() {
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let mut initial_batch = engine::CpuMeshBatch::empty();
    world.emit_surface(&mut initial_batch);

    // Locate a sloped block (45° ramp cell where mid-cell surface height lies strictly between 0.5m increments)
    let mut slope_u = 0.0f32;
    let mut slope_z = -4.25f32;
    let mut slope_mid_y = 0.0f32;
    'search: for cu in -12..12 {
        for cv in -12..-5 {
            let (u, v) = ((cu as f32) * 0.5 + 0.25, (cv as f32) * 0.5 + 0.25);
            let sample_y = world.constrain_to_surface(Vec3::new(u, 18.0, v), 0.0).y - 16.0;
            let rem = sample_y.rem_euclid(0.5);
            if rem > 0.10 && rem < 0.40 {
                (slope_u, slope_z, slope_mid_y) = (u, v, sample_y);
                break 'search;
            }
        }
    }
    assert!(slope_mid_y > 0.0, "Must locate a sloped ramp block on the Temperate face");

    // Position avatar 0.75m south of the sloped block, looking north (-Z) directly at the sloped block
    let stand_pos = world.constrain_to_surface(Vec3::new(slope_u, 18.0, slope_z + 0.75), 1.0);
    let controls = KeyMotion::idle();
    let attire = YouthAttire::squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::IronShovel);
    let mut avatar = TroupeAvatar::near_spawn(Box::new(PlanetaryAvatar::at(stand_pos)), stand_pos, &world, attire);
    avatar.apply_attire(attire);
    avatar.rotate_view(0.0, 220.0);

    // Mine the sloped block with the shovel in the left hand while the right hand is empty
    assert!(avatar.use_left_hand(&mut world), "Shovel in left hand with empty right hand must excavate the sloped block");

    let after_mid_y = world.constrain_to_surface(Vec3::new(slope_u, 18.0, slope_z), 0.0).y - 16.0;
    assert!(after_mid_y < slope_mid_y, "Excavating the sloped block must lower its surface height");

    assert_eq!(world.face_count(), 6, "Cube planet must expose 6 independent face slices");
    let mut full_planet_ledger = SurfaceLedger::empty();
    world.emit_surface(&mut full_planet_ledger);
    let mut modified_face_ledger = SurfaceLedger::empty();
    let modified_idx = world.emit_modified_face(&mut modified_face_ledger);
    assert_eq!(modified_idx, 0, "Temperate (+Y) face must be identified as the modified face");
    assert!(
        modified_face_ledger.count() > 0 && modified_face_ledger.count() * 4 < full_planet_ledger.count(),
        "emit_modified_face must emit only the single modified planetary face rather than all 6 faces"
    );

    for _ in 0..10 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
        let mut frame_batch = SurfaceLedger::empty();
        avatar.emit_appearance(&mut frame_batch);
        assert!(frame_batch.count() > 0);
    }

    // The mined block landed in the previously empty right hand, so right-hand action places it back
    assert!(avatar.use_right_hand(&mut world), "Mined block in right hand must be placeable with right hand");
}
