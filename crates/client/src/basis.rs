use glam::{Mat4, Quat, Vec3};
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::orientation::ViewOrientation;

/// Local coordinate basis for an avatar on a curved planetary surface.
pub struct OrientationBasis {
    forward: Vec3,
    right: Vec3,
    up: Vec3,
}

impl OrientationBasis {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(forward: Vec3, right: Vec3, up: Vec3) -> Self {
        Self { forward, right, up }
    }

    /// Secondary constructor aligning basis with surface up vector and view orientation.
    pub fn aligned(surface_up: Vec3, look: &dyn ViewOrientation) -> Self {
        let up = surface_up.normalize();
        let arbitrary_ref = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
        let tangent = up.cross(arbitrary_ref).normalize();
        let bitangent = tangent.cross(up).normalize();

        let surface_rot = Quat::from_mat3(&glam::Mat3::from_cols(tangent, up, bitangent));
        let view_rot = look.combine_with_surface(surface_rot);

        let forward = view_rot * Vec3::NEG_Z;
        let right = view_rot * Vec3::X;

        Self::new(forward, right, up)
    }

    pub fn drive_locomotion(
        &self,
        controls: &dyn KeyboardControls,
        target: &mut dyn LocomotionTarget,
    ) {
        controls.drive_locomotion(self.forward, self.right, target);
    }

    pub fn view_matrix_at(&self, position: Vec3, eye_height: f32) -> Mat4 {
        let eye = position + self.up * eye_height;
        let target = eye + self.forward;
        Mat4::look_at_rh(eye, target, self.up)
    }
}
