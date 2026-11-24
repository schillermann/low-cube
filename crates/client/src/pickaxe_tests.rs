use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, MeshTarget, MiningEdge, PlanetaryWorld, Quad,
    SatchelManifest, SurfaceFind, TerrainMesh as _, WorldEarthworks as _,
};
use crate::attire::{ArtisanTome, BeltSatchel, HeldRelic, YouthAttire};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarPresence as _, PlanetaryAvatar,
};
use crate::controls::KeyMotion;

struct QuadCounter {
    count: usize,
}

impl QuadCounter {
    const fn empty() -> Self {
        Self { count: 0 }
    }
}

impl MeshTarget for QuadCounter {
    fn push_quad(&mut self, _quad: Quad) {
        self.count += 1;
    }
}

#[test]
fn test_bare_hands_and_spade_reject_stone_and_ores_while_digging_earth_and_planks() {
    for hard in [
        BlockMedium::QuarriedStone,
        BlockMedium::CopperOre,
        BlockMedium::IronOre,
        BlockMedium::GoldOre,
        BlockMedium::CrustRock,
    ] {
        assert!(!hard.yields_to(MiningEdge::BarePalm), "{hard:?} must reject bare hands and spades");
    }
    for soft in [BlockMedium::TurfEarth, BlockMedium::Sand, BlockMedium::PlanedPlanks] {
        assert!(soft.yields_to(MiningEdge::BarePalm), "{soft:?} must remain diggable without pickaxe");
    }

    let mut world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let stand = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let probe = Vec3::new(0.25, 16.0, -0.75);
    let ground_y = world.constrain_to_surface(probe, 0.0).y;
    let aim_eye = Vec3::new(0.25, stand.y + 0.8, 0.25);
    let aim_dir = (Vec3::new(0.25, ground_y + 0.1, -0.75) - aim_eye).normalize();

    for medium in [BlockMedium::QuarriedStone, BlockMedium::GoldOre] {
        assert!(world.place_molded_surface_block(
            aim_eye,
            aim_dir,
            stand,
            BlockMold::cube().in_medium(medium),
        ));
        let top_before = world.constrain_to_surface(probe, 0.0).y;
        let mut miner = PlanetaryAvatar::at(stand);
        miner.rotate_degrees(0.0, 70.5);
        miner.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::EmptyPalm));
        for _ in 0..6 {
            assert!(!miner.use_right_hand(&mut world), "IronSpade must not mine {medium:?}");
            assert!(!miner.use_left_hand(&mut world), "Bare palm must not mine {medium:?}");
        }
        for _ in 0..80 {
            miner.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
            assert!(!miner.step_digging(&mut world), "Sustained hold must not scar or harvest {medium:?}");
        }
        let _ = miner.release_hand(&mut world);
        assert!(
            (world.constrain_to_surface(probe, 0.0).y - top_before).abs() < 1e-4,
            "{medium:?} height must remain intact against bare hands and spade"
        );
        assert_eq!(world.harvest_surface(aim_eye, aim_dir), Some(medium));
    }
}

#[test]
fn test_crafting_flint_pickaxe_cleaves_stone_copper_and_iron_but_rejects_gold_and_crust() {
    let satchel = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::DeadwoodBough, 1)
        .with_stowed_relic(HeldRelic::FlintStone, 3);
    let squire = YouthAttire::squire();
    let tome = ArtisanTome::new(19, true);
    assert_eq!(tome.selected_entry().product(), HeldRelic::FlintPickaxe);
    let (_, satchel_after, armed_flint, crafted) = tome.craft_selected(1.0, satchel, squire);
    assert!(crafted, "1 DeadwoodBough + 3 FlintStone must craft FlintPickaxe");
    assert_eq!(armed_flint.held_pair(), (HeldRelic::FlintPickaxe, HeldRelic::EmptyPalm));
    assert_eq!(satchel_after.resource_tally(HeldRelic::FlintStone, armed_flint), 0);

    let mut world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let stand = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let probe = Vec3::new(0.25, 16.0, -0.75);
    let ground_y = world.constrain_to_surface(probe, 0.0).y;
    let aim_eye = Vec3::new(0.25, stand.y + 0.8, 0.25);
    let aim_dir = (Vec3::new(0.25, ground_y + 0.1, -0.75) - aim_eye).normalize();

    let mut miner = PlanetaryAvatar::at(stand);
    miner.rotate_degrees(0.0, 70.5);
    miner.apply_attire(armed_flint);

    for mineable in [BlockMedium::QuarriedStone, BlockMedium::CopperOre, BlockMedium::IronOre] {
        assert!(world.place_molded_surface_block(
            aim_eye,
            aim_dir,
            stand,
            BlockMold::cube().in_medium(mineable),
        ));
        let elevated = world.constrain_to_surface(probe, 0.0).y;
        assert!(!miner.use_right_hand(&mut world));
        assert!(!miner.use_right_hand(&mut world));
        assert!(miner.use_right_hand(&mut world), "3rd FlintPickaxe strike must harvest {mineable:?}");
        let _ = miner.release_hand(&mut world);
        assert!((elevated - world.constrain_to_surface(probe, 0.0).y - 0.5).abs() < 1e-4);
    }

    for deep_hard in [BlockMedium::GoldOre, BlockMedium::CrustRock] {
        assert!(world.place_molded_surface_block(
            aim_eye,
            aim_dir,
            stand,
            BlockMold::cube().in_medium(deep_hard),
        ));
        let elevated = world.constrain_to_surface(probe, 0.0).y;
        for _ in 0..6 {
            assert!(!miner.use_right_hand(&mut world), "FlintPickaxe must bounce off {deep_hard:?}");
        }
        let _ = miner.release_hand(&mut world);
        assert!((world.constrain_to_surface(probe, 0.0).y - elevated).abs() < 1e-4);
        assert_eq!(world.harvest_surface(aim_eye, aim_dir), Some(deep_hard));
    }
}

#[test]
fn test_crafting_iron_pickaxe_cleaves_gold_and_crust_and_syncs_multiplayer_and_survivor_manifest() {
    let satchel = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::PlankBlock, 1)
        .with_stowed_relic(HeldRelic::IronOre, 2);
    let squire = YouthAttire::squire();
    let tome = ArtisanTome::new(20, true);
    assert_eq!(tome.selected_entry().product(), HeldRelic::IronPickaxe);
    let (_, _, armed_iron, crafted) = tome.craft_selected(1.0, satchel, squire);
    assert!(crafted, "1 PlankBlock + 2 IronOre must craft IronPickaxe");
    assert_eq!(armed_iron.held_pair(), (HeldRelic::IronPickaxe, HeldRelic::EmptyPalm));

    let mut manifest = SatchelManifest::parse_server_kit("deadwood_bough:1, flint_stone:3, plank_block:1, iron_ore:2", "test");
    assert!(manifest.apply_recipe(19) && manifest.apply_recipe(20));
    let mut wire = Vec::new();
    manifest.inscribe_sparse(&mut wire);
    let (decoded_manifest, _) = SatchelManifest::decode_sparse(&wire).expect("decode manifest");
    assert_eq!(decoded_manifest.count_of(HeldRelic::FlintPickaxe.code()), 1);
    assert_eq!(decoded_manifest.count_of(HeldRelic::IronPickaxe.code()), 1);

    let mut world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let stand = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let probe = Vec3::new(0.25, 16.0, -0.75);
    let ground_y = world.constrain_to_surface(probe, 0.0).y;
    let aim_eye = Vec3::new(0.25, stand.y + 0.8, 0.25);
    let aim_dir = (Vec3::new(0.25, ground_y + 0.1, -0.75) - aim_eye).normalize();

    let mut miner = PlanetaryAvatar::at(stand);
    miner.rotate_degrees(0.0, 70.5);
    miner.apply_attire(armed_iron);

    for deep_hard in [BlockMedium::GoldOre, BlockMedium::CrustRock] {
        assert!(world.place_molded_surface_block(
            aim_eye,
            aim_dir,
            stand,
            BlockMold::cube().in_medium(deep_hard),
        ));
        let elevated = world.constrain_to_surface(probe, 0.0).y;
        assert!(!miner.use_right_hand(&mut world));
        assert!(miner.use_right_hand(&mut world), "2nd IronPickaxe strike must harvest {deep_hard:?}");
        let _ = miner.release_hand(&mut world);
        assert!((elevated - world.constrain_to_surface(probe, 0.0).y - 0.5).abs() < 1e-4);
    }

    let dual_picks = YouthAttire::squire().with_relics(HeldRelic::FlintPickaxe, HeldRelic::IronPickaxe);
    let decoded = YouthAttire::from_multiplayer_bytes(dual_picks.multiplayer_bytes());
    assert_eq!(decoded.held_pair(), (HeldRelic::FlintPickaxe, HeldRelic::IronPickaxe));

    let mut bare_quads = QuadCounter::empty();
    let mut armed_quads = QuadCounter::empty();
    let mut avatar = PlanetaryAvatar::at(stand);
    avatar.emit_appearance(&mut bare_quads);
    avatar.apply_attire(dual_picks);
    avatar.emit_appearance(&mut armed_quads);
    assert!(armed_quads.count > bare_quads.count + 40, "Dual pickaxes must render 3D haft and head geometry");

    assert!(avatar.press_world_key(KeyCode::KeyE, &mut world));
    assert!(avatar.press_world_key(KeyCode::KeyE, &mut world), "Double-tap E must cast FlintPickaxe onto ground");
    let mut ground_quads_before = QuadCounter::empty();
    world.emit_surface(&mut ground_quads_before);
    assert_eq!(SurfaceFind::from_code(SurfaceFind::FlintPickaxe.code()), SurfaceFind::FlintPickaxe);
    assert_eq!(SurfaceFind::from_code(SurfaceFind::IronPickaxe.code()), SurfaceFind::IronPickaxe);
    assert!(ground_quads_before.count > 0);
}
