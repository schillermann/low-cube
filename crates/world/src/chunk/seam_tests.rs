use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::{
    PlanetaryNeighborhood, SparsePlanetaryGrid, StratifiedVolume, VolumeCoord,
    VoxelCoord, VoxelNeighborhood, VOLUME_EDGE_LENGTH,
};

#[test]
fn test_cross_chunk_boundary_occlusion_culling() {
    let mut grid = SparsePlanetaryGrid::planetary(8, 1);
    let mut chunk_a = StratifiedVolume::planetary();
    let mut chunk_b = StratifiedVolume::planetary();

    // Fill adjacent edge in chunk_a at x = 15
    for z in 0..VOLUME_EDGE_LENGTH {
        for y in 0..VOLUME_EDGE_LENGTH {
            chunk_a.place(VoxelCoord::new(15, y, z), 1);
        }
    }

    // Fill adjacent edge in chunk_b at x = 0
    for z in 0..VOLUME_EDGE_LENGTH {
        for y in 0..VOLUME_EDGE_LENGTH {
            chunk_b.place(VoxelCoord::new(0, y, z), 1);
        }
    }

    let coord_a = VolumeCoord::new(0, 0, 0);
    let coord_b = VolumeCoord::new(1, 0, 0);
    grid.install_volume(coord_a, Box::new(chunk_a));
    grid.install_volume(coord_b, Box::new(chunk_b));

    // Test neighborhood query across boundary
    let neighborhood_a = PlanetaryNeighborhood::new(&grid, coord_a);
    // At x = 16 (relative to chunk A), this maps to chunk B x = 0: should be occluding!
    assert!(neighborhood_a.occluding_at(16, 0, 0));
    assert!(neighborhood_a.occluding_at(16, 5, 5));

    // At x = -1 (open void): should not be occluding
    assert!(!neighborhood_a.occluding_at(-1, 0, 0));
}

#[test]
fn test_90_degree_seam_quad_alignment_at_corner() {
    let mut grid = SparsePlanetaryGrid::planetary(8, 1);
    let mut chunk = StratifiedVolume::planetary();

    // Place a corner line of voxels at top-right (x = 15, y = 15, z in 0..16)
    for z in 0..VOLUME_EDGE_LENGTH {
        chunk.place(VoxelCoord::new(15, 15, z), 1);
    }

    let coord = VolumeCoord::new(0, 0, 0);
    grid.install_volume(coord, Box::new(chunk));

    let neighborhood = PlanetaryNeighborhood::new(&grid, coord);

    // Collect quads
    struct QuadCollector {
        quads: Vec<Quad>,
    }
    impl MeshTarget for QuadCollector {
        fn push_quad(&mut self, quad: Quad) {
            self.quads.push(quad);
        }
    }

    let mut collector = QuadCollector { quads: Vec::new() };
    grid.deliver_volume(coord, |c| {
        c.emit_mesh(Vec3::ZERO, &neighborhood, &mut collector);
    });

    // The corner strip of 16 voxels should have:
    // 1 merged +Y quad (normal Y) from x=15 to 16, y=16, z=0 to 16
    // 1 merged +X quad (normal X) from x=16, y=15 to 16, z=0 to 16
    let y_quads = collector.quads.iter().filter(|q| {
        q.transfer(|_, _, _, _, n, _| n == Vec3::Y)
    }).count();
    let x_quads = collector.quads.iter().filter(|q| {
        q.transfer(|_, _, _, _, n, _| n == Vec3::X)
    }).count();

    assert_eq!(y_quads, 1);
    assert_eq!(x_quads, 1);
}
