use glam::{Quat, Vec3};
use crate::frame::SurfaceFrame;
use crate::wire_vector::{WireSlice, WireVector};

/// Orthonormal coordinate axes of a drivable vehicle assembly on a planetary surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleAxes {
    tangent: Vec3,
    bitangent: Vec3,
    normal: Vec3,
}

impl VehicleAxes {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tangent: Vec3, bitangent: Vec3, normal: Vec3) -> Self {
        Self { tangent, bitangent, normal }
    }

    pub const fn normal(self) -> Vec3 {
        self.normal
    }

    pub(in crate::relief) fn surface_frame(self) -> SurfaceFrame {
        SurfaceFrame::new(self.tangent, self.bitangent, self.normal, 0.0)
    }

    pub(in crate::relief) fn interpolated(self, target: Self, alpha: f32) -> Self {
        let raw_n = self.normal.lerp(target.normal, alpha);
        let n = if raw_n.length_squared() > 1e-6 { raw_n.normalize() } else { target.normal };
        let raw_t = self.tangent.lerp(target.tangent, alpha);
        let t_ortho = raw_t - n * raw_t.dot(n);
        let t = if t_ortho.length_squared() > 1e-6 { t_ortho.normalize() } else { target.tangent };
        let raw_b = self.bitangent.lerp(target.bitangent, alpha);
        let b_ortho = raw_b - n * raw_b.dot(n) - t * raw_b.dot(t);
        let b = if b_ortho.length_squared() > 1e-6 { b_ortho.normalize() } else { target.bitangent };
        Self::new(t, b, n)
    }

    pub(in crate::relief) fn inscribe(self, out: &mut Vec<u8>) {
        WireVector::new(self.tangent).inscribe(out);
        WireVector::new(self.bitangent).inscribe(out);
        WireVector::new(self.normal).inscribe(out);
    }

    pub(in crate::relief) fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 36 {
            return None;
        }
        let t = WireSlice::new(&bytes[0..12]).vec3()?;
        let b = WireSlice::new(&bytes[12..24]).vec3()?;
        let n = WireSlice::new(&bytes[24..36]).vec3()?;
        if t.length_squared() < 0.25 || b.length_squared() < 0.25 || n.length_squared() < 0.25 {
            return None;
        }
        Some((Self::new(t.normalize(), b.normalize(), n.normalize()), 36))
    }

    pub(crate) fn grid_basis(self, frame: &SurfaceFrame) -> (i32, i32, i32, i32) {
        let unfold = Quat::from_rotation_arc(self.normal.normalize_or_zero(), frame.normal());
        let (t_flat, b_flat) = (unfold * self.tangent, unfold * self.bitangent);
        let (du, dv) = (t_flat.dot(frame.tangent()), t_flat.dot(frame.bitangent()));
        let (rxx, rxy) = if du.abs() >= dv.abs() {
            (if du >= 0.0 { 1 } else { -1 }, 0)
        } else {
            (0, if dv >= 0.0 { 1 } else { -1 })
        };
        let (bu, bv) = (b_flat.dot(frame.tangent()), b_flat.dot(frame.bitangent()));
        let (ryx, ryy) = if rxx != 0 {
            (0, if bv >= 0.0 { 1 } else { -1 })
        } else {
            (if bu >= 0.0 { 1 } else { -1 }, 0)
        };
        (rxx, rxy, ryx, ryy)
    }

    pub(in crate::relief) fn rotated(self, rot: Quat) -> Self {
        Self::new(
            (rot * self.tangent).normalize_or_zero(),
            (rot * self.bitangent).normalize_or_zero(),
            (rot * self.normal).normalize_or_zero(),
        )
    }
}
