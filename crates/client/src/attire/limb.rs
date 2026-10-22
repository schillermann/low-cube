use std::f32::consts::PI;
use glam::Vec3;
use world::{BlockMold, MeshTarget, Quad};

/// Local 3D anatomical coordinate frame for positioning faceted limbs and garments.
#[derive(Clone, Copy, Debug)]
pub struct LimbFrame {
    origin: Vec3,
    right: Vec3,
    up: Vec3,
    forward: Vec3,
}

impl LimbFrame {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Vec3, right: Vec3, up: Vec3, forward: Vec3) -> Self {
        Self { origin, right, up, forward }
    }

    /// Maps local coordinates (x: right, y: up, z: forward) into planetary world space.
    pub fn point(&self, x: f32, y: f32, z: f32) -> Vec3 {
        self.origin + self.right * x + self.up * y + self.forward * z
    }

    /// Translates the frame origin by local coordinates.
    pub fn shifted(&self, x: f32, y: f32, z: f32) -> Self {
        Self::new(self.point(x, y, z), self.right, self.up, self.forward)
    }

    /// Uniformly scales the local coordinate axes around the frame origin.
    pub fn scaled(&self, factor: f32) -> Self {
        Self::new(self.origin, self.right * factor, self.up * factor, self.forward * factor)
    }

    /// Pitches the local up and forward axes around the right axis by radians.
    pub fn pitched(&self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        let next_up = (self.up * c + self.forward * s).normalize();
        let next_fwd = (self.forward * c - self.up * s).normalize();
        Self::new(self.origin, self.right, next_up, next_fwd)
    }

    /// Rotates the local right and forward axes around the up axis by radians.
    pub fn yawed(&self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        let next_right = (self.right * c + self.forward * s).normalize();
        let next_fwd = (self.forward * c - self.right * s).normalize();
        Self::new(self.origin, next_right, self.up, next_fwd)
    }

    /// Emits a palm-molded block resting upright on the hollow upturned surface of the left or right palm.
    pub fn emit_cradled_block(&self, side: f32, local_origin: Vec3, mold: BlockMold, scale: f32, side_col: Vec3, top_col: Vec3, target: &mut dyn MeshTarget) {
        let center = self.point(local_origin.x, local_origin.y, local_origin.z);
        let (s, c, r, u) = ((side * 0.198).sin(), (side * 0.198).cos(), self.up * side, -self.right * side);
        mold.emit_miniature(center, r * c - self.forward * s, u, self.forward * c + r * s, scale, side_col, top_col, target);
    }

    /// Emits a 3D molded block aligned directly with this frame's right, up, and forward axes.
    pub fn emit_upright_block(&self, local_origin: Vec3, mold: BlockMold, scale: f32, side_col: Vec3, top_col: Vec3, target: &mut dyn MeshTarget) {
        mold.emit_miniature(self.point(local_origin.x, local_origin.y, local_origin.z), self.right, self.up, self.forward, scale, side_col, top_col, target);
    }

    /// Emits a filled background plaque at z_bg = 0.020 bordered by a 4-bar frame at z_rim = 0.018.
    pub fn emit_framed_plaque(&self, rx: f32, ry: f32, t: f32, bg: Vec3, rim: Vec3, target: &mut dyn MeshTarget) {
        let (z_rim, z_bg, n) = (0.018, 0.020, (self.up - self.forward).normalize_or_zero());
        target.push_quad(Quad::new(self.point(-rx, -ry, z_bg), self.point(rx, -ry, z_bg), self.point(rx, ry, z_bg), self.point(-rx, ry, z_bg), n, bg));
        for (a, b) in [
            (Vec3::new(-rx, ry, z_rim), Vec3::new(rx, ry, z_rim)),
            (Vec3::new(-rx, -ry, z_rim), Vec3::new(rx, -ry, z_rim)),
            (Vec3::new(-rx, -ry, z_rim), Vec3::new(-rx, ry, z_rim)),
            (Vec3::new(rx, -ry, z_rim), Vec3::new(rx, ry, z_rim)),
        ] {
            AxialLimb::new(a, b, t, t, rim).emit(self, true, target);
        }
    }
}

/// Interface for faceted polyhedral limb and garment solids emitting into a MeshTarget.
pub trait FacetedSolid {
    fn emit(&self, frame: &LimbFrame, capped: bool, target: &mut dyn MeshTarget);
}

/// Faceted hexagonal segment aligned along an arbitrary 3D bone axis (for forearms, thumbs, and bent finger joints).
#[derive(Clone, Copy, Debug)]
pub struct AxialLimb {
    start: Vec3,
    end: Vec3,
    r_start: f32,
    r_end: f32,
    pigment: Vec3,
}

impl AxialLimb {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(start: Vec3, end: Vec3, r_start: f32, r_end: f32, pigment: Vec3) -> Self {
        Self { start, end, r_start, r_end, pigment }
    }
}

impl FacetedSolid for AxialLimb {
    fn emit(&self, frame: &LimbFrame, capped: bool, target: &mut dyn MeshTarget) {
        let p0_center = frame.point(self.start.x, self.start.y, self.start.z);
        let p1_center = frame.point(self.end.x, self.end.y, self.end.z);
        let delta = p1_center - p0_center;
        if delta.length_squared() <= 1e-8 {
            return;
        }
        let axis = delta.normalize();
        let ref_vec = if axis.dot(frame.up).abs() < 0.85 { frame.up } else { frame.right };
        let u_vec = axis.cross(ref_vec).normalize();
        let v_vec = u_vec.cross(axis).normalize();
        let segs = 6usize;
        let step = (2.0 * PI) / (segs as f32);

        for i in 0..segs {
            let (s0, c0) = ((i as f32) * step).sin_cos();
            let (s1, c1) = (((i + 1) as f32) * step).sin_cos();
            let dir0 = u_vec * c0 + v_vec * s0;
            let dir1 = u_vec * c1 + v_vec * s1;

            let v0 = p0_center + dir0 * self.r_start;
            let v1 = p0_center + dir1 * self.r_start;
            let v2 = p1_center + dir1 * self.r_end;
            let v3 = p1_center + dir0 * self.r_end;

            let normal = (v1 - v0).cross(v3 - v0).normalize_or_zero();
            let valid_n = if normal.length_squared() > 0.5 { normal } else { dir0 };
            target.push_quad(Quad::new(v0, v1, v2, v3, valid_n, self.pigment));

            if capped {
                target.push_quad(Quad::new(p0_center, v0, v1, v1, -axis, self.pigment));
                target.push_quad(Quad::new(p1_center, v3, v2, v2, axis, self.pigment));
            }
        }
    }
}

/// Faceted hexagonal or octagonal tapered prism segment for stylized low-poly torso and boots.
#[derive(Clone, Copy, Debug)]
pub struct TaperedPrism {
    segments: usize,
    bottom_center: Vec3,
    top_center: Vec3,
    bottom_rx: f32,
    bottom_rz: f32,
    top_rx: f32,
    top_rz: f32,
    pigment: Vec3,
}

impl TaperedPrism {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(segments: usize, bottom_center: Vec3, top_center: Vec3, bottom_rx: f32, bottom_rz: f32, top_rx: f32, top_rz: f32, pigment: Vec3) -> Self {
        Self { segments, bottom_center, top_center, bottom_rx, bottom_rz, top_rx, top_rz, pigment }
    }

    /// Secondary constructor for a 6-sided limb or garment segment with uniform radius per ring.
    pub fn hexagonal(bottom_center: Vec3, top_center: Vec3, bottom_r: f32, top_r: f32, pigment: Vec3) -> Self {
        Self::new(6, bottom_center, top_center, bottom_r, bottom_r, top_r, top_r, pigment)
    }
}

impl FacetedSolid for TaperedPrism {
    fn emit(&self, frame: &LimbFrame, capped: bool, target: &mut dyn MeshTarget) {
        let segs = self.segments.max(4);
        let step = (2.0 * PI) / (segs as f32);

        for i in 0..segs {
            let (s0, c0) = (((i as f32) + 0.5) * step).sin_cos();
            let (s1, c1) = (((i as f32) + 1.5) * step).sin_cos();

            let p0 = frame.point(self.bottom_center.x + self.bottom_rx * c0, self.bottom_center.y, self.bottom_center.z + self.bottom_rz * s0);
            let p1 = frame.point(self.bottom_center.x + self.bottom_rx * c1, self.bottom_center.y, self.bottom_center.z + self.bottom_rz * s1);
            let p2 = frame.point(self.top_center.x + self.top_rx * c1, self.top_center.y, self.top_center.z + self.top_rz * s1);
            let p3 = frame.point(self.top_center.x + self.top_rx * c0, self.top_center.y, self.top_center.z + self.top_rz * s0);

            let cross = (p1 - p0).cross(p3 - p0);
            let normal = if cross.length_squared() > 1e-8 {
                cross.normalize()
            } else {
                (frame.point(c0, 0.0, s0) - frame.point(0.0, 0.0, 0.0)).normalize()
            };
            target.push_quad(Quad::new(p0, p1, p2, p3, normal, self.pigment));

            if capped {
                let top_mid = frame.point(self.top_center.x, self.top_center.y, self.top_center.z);
                let top_n = (p3 - top_mid).cross(p2 - top_mid).normalize_or_zero();
                if top_n.length_squared() > 0.5 {
                    target.push_quad(Quad::new(top_mid, p3, p2, p2, top_n, self.pigment));
                }
            }
        }
    }
}
