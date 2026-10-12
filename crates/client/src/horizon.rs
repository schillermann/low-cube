use glam::{Mat4, Quat, Vec3};
use crate::orientation::ViewOrientation;

/// Interface for viewport view-matrix generation without leaking raw vector internals ("Tell, Don't Ask").
pub trait ViewFrame: Send + Sync {
    fn view_matrix_at(&self, position: Vec3, eye_height: f32) -> Mat4;
}

/// Camera horizon viewpoint aligned with curved planetary surface orientation and player look direction.
pub struct CameraHorizon {
    surface_rot: Quat,
    view_rot: Quat,
}

impl CameraHorizon {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(surface_rot: Quat, view_rot: Quat) -> Self {
        Self { surface_rot, view_rot }
    }

    /// Secondary constructor combining player look direction with planetary surface gravity orientation.
    pub fn aligned(surface_rot: Quat, look: &dyn ViewOrientation) -> Self {
        let view_rot = look.combine_with_surface(surface_rot);
        Self::new(surface_rot, view_rot)
    }
}

impl ViewFrame for CameraHorizon {
    fn view_matrix_at(&self, position: Vec3, eye_height: f32) -> Mat4 {
        let up = self.surface_rot * Vec3::Y;
        let forward = self.view_rot * Vec3::NEG_Z;
        let eye = position + up * eye_height;
        let target = eye + forward;
        Mat4::look_at_rh(eye, target, up)
    }
}

/// Decorator applying smooth camera horizon roll around the view axis during seam traversal.
pub struct HorizonTilt<'a> {
    origin: &'a dyn ViewFrame,
    roll_radians: f32,
}

impl<'a> HorizonTilt<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: &'a dyn ViewFrame, roll_radians: f32) -> Self {
        Self { origin, roll_radians }
    }
}

impl<'a> ViewFrame for HorizonTilt<'a> {
    fn view_matrix_at(&self, position: Vec3, eye_height: f32) -> Mat4 {
        let base_view = self.origin.view_matrix_at(position, eye_height);
        Mat4::from_rotation_z(self.roll_radians) * base_view
    }
}
