/// Abstraction for the polar aurora atmospheric canopy.
pub trait AuroraCanopy: Send + Sync {
    fn canopy_bounds(&self) -> [f32; 3];
}

/// Atmospheric canopy for Arctic auroras calibrated to planetary dimensions (`3` private fields).
pub struct CalibratedCanopy {
    face_half: f32,
    radius: f32,
    ceiling_altitude: f32,
}

impl CalibratedCanopy {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(face_half: f32, radius: f32, ceiling_altitude: f32) -> Self {
        Self {
            face_half,
            radius,
            ceiling_altitude,
        }
    }

    /// Secondary constructor calibrating canopy dimensions from block scale and bounds.
    pub fn calibrated(world_blocks: u32, min_size: f32, max_size: f32) -> Self {
        let face_half = (world_blocks as f32) * 0.5;
        let ceiling_altitude = (face_half * 0.40).clamp(8.0, 48.0);
        let target_radius = (face_half * 0.60).clamp(min_size, max_size);
        let radius = target_radius.min(face_half * 0.80);
        Self::new(face_half, radius, ceiling_altitude)
    }

    /// Tertiary constructor calibrating canopy with planetary defaults.
    pub fn planetary(world_blocks: u32) -> Self {
        Self::calibrated(world_blocks, 14.0, 96.0)
    }

    /// Tell-Don't-Ask functional inspection for testing.
    #[cfg(test)]
    pub fn transfer<R, F: FnOnce(f32, f32, f32) -> R>(&self, inspector: F) -> R {
        inspector(self.face_half, self.radius, self.ceiling_altitude)
    }
}

impl AuroraCanopy for CalibratedCanopy {
    fn canopy_bounds(&self) -> [f32; 3] {
        [self.face_half, self.radius, self.ceiling_altitude]
    }
}

/// Null object pattern canopy emitting zeroed dimensions (`0` fields).
#[cfg(test)]
pub struct VoidCanopy;

#[cfg(test)]
impl VoidCanopy {
    /// Primary constructor doing nothing except creating instance.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
impl AuroraCanopy for VoidCanopy {
    fn canopy_bounds(&self) -> [f32; 3] {
        [0.0, 0.0, 0.0]
    }
}
