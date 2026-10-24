use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::GridCell;
use crate::frame::SurfaceFrame;

/// Incandescent molten sphere at the planetary heart: lethal on contact and self-luminous when rendered.
#[derive(Clone, Copy, Debug)]
pub struct MagmaCore {
    radius: f32,
    half_blocks: i32,
    stride: i32,
}

impl MagmaCore {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(radius: f32, half_blocks: i32, stride: i32) -> Self {
        Self { radius, half_blocks, stride }
    }

    fn half_size(&self) -> f32 {
        (self.half_blocks as f32) * 0.5
    }

    /// Elevation step of a breached shaft bottom, resting on the molten summit beneath the face center.
    pub fn summit_steps(&self) -> i32 {
        ((self.radius - self.half_size()) * 2.0).floor() as i32
    }

    /// Indicates whether a point at tangential (u, v) and face elevation lies within `reach` of the molten surface.
    pub fn engulfs(&self, u: f32, v: f32, elev: f32, reach: f32) -> bool {
        Vec3::new(u, v, self.half_size() + elev).length() <= self.radius + reach
    }

    /// Face elevation of the molten crest directly beneath (u, v), or the planetary center plane beside the sphere.
    pub fn crest_at(&self, u: f32, v: f32) -> f32 {
        let span = self.radius * self.radius - u * u - v * v;
        if span > 0.0 { span.sqrt() - self.half_size() } else { -self.half_size() }
    }

    /// Emits the radiant spherical patch anchored at this cell; strided patches tile the face's sixth of the sphere.
    pub fn emit_patch(&self, cell: GridCell, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        if (u + self.half_blocks).rem_euclid(self.stride) != 0 || (v + self.half_blocks).rem_euclid(self.stride) != 0 {
            return;
        }
        let ((u0, _), (v0, _), span) = (cell.u_bounds(), cell.v_bounds(), (self.stride as f32) * 0.5);
        let (u1, v1) = ((u0 + span).min(self.half_size()), (v0 + span).min(self.half_size()));
        let sphere = |a: f32, b: f32| frame.point_at(a, b, 0.0).normalize() * self.radius;
        let (p0, p1, p2, p3) = (sphere(u0, v0), sphere(u1, v0), sphere(u1, v1), sphere(u0, v1));
        let normal = (p0 + p1 + p2 + p3).normalize();
        let swirl = ((normal.x * 7.0 + normal.y * 3.0).sin() * (normal.z * 5.0 - normal.x * 2.0).cos()) * 0.5 + 0.5;
        let color = Vec3::new(1.0, 0.28, 0.03).lerp(Vec3::new(1.0, 0.82, 0.32), swirl);
        target.push_quad(Quad::glowing(p0, p1, p2, p3, normal, color, 0.9));
    }
}
