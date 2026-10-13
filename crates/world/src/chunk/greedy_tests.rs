use glam::Vec3;
use crate::{CountedMeshTarget, MeshTarget, Quad};
use super::{
    FacetSlice, StratifiedVolume, VoidNeighborhood, VoxelCoord, VoxelVolume,
    VOLUME_EDGE_LENGTH,
};

struct SilentCollector;

impl MeshTarget for SilentCollector {
    fn push_quad(&mut self, _quad: Quad) {}
}

#[test]
fn test_facet_slice_uniform_merge_single_quad() {
    let mut slice = FacetSlice::new();
    for v in 0..VOLUME_EDGE_LENGTH {
        for u in 0..VOLUME_EDGE_LENGTH {
            slice.record(u, v, 1);
        }
    }

    let mut merged_rectangles = Vec::new();
    slice.merge_into(&mut |u, v, w, h, id| {
        merged_rectangles.push((u, v, w, h, id));
    });

    assert_eq!(merged_rectangles.len(), 1);
    assert_eq!(merged_rectangles[0], (0, 0, 16, 16, 1));
}

#[test]
fn test_greedy_chunk_2x2x2_block_quad_reduction() {
    let mut chunk = StratifiedVolume::planetary();

    // Place a 2x2x2 solid cube
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                chunk.place(VoxelCoord::new(x, y, z), 1);
            }
        }
    }

    let mut collector = SilentCollector;
    let mut counted = CountedMeshTarget::new(&mut collector);
    chunk.emit_mesh(Vec3::ZERO, &VoidNeighborhood, &mut counted);

    // Naive meshing with face culling would emit 24 individual 1x1 quads.
    // Greedy meshing merges each 2x2 face into 1 single quad, giving exactly 6 quads!
    assert_eq!(counted.count(), 6);
    let naive_quad_count = 24.0;
    let greedy_quad_count = counted.count() as f64;
    let reduction_ratio = 1.0 - (greedy_quad_count / naive_quad_count);
    assert!(
        reduction_ratio >= 0.70,
        "Reduction ratio {} must be at least 70%",
        reduction_ratio
    );
}

#[test]
fn test_greedy_chunk_full_solid_16x16x16_quad_reduction() {
    let mut chunk = StratifiedVolume::planetary();

    // Fill entire chunk
    for z in 0..VOLUME_EDGE_LENGTH {
        for y in 0..VOLUME_EDGE_LENGTH {
            for x in 0..VOLUME_EDGE_LENGTH {
                chunk.place(VoxelCoord::new(x, y, z), 1);
            }
        }
    }

    let mut collector = SilentCollector;
    let mut counted = CountedMeshTarget::new(&mut collector);
    chunk.emit_mesh(Vec3::ZERO, &VoidNeighborhood, &mut counted);

    // Naive without greedy: 6 * 16 * 16 = 1536 quads.
    // Greedy meshing: 6 quads (1 per 16x16 face).
    assert_eq!(counted.count(), 6);
    let naive_quads = 1536.0;
    let reduction = 1.0 - (6.0 / naive_quads);
    assert!(reduction > 0.99, "Reduction {} exceeds 99%", reduction);
}
