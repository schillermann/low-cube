mod console;
mod nozzle;

use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::cell::GridCell;
use super::mold::MoldArchetype;
use super::timber_grain::BlockMedium;
use super::wedge::WedgeHeading;

pub use self::console::FlightConsoleMesh;
pub use self::nozzle::RocketNozzleMesh;

/// Six-way physical exhaust direction of a mounted `RocketNozzle` block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NozzleAxis {
    Downward,
    Lateral(WedgeHeading),
    Upward,
}

impl NozzleAxis {
    /// Resolves nozzle exhaust orientation from form archetype (`0..3`) and cardinal wedge heading.
    pub const fn from_mold(archetype: MoldArchetype, heading: WedgeHeading) -> Self {
        match archetype.code() % 3 {
            0 => Self::Downward,
            1 => Self::Lateral(heading),
            _ => Self::Upward,
        }
    }

    /// Outward exhaust vector in the ship's local `SurfaceFrame`.
    pub fn exhaust_vector(self, frame: &SurfaceFrame) -> Vec3 {
        match self {
            Self::Downward => -frame.normal(),
            Self::Lateral(heading) => heading.forward_vector(frame),
            Self::Upward => frame.normal(),
        }
    }

    /// Physical reaction thrust vector (`-exhaust_vector`) pushing the craft opposite the nozzle bell.
    pub fn thrust_vector(self, frame: &SurfaceFrame) -> Vec3 {
        -self.exhaust_vector(frame)
    }

    /// Evaluates visual plume intensity `[0.0..=1.0]` from continuous `(vertical_burn, planar_burn)` state.
    pub fn plume_glow(self, motion: (f32, f32)) -> f32 {
        match self {
            Self::Downward if motion.0 > 0.04 => motion.0.clamp(0.0, 1.2),
            Self::Upward if motion.0 < -0.04 => (-motion.0).clamp(0.0, 1.2),
            Self::Lateral(_) if motion.1.abs() > 0.04 => motion.1.abs().clamp(0.0, 1.2),
            _ => 0.0,
        }
    }
}

/// Sculpted 3D geometry dispatcher for `FlightConsole` and `RocketNozzle` spacefaring blocks.
pub struct SpaceBlockMesh {
    medium: BlockMedium,
    heading: WedgeHeading,
    archetype: MoldArchetype,
    motion: (f32, f32),
}

impl SpaceBlockMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_motion(
        medium: BlockMedium,
        heading: WedgeHeading,
        archetype: MoldArchetype,
        motion: (f32, f32),
    ) -> Self {
        Self { medium, heading, archetype, motion }
    }

    /// Secondary constructor for static unpowered world and hand placement rendering.
    pub const fn parked(medium: BlockMedium, heading: WedgeHeading, archetype: MoldArchetype) -> Self {
        Self::with_motion(medium, heading, archetype, (0.0, 0.0))
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        match self.medium {
            BlockMedium::FlightConsole => {
                FlightConsoleMesh::new(self.heading, self.motion.1).emit(cell, floor_elev, frame, target);
            }
            _ => {
                let axis = NozzleAxis::from_mold(self.archetype, self.heading);
                RocketNozzleMesh::new(axis, axis.plume_glow(self.motion)).emit(cell, floor_elev, frame, target);
            }
        }
    }
}
