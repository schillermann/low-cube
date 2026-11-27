use glam::Vec3;
use crate::{
    BiomedQuadSphere, BlockMedium, BlockMold, BodyId, CardinalTurn, HeliosphereWorld, MeshTarget,
    MoldArchetype, PlanetaryWorld, Quad, SightCone, SolarEphemeris, SolarProportions, TerrainMesh,
    WorldEarthworks, WorldFact, OFFWORLD_IDLE_TTL_SECS,
};

struct EmeraldCounter {
    emerald_quads: usize,
}

impl MeshTarget for EmeraldCounter {
    fn push_quad(&mut self, quad: Quad) {
        let glowing = quad.radiate(|g| g > 0.9);
        if glowing {
            quad.transfer(|_, _, _, _, _, col| {
                if col.y > 0.95 && col.z > 0.50 && col.x < 0.65 {
                    self.emerald_quads += 1;
                }
            });
        }
    }
}

#[test]
fn test_offworld_vault_evicts_idle_body_into_compact_sector_snapshots_and_rehydrates_cleanly() {
    let prop = SolarProportions::true_scale(128);
    let earth = BiomedQuadSphere::barren(prop.earth_blocks());
    let mut helio = HeliosphereWorld::assembled(earth, SolarEphemeris::new(prop, 0.0), true);

    let (mars_probe, mars_half) = crate::AlienWorld::build_body(BodyId::MARS, prop, false);
    let g0 = mars_probe.constrain_to_surface(Vec3::new(0.25, mars_half + 6.0, 0.25), 0.0);
    let g1 = mars_probe.constrain_to_surface(Vec3::new(0.75, mars_half + 6.0, 0.25), 0.0);
    let g2 = mars_probe.constrain_to_surface(Vec3::new(1.25, mars_half + 6.0, 0.25), 0.0);
    let beacon_mold = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::LaserBeacon);

    // Mutate Mars in the same 4m x 4m sector: 2 harvests + 1 LaserBeacon placement = 3 raw raycast facts
    assert!(helio.harvest_offworld(BodyId::MARS, g0 + Vec3::Y * 1.2, Vec3::NEG_Y).is_some());
    assert!(helio.harvest_offworld(BodyId::MARS, g1 + Vec3::Y * 1.2, Vec3::NEG_Y).is_some());
    let place = WorldFact::MoldPlaced {
        eye: g2 + Vec3::Y * 1.2,
        direction: Vec3::NEG_Y,
        avatar_pos: g2 + Vec3::Z * 2.5,
        mold: beacon_mold,
    };
    assert!(helio.enact_offworld(BodyId::MARS, &place));
    assert_eq!(helio.warm_offworld_count(), 1, "Mars must be warm immediately after off-world mutations");
    assert_eq!(helio.cold_chronicle_count(BodyId::MARS), 3, "Before cold eviction, 3 raw raycast facts are queued");

    // While Mars is inhabited, stepping past TTL must keep Mars warm in RAM
    let _ = helio.step_active_zones(120.0, 0, &[BodyId::MARS]);
    let _ = helio.step_active_zones(120.0, 0, &[BodyId::MARS]);
    assert_eq!(helio.warm_offworld_count(), 1, "Inhabited off-world body must never be evicted from RAM");

    // Once uninhabited (`active_bodies = &[]`), stepping past OFFWORLD_IDLE_TTL_SECS folds & drops QuadSphere
    let _ = helio.step_active_zones(OFFWORLD_IDLE_TTL_SECS + 5.0, 0, &[]);
    assert_eq!(helio.warm_offworld_count(), 0, "Uninhabited Mars must drop QuadSphere from RAM after Idle-TTL");
    assert_eq!(
        helio.cold_chronicle_count(BodyId::MARS),
        1,
        "Cold eviction must fold 3 raycast facts in the same sector into 1 raycast-free SectorSnapshot"
    );

    // Cold Mars still reports its placed LaserBeacon directly from the compact SectorSnapshot without waking QuadSphere!
    let mut cold_beacons = Vec::new();
    helio.collect_offworld_medium_sites(BlockMedium::LaserBeacon, &mut cold_beacons);
    assert_eq!(cold_beacons.len(), 1);
    assert_eq!(cold_beacons[0].0, BodyId::MARS);
    assert_eq!(helio.warm_offworld_count(), 0, "Scanning cold SectorSnapshots must not rehydrate QuadSphere");

    // Returning to Mars rehydrates identical sector state (placed LaserBeacon + 2 excavated shafts)
    let temp_face = crate::ClimateZone::Temperate.index() as u8;
    let rehydrated = helio.capture_offworld_snapshot(BodyId::MARS, temp_face, 0, 0);
    assert_eq!(helio.warm_offworld_count(), 1);
    assert!(
        rehydrated.columns().blocks().iter().any(|b| b.stamp()[2] == BlockMedium::LaserBeacon.code()),
        "Rehydrated Mars sector must preserve placed LaserBeacon"
    );
    assert_eq!(
        rehydrated.columns().shafts().len(),
        2,
        "Rehydrated Mars sector must preserve both excavated shafts"
    );
}

#[test]
fn test_singleplayer_laser_beacon_shines_from_orbit_above_horizon_and_hides_behind_planet() {
    let prop = SolarProportions::true_scale(128);
    let earth_half = prop.earth_half();
    let earth = BiomedQuadSphere::barren(prop.earth_blocks());
    let mut helio = HeliosphereWorld::assembled(earth, SolarEphemeris::new(prop, 0.0), true);

    let g = helio.constrain_to_surface(Vec3::new(0.25, earth_half + 4.0, 0.25), 0.0);
    let beacon_mold = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::LaserBeacon);
    assert!(helio.place_molded_surface_block(g + Vec3::Y * 1.5, Vec3::NEG_Y, g + Vec3::Z * 2.5, beacon_mold));

    let mut orbit_above = EmeraldCounter { emerald_quads: 0 };
    let cone_above = SightCone::from_view(Vec3::new(0.0, earth_half + 1_200.0, 0.0), Vec3::NEG_Y);
    helio.emit_cosmic_field(cone_above, &mut orbit_above);
    assert!(
        orbit_above.emerald_quads >= 2,
        "Orbital viewer above Temperate must see the coherent emerald OpticalBeacon"
    );

    let mut orbit_behind = EmeraldCounter { emerald_quads: 0 };
    let cone_behind = SightCone::from_view(Vec3::new(0.0, -(earth_half + 1_200.0), 0.0), Vec3::Y);
    helio.emit_cosmic_field(cone_behind, &mut orbit_behind);
    assert_eq!(
        orbit_behind.emerald_quads,
        0,
        "Orbital viewer on Volcanic side must see 0 OpticalBeacon quads through the planet"
    );
}
