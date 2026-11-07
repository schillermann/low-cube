use glam::Vec3;
use crate::{
    CountedMeshTarget, CrateredMeshTarget, MeshTarget, Quad, TurfMeshTarget,
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
    let mut block_ledger = QuadLedger::empty();
    EarthClods::new(GridCell::new(0, 0), 0.0).emit_terraced(grass_col, false, &frame, &mut block_ledger);

    let grass_quads = block_ledger.quads.iter().filter(|q| q.radiate(|g| g > 1.5)).count();
    assert_eq!(grass_quads, 18, "Verdant dirt block must emit 6 30°-splayed 3-wing cross-billboard fans (18 quads = 126 cutout blades)");

    let mut dug_ledger = QuadLedger::empty();
    let mut cratered = CrateredMeshTarget::new(&mut dug_ledger, 2);
    for quad in &block_ledger.quads {
        cratered.push_quad(*quad);
    }
    let dug_grass = dug_ledger.quads.iter().filter(|q| q.radiate(|g| g > 1.5)).count();
    assert_eq!(dug_grass, 0, "Digging stages 1..=3 must suppress 3D grass blade pyramids on cratered blocks");
}

#[test]
fn test_local_camera_meadow_is_constant_o1_across_world_cube_scales() {
    use crate::{BiomedCubeWorld, PlanetaryWorld};

    let small_world = BiomedCubeWorld::beveled(16, 1.0);
    let large_world = BiomedCubeWorld::beveled(64, 1.0);
    let (mut s_surf, mut l_surf) = (QuadLedger::empty(), QuadLedger::empty());
    small_world.emit_face(0, &mut s_surf);
    large_world.emit_face(0, &mut l_surf);

    let (mut small_meadow, mut large_meadow) = (QuadLedger::empty(), QuadLedger::empty());
    small_world.emit_meadow_around(Vec3::new(0.0, 9.0, 0.0), Vec3::NEG_Z, &mut small_meadow);
    large_world.emit_meadow_around(Vec3::new(0.0, 33.0, 0.0), Vec3::NEG_Z, &mut large_meadow);

    assert!(small_meadow.quads.len() > 1400 && small_meadow.quads.len() < 3500);
    assert!(
        (small_meadow.quads.len() as i32 - large_meadow.quads.len() as i32).abs() < 150,
        "Camera-centered local meadow must remain O(1) bounded (~2200 quads) even when world-cube area scales up 16x"
    );
}
