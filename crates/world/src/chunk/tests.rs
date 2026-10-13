use glam::Vec3;
use crate::{CountedMeshTarget, MeshTarget, Quad};
use super::{
    MonolithVolume, SparsePlanetaryGrid, StratifiedVolume, VoidNeighborhood, VoidVolume,
    VolumeCoord, VoxelCoord, VoxelNeighborhood, VoxelVolume,
};

struct SilentMeshCollector;

impl MeshTarget for SilentMeshCollector {
    fn push_quad(&mut self, _quad: Quad) {}
}

struct OccludingNeighborhood;

impl VoxelNeighborhood for OccludingNeighborhood {
    fn occluding_at(&self, _x: i32, _y: i32, _z: i32) -> bool {
        true
    }

    fn color_at(&self, _x: i32, _y: i32, _z: i32) -> Vec3 {
        Vec3::ONE
    }
}

#[test]
fn test_void_volume_zero_size_and_empty_emission() {
    assert_eq!(std::mem::size_of::<VoidVolume>(), 0);
    let void = VoidVolume::new();
    assert!(!void.occluding_at(VoxelCoord::new(0, 0, 0)));

    let mut inspected_solid = true;
    void.inspect_voxel(VoxelCoord::new(0, 0, 0), &mut |solid, _| {
        inspected_solid = solid;
    });
    assert!(!inspected_solid);

    let mut collector = SilentMeshCollector;
    let mut counted = CountedMeshTarget::new(&mut collector);
    void.emit_mesh(Vec3::ZERO, &VoidNeighborhood, &mut counted);
    assert_eq!(counted.count(), 0);
}

#[test]
fn test_monolith_volume_face_culling_and_mesh_emission() {
    let monolith = MonolithVolume::new(Vec3::new(0.5, 0.5, 0.5));
    assert!(monolith.occluding_at(VoxelCoord::new(0, 0, 0)));

    // In open void: all 6 faces emitted
    let mut collector = SilentMeshCollector;
    let mut counted = CountedMeshTarget::new(&mut collector);
    monolith.emit_mesh(Vec3::ZERO, &VoidNeighborhood, &mut counted);
    assert_eq!(counted.count(), 6);

    // Fully surrounded by occluding volumes: 0 faces emitted
    let mut counted_occluded = CountedMeshTarget::new(&mut collector);
    monolith.emit_mesh(Vec3::ZERO, &OccludingNeighborhood, &mut counted_occluded);
    assert_eq!(counted_occluded.count(), 0);
}

#[test]
fn test_stratified_volume_culling_internal_faces() {
    let mut stratified = StratifiedVolume::planetary();

    // Place a 2x2x2 cube of stone (id 1)
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                stratified.place(VoxelCoord::new(x, y, z), 1);
            }
        }
    }

    // In a 2x2x2 solid block, 24 internal faces are culled, and greedy meshing
    // merges the 4 exposed coplanar faces on each of the 6 sides into 1 single quad.
    // Exactly 6 outer quads are emitted.
    let mut collector = SilentMeshCollector;
    let mut counted = CountedMeshTarget::new(&mut collector);
    stratified.emit_mesh(Vec3::ZERO, &VoidNeighborhood, &mut counted);
    assert_eq!(counted.count(), 6);
}

#[test]
fn test_sparse_planetary_grid_hollow_core_savings() {
    // 128^3 pocket moonlet (8x8x8 volumes)
    let moonlet = SparsePlanetaryGrid::planetary(8, 1);
    assert_eq!(moonlet.total_volume_count(), 512);
    assert_eq!(moonlet.hollow_core_volume_count(), 216);
    let moonlet_saving_ratio = 216.0 / 512.0;
    assert!(moonlet_saving_ratio > 0.42);

    // 1024^3 standard terrestrial world (64x64x64 volumes)
    let terrestrial = SparsePlanetaryGrid::planetary(64, 1);
    assert_eq!(terrestrial.total_volume_count(), 262_144);
    assert_eq!(terrestrial.hollow_core_volume_count(), 238_328);
    let terrestrial_saving_ratio = 238_328.0 / 262_144.0;
    assert!(terrestrial_saving_ratio > 0.909);

    // Core coordinates resolve to VoidVolume without allocation
    let core_coord = VolumeCoord::new(10, 10, 10);
    assert!(terrestrial.in_hollow_core(core_coord));
    terrestrial.deliver_volume(core_coord, |volume| {
        let mut is_solid = true;
        volume.inspect_voxel(VoxelCoord::new(0, 0, 0), &mut |solid, _| {
            is_solid = solid;
        });
        assert!(!is_solid);
    });
}
