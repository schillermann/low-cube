use glam::{DVec3, Vec3};
use crate::{
    BiomedQuadSphere, BlockMedium, BlockMold, Brad64, CelestialSystem, ClimateZone, CosmicFrame,
    CosmicPos, CountedMeshTarget, GravityBubble, GravityField, LunarWorld, MeshTarget, Quad,
    SolarEphemeris, SolarProportions, TerrainMesh, WorldEarthworks,
};

struct VoidSink;

impl MeshTarget for VoidSink {
    fn push_quad(&mut self, _quad: Quad) {}
}

#[test]
fn cosmic_pos_preserves_sub_millimeter_precision_at_thirty_au() {
    let neptune_au = 30.1 * SolarProportions::EPSTEIN_HOUR_AU_CAP;
    let cam_d = DVec3::new(neptune_au, -142_000.0, 88_500.0);
    let cam = CosmicPos::from_dvec3(cam_d);
    let bolt = cam.shifted_ticks(12_503, -4, 80_001);
    let rel = bolt.relative_f32(cam);
    assert!((rel.x - 1.2503).abs() < 1e-6, "got {}", rel.x);
    assert!((rel.y - (-0.0004)).abs() < 1e-6, "got {}", rel.y);
    assert!((rel.z - 8.0001).abs() < 1e-5, "got {}", rel.z);

    let mut wire = Vec::new();
    bolt.inscribe(&mut wire);
    let (decoded, used) = CosmicPos::decode(&wire).expect("24-byte CosmicPos");
    assert_eq!(used, 24);
    assert_eq!(decoded, bolt);
}

#[test]
fn brad64_and_kepler_ephemeris_are_cross_os_bit_deterministic() {
    for deg in [0.0_f64, 15.0, 45.0, 90.0, 137.5, 180.0, 225.0, 311.25, -73.4] {
        let rad = deg.to_radians();
        let (s, c) = Brad64::from_radians(rad).sin_cos_f64();
        let (ref_s, ref_c) = rad.sin_cos();
        assert!((s - ref_s).abs() < 1e-12, "sin mismatch at {deg}: {s} vs {ref_s}");
        assert!((c - ref_c).abs() < 1e-12, "cos mismatch at {deg}: {c} vs {ref_c}");
        let y = ref_s * 3.7;
        let x = ref_c * 3.7;
        let atan = Brad64::atan2_radians(y, x);
        let expected = rad.rem_euclid(std::f64::consts::TAU);
        let diff = (atan - expected).abs().min(std::f64::consts::TAU - (atan - expected).abs());
        assert!(diff < 1e-11, "atan2 mismatch at {deg}: {atan} vs {expected}");
    }

    let props = SolarProportions::true_scale(128);
    let eph_a = SolarEphemeris::new(props, 3_600.0);
    let eph_b = SolarEphemeris::new(props, 3_600.0);
    assert_eq!(eph_a.deterministic_digest(), eph_b.deterministic_digest());
    assert_ne!(eph_a.deterministic_digest(), eph_a.advanced(0.5).deterministic_digest());

    let bubble = GravityBubble::from_ephemeris(eph_a);
    let snaps = eph_a.snapshot_all(0.0);
    let luna = snaps[4];
    let approach_luna = luna.cosmic_locus().relative_f32(CosmicPos::origin()) + Vec3::new(0.0, 24.0, 0.0);
    let handoff = bubble
        .evaluate_handoff(eph_a, approach_luna, Vec3::ZERO)
        .expect("approaching Luna enters Luna Hill sphere");
    assert_eq!(handoff.next_frame(), CosmicFrame::Body( luna.id() ));
}

#[test]
fn quad_sphere_projects_six_shells_onto_true_sphere_with_zero_seam_gap() {
    let radius = 16.0_f32;
    let planet = BiomedQuadSphere::barren(32);

    for zone in ClimateZone::all() {
        let frame = zone.surface_frame(radius);
        for &(u, v) in &[
            (0.0, 0.0),
            (7.5, -4.25),
            (radius, 0.0),
            (-radius, 6.0),
            (radius, radius),
            (-radius, -radius),
        ] {
            let p = frame.point_at(u, v, 0.0);
            assert!(
                (p.length() - radius).abs() < 1e-4,
                "Zone {zone:?} ({u},{v}) must lie on exact sphere r={radius}, got r={}",
                p.length()
            );
            let (du, dv) = frame.decompose(p);
            assert!((du - u).abs() < 1e-4 && (dv - v).abs() < 1e-4);
            let up = planet.surface_up_at(p);
            assert!((up - p.normalize()).length() < 1e-4);
        }
    }

    let temp_edge = ClimateZone::Temperate.surface_frame(radius).point_at(radius, 5.0, 0.0);
    let desert_edge = ClimateZone::Desert.surface_frame(radius).point_at(-5.0, radius, 0.0);
    assert!(
        (temp_edge - desert_edge).length() < 1e-5,
        "Adjacent QuadSphere shells must meet with 0.0m seam gap, got {:?}",
        temp_edge - desert_edge
    );
}

#[test]
fn untouched_quad_sphere_uses_zero_brick_ram_until_real_edit() {
    let mut earth = BiomedQuadSphere::of_size(64);
    assert_eq!(earth.allocated_brick_bytes(), 0, "Untouched BiomedQuadSphere must use 0 Bytes Brick8 RAM");

    let mut void = VoidSink;
    let mut sink = CountedMeshTarget::new(&mut void);
    earth.emit_face_patch(0, 0, &mut sink);
    assert!(sink.count() > 0, "Height-clipmap must emit terrain quads with 0 Brick8 RAM");
    assert_eq!(earth.allocated_brick_bytes(), 0, "Clipmap emission must not allocate Brick8s");

    let luna = LunarWorld::authoritative();
    assert_eq!(luna.allocated_brick_bytes(), 0, "Untouched LunarWorld must use 0 Bytes Brick8 RAM");

    let helio = CelestialSystem::barren_heliosphere(SolarProportions::compressed(32, 0.02), 1.0);
    assert_eq!(helio.allocated_brick_bytes(), 0, "Untouched HeliosphereWorld must use 0 Bytes Brick8 RAM");

    let placed = earth.place_molded_surface_block(
        Vec3::new(2.25, 34.0, 2.25),
        Vec3::NEG_Y,
        Vec3::new(0.25, 33.0, 0.25),
        BlockMold::cube().in_medium(BlockMedium::SpawnStone),
    );
    assert!(placed, "Building a house block on QuadSphere must succeed");
    assert!(
        earth.allocated_brick_bytes() > 0,
        "Placing a block must instantiate only the affected 8x8x8 Brick8"
    );
}
