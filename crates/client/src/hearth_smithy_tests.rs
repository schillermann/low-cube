use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, BodyId, CelestialSystem, CosmicFrame, MeshTarget,
    PlanetaryWorld, PresenceAttire, PresenceKinematics, Quad, SolarProportions, SurfaceFind,
    TerrainMesh as _, WorldEarthworks as _,
};
use crate::attire::{ArtisanTome, BeltSatchel, CraftVenue, HeldRelic, YouthAttire};
use crate::avatar::{Avatar as _, PlanetaryAvatar};
use crate::controls::{ActionBindings, InputTrigger};
use crate::earthworks::{
    AtmosphericBloomery, ColdStone, CulinaryOven, HearthMedium, MealOutcome, PalmEarthworks,
    SurfaceReach, ThermalHearth as _, VacuumForge, WorkstationStation,
};

struct QuadCounter {
    count: usize,
}

impl MeshTarget for QuadCounter {
    fn push_quad(&mut self, _quad: Quad) {
        self.count += 1;
    }
}

#[test]
fn test_baking_oven_roasts_boar_meat_rejects_ores_and_vacuum_and_boosts_nourishment() {
    let squire = YouthAttire::squire().with_relics(HeldRelic::BoarMeat, HeldRelic::DeadwoodBough);
    let satchel = BeltSatchel::empty().reconcile_attire(squire);
    let oven = CulinaryOven::new(ColdStone::new());

    assert!(
        oven.fire_charge(squire.held_pair(), HearthMedium::HardVacuum, satchel, squire, 1.0).is_none(),
        "Wood-fired BakingOven must refuse combustion in oxygen-free HardVacuum"
    );

    let ore_hands = YouthAttire::squire().with_relics(HeldRelic::IronOre, HeldRelic::DeadwoodBough);
    let ore_satchel = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::BoarMeat, 1)
        .reconcile_attire(ore_hands);
    assert!(
        oven.fire_charge(ore_hands.held_pair(), HearthMedium::AmbientAir, ore_satchel, ore_hands, 1.0).is_none(),
        "250 C BakingOven must reject IronOre smelting attempts"
    );

    let (baked_satchel, baked_attire, code) = oven
        .fire_charge(squire.held_pair(), HearthMedium::AmbientAir, satchel, squire, 1.0)
        .expect("BakingOven must bake BoarMeat + DeadwoodBough in AmbientAir");
    assert_eq!(code, 100);
    assert_eq!(baked_attire.held_pair(), (HeldRelic::RoastedBoar, HeldRelic::EmptyPalm));
    assert_eq!(baked_satchel.resource_tally(HeldRelic::BoarMeat, baked_attire), 0);
    assert_eq!(MealOutcome::from_relic(HeldRelic::RoastedBoar), MealOutcome::OvenRoast);
    assert_eq!(MealOutcome::from_relic(HeldRelic::BoarMeat), MealOutcome::RawVenison);

    // End-to-end in-world placement, baking without eating raw meat, and eating RoastedBoar:
    let mut world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let stand = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let probe = Vec3::new(0.25, 16.0, -0.75);
    let ground_y = world.constrain_to_surface(probe, 0.0).y;
    let aim_eye = Vec3::new(0.25, stand.y + 0.8, 0.25);
    let aim_dir = (Vec3::new(0.25, ground_y + 0.1, -0.75) - aim_eye).normalize();

    assert!(world.place_molded_surface_block(
        aim_eye,
        aim_dir,
        stand,
        BlockMold::cube().in_medium(BlockMedium::BakingOven),
    ));
    let mut baker = PlanetaryAvatar::at(stand);
    baker.rotate_degrees(0.0, 70.5);
    baker.apply_attire(squire);
    assert!(baker.use_right_hand(&mut world), "Clicking placed BakingOven with BoarMeat + DeadwoodBough must bake RoastedBoar");

    // Look up at the sky and eat the freshly baked RoastedBoar:
    baker.rotate_degrees(0.0, -70.5);
    assert!(baker.use_right_hand(&mut world), "Clicking skyward with RoastedBoar must savor hearty roast");
}

#[test]
fn test_bloomery_forge_smelts_copper_and_iron_in_air_and_forges_space_titanium_exclusively_in_vacuum() {
    let forge = VacuumForge::new(AtmosphericBloomery::new(ColdStone::new()));
    let squire = YouthAttire::squire();

    // 1. Atmospheric smelting: CopperOre & IronOre succeed; CrustBlock Titanium fails due to O2/N2 embrittlement!
    let ti_air_attire = squire.with_relics(HeldRelic::CrustBlock, HeldRelic::IronOre);
    let ti_air_satchel = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::DeadwoodBough, 2)
        .reconcile_attire(ti_air_attire);
    assert!(
        forge
            .fire_charge(ti_air_attire.held_pair(), HearthMedium::AmbientAir, ti_air_satchel, ti_air_attire, 1.0)
            .is_none(),
        "Atmospheric BloomeryForge must reject CrustBlock Titanium due to oxygen/nitrogen embrittlement"
    );

    let fe_attire = squire.with_relics(HeldRelic::IronOre, HeldRelic::DeadwoodBough);
    let fe_satchel = BeltSatchel::empty().reconcile_attire(fe_attire);
    let (_, after_fe, fe_code) = forge
        .fire_charge(fe_attire.held_pair(), HearthMedium::AmbientAir, fe_satchel, fe_attire, 1.0)
        .expect("Atmospheric BloomeryForge must smelt IronOre + DeadwoodBough");
    assert_eq!((fe_code, after_fe.held_pair().0), (102, HeldRelic::IronBloom));

    let cu_attire = squire.with_relics(HeldRelic::CopperOre, HeldRelic::DeadwoodBough);
    let cu_satchel = BeltSatchel::empty().reconcile_attire(cu_attire);
    let (_, after_cu, cu_code) = forge
        .fire_charge(cu_attire.held_pair(), HearthMedium::AmbientAir, cu_satchel, cu_attire, 1.0)
        .expect("Atmospheric BloomeryForge must smelt CopperOre + DeadwoodBough");
    assert_eq!((cu_code, after_cu.held_pair().0), (101, HeldRelic::CopperBloom));

    // 2. HardVacuum: wood-fired IronOre fails, while Space-Forged CrustBlock + IronOre yields 2 TitaniumBillet!
    assert!(
        forge
            .fire_charge(fe_attire.held_pair(), HearthMedium::HardVacuum, fe_satchel, fe_attire, 1.0)
            .is_none(),
        "Wood-fired smelting without CrustBlock must fail in oxygen-free HardVacuum"
    );

    let vac_attire = squire.with_relics(HeldRelic::CrustBlock, HeldRelic::IronOre);
    let vac_satchel = BeltSatchel::empty().reconcile_attire(vac_attire);
    let (ti_satchel, ti_attire, ti_code) = forge
        .fire_charge(vac_attire.held_pair(), HearthMedium::HardVacuum, vac_satchel, vac_attire, 1.0)
        .expect("VacuumForge must reduce CrustBlock + IronOre into Space-Forged TitaniumBillet in HardVacuum");
    assert_eq!(ti_code, 103);
    assert_eq!(ti_attire.held_pair().0, HeldRelic::TitaniumBillet);
    assert_eq!(ti_satchel.resource_tally(HeldRelic::TitaniumBillet, ti_attire), 2);

    // 3. Verify PlanetaryWorld::vacuum_medium_at across Earth surface, high orbit, and airless Luna:
    let mut cosmos = CelestialSystem::barren_heliosphere(SolarProportions::compressed(16, 0.001), 0.5);
    assert!(!cosmos.vacuum_medium_at(Vec3::new(0.25, 8.5, 0.25)), "Earth surface has atmosphere");
    assert!(cosmos.vacuum_medium_at(Vec3::new(0.25, 500.0, 0.25)), "Orbital space above Earth is HardVacuum");
    cosmos.anchor_to_frame(CosmicFrame::Body(BodyId::LUNA), Vec3::new(0.25, 4.5, 0.25));
    assert!(cosmos.vacuum_medium_at(Vec3::new(0.25, 4.5, 0.25)), "Airless Luna surface is HardVacuum");
}

#[test]
fn test_smithy_anvil_opens_forge_venue_in_world_crafts_tools_and_enforces_multiplayer_vacuum_ward() {
    // 1. Craft BakingOven (#30), BloomeryForge (#31), and SmithyAnvil (#32) in FolioTab::Blocks:
    let squire = YouthAttire::squire();
    let builder_stock = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::QuarriedStone, 7)
        .with_stowed_relic(HeldRelic::DeadwoodBough, 1)
        .with_stowed_relic(HeldRelic::TimberTorch, 1)
        .with_stowed_relic(HeldRelic::TimberBlock, 1);
    for (idx, expected) in [
        (30usize, HeldRelic::BakingOven),
        (31, HeldRelic::BloomeryForge),
        (32, HeldRelic::SmithyAnvil),
    ] {
        let tome = ArtisanTome::new(idx, true);
        assert_eq!(tome.venue(), CraftVenue::FieldHands);
        let (_, _, out_attire, ok) = tome.craft_selected(1.0, builder_stock, squire);
        assert!(ok && out_attire.held_pair().0 == expected, "Field Blocks recipe #{idx} must craft {expected:?}");
    }

    // 2. Place SmithyAnvil in world, click it to open AnvilOpen folio, and forge TitaniumPickaxe (#29):
    let mut world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let stand = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let probe = Vec3::new(0.25, 16.0, -0.75);
    let ground_y = world.constrain_to_surface(probe, 0.0).y;
    let aim_eye = Vec3::new(0.25, stand.y + 0.8, 0.25);
    let aim_dir = (Vec3::new(0.25, ground_y + 0.1, -0.75) - aim_eye).normalize();

    for medium in [BlockMedium::BakingOven, BlockMedium::BloomeryForge, BlockMedium::SmithyAnvil] {
        assert!(world.place_molded_surface_block(aim_eye, aim_dir, stand, BlockMold::cube().in_medium(medium)));
    }
    let mut mesh = QuadCounter { count: 0 };
    world.emit_surface(&mut mesh);
    assert!(mesh.count > 40, "Placed BakingOven, BloomeryForge, and SmithyAnvil must emit sculpted 3D quads");

    // Top block at probe is SmithyAnvil: holding IronOre suppresses placement hologram over BloomeryForge, while SmithyAnvil block still previews cleanly:
    let reach = SurfaceReach::new(aim_eye, aim_dir, stand, true);
    let anvil_builder = squire.with_relics(HeldRelic::SmithyAnvil, HeldRelic::EmptyPalm);
    let ew_builder = PalmEarthworks::empty().reconcile_attire(anvil_builder);
    assert!(ew_builder.planned_mount(reach, anvil_builder, &world).is_some(), "Holding SmithyAnvil must still preview placement");
    let plank_smith = squire.with_relics(HeldRelic::PlankBlock, HeldRelic::EmptyPalm);
    let ew_smith = PalmEarthworks::empty().reconcile_attire(plank_smith);
    assert!(ew_smith.planned_mount(reach, plank_smith, &world).is_none(), "Aiming PlankBlock at SmithyAnvil opens forge instead of showing placement hologram");

    // Interact via WorkstationStation to open AnvilOpen ArtisanTome and forge TitaniumSpade (#27):
    let smith_attire = squire.with_relics(HeldRelic::TitaniumBillet, HeldRelic::PlankBlock);
    let smith_satchel = BeltSatchel::empty().reconcile_attire(smith_attire);
    let (satchel_open, anvil_tome, _, opened) = WorkstationStation::new(BlockMedium::SmithyAnvil, HearthMedium::AmbientAir)
        .interact(1.0, smith_satchel, ArtisanTome::closed(), smith_attire, &mut world)
        .expect("Clicking SmithyAnvil must open ArtisanTome at CraftVenue::SmithyAnvil");
    assert!(opened && anvil_tome.open() && anvil_tome.venue() == CraftVenue::SmithyAnvil);

    let bindings = ActionBindings::seeded();
    let (ready_anvil, s2, a2, ok_f) = anvil_tome.dispatch_trigger(InputTrigger::Key(KeyCode::KeyF), &bindings, satchel_open, smith_attire);
    assert!(ok_f && ready_anvil.selected_entry().product() == HeldRelic::TitaniumSpade);
    let (closed_after, _, forged_attire, forged_ok) = ready_anvil.craft_selected(1.0, s2, a2);
    assert!(forged_ok && !closed_after.open() && forged_attire.held_pair().0 == HeldRelic::TitaniumSpade);

    // Round-trip new SurfaceFind codes:
    for find in [
        SurfaceFind::RoastedBoar,
        SurfaceFind::CopperBloom,
        SurfaceFind::IronBloom,
        SurfaceFind::TitaniumBillet,
    ] {
        assert_eq!(SurfaceFind::from_code(find.code()), find);
    }

    // 3. Verify SolarProportions::vacuum_locus matches atmospheric vs. vacuum anti-cheat boundaries:
    let prop = SolarProportions::true_scale(16);
    assert!(!prop.vacuum_locus(CosmicFrame::Body(BodyId::EARTH), Vec3::new(0.25, 8.5, 0.25)));
    assert!(prop.vacuum_locus(CosmicFrame::Body(BodyId::EARTH), Vec3::new(0.0, 200.0, 0.0)));
    assert!(prop.vacuum_locus(CosmicFrame::Body(BodyId::LUNA), Vec3::new(0.25, 4.5, 0.25)));
    let _ = PresenceKinematics::new(stand, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
    let _ = PresenceAttire::squire();
}
