use glam::Vec3;
use crate::{
    CountedMeshTarget, CrateredMeshTarget, MeshTarget, Quad, TurfMeshTarget, WorldEarthworks,
};

struct QuadLedger {
    quads: Vec<Quad>,
}

impl QuadLedger {
    fn empty() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for QuadLedger {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

#[test]
fn test_mesh_target_decorators_composition() {
    let mut crater_ledger = QuadLedger::empty();
    {
        let mut counted = CountedMeshTarget::new(&mut crater_ledger);
        let mut cratered = CrateredMeshTarget::new(&mut counted, 2);
        let corners = [Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 1.0), Vec3::Z];
        let mut turf = TurfMeshTarget::new(&mut cratered, Vec3::Y, corners, Vec3::X, Vec3::Z, false);
        turf.push_quad(Quad::new(corners[0], corners[1], corners[2], corners[3], Vec3::Y, Vec3::ONE));
        turf.seal_underside();
        assert_eq!(counted.count(), 18, "Two quads through stage-2 CrateredMeshTarget must emit 18 crater sub-quads");
    }
    assert_eq!(crater_ledger.quads.len(), 18);
}

#[test]
fn test_low_poly_grass_emits_3d_pyramid_blades_on_verdant_turf() {
    use crate::block::{EarthClods, GridCell};
    use crate::topology::ClimateZone;

    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let grass_col = Vec3::new(0.28, 0.65, 0.25);
    let cell = GridCell::new(0, 0);
    let mut block_ledger = QuadLedger::empty();
    EarthClods::new(cell, 0.0).emit_terraced(grass_col, false, &frame, &mut block_ledger);

    let contour_top_quads = block_ledger.quads.iter().filter(|q| {
        q.radiate(|g| g.abs() < 1e-4) && q.transfer(|v0, v1, v2, v3, n, col| {
            n.y > 0.9 && (v0.y + v1.y + v2.y + v3.y) * 0.25 > 16.501 && col.y > col.x * 1.15
        })
    }).count();
    assert!(contour_top_quads >= 4, "Verdant dirt block must emit multi-depth 3D contour terrace top quads above 16.5m, got {contour_top_quads}");
    let blade_tuft_quads = block_ledger.quads.iter().filter(|q| {
        q.radiate(|g| g >= 2.0)
    }).count();
    assert_eq!(blade_tuft_quads, 18, "Verdant dirt block must emit the original 6 3-wing grass tufts (18 quads)");
    let turf_rim_quads = block_ledger.quads.iter().filter(|q| {
        q.transfer(|v0, v1, v2, v3, n, col| {
            n.y.abs() < 0.1 && (v0.y + v1.y + v2.y + v3.y) * 0.25 > 16.34 && col.y > col.x * 1.15
        })
    }).count();
    assert!(turf_rim_quads >= 32, "Verdant dirt block must emit overhanging green turf rim and sod fringes on all 4 flanks, got {turf_rim_quads}");

    let mut fan_ledger = QuadLedger::empty();
    let mut meadow = crate::NaturalMeadowTarget::for_cell(&mut fan_ledger, cell, frame.normal());
    let corners = cell.corner_points(&frame, 0.5);
    meadow.push_quad(Quad::glowing(corners[0], corners[1], corners[2], corners[3], frame.normal(), grass_col, 0.02));
    assert!(fan_ledger.quads.len() >= 9, "NaturalMeadowTarget must emit 3D contour terraces and original grass tufts on a verdant cell, got {}", fan_ledger.quads.len());

    let mut dug_ledger = QuadLedger::empty();
    let mut cratered = CrateredMeshTarget::new(&mut dug_ledger, 2);
    cratered.push_quad(Quad::new(corners[0], corners[1], corners[2], corners[3], frame.normal(), grass_col));
    let mut dug_turf = QuadLedger::empty();
    let mut dug_meadow = crate::NaturalMeadowTarget::for_cell(&mut dug_turf, cell, frame.normal());
    for quad in &dug_ledger.quads {
        dug_meadow.push_quad(*quad);
    }
    assert_eq!(dug_turf.quads.len(), 0, "Digging stages 1..=3 must suppress 3D sod clods on cratered sub-quads");
}

#[test]
fn test_local_camera_meadow_is_constant_o1_across_world_cube_scales() {
    use crate::{BiomedQuadSphere, TerrainMesh};

    let small_world = BiomedQuadSphere::of_size(96);
    let large_world = BiomedQuadSphere::of_size(160);
    let (mut s_surf, mut l_surf) = (QuadLedger::empty(), QuadLedger::empty());
    small_world.emit_face(0, &mut s_surf);
    large_world.emit_face(0, &mut l_surf);

    let (mut small_meadow, mut large_meadow) = (QuadLedger::empty(), QuadLedger::empty());
    small_world.emit_meadow_around(Vec3::new(0.0, 49.0, 0.0), Vec3::NEG_Z, &mut small_meadow);
    large_world.emit_meadow_around(Vec3::new(0.0, 81.0, 0.0), Vec3::NEG_Z, &mut large_meadow);

    assert!(small_meadow.quads.len() > 15000 && small_meadow.quads.len() < 75000, "got {}", small_meadow.quads.len());
    assert!(
        (small_meadow.quads.len() as i32 - large_meadow.quads.len() as i32).abs() < 12000,
        "Camera-centered local meadow must remain O(1) bounded (got {} vs {}) even when world-cube area scales up",
        small_meadow.quads.len(),
        large_meadow.quads.len()
    );
}

#[test]
fn test_sector_patch_grid_and_toroidal_meadow_ring_update_o1_subpatches() {
    use crate::{BiomedQuadSphere, MeadowRing, PlanetaryWorld, TerrainMesh, MEADOW_RING_SLOTS};

    let mut world = BiomedQuadSphere::barren(64);
    assert_eq!(world.face_patch_count(0), 64, "64-block face (128 half-meter cells) must partition into 8x8 = 64 sectors");
    let mut init_buf = QuadLedger::empty();
    world.emit_face_patch(0, 0, &mut init_buf);
    assert!(!init_buf.quads.is_empty());

    let mut ring = MeadowRing::cold();
    let center0 = Vec3::new(0.25, 34.0, 0.25);
    let mut warm_count = 0usize;
    while world.drain_stale_meadow_patch(center0, &mut ring, &mut QuadLedger::empty()).is_some() {
        warm_count += 1;
    }
    assert_eq!(warm_count, MEADOW_RING_SLOTS);
    assert!(world.drain_stale_meadow_patch(center0, &mut ring, &mut QuadLedger::empty()).is_none());

    let g = world.constrain_to_surface(Vec3::new(0.25, 50.0, 0.25), 0.0).y;
    assert!(world.loosen_surface(Vec3::new(0.25, g + 1.5, 0.25), Vec3::NEG_Y, 2).is_some());
    let mut dirty_terrain = 0usize;
    while world.drain_modified_patch(&mut QuadLedger::empty()).is_some() {
        dirty_terrain += 1;
    }
    assert_eq!(dirty_terrain, 1, "Single dig crack must invalidate only 1 of 64 sectors");
    assert!(!world.pending_slices(), "Draining sector patches must clear terrain slice bit");

    let mut dig_meadow = 0usize;
    while world.drain_stale_meadow_patch(center0, &mut ring, &mut QuadLedger::empty()).is_some() {
        dig_meadow += 1;
    }
    assert_eq!(dig_meadow, 1, "Single dig crack must refresh only 1 of 81 toroidal meadow slots");

    let center1 = Vec3::new(9.0, 34.0, 0.25);
    let mut step_meadow = 0usize;
    while world.drain_stale_meadow_patch(center1, &mut ring, &mut QuadLedger::empty()).is_some() {
        step_meadow += 1;
    }
    assert!(
        step_meadow > 0 && step_meadow <= 30,
        "Crossing one 8m sector while driving must keep >= 51 of 81 grass patches in VRAM (updated {step_meadow})"
    );
}
