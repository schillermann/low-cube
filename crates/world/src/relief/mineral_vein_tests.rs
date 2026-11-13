use super::*;
use crate::block::{BlockMedium, BlockMold, GridCell};
use crate::topology::ClimateZone;
use crate::{BiomedCubeWorld, CompanionPresence, PresenceKinematics, WorldFact};
use glam::{Quat, Vec3};

struct TallPlateau;

impl ReliefElevation for TallPlateau {
    fn elevation_at(&self, _u: i32, _v: i32) -> i32 {
        6
    }

    fn surface_height_at(&self, _u: f32, _v: f32) -> f32 {
        3.0
    }
}

#[test]
fn test_air_exposure_only_reveals_cells_touching_excavated_pit() {
    let mut auth = TerracedRelief::with_veins(
        Box::new(TallPlateau),
        32,
        Box::new(LivingVeins::with_secret(0xCAFE_BABE_1234_5678, 0x9E37_79B9_7F4A_7C15)),
    );
    let mut hollow = TerracedRelief::with_veins(
        Box::new(TallPlateau),
        32,
        Box::new(HollowVeins::untouched()),
    );

    // Before digging, zero subterranean cells are disclosed anywhere.
    for step in -10..6 {
        assert_eq!(auth.exposed_ore_at(0, 0, step), None);
        assert_eq!(hollow.exposed_ore_at(0, 0, step), None);
    }

    // Excavate a shaft at (4, 4) and verify disclosures ONLY reference (4, 4) floor or immediate ±1 wall neighbors.
    let mut all_disclosed = Vec::new();
    for step in (-8..6).rev() {
        let mid = (step as f32) * 0.5 + 0.25;
        let _ = auth.harvest_layer(4, 4, mid);
        auth.drain_disclosures(&mut all_disclosed);
    }
    for stratum in &all_disclosed {
        stratum.transfer(|u, v, _, med| {
            let manhattan = (u - 4).abs() + (v - 4).abs();
            assert!(manhattan <= 1, "Unexposed interior block ({u}, {v}) behind solid wall leaked!");
            assert!(med.mineral_ore(), "Only genuine mineral ores need disclosure packets");
        });
        hollow.accept_disclosure(*stratum);
    }

    // Cells 2 steps away behind solid rock remain completely undisclosed on both server and client.
    for step in -8..6 {
        assert_eq!(auth.exposed_ore_at(6, 4, step), None);
        assert_eq!(hollow.exposed_ore_at(6, 4, step), None);
    }
}

#[test]
fn test_height_stratigraphy_and_sector_saturation_curb_strip_mining() {
    let mut relief = TerracedRelief::with_veins(
        Box::new(TallPlateau),
        64,
        Box::new(LivingVeins::with_secret(0x1122_3344_5566_7788, 0xAABB_CCDD_EEFF_0011)),
    );
    let (mut shallow_copper, mut mid_iron, mut deep_gold) = (0usize, 0usize, 0usize);
    let (mut first_vein_sector_hits, mut fresh_sector_hits) = (0usize, 0usize);

    // Dig several shafts all the way down to CrustVault across fresh 16x16 sectors.
    for idx in 0..12 {
        let (su, sv) = ((idx % 4) * 16 - 24, (idx / 4) * 16 - 16);
        let floor = relief.crust_floor_at(su, sv);
        let mut disclosures = Vec::new();
        for step in (floor..6).rev() {
            let _ = relief.harvest_layer(su, sv, (step as f32) * 0.5 + 0.25);
            relief.drain_disclosures(&mut disclosures);
        }
        fresh_sector_hits += disclosures.len();
        for d in disclosures {
            d.transfer(|u, v, step, med| {
                let nat = relief.natural_elevation_at(u, v);
                let depth = nat - step;
                let above_crust = step - (relief.crust_floor_at(u, v) + 4);
                match med {
                    BlockMedium::CopperOre => {
                        assert!(depth >= 2 && depth <= 5, "CopperOre appeared outside upper-mid zone at depth {depth}");
                        shallow_copper += 1;
                    }
                    BlockMedium::IronOre => {
                        assert!(depth >= 4, "IronOre appeared too shallow at depth {depth}");
                        mid_iron += 1;
                    }
                    BlockMedium::GoldOre => {
                        assert!(above_crust <= 4, "GoldOre appeared far above deep crust zone ({above_crust})");
                        deep_gold += 1;
                    }
                    other => panic!("Unexpected medium in vein disclosure: {other:?}"),
                }
            });
        }
    }
    assert!(shallow_copper > 0 && mid_iron > 0 && deep_gold > 0, "All three stratified ores must spawn in their geological bands");

    // Strip-mining 12 shafts crammed inside a SINGLE already-mined 16x16 sector yields fewer new sprouts due to saturation.
    let mut saturated = TerracedRelief::with_veins(
        Box::new(TallPlateau),
        64,
        Box::new(LivingVeins::with_secret(0x1122_3344_5566_7788, 0xAABB_CCDD_EEFF_0011)),
    );
    for idx in 0..12 {
        let (su, sv) = ((idx % 4) * 3, (idx / 4) * 3);
        let floor = saturated.crust_floor_at(su, sv);
        let mut disclosures = Vec::new();
        for step in (floor..6).rev() {
            let _ = saturated.harvest_layer(su, sv, (step as f32) * 0.5 + 0.25);
            saturated.drain_disclosures(&mut disclosures);
        }
        first_vein_sector_hits += disclosures.len();
    }
    assert!(
        first_vein_sector_hits < fresh_sector_hits,
        "Sector saturation must penalize concentrated strip-mining ({first_vein_sector_hits}) vs exploring fresh sectors ({fresh_sector_hits})"
    );
}

#[test]
fn test_world_fact_and_companion_presence_binary_roundtrip_and_zero_knowledge_enactment() {
    let mut client_world = BiomedCubeWorld::hollow_beveled(32, 1.0);
    let fact = WorldFact::StratumDisclosed {
        face: 0,
        stratum: ExposedStratum::new((1, 1, -2), BlockMedium::GoldOre),
    };
    let place = WorldFact::MoldPlaced {
        eye: Vec3::new(0.25, 18.0, 0.25),
        direction: Vec3::NEG_Y,
        avatar_pos: Vec3::new(2.25, 17.0, 2.25),
        mold: BlockMold::cube().in_medium(BlockMedium::CopperOre),
    };
    let mut buf = Vec::new();
    fact.inscribe(&mut buf);
    place.inscribe(&mut buf);
    let (d1, n1) = WorldFact::decode(&buf).expect("decode first fact");
    let (d2, n2) = WorldFact::decode(&buf[n1..]).expect("decode second fact");
    assert_eq!(n1 + n2, buf.len());
    assert_eq!(d1, fact);
    assert_eq!(d2, place);
    assert!(d1.enact_on(&mut client_world));
    assert!(d2.enact_on(&mut client_world));

    let kin = PresenceKinematics::new(Vec3::new(1.0, 17.0, -2.0), Quat::IDENTITY, Quat::IDENTITY, (0.8, 0.0));
    let pres = CompanionPresence::new(7, kin, (1.2, 0.8, 0.0, 0.0), ([1, 2, 3, 0, 0, 0, 4, 2, 1], [5, 5, 5, 5]));
    let mut p_buf = Vec::new();
    pres.inscribe(&mut p_buf);
    let (decoded_pres, p_len) = CompanionPresence::decode(&p_buf).expect("decode presence");
    assert_eq!(p_len, p_buf.len());
    assert_eq!(decoded_pres, pres);

    // Ensure MineralSeam emits geometry without protruding outward.
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    struct Counter(usize);
    impl crate::MeshTarget for Counter {
        fn push_quad(&mut self, _quad: crate::Quad) { self.0 += 1; }
    }
    let mut c = Counter(0);
    crate::MineralSeam::new(BlockMedium::GoldOre).emit_ore_cube(GridCell::new(0, 0), 0.0, &frame, &mut c);
    assert!(c.0 > 20);
}
