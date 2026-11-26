use glam::{Quat, Vec3};
use world::{
    AttitudeGuidance, BlockMedium, BlockMold, CelestialSystem, MoldArchetype, PilotOrder, Quad,
    RelicCatalog, SatchelManifest, SurfaceContact, VehicleAssembly, VehicleAxes, VehiclePart,
};
use crate::attire::{ArtisanTome, BeltSatchel, HeldRelic, YouthAttire};
use super::reentry_step::AtmosphericEntryStep;
use super::space_step::{InertialDrift, SpaceStep};

struct GlowProbe {
    quads: usize,
    peak_glow: f32,
}

impl world::MeshTarget for GlowProbe {
    fn push_quad(&mut self, quad: Quad) {
        self.quads += 1;
        let g = quad.radiate(|v| v);
        if g > self.peak_glow {
            self.peak_glow = g;
        }
    }
}

fn assemble_hull(hull_medium: BlockMedium, block_count: i32) -> VehicleAssembly {
    let mut parts = vec![
        VehiclePart::new((0, 0), 0.0, BlockMold::shape_from_stamp([0, 0, BlockMedium::CabinSeat.code()])),
        VehiclePart::new((0, 1), 0.0, BlockMold::shape_from_stamp([0, 0, BlockMedium::FlightConsole.code()])),
        VehiclePart::new((0, 0), -0.5, BlockMold::shape_from_stamp([0, 0, BlockMedium::FusionTorch.code()])),
    ];
    for idx in 1..=block_count {
        parts.push(VehiclePart::new(
            (idx, 0),
            0.0,
            BlockMold::shape_from_stamp([MoldArchetype::RampWedge.code(), 0, hull_medium.code()]),
        ));
    }
    VehicleAssembly::new(Vec3::new(0.0, 24.0, 0.0), VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y), Vec3::Z, parts)
}

#[test]
fn test_titanium_and_carbon_silicate_craft_serialize_and_sculpt_3d_laced_meshes() {
    let squire = YouthAttire::squire();
    let stocked = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::TitaniumBillet, 2)
        .with_stowed_relic(HeldRelic::IronBloom, 1)
        .with_stowed_relic(HeldRelic::GoldOre, 1)
        .with_stowed_relic(HeldRelic::FlintStone, 2);

    let (_, _, _, rejected_field) = ArtisanTome::new(21, true).craft_selected(1.0, stocked, squire);
    assert!(!rejected_field, "TitaniumHull forging must require SmithyAnvil");
    let (_, satchel_ti, attire_ti, ok_ti) = ArtisanTome::at_anvil(21, true).craft_selected(1.0, stocked, squire);
    assert!(ok_ti && satchel_ti.resource_tally(HeldRelic::TitaniumHull, attire_ti) == 4);

    let (_, satchel_cs, attire_cs, ok_cs) = ArtisanTome::new(22, true).craft_selected(-1.0, satchel_ti, attire_ti);
    assert!(ok_cs && satchel_cs.resource_tally(HeldRelic::CarbonSilicate, attire_cs) == 4);
    assert_eq!(attire_cs.held_pair(), (HeldRelic::TitaniumHull, HeldRelic::CarbonSilicate));

    let wire = attire_cs.multiplayer_bytes();
    assert_eq!(YouthAttire::from_multiplayer_bytes(wire).held_pair(), (HeldRelic::TitaniumHull, HeldRelic::CarbonSilicate));

    let mut manifest = SatchelManifest::empty();
    manifest.credit_code(RelicCatalog::from_medium(BlockMedium::IronOre), 2);
    manifest.credit_code(RelicCatalog::from_medium(BlockMedium::CrustRock), 1);
    manifest.credit_code(HeldRelic::DeadwoodBough.code(), 1);
    assert!(manifest.apply_recipe(103), "Vacuum BloomeryForge #103 must reduce CrustRock + IronOre into 2 TitaniumBillet");
    assert!(manifest.apply_recipe(102), "BloomeryForge #102 must reduce IronOre + DeadwoodBough into IronBloom");
    assert!(manifest.apply_recipe(21) && manifest.count_of(32) == 4);
    let mut sparse = Vec::new();
    manifest.inscribe_sparse(&mut sparse);
    let (decoded, _) = SatchelManifest::decode_sparse(&sparse).expect("sparse manifest");
    assert_eq!(decoded.count_of(32), 4);

    let ship_ti = assemble_hull(BlockMedium::TitaniumHull, 2);
    let ship_cs = assemble_hull(BlockMedium::CarbonSilicate, 2);
    let (mut probe_ti, mut probe_cs) = (GlowProbe { quads: 0, peak_glow: 0.0 }, GlowProbe { quads: 0, peak_glow: 0.0 });
    ship_ti.emit(&mut probe_ti);
    ship_cs.emit(&mut probe_cs);
    assert!(probe_ti.quads > 20 && probe_cs.quads > 20);
    assert!(probe_cs.peak_glow >= 0.18, "Carbon-silicate lace plating must emit signature cyan-blue radiance");
}

#[test]
fn test_aerospace_keel_reduces_dry_mass_and_unlocks_high_g_epstein_burn() {
    let timber_ship = assemble_hull(BlockMedium::RawTimber, 10);
    let titanium_ship = assemble_hull(BlockMedium::TitaniumHull, 10);
    let lace_ship = assemble_hull(BlockMedium::CarbonSilicate, 10);

    assert!(!timber_ship.aerospace_framed() && !timber_ship.thermal_shielded());
    assert!(titanium_ship.aerospace_framed() && !titanium_ship.thermal_shielded());
    assert!(lace_ship.aerospace_framed() && lace_ship.thermal_shielded());

    let burn = PilotOrder::from_channels(Vec3::Y, Vec3::ZERO, Vec3::ZERO, AttitudeGuidance::Engaged);
    let dv_timber = timber_ship.sample_impulse(&burn).keel_fusion_velocity_delta(timber_ship.aerospace_framed(), 1.0).length();
    let dv_titanium = titanium_ship.sample_impulse(&burn).keel_fusion_velocity_delta(titanium_ship.aerospace_framed(), 1.0).length();
    let dv_lace = lace_ship.sample_impulse(&burn).keel_fusion_velocity_delta(lace_ship.aerospace_framed(), 1.0).length();

    assert!(dv_titanium > dv_timber + 20.0, "Titanium keel must out-accelerate heavy timber hull ({dv_titanium} vs {dv_timber})");
    assert!(dv_lace >= dv_titanium, "Carbon-silicate lace must be at least as agile as titanium ({dv_lace} vs {dv_titanium})");
}

#[test]
fn test_carbon_silicate_aerobrakes_smoothly_during_steep_reentry_without_buffeting() {
    let world = CelestialSystem::barren_authoritative(32, 1.0);
    let ground = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0);
    let entry_pos = ground + Vec3::Y * 6.0;
    let plunge_vel = Vec3::new(15.0, -90.0, 0.0);
    let spin = Vec3::new(0.8, 0.0, -0.6);

    let mut timber_ship = assemble_hull(BlockMedium::RawTimber, 4);
    timber_ship.translate_to(entry_pos);
    let mut lace_ship = assemble_hull(BlockMedium::CarbonSilicate, 4);
    lace_ship.translate_to(entry_pos);

    let s_timber = AtmosphericEntryStep::new(
        InertialDrift::new(entry_pos, plunge_vel, spin),
        &timber_ship,
        world.as_ref(),
    )
    .resolve(0.05);
    let s_lace = AtmosphericEntryStep::new(
        InertialDrift::new(entry_pos, plunge_vel, spin),
        &lace_ship,
        world.as_ref(),
    )
    .resolve(0.05);

    let (v_tim, w_tim) = s_timber.transfer(|_, v, w, _: Quat| (v, w));
    let (v_lac, w_lac) = s_lace.transfer(|_, v, w, _: Quat| (v, w));
    assert!(v_lac.length() < v_tim.length(), "Carbon-silicate tiles must aerobrake steeper than bare timber");
    assert!(w_lac.length() < spin.length(), "Carbon-silicate hull must damp rotational buffeting");
    assert!(w_tim.length() != w_lac.length(), "Unshielded timber hull must experience distinct atmospheric buffeting");
    assert_eq!(SurfaceContact::GasEnvelope { drag: 1.5 }.resolve_gas(0.0, |d| d), 1.5);
}
