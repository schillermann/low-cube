use glam::Vec3;
use crate::{
    ColoredMeshTarget, CountedMeshTarget, CrateredMeshTarget, MeshTarget, OffsetMeshTarget, Quad,
    TintedMeshTarget, TurfMeshTarget,
};

struct QuadCatcher {
    quads: Vec<Quad>,
}

impl QuadCatcher {
    fn empty() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for QuadCatcher {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

#[test]
fn test_mesh_target_decorators_composition() {
    let mut catcher = QuadCatcher::empty();
    let original = Quad::new(
        Vec3::ZERO,
        Vec3::X,
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::Y,
        Vec3::Z,
        Vec3::ONE,
    );

    {
        let mut counted = CountedMeshTarget::new(&mut catcher);
        let mut tinted = TintedMeshTarget::new(&mut counted, Vec3::new(0.5, 0.5, 0.5));
        let mut colored = ColoredMeshTarget::new(&mut tinted, Vec3::new(1.0, 0.0, 0.0));
        let mut offset = OffsetMeshTarget::new(&mut colored, Vec3::new(10.0, 20.0, 30.0));

        offset.push_quad(original);
        assert_eq!(counted.count(), 1);
    }

    assert_eq!(catcher.quads.len(), 1);
    let emitted = catcher.quads[0];
    emitted.transfer(|v0, _v1, _v2, _v3, normal, color| {
        assert_eq!(v0, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(normal, Vec3::Z);
        // Colored set to red (1, 0, 0), then Tinted multiplied by 0.5 -> (0.5, 0.0, 0.0)
        assert_eq!(color, Vec3::new(0.5, 0.0, 0.0));
    });

    let mut crater_catcher = QuadCatcher::empty();
    {
        let mut cratered = CrateredMeshTarget::new(&mut crater_catcher, 2);
        let corners = [Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 1.0), Vec3::Z];
        let mut turf = TurfMeshTarget::new(&mut cratered, Vec3::Y, corners, Vec3::X, Vec3::Z, false);
        turf.push_quad(Quad::new(corners[0], corners[1], corners[2], corners[3], Vec3::Y, Vec3::ONE));
        turf.seal_underside();
    }
    assert_eq!(crater_catcher.quads.len(), 18, "Two quads through stage-2 CrateredMeshTarget must emit 18 crater sub-quads");
}
