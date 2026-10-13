use glam::Vec3;
use crate::MeshTarget;
use super::{
    GreedyMesh, VoxelCoord, VoxelNeighborhood, VoxelVolume, VOLUME_EDGE_LENGTH,
    VOXEL_COUNT_PER_VOLUME,
};

/// Heterogeneous geological volume storing stratified strata using an indexed palette.
pub struct StratifiedVolume {
    palette: [Vec3; 16],
    voxels: Box<[u8; VOXEL_COUNT_PER_VOLUME]>,
}

impl StratifiedVolume {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(palette: [Vec3; 16], voxels: Box<[u8; VOXEL_COUNT_PER_VOLUME]>) -> Self {
        Self { palette, voxels }
    }

    /// Secondary constructor initializing with standard planetary earth/rock palette.
    pub fn planetary() -> Self {
        let mut palette = [Vec3::ZERO; 16];
        palette[1] = Vec3::new(0.5, 0.5, 0.5); // Stone
        palette[2] = Vec3::new(0.45, 0.30, 0.15); // Dirt
        palette[3] = Vec3::new(0.76, 0.70, 0.50); // Sand
        palette[4] = Vec3::new(0.90, 0.92, 0.95); // Snow
        palette[5] = Vec3::new(0.20, 0.20, 0.22); // Bedrock
        let voxels = Box::new([0u8; VOXEL_COUNT_PER_VOLUME]);
        Self::new(palette, voxels)
    }

    /// Places a voxel material at the specified discrete coordinates.
    pub fn place(&mut self, coord: VoxelCoord, material_id: u8) {
        self.voxels[coord.linear_index()] = material_id;
    }

    fn color_for(&self, id: u8) -> Vec3 {
        let idx = (id as usize) & 0x0F;
        self.palette[idx]
    }
}

impl VoxelVolume for StratifiedVolume {
    fn occluding_at(&self, coord: VoxelCoord) -> bool {
        self.voxels[coord.linear_index()] != 0
    }

    fn inspect_voxel(&self, coord: VoxelCoord, consumer: &mut dyn FnMut(bool, Vec3)) {
        let id = self.voxels[coord.linear_index()];
        let solid = id != 0;
        let color = self.color_for(id);
        consumer(solid, color);
    }

    fn emit_mesh(
        &self,
        origin: Vec3,
        neighborhood: &dyn VoxelNeighborhood,
        target: &mut dyn MeshTarget,
    ) {
        let greedy = GreedyMesh::new(&self.palette);
        let size = VOLUME_EDGE_LENGTH as i32;
        greedy.emit_volume_mesh(
            origin,
            |x, y, z| {
                if x >= 0 && x < size && y >= 0 && y < size && z >= 0 && z < size {
                    let coord = VoxelCoord::new(x as usize, y as usize, z as usize);
                    self.voxels[coord.linear_index()]
                } else {
                    0
                }
            },
            neighborhood,
            target,
        );
    }
}
