use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::mold::MoldArchetype;
use super::wedge::WedgeHeading;

/// Surface-mounted field camera standing on three splayed tripod legs with an 8-way rotatable lens housing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldCameraTripod {
    heading: WedgeHeading,
    archetype: MoldArchetype,
}

impl FieldCameraTripod {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(heading: WedgeHeading, archetype: MoldArchetype) -> Self {
        Self { heading, archetype }
    }

    /// Resolves the 3D world position of the tripod optical hub, base forward heading, and surface normal.
    pub fn tripod_hub(self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame) -> (Vec3, Vec3, Vec3) {
        let (uc, vc) = cell.center();
        let up = frame.normal();
        let base_fwd = self.heading.forward_vector(frame);
        let origin = frame.point_at(uc, vc, floor_elev);
        (origin + up * 0.255, base_fwd, up)
    }

    /// Emits the three splayed tripod legs, brass crown hub, and rotated camera housing.
    pub fn emit(self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (uc, vc) = cell.center();
        let up = frame.normal();
        let base_fwd = self.heading.forward_vector(frame);
        let base_right = base_fwd.cross(up).normalize_or_zero();
        let (s, c) = self.archetype.placed_camera_yaw().sin_cos();
        let fwd = (base_fwd * c + base_right * s).normalize_or_zero();
        let right = fwd.cross(up).normalize_or_zero();
        let origin = frame.point_at(uc, vc, floor_elev);
        let (leather, mahogany, brass, glass, flash) = (
            Vec3::new(0.18, 0.19, 0.24),
            Vec3::new(0.46, 0.25, 0.14),
            Vec3::new(0.88, 0.72, 0.28),
            Vec3::new(0.32, 0.68, 0.88),
            Vec3::new(0.95, 0.92, 0.82),
        );

        for leg_idx in 0..3 {
            let angle = (leg_idx as f32) * (2.0 * std::f32::consts::FRAC_PI_3);
            let (ls, lc) = angle.sin_cos();
            let radial = (fwd * lc + right * ls).normalize_or_zero();
            let lateral = radial.cross(up).normalize_or_zero();
            Self::emit_splayed_leg(origin, up, radial, lateral, mahogany, brass, target);
        }

        let emit_box_local = |x0: f32, x1: f32, y0: f32, y1: f32, z0: f32, z1: f32, w_col: Vec3, t_col: Vec3, out: &mut dyn MeshTarget| {
            let pt = |x: f32, y: f32, z: f32| origin + right * x + up * y + fwd * z;
            let [a0, a1, a2, a3] = [pt(x0, y0, z0), pt(x1, y0, z0), pt(x1, y0, z1), pt(x0, y0, z1)];
            let [b0, b1, b2, b3] = [pt(x0, y1, z0), pt(x1, y1, z0), pt(x1, y1, z1), pt(x0, y1, z1)];
            out.push_quad(Quad::new(b0, b1, b2, b3, up, t_col));
            out.push_quad(Quad::new(a0, a3, a2, a1, -up, w_col));
            for ([p0, p1, p2, p3], n) in [([a0, a1, b1, b0], -fwd), ([a1, a2, b2, b1], right), ([a2, a3, b3, b2], fwd), ([a3, a0, b0, b3], -right)] {
                out.push_quad(Quad::new(p0, p1, p2, p3, n, w_col));
            }
        };
        emit_box_local(-0.034, 0.034, 0.155, 0.185, -0.034, 0.034, brass, brass, target);
        emit_box_local(-0.11, 0.11, 0.185, 0.325, -0.075, 0.075, leather, mahogany, target);
        emit_box_local(-0.048, 0.048, 0.220, 0.300, 0.075, 0.135, brass, brass, target);
        emit_box_local(-0.034, 0.034, 0.233, 0.287, 0.135, 0.152, glass, glass, target);
        emit_box_local(-0.068, -0.028, 0.325, 0.360, -0.02, 0.02, brass, brass, target);
        emit_box_local(0.024, 0.076, 0.325, 0.375, -0.015, 0.025, brass, flash, target);
    }

    fn emit_splayed_leg(
        origin: Vec3,
        up: Vec3,
        radial: Vec3,
        lateral: Vec3,
        wood_col: Vec3,
        brass_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let (r_top, y_top, r_bot, y_bot, w, d) = (0.020, 0.165, 0.185, 0.016, 0.014, 0.014);
        let top_center = origin + up * y_top + radial * r_top;
        let bot_center = origin + up * y_bot + radial * r_bot;
        let [t0, t1, t2, t3] = [
            top_center - lateral * w - radial * d,
            top_center + lateral * w - radial * d,
            top_center + lateral * w + radial * d,
            top_center - lateral * w + radial * d,
        ];
        let [b0, b1, b2, b3] = [
            bot_center - lateral * w - radial * d,
            bot_center + lateral * w - radial * d,
            bot_center + lateral * w + radial * d,
            bot_center - lateral * w + radial * d,
        ];
        target.push_quad(Quad::new(t0, t1, t2, t3, up, wood_col));
        target.push_quad(Quad::new(b0, b3, b2, b1, -up, brass_col));
        let axis = (top_center - bot_center).normalize_or_zero();
        let out_norm = radial.cross(lateral).cross(axis).normalize_or_zero();
        for ([p0, p1, p2, p3], n) in [
            ([b0, b1, t1, t0], -out_norm),
            ([b1, b2, t2, t1], lateral),
            ([b2, b3, t3, t2], out_norm),
            ([b3, b0, t0, t3], -lateral),
        ] {
            target.push_quad(Quad::new(p0, p1, p2, p3, n, wood_col));
        }

        let foot_base = origin + up * 0.002 + radial * r_bot;
        let foot_top = origin + up * y_bot + radial * r_bot;
        let fw = 0.020;
        let [fa0, fa1, fa2, fa3] = [
            foot_base - lateral * fw - radial * fw,
            foot_base + lateral * fw - radial * fw,
            foot_base + lateral * fw + radial * fw,
            foot_base - lateral * fw + radial * fw,
        ];
        let [fb0, fb1, fb2, fb3] = [
            foot_top - lateral * fw - radial * fw,
            foot_top + lateral * fw - radial * fw,
            foot_top + lateral * fw + radial * fw,
            foot_top - lateral * fw + radial * fw,
        ];
        target.push_quad(Quad::new(fb0, fb1, fb2, fb3, up, brass_col));
        target.push_quad(Quad::new(fa0, fa3, fa2, fa1, -up, brass_col));
        for ([p0, p1, p2, p3], n) in [
            ([fa0, fa1, fb1, fb0], -radial),
            ([fa1, fa2, fb2, fb1], lateral),
            ([fa2, fa3, fb3, fb2], radial),
            ([fa3, fa0, fb0, fb3], -lateral),
        ] {
            target.push_quad(Quad::new(p0, p1, p2, p3, n, brass_col));
        }
    }
}
