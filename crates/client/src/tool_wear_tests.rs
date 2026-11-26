use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, MeshTarget, PlanetaryWorld, Quad, SatchelManifest,
    SurfaceFind, TerrainMesh as _, WorldEarthworks as _,
};
use crate::attire::{
    ArtisanTome, BeltSatchel, BodyPose, HeldRelic, MoldedWardrobe, SightMode, StrideRhythm,
    ToolCondition, Wardrobe as _, YouthAttire, YouthPhysique,
};
use crate::avatar::{Avatar as _, AvatarMotor as _, PlanetaryAvatar};

struct MeshDigest {
    vertices: Vec<Vec3>,
}

impl MeshDigest {
    const fn empty() -> Self {
        Self { vertices: Vec::new() }
    }
}

impl MeshTarget for MeshDigest {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, v1, v2, v3, _, _| {
            self.vertices.extend_from_slice(&[v0, v1, v2, v3]);
        });
    }
}

#[test]
fn test_four_material_tiers_enforce_realistic_recipes_strike_cadence_and_mining_hardness() {
    let squire = YouthAttire::squire();
    let stocked = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::DeadwoodBough, 5)
        .with_stowed_relic(HeldRelic::FlintStone, 3)
        .with_stowed_relic(HeldRelic::PlankBlock, 8)
        .with_stowed_relic(HeldRelic::CopperOre, 4)
        .with_stowed_relic(HeldRelic::IronOre, 9)
        .with_stowed_relic(HeldRelic::TitaniumHull, 4);

    // Flint no longer crafts IronSpade, WoodmanAxe, or KnightlySword:
    let flint_only = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::DeadwoodBough, 2)
        .with_stowed_relic(HeldRelic::FlintStone, 4);
    for iron_recipe in [0usize, 5, 6] {
        let (_, _, _, ok) = ArtisanTome::new(iron_recipe, true).craft_selected(1.0, flint_only, squire);
        assert!(!ok, "Recipe #{iron_recipe} must require IronOre + PlankBlock, never FlintStone");
    }

    // Craft all 4 spade tiers and all 4 pickaxe tiers + forged axe & sword:
    for (idx, expected) in [
        (25usize, HeldRelic::WoodenSpade),
        (26, HeldRelic::CopperSpade),
        (5, HeldRelic::IronSpade),
        (27, HeldRelic::TitaniumSpade),
        (19, HeldRelic::FlintPickaxe),
        (28, HeldRelic::CopperPickaxe),
        (20, HeldRelic::IronPickaxe),
        (29, HeldRelic::TitaniumPickaxe),
        (0, HeldRelic::WoodmanAxe),
        (6, HeldRelic::KnightlySword),
    ] {
        let tome = ArtisanTome::new(idx, true);
        assert_eq!(tome.selected_entry().product(), expected);
        let (_, _, armed, ok) = tome.craft_selected(1.0, stocked, squire);
        assert!(ok && armed.held_pair().0 == expected, "Recipe #{idx} must craft {expected:?}");
    }

    // Verify spade excavation strike progression: Wood (4) -> Copper (3) -> Iron (2) -> Titanium (1)
    let mut world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let stand = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let probe = Vec3::new(0.25, 16.0, -0.75);
    let ground_y = world.constrain_to_surface(probe, 0.0).y;
    let aim_eye = Vec3::new(0.25, stand.y + 0.8, 0.25);
    let aim_dir = (Vec3::new(0.25, ground_y + 0.1, -0.75) - aim_eye).normalize();

    for (spade, strikes_needed) in [
        (HeldRelic::WoodenSpade, 4usize),
        (HeldRelic::CopperSpade, 3),
        (HeldRelic::IronSpade, 2),
        (HeldRelic::TitaniumSpade, 1),
    ] {
        assert!(world.place_molded_surface_block(aim_eye, aim_dir, stand, BlockMold::cube().in_medium(BlockMedium::TurfEarth)));
        let mut digger = PlanetaryAvatar::at(stand);
        digger.rotate_degrees(0.0, 70.5);
        digger.apply_attire(squire.with_relics(spade, HeldRelic::EmptyPalm));
        for pre in 1..strikes_needed {
            assert!(!digger.use_right_hand(&mut world), "{spade:?} strike {pre} must loosen before harvest");
        }
        assert!(digger.use_right_hand(&mut world), "{spade:?} must harvest earth on strike {strikes_needed}");
        let _ = digger.release_hand(&mut world);
    }

    // CopperPickaxe cleaves IronOre (3 strikes) but bounces off CrustRock; TitaniumPickaxe cleaves CrustRock in 1 strike:
    assert!(world.place_molded_surface_block(aim_eye, aim_dir, stand, BlockMold::cube().in_medium(BlockMedium::IronOre)));
    let mut copper_miner = PlanetaryAvatar::at(stand);
    copper_miner.rotate_degrees(0.0, 70.5);
    copper_miner.apply_attire(squire.with_relics(HeldRelic::CopperPickaxe, HeldRelic::EmptyPalm));
    assert!(!copper_miner.use_right_hand(&mut world) && !copper_miner.use_right_hand(&mut world));
    assert!(copper_miner.use_right_hand(&mut world), "CopperPickaxe must cleave IronOre on 3rd strike");
    let _ = copper_miner.release_hand(&mut world);

    assert!(world.place_molded_surface_block(aim_eye, aim_dir, stand, BlockMold::cube().in_medium(BlockMedium::CrustRock)));
    assert!(!copper_miner.use_right_hand(&mut world), "CopperPickaxe must bounce off deep CrustRock");
    let _ = copper_miner.release_hand(&mut world);

    let mut ti_miner = PlanetaryAvatar::at(stand);
    ti_miner.rotate_degrees(0.0, 70.5);
    ti_miner.apply_attire(squire.with_relics(HeldRelic::TitaniumPickaxe, HeldRelic::EmptyPalm));
    assert!(ti_miner.use_right_hand(&mut world), "TitaniumPickaxe must cleave CrustRock in 1 strike");
}

#[test]
fn test_tool_wear_progresses_visually_across_four_conditions_survives_stow_and_shatters_at_limit() {
    let mut world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let squire = YouthAttire::squire().with_relics(HeldRelic::WoodenSpade, HeldRelic::EmptyPalm);
    let mut satchel = BeltSatchel::empty().reconcile_attire(squire);
    let mut attire = squire;

    // Endurance of WoodenSpade is 24; step through Pristine -> Scuffed -> Chipped -> Splintered:
    assert_eq!(satchel.hand_condition(1.0, attire), ToolCondition::Pristine);
    (satchel, attire) = satchel.strain_hand(1.0, 7, attire, &mut world);
    assert_eq!(satchel.hand_condition(1.0, attire), ToolCondition::Scuffed);
    (satchel, attire) = satchel.strain_hand(1.0, 8, attire, &mut world);
    assert_eq!(satchel.hand_condition(1.0, attire), ToolCondition::Chipped);
    (satchel, attire) = satchel.strain_hand(1.0, 5, attire, &mut world);
    assert_eq!(satchel.hand_condition(1.0, attire), ToolCondition::Splintered);

    // Stowing and redrawing must preserve Splintered wear rather than repairing for free:
    let (stowed_satchel, unarmed, ok_stow) = satchel.transfer_hand(1.0, attire);
    assert!(ok_stow && unarmed.held_pair().0 == HeldRelic::EmptyPalm);
    let pose = BodyPose::new(Vec3::new(0.0, 9.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let (mut fresh_tray, mut worn_tray) = (MeshDigest::empty(), MeshDigest::empty());
    BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::WoodenSpade, 1)
        .with_open(true)
        .emit_for_attire(&pose, unarmed, &mut fresh_tray);
    stowed_satchel.with_open(true).emit_for_attire(&pose, unarmed, &mut worn_tray);
    assert_ne!(
        fresh_tray.vertices, worn_tray.vertices,
        "Open satchel compartment and legend must visually distinguish Splintered from Pristine tool"
    );

    let (opened_satchel, _) = stowed_satchel.toggle_open();
    let (redrawn_satchel, redrawn_attire, ok_draw) = opened_satchel.transfer_hand(1.0, unarmed);
    assert!(ok_draw && redrawn_attire.held_pair().0 == HeldRelic::WoodenSpade);
    assert_eq!(redrawn_satchel.hand_condition(1.0, redrawn_attire), ToolCondition::Splintered);

    // Distinct 3D palm geometry across all 4 conditions (including multiplayer PresenceAttire replication):
    let mut condition_meshes = Vec::new();
    for stage in [
        ToolCondition::Pristine,
        ToolCondition::Scuffed,
        ToolCondition::Chipped,
        ToolCondition::Splintered,
    ] {
        let molds = world::PalmMolds::new(stage.into_mold(), BlockMold::cube());
        let presence = world::PresenceAttire::new(
            squire.with_relics(HeldRelic::IronPickaxe, HeldRelic::EmptyPalm).multiplayer_bytes(),
            squire.region_bytes(),
            molds,
        );
        let mut digest = MeshDigest::empty();
        MoldedWardrobe::from_presence(presence).emit(&pose, SightMode::FirstPersonSight, &mut digest);
        assert!(!digest.vertices.is_empty());
        condition_meshes.push(digest.vertices);
    }
    for pair in condition_meshes.windows(2) {
        assert_ne!(pair[0], pair[1], "Each wear stage must sculpt distinct 3D tool geometry");
    }

    // Final 4 wear units shatter WoodenSpade (`20 + 4 == 24`) to EmptyPalm and sync SatchelManifest:
    let (broken_satchel, broken_attire) = redrawn_satchel.strain_hand(1.0, 4, redrawn_attire, &mut world);
    assert_eq!(broken_attire.held_pair().0, HeldRelic::EmptyPalm, "Exhausted tool must shatter out of hand");
    assert_eq!(broken_satchel.resource_tally(HeldRelic::WoodenSpade, broken_attire), 0);

    // Verify Copper outlasts Wood/Flint, Iron outlasts Copper, and Titanium outlasts Iron:
    assert!(HeldRelic::WoodenSpade.endurance_span() < HeldRelic::CopperSpade.endurance_span());
    assert!(HeldRelic::CopperSpade.endurance_span() < HeldRelic::IronSpade.endurance_span());
    assert!(HeldRelic::IronSpade.endurance_span() < HeldRelic::TitaniumSpade.endurance_span());
    assert!(HeldRelic::FlintPickaxe.endurance_span() < HeldRelic::CopperPickaxe.endurance_span());
    assert!(HeldRelic::CopperPickaxe.endurance_span() < HeldRelic::IronPickaxe.endurance_span());
    assert!(HeldRelic::IronPickaxe.endurance_span() < HeldRelic::TitaniumPickaxe.endurance_span());

    let mut manifest = SatchelManifest::parse_server_kit("wooden_spade:1, titanium_pickaxe:1", "test");
    assert!(manifest.consume_edible(HeldRelic::WoodenSpade.code()));
    assert_eq!(manifest.count_of(HeldRelic::WoodenSpade.code()), 0);
    assert!(!manifest.consume_edible(HeldRelic::GoldOre.code()), "Non-expendable ore must still be guarded");

    // Casting new tool tiers onto planetary surface round-trips SurfaceFind codes and ground meshes:
    let spawn = world.constrain_to_surface(Vec3::new(2.25, 16.0, 2.25), 1.0);
    let mut dropper = PlanetaryAvatar::at(spawn);
    dropper.rotate_degrees(0.0, 68.0);
    dropper.apply_attire(YouthAttire::squire().with_relics(HeldRelic::TitaniumPickaxe, HeldRelic::EmptyPalm));
    assert!(dropper.press_world_key(KeyCode::KeyE, &mut world));
    assert!(dropper.press_world_key(KeyCode::KeyE, &mut world));
    let mut surface_mesh = MeshDigest::empty();
    world.emit_surface(&mut surface_mesh);
    assert!(!surface_mesh.vertices.is_empty());
    for find in [
        SurfaceFind::WoodenSpade,
        SurfaceFind::CopperSpade,
        SurfaceFind::TitaniumSpade,
        SurfaceFind::CopperPickaxe,
        SurfaceFind::TitaniumPickaxe,
    ] {
        assert_eq!(SurfaceFind::from_code(find.code()), find);
    }
}
