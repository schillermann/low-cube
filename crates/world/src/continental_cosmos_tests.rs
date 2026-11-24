use glam::Vec3;
use crate::{
    AsteroidBelt, BodyId, CelestialSystem, CountedMeshTarget, MeshTarget, Quad, SolarProportions,
    SurfaceContact,
};

struct NullSink;
impl MeshTarget for NullSink {
    fn push_quad(&mut self, _quad: Quad) {}
}

#[test]
fn test_continental_4096_heliosphere_initializes_lazily_and_keeps_outer_belt_under_one_hour() {
    let prop = SolarProportions::true_scale(4_096);
    assert_eq!(prop.voxel_blocks(1.0), 4_096, "Earth voxel_blocks must scale up to 4096");
    assert_eq!(prop.voxel_blocks(0.949), 3_888, "Venus must exceed old 512 cap cleanly");
    assert!(
        prop.chemical_speed_cap() > 240.0,
        "Chemical rocket speed cap must scale with sqrt(size) to clear 200.5 m/s escape velocity on 4096³"
    );

    // Worst-case 4.40 AU conjunction transit (Earth aphelion 1.20 AU -> outer Asteroid Belt 3.20 AU) at 8G (78.5 m/s²)
    let worst_dist = 4.40 * prop.astronomical_unit();
    let accel_8g = 78.5_f64;
    let flip_burn_secs = 2.0 * (worst_dist / accel_8g).sqrt();
    let peak_speed = accel_8g * (flip_burn_secs * 0.5);
    assert!(
        flip_burn_secs < 3_600.0,
        "Epstein 8G Flip-&-Burn to outer Mars-Jupiter belt must stay < 1h even at max 4096³ scale, got {flip_burn_secs:.1}s"
    );
    assert!(
        peak_speed < f64::from(prop.fusion_speed_cap()),
        "Fusion speed ceiling must not throttle peak Brachistochrone velocity ({peak_speed:.0} m/s)"
    );

    let world = CelestialSystem::barren_heliosphere(prop, 1.0);
    assert_eq!(
        world.surface_contact(Vec3::new(0.0, 3_500.0, 0.0), 0.0),
        SurfaceContact::Vacuum,
        "High altitude above 4096³ Earth must report Vacuum instead of bouncing rockets off phantom ground"
    );
    assert!(
        world.surface_contact(Vec3::new(0.25, 2_050.0, 0.25), 0.0).landable(),
        "Low altitude over 4096³ Earth crust must report SolidCrust"
    );

    let jup = world.body_center(BodyId::JUPITER).expect("Jupiter center");
    assert!(
        world.surface_contact(jup, 0.0).gaseous(),
        "Gas giant interior must report GasEnvelope rather than SolidCrust"
    );

    let mut sink = NullSink;
    let mut patch_counter = CountedMeshTarget::new(&mut sink);
    world.emit_face_patch(0, 0, &mut patch_counter);
    assert!(patch_counter.count() > 0, "OrbitalShell macro-patch on 4096³ face must emit horizon facets");
}

#[test]
fn test_belt_asteroid_awakens_as_landable_crust_cube() {
    let prop = SolarProportions::compressed(128, 0.02);
    let world = CelestialSystem::barren_heliosphere(prop, 1.0);
    let eph = world.celestial_atlas().ephemeris().expect("ephemeris");
    let sun = eph.snapshot_all(0.0)[0];
    let belt = AsteroidBelt::new(prop, eph.epoch_secs());
    let au = prop.astronomical_unit();

    let mut found_rock = None;
    for step in 0..240 {
        let angle = (step as f64) * 0.026;
        let r = 2.65 * au;
        let probe = sun.center() + glam::DVec3::new(r * angle.cos(), 0.0, r * angle.sin());
        if let Some(rock) = belt.nearest_rock(probe, sun.center(), sun.gm(), 420.0) {
            found_rock = Some(rock);
            break;
        }
    }
    let rock = found_rock.expect("procedural belt asteroid between 2.2 and 3.2 AU");
    let approach = rock.center().as_vec3() + Vec3::new(0.25, rock.half_size() + 2.0, 0.25);
    let contact = world.surface_contact(approach, 0.0);
    assert!(contact.landable(), "BeltRock asteroid must materialize on demand as a landable voxel crust");
}

#[test]
fn test_continental_4096_forested_world_isolates_climate_zones_and_lazy_sector_groves() {
    use crate::{
        BiomedCubeWorld, ClimateZone, FacetedReliefFace, ForestedFace, PlanetaryWorld, TerrainMesh,
    };

    let relief = Box::new(FacetedReliefFace::in_zone(ClimateZone::Temperate, 2_048.0, 8_192));
    let forested = ForestedFace::in_zone(relief, ClimateZone::Temperate, 2_048.0);
    assert_eq!(
        forested.materialized_grove_count(),
        0,
        "4096³ ForestedFace must allocate zero eager tree groves at startup"
    );

    let mut planet = BiomedCubeWorld::beveled(4_096, 1.0);
    let stand = planet.constrain_to_surface(Vec3::new(0.25, 2_050.0, 0.25), 0.0);
    let mask = planet.visible_slice_mask(stand);
    let terrain_bits = mask & 0b0011_1111;
    let flora_bits = (mask >> 6) & 0b0011_1111;
    assert_eq!(
        terrain_bits.count_ones(),
        1,
        "Standing inside Temperate on 4096³ Earth must expose only 1 ClimateZone face, got mask {terrain_bits:#08b}"
    );
    assert_eq!(
        flora_bits,
        0,
        "4096³ continental flora slices must remain 0 because local trees stream via MeadowRing"
    );
    assert!(!planet.step_surface(1.0 / 120.0), "Untouched 4096³ ClimateZones must sleep during step_surface");
}
