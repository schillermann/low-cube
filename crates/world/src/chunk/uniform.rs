use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::{VoxelCoord, VoxelNeighborhood, VoxelVolume, VOLUME_EDGE_LENGTH};

/// Monolithic volume representing unbroken homogeneous rock or bedrock.
///
/// Stores only a single material color without allocating per-voxel storage.
#[derive(Clone, Copy, Debug)]
pub struct MonolithVolume {
    color: Vec3,
}

impl MonolithVolume {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(color: Vec3) -> Self {
        Self { color }
    }
}

impl VoxelVolume for MonolithVolume {
    fn occluding_at(&self, _coord: VoxelCoord) -> bool {
        true
    }

    fn inspect_voxel(&self, _coord: VoxelCoord, consumer: &mut dyn FnMut(bool, Vec3)) {
        consumer(true, self.color);
    }

    fn emit_mesh(
        &self,
        origin: Vec3,
        neighborhood: &dyn VoxelNeighborhood,
        target: &mut dyn MeshTarget,
    ) {
        let size = VOLUME_EDGE_LENGTH as f32;
        let c = self.color;

        // Face -X (x = 0): Check neighbor at x = -1
        if !neighborhood.occluding_at(-1, 0, 0) {
            target.push_quad(Quad::new(
                origin + Vec3::new(0.0, 0.0, size),
                origin + Vec3::new(0.0, 0.0, 0.0),
                origin + Vec3::new(0.0, size, 0.0),
                origin + Vec3::new(0.0, size, size),
                Vec3::NEG_X,
                c,
            ));
        }

        // Face +X (x = size): Check neighbor at x = size
        if !neighborhood.occluding_at(size as i32, 0, 0) {
            target.push_quad(Quad::new(
                origin + Vec3::new(size, 0.0, 0.0),
                origin + Vec3::new(size, 0.0, size),
                origin + Vec3::new(size, size, size),
                origin + Vec3::new(size, size, 0.0),
                Vec3::X,
                c,
            ));
        }

        // Face -Y (y = 0): Check neighbor at y = -1
        if !neighborhood.occluding_at(0, -1, 0) {
            target.push_quad(Quad::new(
                origin + Vec3::new(0.0, 0.0, 0.0),
                origin + Vec3::new(size, 0.0, 0.0),
                origin + Vec3::new(size, 0.0, size),
                origin + Vec3::new(0.0, 0.0, size),
                Vec3::NEG_Y,
                c,
            ));
        }

        // Face +Y (y = size): Check neighbor at y = size
        if !neighborhood.occluding_at(0, size as i32, 0) {
            target.push_quad(Quad::new(
                origin + Vec3::new(0.0, size, size),
                origin + Vec3::new(size, size, size),
                origin + Vec3::new(size, size, 0.0),
                origin + Vec3::new(0.0, size, 0.0),
                Vec3::Y,
                c,
            ));
        }

        // Face -Z (z = 0): Check neighbor at z = -1
        if !neighborhood.occluding_at(0, 0, -1) {
            target.push_quad(Quad::new(
                origin + Vec3::new(size, 0.0, 0.0),
                origin + Vec3::new(0.0, 0.0, 0.0),
                origin + Vec3::new(0.0, size, 0.0),
                origin + Vec3::new(size, size, 0.0),
                Vec3::NEG_Z,
                c,
            ));
        }

        // Face +Z (z = size): Check neighbor at z = size
        if !neighborhood.occluding_at(0, 0, size as i32) {
            target.push_quad(Quad::new(
                origin + Vec3::new(0.0, 0.0, size),
                origin + Vec3::new(size, 0.0, size),
                origin + Vec3::new(size, size, size),
                origin + Vec3::new(0.0, size, size),
                Vec3::Z,
                c,
            ));
        }
    }
}
