mod schematic;

#[cfg(test)]
mod tests;

use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::wedge::WedgeHeading;
use self::schematic::CadSchematic;

/// Vertical local coordinate frame of the wall-mounted aerospace CAD blueprint tablet.
#[derive(Clone, Copy, Debug)]
pub(super) struct SlatePosture {
    origin: Vec3,
    right: Vec3,
    rise: Vec3,
    face_normal: Vec3,
}

impl SlatePosture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: Vec3, right: Vec3, rise: Vec3, face_normal: Vec3) -> Self {
        Self {
            origin,
            right,
            rise,
            face_normal,
        }
    }

    pub(super) fn point(&self, x: f32, s: f32, w: f32) -> Vec3 {
        self.origin + self.right * x + self.rise * s + self.face_normal * w
    }

    pub(super) fn emit_slab(
        &self,
        bounds: [f32; 6],
        wall_col: Vec3,
        face_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let [x0, x1, s0, s1, w0, w1] = bounds;
        let [a0, a1, a2, a3] = [
            self.point(x0, s0, w0),
            self.point(x1, s0, w0),
            self.point(x1, s1, w0),
            self.point(x0, s1, w0),
        ];
        let [b0, b1, b2, b3] = [
            self.point(x0, s0, w1),
            self.point(x1, s0, w1),
            self.point(x1, s1, w1),
            self.point(x0, s1, w1),
        ];
        target.push_quad(Quad::new(b0, b1, b2, b3, self.face_normal, face_col));
        target.push_quad(Quad::new(a0, a3, a2, a1, -self.face_normal, wall_col));
        target.push_quad(Quad::new(a0, a1, b1, b0, -self.rise, wall_col));
        target.push_quad(Quad::new(b3, b2, a2, a3, self.rise, wall_col));
        target.push_quad(Quad::new(a0, b0, b3, a3, -self.right, wall_col));
        target.push_quad(Quad::new(b1, a1, a2, b2, self.right, wall_col));
    }

    pub(super) fn emit_glow_rect(
        &self,
        rect: [f32; 4],
        w: f32,
        col: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        let [x0, x1, s0, s1] = rect;
        target.push_quad(Quad::glowing(
            self.point(x0, s0, w),
            self.point(x1, s0, w),
            self.point(x1, s1, w),
            self.point(x0, s1, w),
            self.face_normal,
            col,
            glow,
        ));
    }
}

/// Vertically mounted aerospace CAD blueprint tablet with anodized frame, docking clasps, and rear wall cleats.
#[derive(Clone, Copy, Debug)]
pub struct BlueprintBlockMesh {
    heading: WedgeHeading,
}

impl BlueprintBlockMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(heading: WedgeHeading) -> Self {
        Self { heading }
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (uc, vc) = cell.center();
        let fwd = self.heading.forward_vector(frame);
        let up = frame.normal();
        let right = fwd.cross(up).normalize_or_zero();
        let floor_center = frame.point_at(uc, vc, floor_elev);

        let mount_origin = floor_center + fwd * 0.230 + up * 0.062;
        let slate = SlatePosture::new(mount_origin, right, up, -fwd);

        self.emit_oak_chassis(&slate, target);
        self.emit_wall_cleats(&slate, target);
        self.emit_brass_hardware(&slate, target);
        self.emit_side_griffel(&slate, target);
        CadSchematic::new(slate).emit(target);
    }

    fn emit_wall_cleats(&self, slate: &SlatePosture, target: &mut dyn MeshTarget) {
        let forged_iron = Vec3::new(0.20, 0.22, 0.26);
        let brass_stud = Vec3::new(0.92, 0.76, 0.28);

        for (cx0, cx1) in [(-0.164f32, -0.116f32), (0.116f32, 0.164f32)] {
            let cx = (cx0 + cx1) * 0.5;
            for (s0, s1) in [(0.064f32, 0.116f32), (0.238f32, 0.290f32)] {
                let cs = (s0 + s1) * 0.5;
                slate.emit_slab([cx0, cx1, s0, s1, -0.016, -0.008], forged_iron, forged_iron, target);
                slate.emit_slab([cx - 0.007, cx + 0.007, cs - 0.007, cs + 0.007, -0.018, -0.016], brass_stud, brass_stud, target);
            }
        }
    }

    fn emit_oak_chassis(&self, slate: &SlatePosture, target: &mut dyn MeshTarget) {
        let oak_dark = Vec3::new(0.27, 0.16, 0.09);
        let oak_bezel = Vec3::new(0.38, 0.23, 0.13);
        let oak_batten = Vec3::new(0.21, 0.12, 0.07);

        slate.emit_slab([-0.220, 0.220, 0.000, 0.376, 0.000, 0.032], oak_dark, oak_dark, target);
        slate.emit_slab([-0.205, 0.205, 0.072, 0.108, -0.008, 0.000], oak_batten, oak_batten, target);
        slate.emit_slab([-0.205, 0.205, 0.246, 0.282, -0.008, 0.000], oak_batten, oak_batten, target);

        for bar in [
            [-0.220, 0.220, 0.000, 0.056, 0.032, 0.038],
            [-0.220, 0.220, 0.332, 0.376, 0.032, 0.038],
            [-0.220, -0.184, 0.056, 0.332, 0.032, 0.038],
            [0.184, 0.220, 0.056, 0.332, 0.032, 0.038],
        ] {
            slate.emit_slab(bar, oak_dark, oak_bezel, target);
        }
    }

    fn emit_brass_hardware(&self, slate: &SlatePosture, target: &mut dyn MeshTarget) {
        let brass = Vec3::new(0.88, 0.71, 0.24);
        let brass_rim = Vec3::new(0.68, 0.52, 0.16);
        let brass_rivet = Vec3::new(0.98, 0.85, 0.38);
        let compass_glow = Vec3::new(1.00, 0.84, 0.30);

        for (sx, horiz_x, vert_x) in [
            (-1.0f32, [-0.226f32, -0.156f32], [-0.226f32, -0.194f32]),
            (1.0f32, [0.156f32, 0.226f32], [0.194f32, 0.226f32]),
        ] {
            let rx = sx * 0.206;
            slate.emit_slab([horiz_x[0], horiz_x[1], -0.005, 0.024, -0.004, 0.043], brass_rim, brass, target);
            slate.emit_slab([vert_x[0], vert_x[1], 0.024, 0.068, -0.004, 0.043], brass_rim, brass, target);
            slate.emit_slab([rx - 0.007, rx + 0.007, 0.006, 0.020, 0.043, 0.047], brass, brass_rivet, target);

            slate.emit_slab([horiz_x[0], horiz_x[1], 0.352, 0.381, -0.004, 0.043], brass_rim, brass, target);
            slate.emit_slab([vert_x[0], vert_x[1], 0.308, 0.352, -0.004, 0.043], brass_rim, brass, target);
            slate.emit_slab([rx - 0.007, rx + 0.007, 0.356, 0.370, 0.043, 0.047], brass, brass_rivet, target);
        }

        slate.emit_slab([-0.024, 0.024, 0.011, 0.045, 0.038, 0.044], brass_rim, brass, target);
        slate.emit_slab([-0.017, 0.017, 0.005, 0.051, 0.038, 0.044], brass_rim, brass, target);
        slate.emit_glow_rect([-0.010, 0.010, 0.018, 0.038], 0.046, compass_glow, 0.84, target);
    }

    fn emit_side_griffel(&self, slate: &SlatePosture, target: &mut dyn MeshTarget) {
        let leather_strap = Vec3::new(0.36, 0.20, 0.11);
        let turned_walnut = Vec3::new(0.50, 0.30, 0.16);
        let brass_nib = Vec3::new(0.92, 0.76, 0.28);

        slate.emit_slab([0.218, 0.245, 0.168, 0.220, 0.004, 0.034], leather_strap, leather_strap, target);
        slate.emit_slab([0.224, 0.241, 0.082, 0.292, 0.009, 0.028], turned_walnut, turned_walnut, target);
        slate.emit_slab([0.223, 0.242, 0.292, 0.314, 0.008, 0.029], brass_nib, brass_nib, target);
        slate.emit_slab([0.225, 0.240, 0.068, 0.082, 0.010, 0.027], brass_nib, brass_nib, target);
        slate.emit_slab([0.228, 0.237, 0.044, 0.068, 0.013, 0.024], brass_nib, brass_nib, target);
    }
}
