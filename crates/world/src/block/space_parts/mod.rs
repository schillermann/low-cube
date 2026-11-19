mod attitude_nozzle;
mod console;
mod nozzle;
mod orrery;
mod rocket_camera;

use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::cell::GridCell;
use super::mold::MoldArchetype;
use super::timber_grain::BlockMedium;
use super::wedge::WedgeHeading;

pub use self::attitude_nozzle::AttitudeNozzleMesh;
pub use self::console::FlightConsoleMesh;
pub use self::nozzle::RocketNozzleMesh;
pub use self::orrery::ConsoleOrrery;
pub use self::rocket_camera::{RocketCameraMesh, SensorPodAxis};

/// Propulsion mode of a mounted `RocketNozzle` block toggled via mouse wheel scroll (`Downward` main bell vs. `Attitude` Steuerdüse).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NozzleAxis {
    Downward,
    Attitude(WedgeHeading),
}

impl NozzleAxis {
    /// Resolves main rocket bell (`0`) vs. reaction-control Steuerdüse (`1`) from form archetype and wedge heading.
    pub const fn from_mold(archetype: MoldArchetype, heading: WedgeHeading) -> Self {
        match archetype.code() % 2 {
            0 => Self::Downward,
            _ => Self::Attitude(heading),
        }
    }

    /// Indicates whether this thruster is a reaction-control Steuerdüse pod rather than a main bell nozzle.
    pub const fn attitude_pod(self) -> bool {
        matches!(self, Self::Attitude(_))
    }

    /// Primary outward exhaust vector in the ship's local `SurfaceFrame` (pointing outward from hull wall).
    pub fn exhaust_vector(self, frame: &SurfaceFrame) -> Vec3 {
        match self {
            Self::Downward => -frame.normal(),
            Self::Attitude(heading) => -heading.forward_vector(frame),
        }
    }

    /// Primary reaction thrust vector (`-exhaust_vector`) pushing the craft opposite the nozzle cone.
    pub fn thrust_vector(self, frame: &SurfaceFrame) -> Vec3 {
        -self.exhaust_vector(frame)
    }

    /// Evaluates visual plume intensity `[0.0..=1.2]` from continuous `(vertical_burn, planar_burn)` state.
    pub fn plume_glow(self, motion: (f32, f32)) -> f32 {
        match self {
            Self::Downward if motion.0 > 0.04 => motion.0.clamp(0.0, 1.2),
            Self::Attitude(_) if motion.1.abs() > 0.04 => motion.1.abs().clamp(0.0, 1.2),
            Self::Attitude(_) if motion.0 < -0.04 => (-motion.0).clamp(0.0, 1.2),
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
                FlightConsoleMesh::new(self.heading, self.motion).emit(cell, floor_elev, frame, target);
            }
            BlockMedium::RocketCamera => {
                RocketCameraMesh::new(SensorPodAxis::from_mold(self.archetype, self.heading))
                    .emit(cell, floor_elev, frame, target);
            }
            _ => match NozzleAxis::from_mold(self.archetype, self.heading) {
                axis @ NozzleAxis::Downward => {
                    RocketNozzleMesh::new(axis, axis.plume_glow(self.motion)).emit(cell, floor_elev, frame, target);
                }
                NozzleAxis::Attitude(heading) => {
                    AttitudeNozzleMesh::new(heading, self.motion).emit(cell, floor_elev, frame, target);
                }
            },
        }
    }
}
