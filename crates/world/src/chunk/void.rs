use glam::Vec3;
use crate::MeshTarget;
use super::{VoxelCoord, VoxelNeighborhood, VoxelVolume};

/// Null Object representing pure spatial void in the hollow planetary core or outer atmosphere.
///
/// Occupies strictly 0 bytes of memory, fulfilling Pure OOP principles by eliminating NULL.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VoidVolume;

impl VoidVolume {
    /// Primary constructor for a 0-byte void volume.
    pub fn new() -> Self {
        Self
    }
}

impl VoxelVolume for VoidVolume {
    fn occluding_at(&self, _coord: VoxelCoord) -> bool {
        false
    }

    fn inspect_voxel(&self, _coord: VoxelCoord, consumer: &mut dyn FnMut(bool, Vec3)) {
        consumer(false, Vec3::ZERO);
    }

    fn emit_mesh(
        &self,
        _origin: Vec3,
        _neighborhood: &dyn VoxelNeighborhood,
        _target: &mut dyn MeshTarget,
    ) {
        // Pure void emits zero geometry.
    }
}
