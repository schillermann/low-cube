use std::collections::HashMap;
use glam::Vec3;
use crate::MeshTarget;
use super::{
    PlanetaryNeighborhood, VoidNeighborhood, VoidVolume, VolumeCoord, VoxelVolume,
    VOLUME_EDGE_LENGTH,
};

/// Sparse spatial partition for cubic planetary worlds with an encapsulated hollow core void.
pub struct SparsePlanetaryGrid {
    side_volumes: usize,
    crust_depth_volumes: usize,
    crust: HashMap<VolumeCoord, Box<dyn VoxelVolume>>,
    void: VoidVolume,
}

impl SparsePlanetaryGrid {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        side_volumes: usize,
        crust_depth_volumes: usize,
        crust: HashMap<VolumeCoord, Box<dyn VoxelVolume>>,
        void: VoidVolume,
    ) -> Self {
        Self {
            side_volumes,
            crust_depth_volumes,
            crust,
            void,
        }
    }

    /// Secondary constructor initializing a planetary grid of given side dimension in volume partitions.
    pub fn planetary(side_volumes: usize, crust_depth_volumes: usize) -> Self {
        Self::new(
            side_volumes,
            crust_depth_volumes,
            HashMap::new(),
            VoidVolume::new(),
        )
    }

    /// Internal predicate testing whether a discrete coordinate lies inside the hollow core abyss.
    pub fn in_hollow_core(&self, coord: VolumeCoord) -> bool {
        coord.transfer(|x, y, z| {
            let low = self.crust_depth_volumes as i32;
            let high = (self.side_volumes as i32) - 1 - low;
            x >= low && x <= high && y >= low && y <= high && z >= low && z <= high
        })
    }

    /// Inserts or replaces a crust volume at the specified coordinate.
    pub fn install_volume(&mut self, coord: VolumeCoord, volume: Box<dyn VoxelVolume>) {
        self.crust.insert(coord, volume);
    }

    /// Delivers the volume at the specified coordinate to a consumer without exposing pointers.
    pub fn deliver_volume<R>(
        &self,
        coord: VolumeCoord,
        consumer: impl FnOnce(&dyn VoxelVolume) -> R,
    ) -> R {
        if self.in_hollow_core(coord) {
            consumer(&self.void)
        } else if let Some(volume) = self.crust.get(&coord) {
            consumer(volume.as_ref())
        } else {
            consumer(&self.void)
        }
    }

    /// Total number of theoretical volume partitions across the entire planetary cube.
    pub fn total_volume_count(&self) -> usize {
        self.side_volumes * self.side_volumes * self.side_volumes
    }

    /// Number of 0-byte void volumes encapsulated within the hollow core.
    pub fn hollow_core_volume_count(&self) -> usize {
        let low = self.crust_depth_volumes * 2;
        if self.side_volumes > low {
            let core_side = self.side_volumes - low;
            core_side * core_side * core_side
        } else {
            0
        }
    }

    /// Instructs all active volumes in the grid to emit their visible mesh geometry into a target.
    pub fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let neighborhood = VoidNeighborhood;
        for (coord, volume) in &self.crust {
            coord.spatial_origin(|origin| {
                volume.emit_mesh(origin, &neighborhood, target);
            });
        }
    }

    /// Tells each crust volume to emit mesh resolving 3D cross-seam occlusion via PlanetaryNeighborhood.
    pub fn emit_planetary_surface(&self, half_size: f32, target: &mut dyn MeshTarget) {
        let base_offset = Vec3::splat(-half_size);
        let span = VOLUME_EDGE_LENGTH as f32;
        for (coord, volume) in &self.crust {
            let origin = coord.transfer(|cx, cy, cz| {
                base_offset + Vec3::new(cx as f32 * span, cy as f32 * span, cz as f32 * span)
            });
            let neighborhood = PlanetaryNeighborhood::new(self, *coord);
            volume.emit_mesh(origin, &neighborhood, target);
        }
    }
}
