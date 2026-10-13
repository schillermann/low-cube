use glam::Vec3;
use super::{SparsePlanetaryGrid, VolumeCoord, VoxelCoord, VoxelNeighborhood, VOLUME_EDGE_LENGTH};

/// Seamless voxel neighborhood resolving cross-volume and planetary seam neighbors via 3D grid coordinates.
pub struct PlanetaryNeighborhood<'a> {
    grid: &'a SparsePlanetaryGrid,
    volume_coord: VolumeCoord,
}

impl<'a> PlanetaryNeighborhood<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(grid: &'a SparsePlanetaryGrid, volume_coord: VolumeCoord) -> Self {
        Self { grid, volume_coord }
    }

    fn resolve_target_voxel(&self, x: i32, y: i32, z: i32) -> (VolumeCoord, VoxelCoord) {
        let size = VOLUME_EDGE_LENGTH as i32;
        let dx = x.div_euclid(size);
        let dy = y.div_euclid(size);
        let dz = z.div_euclid(size);

        let lx = x.rem_euclid(size) as usize;
        let ly = y.rem_euclid(size) as usize;
        let lz = z.rem_euclid(size) as usize;

        let target_volume = self.volume_coord.transfer(|cx, cy, cz| {
            VolumeCoord::new(cx + dx, cy + dy, cz + dz)
        });
        let target_voxel = VoxelCoord::new(lx, ly, lz);
        (target_volume, target_voxel)
    }
}

impl<'a> VoxelNeighborhood for PlanetaryNeighborhood<'a> {
    fn occluding_at(&self, x: i32, y: i32, z: i32) -> bool {
        let (target_volume, target_voxel) = self.resolve_target_voxel(x, y, z);
        self.grid.deliver_volume(target_volume, |volume| {
            volume.occluding_at(target_voxel)
        })
    }

    fn color_at(&self, x: i32, y: i32, z: i32) -> Vec3 {
        let (target_volume, target_voxel) = self.resolve_target_voxel(x, y, z);
        let mut resolved = Vec3::ZERO;
        self.grid.deliver_volume(target_volume, |volume| {
            volume.inspect_voxel(target_voxel, &mut |solid, color| {
                if solid {
                    resolved = color;
                }
            });
        });
        resolved
    }
}
