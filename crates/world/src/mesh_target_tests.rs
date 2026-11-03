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
