use glam::Vec3;
use crate::{Block, CubeFace, Face, PlanarFace, StoneBlock};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;

#[test]
fn test_voxel_cluster_decorates_face() {
    let face = Box::new(PlanarFace::on_cube(CubeFace::Top, Vec3::ONE, 8.0, 16));
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 8.0);
    let blocks: Vec<Box<dyn Block>> = vec![
        Box::new(StoneBlock::cube(0, 0, 0.0)),
    ];
    let cluster = VoxelCluster::new(face, frame, blocks);

    assert_eq!(cluster.surface_up(), Vec3::Y);
    assert_eq!(cluster.pull_direction(), Vec3::NEG_Y);

    // On the cube at (0.5, 0.5): base floor is 8.0 + 1.0 (cube) + 1.0 (player) = 10.0
    let supported = cluster.constrain(Vec3::new(0.5, 5.0, 0.5), 1.0);
    assert_eq!(supported.y, 10.0);

    // Outside the cube at (2.0, 2.0): base floor is 8.0 + 1.0 = 9.0
    let flat = cluster.constrain(Vec3::new(2.0, 5.0, 2.0), 1.0);
    assert_eq!(flat.y, 9.0);
}
