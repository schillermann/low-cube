use glam::Vec3;
use world::{MeshTarget, Quad};
use super::super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};

/// Orthogonal 4-sided pedestal plinth with flat walls and distinct flank/crown tones.
#[derive(Clone, Copy, Debug)]
pub(super) struct SquarePlinth {
    bounds: (Vec3, Vec3),
    tones: (Vec3, Vec3),
}

impl SquarePlinth {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(bounds: (Vec3, Vec3), tones: (Vec3, Vec3)) -> Self {
        Self { bounds, tones }
    }
}

impl FacetedSolid for SquarePlinth {
    fn emit(&self, frame: &LimbFrame, capped: bool, target: &mut dyn MeshTarget) {
        let (min, max) = self.bounds;
        let (wall_col, top_col) = self.tones;
        let [b0, b1, b2, b3] = [
            frame.point(min.x, min.y, min.z),
            frame.point(max.x, min.y, min.z),
            frame.point(max.x, min.y, max.z),
            frame.point(min.x, min.y, max.z),
        ];
        let [t0, t1, t2, t3] = [
            frame.point(min.x, max.y, min.z),
            frame.point(max.x, max.y, min.z),
            frame.point(max.x, max.y, max.z),
            frame.point(min.x, max.y, max.z),
        ];
        for (v0, v1, v2, v3, col) in [
            (t0, t1, t2, t3, top_col),
            (b2, b3, t3, t2, wall_col),
            (b0, b1, t1, t0, wall_col),
            (b1, b2, t2, t1, wall_col),
            (b3, b0, t0, t3, wall_col),
        ] {
            let normal = (v1 - v0).cross(v3 - v0).normalize_or_zero();
            target.push_quad(Quad::new(v0, v1, v2, v3, normal, col));
        }
        if capped {
            let bottom_n = (b3 - b0).cross(b1 - b0).normalize_or_zero();
            target.push_quad(Quad::new(b0, b3, b2, b1, bottom_n, wall_col));
        }
    }
}

/// Rectangular sloped diagonal strut (`Kopfband`) aligned in the local XY crane plane.
#[derive(Clone, Copy, Debug)]
pub(super) struct SlopedBeam {
    endpoints: (Vec3, Vec3),
    section: (f32, f32),
    tones: (Vec3, Vec3),
}

impl SlopedBeam {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(endpoints: (Vec3, Vec3), section: (f32, f32), tones: (Vec3, Vec3)) -> Self {
        Self { endpoints, section, tones }
    }
}

impl FacetedSolid for SlopedBeam {
    fn emit(&self, frame: &LimbFrame, _capped: bool, target: &mut dyn MeshTarget) {
        let (p0, p1) = self.endpoints;
        let (half_w, half_h) = self.section;
        let (flank_col, top_col) = self.tones;
        let dir = (p1 - p0).normalize_or_zero();
        let w = Vec3::new(0.0, 0.0, half_w);
        let h = Vec3::Z.cross(dir).normalize_or_zero() * half_h;
        let map = |v: Vec3| frame.point(v.x, v.y, v.z);
        let [b0, b1, b2, b3] = [map(p0 - w - h), map(p0 + w - h), map(p0 + w + h), map(p0 - w + h)];
        let [t0, t1, t2, t3] = [map(p1 - w - h), map(p1 + w - h), map(p1 + w + h), map(p1 - w + h)];
        for (v0, v1, v2, v3, col) in [
            (b2, b3, t3, t2, top_col),
            (b0, b1, t1, t0, flank_col),
            (b1, b2, t2, t1, flank_col),
            (b3, b0, t0, t3, flank_col),
        ] {
            let normal = (v1 - v0).cross(v3 - v0).normalize_or_zero();
            target.push_quad(Quad::new(v0, v1, v2, v3, normal, col));
        }
    }
}

/// Sculpted wild-boar venison haunch with ivory shank bone, ruby game meat, and pale fat rind.
#[derive(Clone, Copy, Debug)]
pub(super) struct BoarHaunch;

impl FacetedSolid for BoarHaunch {
    fn emit(&self, frame: &LimbFrame, _capped: bool, target: &mut dyn MeshTarget) {
        let (bone, ruby_meat, dark_rind, fat_cap) = (
            Vec3::new(0.90, 0.86, 0.76),
            Vec3::new(0.66, 0.18, 0.15),
            Vec3::new(0.42, 0.15, 0.11),
            Vec3::new(0.92, 0.82, 0.72),
        );
        TaperedPrism::new(6, Vec3::new(-0.082, 0.015, -0.012), Vec3::new(-0.068, 0.017, -0.009), 0.015, 0.013, 0.011, 0.010, bone)
            .emit(frame, true, target);
        AxialLimb::new(Vec3::new(-0.070, 0.017, -0.009), Vec3::new(-0.022, 0.022, -0.002), 0.010, 0.012, bone)
            .emit(frame, true, target);
        TaperedPrism::new(7, Vec3::new(-0.026, 0.014, -0.003), Vec3::new(0.034, 0.026, 0.005), 0.034, 0.026, 0.044, 0.034, ruby_meat)
            .emit(frame, true, target);
        TaperedPrism::new(7, Vec3::new(0.032, 0.026, 0.005), Vec3::new(0.066, 0.022, 0.009), 0.043, 0.033, 0.022, 0.018, dark_rind)
            .emit(frame, true, target);
        TaperedPrism::new(6, Vec3::new(-0.012, 0.040, -0.001), Vec3::new(0.044, 0.048, 0.006), 0.026, 0.011, 0.030, 0.012, fat_cap)
            .emit(frame, true, target);
    }
}
