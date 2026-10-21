use glam::Vec3;
use crate::{Block, ClimateZone, Face, PlanarFace, StoneBlock};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;

#[test]
fn test_voxel_cluster_decorates_face() {
    let face = Box::new(PlanarFace::in_zone(ClimateZone::Temperate, Vec3::ONE, 8.0, 16));
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 8.0);
    let blocks: Vec<Box<dyn Block>> = vec![
        Box::new(StoneBlock::cube(0, 0, 0.0)),
    ];
    let cluster = VoxelCluster::new(face, frame, blocks);

    assert_eq!(cluster.surface_up(), Vec3::Y);
    assert_eq!(cluster.pull_direction(), Vec3::NEG_Y);

    // On the cube at (0.25, 0.25): base floor is 8.0 + 0.5 (cube) + 1.0 (player) = 9.5
    let supported = cluster.constrain(Vec3::new(0.25, 5.0, 0.25), 1.0);
    assert_eq!(supported.y, 9.5);

    // Outside the cube at (2.0, 2.0): base floor is 8.0 + 1.0 = 9.0
    let flat = cluster.constrain(Vec3::new(2.0, 5.0, 2.0), 1.0);
    assert_eq!(flat.y, 9.0);
}
