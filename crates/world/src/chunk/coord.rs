use glam::Vec3;

pub const VOLUME_EDGE_LENGTH: usize = 16;
pub const VOXEL_COUNT_PER_VOLUME: usize = VOLUME_EDGE_LENGTH * VOLUME_EDGE_LENGTH * VOLUME_EDGE_LENGTH;

/// 3D integer coordinate identifying a specific discrete volume within the planetary grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VolumeCoord {
    x: i32,
    y: i32,
    z: i32,
}

impl VolumeCoord {
    /// Primary constructor assigning coordinates directly.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Dispatches spatial origin of the volume in world units to a consumer.
    pub fn spatial_origin<R>(&self, consumer: impl FnOnce(Vec3) -> R) -> R {
        let size = VOLUME_EDGE_LENGTH as f32;
        let origin = Vec3::new(
            self.x as f32 * size,
            self.y as f32 * size,
            self.z as f32 * size,
        );
        consumer(origin)
    }

    /// Feeds discrete volume indices to a consumer.
    pub fn transfer<R>(&self, consumer: impl FnOnce(i32, i32, i32) -> R) -> R {
        consumer(self.x, self.y, self.z)
    }
}

/// Local discrete coordinate within a single 16x16x16 volume partition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VoxelCoord {
    x: usize,
    y: usize,
    z: usize,
}

impl VoxelCoord {
    /// Primary constructor verifying bounds.
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        assert!(
            x < VOLUME_EDGE_LENGTH && y < VOLUME_EDGE_LENGTH && z < VOLUME_EDGE_LENGTH,
            "Voxel coordinate ({}, {}, {}) exceeds volume bounds (16)",
            x, y, z
        );
        Self { x, y, z }
    }

    /// Linearized array index for 1D flat storage.
    pub fn linear_index(&self) -> usize {
        self.x + self.y * VOLUME_EDGE_LENGTH + self.z * VOLUME_EDGE_LENGTH * VOLUME_EDGE_LENGTH
    }

    /// Feeds discrete coordinates to a consumer.
    pub fn transfer<R>(&self, consumer: impl FnOnce(usize, usize, usize) -> R) -> R {
        consumer(self.x, self.y, self.z)
    }
}
