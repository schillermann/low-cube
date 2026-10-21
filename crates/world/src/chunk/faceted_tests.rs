use glam::Vec3;
use crate::{CountedMeshTarget, MeshTarget, Quad};
use crate::block::{FormBlock, WedgeHeading};
use crate::frame::SurfaceFrame;
use super::{FacetedVolume, VoidNeighborhood, VoidVolume, VoxelVolume};

struct SilentCollector;

impl MeshTarget for SilentCollector {
    fn push_quad(&mut self, _quad: Quad) {}
}

#[test]
fn test_faceted_chunk_with_wedge_mesh_and_slope_constraint() {
    let void = Box::new(VoidVolume::new());
    // Create a ramp wedge ascending in +U direction (East)
    let wedge = Box::new(FormBlock::wedge(0, 0, 0.0, WedgeHeading::East));
    let faceted = FacetedVolume::adorned(void).with_form(wedge);

    // Check mesh emission
    let mut collector = SilentCollector;
    let mut counted = CountedMeshTarget::new(&mut collector);
    faceted.emit_mesh(Vec3::new(16.0, 0.0, 0.0), &VoidNeighborhood, &mut counted);
    assert!(counted.count() > 0);

    // Frame with normal in +Y
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);

    // At u = 0.25 (middle of the ramp): elevation should be exactly 0.25m
    let base = Vec3::new(0.25, 0.0, 0.25);
    let constrained = faceted.constrain_surface(base, 0.25, 0.25, &frame, 1.0);
    assert_eq!(constrained.y, 1.25); // base (0.25) + distance (1.0) = 1.25m

    // At u = 0.0 (foot of the ramp): elevation should be 0.0m + distance 1.0m = 1.0m
    let foot_base = Vec3::new(0.0, 0.0, 0.25);
    let foot_constrained = faceted.constrain_surface(foot_base, 0.0, 0.25, &frame, 1.0);
    assert_eq!(foot_constrained.y, 1.0);

    // At u = 0.5 (crest of the ramp): elevation should be 0.5m + distance 1.0m = 1.5m
    let crest_base = Vec3::new(0.5, 0.0, 0.25);
    let crest_constrained = faceted.constrain_surface(crest_base, 0.5, 0.25, &frame, 1.0);
    assert_eq!(crest_constrained.y, 1.5);
}

#[test]
fn test_faceted_chunk_with_slab_constraint() {
    let void = Box::new(VoidVolume::new());
    // Create a 25cm rock shelf slab
    let slab = Box::new(FormBlock::slab(0, 0, 0.0));
    let faceted = FacetedVolume::adorned(void).with_form(slab);

    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let base = Vec3::new(0.25, 0.0, 0.25);
    let constrained = faceted.constrain_surface(base, 0.25, 0.25, &frame, 1.0);
    // Slab elevation is 0.25m, plus 1.0m distance = 1.25m
    assert_eq!(constrained.y, 1.25);
}
