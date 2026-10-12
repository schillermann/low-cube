use glam::{Quat, Vec3};
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::orientation::ViewOrientation;

/// Tangential steering heading projecting relative keyboard controls onto planetary surface coordinates.
pub struct SurfaceHeading {
    forward: Vec3,
    right: Vec3,
}

impl SurfaceHeading {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(forward: Vec3, right: Vec3) -> Self {
        Self { forward, right }
    }

    /// Secondary constructor deriving surface-aligned forward and right vectors from view and surface orientation.
    pub fn aligned(surface_rot: Quat, look: &dyn ViewOrientation) -> Self {
        let view_rot = look.combine_with_surface(surface_rot);
        Self::new(view_rot * Vec3::NEG_Z, view_rot * Vec3::X)
    }

    /// Drives locomotion target using encapsulated surface vectors without asking the object for internal data.
    pub fn drive_locomotion(
        &self,
        controls: &dyn KeyboardControls,
        target: &mut dyn LocomotionTarget,
    ) {
        controls.drive_locomotion(self.forward, self.right, target);
    }
}
