use glam::Vec3;
use world::{MeshTarget, Quad};
use super::crown::HeadSculpture;
use super::limb::{FacetedSolid, LimbFrame, TaperedPrism};
use super::pose::BodyPose;

pub(super) fn push_visage_quad(frame: &LimbFrame, pts: [Vec3; 4], color: Vec3, target: &mut dyn MeshTarget) {
    let [p0, p1, p2, p3] = pts.map(|v| frame.point(v.x, v.y, v.z));
    let raw_n = (p1 - p0).cross(p3 - p0);
    let norm = if raw_n.length_squared() > 1e-10 {
        raw_n.normalize()
    } else {
        (frame.point(0.0, 0.0, 1.0) - frame.point(0.0, 0.0, 0.0)).normalize_or_zero()
    };
    target.push_quad(Quad::new(p0, p1, p2, p3, norm, color));
}

/// Sculpted multi-tier cranium, zygomatic cheekbone bevels, tapered jawline, and contoured ears.
#[derive(Clone, Copy, Debug)]
struct CraniumContours {
    skin: Vec3,
    feminine: bool,
}

impl CraniumContours {
    const fn new(skin: Vec3, feminine: bool) -> Self {
        Self { skin, feminine }
    }

    fn emit(&self, pose: &BodyPose, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let warm_shade = self.skin * Vec3::new(0.89, 0.85, 0.83);
        let deep_shade = self.skin * Vec3::new(0.77, 0.71, 0.69);
        let cheek_glow = (self.skin * Vec3::new(1.03, 0.99, 0.98)).min(Vec3::ONE);
        let chin_rx = hw(if self.feminine { 0.036 } else { 0.045 });
        let jaw_rx = hw(if self.feminine { 0.076 } else { 0.085 });

        TaperedPrism::new(
            8,
            Vec3::new(0.0, hs(-0.024), hw(0.022)),
            Vec3::new(0.0, hs(0.022), hw(0.014)),
            chin_rx,
            hw(0.056),
            jaw_rx,
            hw(0.081),
            warm_shade,
        )
        .emit(frame, false, target);
        TaperedPrism::new(
            8,
            Vec3::new(0.0, hs(0.022), hw(0.014)),
            Vec3::new(0.0, hs(0.084), hw(0.010)),
            jaw_rx,
            hw(0.081),
            hw(0.088),
            hw(0.091),
            self.skin,
        )
        .emit(frame, false, target);
        TaperedPrism::new(
            8,
            Vec3::new(0.0, hs(0.084), hw(0.010)),
            Vec3::new(0.0, hs(0.148), hw(0.011)),
            hw(0.088),
            hw(0.091),
            hw(0.089),
            hw(0.094),
            self.skin,
        )
        .emit(frame, false, target);
        TaperedPrism::new(
            8,
            Vec3::new(0.0, hs(0.148), hw(0.011)),
            Vec3::new(0.0, hs(0.200), hw(0.006)),
            hw(0.089),
            hw(0.094),
            hw(0.064),
            hw(0.068),
            self.skin,
        )
        .emit(frame, true, target);

        for side in [-1.0_f32, 1.0_f32] {
            let c_in_lo = Vec3::new(side * hw(0.024), hs(0.058), hw(0.088));
            let c_out_lo = Vec3::new(side * hw(0.064), hs(0.062), hw(0.072));
            let c_out_hi = Vec3::new(side * hw(0.068), hs(0.094), hw(0.075));
            let c_in_hi = Vec3::new(side * hw(0.026), hs(0.092), hw(0.089));
            if side > 0.0 {
                push_visage_quad(frame, [c_out_lo, c_in_lo, c_in_hi, c_out_hi], cheek_glow, target);
            } else {
                push_visage_quad(frame, [c_in_lo, c_out_lo, c_out_hi, c_in_hi], cheek_glow, target);
            }

            TaperedPrism::new(
                6,
                Vec3::new(side * hw(0.088), hs(0.064), hw(-0.004)),
                Vec3::new(side * hw(0.093), hs(0.118), hw(-0.002)),
                hw(0.012),
                hw(0.018),
                hw(0.015),
                hw(0.021),
                warm_shade,
            )
            .emit(frame, true, target);
            TaperedPrism::new(
                4,
                Vec3::new(side * hw(0.094), hs(0.076), hw(-0.001)),
                Vec3::new(side * hw(0.096), hs(0.106), hw(0.001)),
                hw(0.006),
                hw(0.009),
                hw(0.006),
                hw(0.010),
                deep_shade,
            )
            .emit(frame, false, target);
        }
    }
}

/// Faceted 3D triangular nose bridge, tip, and shaded nostril underside.
#[derive(Clone, Copy, Debug)]
struct NasalBridge {
    skin: Vec3,
    feminine: bool,
}

impl NasalBridge {
    const fn new(skin: Vec3, feminine: bool) -> Self {
        Self { skin, feminine }
    }

    fn emit(&self, pose: &BodyPose, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let span = if self.feminine { 0.82 } else { 0.96 };
        let tip_y = if self.feminine { 0.072 } else { 0.068 };
        let dorsal = (self.skin * 1.05).min(Vec3::ONE);
        let flank = self.skin * Vec3::new(0.90, 0.85, 0.83);
        let under_shade = self.skin * Vec3::new(0.76, 0.70, 0.68);

        let root_l = Vec3::new(hw(-0.008 * span), hs(0.120), hw(0.088));
        let root_r = Vec3::new(hw(0.008 * span), hs(0.120), hw(0.088));
        let tip_l = Vec3::new(hw(-0.007 * span), hs(tip_y), hw(0.111));
        let tip_r = Vec3::new(hw(0.007 * span), hs(tip_y), hw(0.111));
        let ala_l = Vec3::new(hw(-0.018 * span), hs(0.061), hw(0.089));
        let ala_r = Vec3::new(hw(0.018 * span), hs(0.061), hw(0.089));
        let sub_l = Vec3::new(hw(-0.011 * span), hs(0.057), hw(0.088));
        let sub_r = Vec3::new(hw(0.011 * span), hs(0.057), hw(0.088));

        push_visage_quad(frame, [tip_r, tip_l, root_l, root_r], dorsal, target);
        push_visage_quad(frame, [ala_r, tip_r, root_r, Vec3::new(hw(0.015 * span), hs(0.114), hw(0.086))], flank, target);
        push_visage_quad(frame, [tip_l, ala_l, Vec3::new(hw(-0.015 * span), hs(0.114), hw(0.086)), root_l], flank, target);
        push_visage_quad(frame, [sub_r, sub_l, tip_l, tip_r], under_shade, target);
    }
}

/// Almond-shaped eyes with iris, pupil, corneal glint, upper lash fold, and arched brows.
#[derive(Clone, Copy, Debug)]
struct OcularGaze {
    skin: Vec3,
    brow_tint: Vec3,
    feminine: bool,
}

impl OcularGaze {
    const fn new(skin: Vec3, brow_tint: Vec3, feminine: bool) -> Self {
        Self { skin, brow_tint, feminine }
    }

    fn emit(&self, pose: &BodyPose, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let sclera = Vec3::new(0.96, 0.96, 0.97);
        let iris = if self.feminine { Vec3::new(0.18, 0.30, 0.36) } else { Vec3::new(0.22, 0.16, 0.11) };
        let pupil = Vec3::new(0.07, 0.06, 0.06);
        let glint = Vec3::new(0.99, 0.99, 1.00);
        let lash_col = if self.feminine {
            Vec3::new(0.14, 0.10, 0.09)
        } else {
            self.skin * Vec3::new(0.76, 0.69, 0.67)
        };

        for side in [-1.0_f32, 1.0_f32] {
            let cx = side * hw(0.036);
            TaperedPrism::new(6, Vec3::new(cx, hs(0.099), hw(0.085)), Vec3::new(cx, hs(0.116), hw(0.085)), hw(0.018), hw(0.008), hw(0.017), hw(0.008), sclera)
                .emit(frame, true, target);
            TaperedPrism::new(6, Vec3::new(cx - side * hw(0.002), hs(0.100), hw(0.089)), Vec3::new(cx - side * hw(0.002), hs(0.115), hw(0.089)), hw(0.010), hw(0.007), hw(0.010), hw(0.007), iris)
                .emit(frame, true, target);
            TaperedPrism::new(4, Vec3::new(cx - side * hw(0.002), hs(0.103), hw(0.092)), Vec3::new(cx - side * hw(0.002), hs(0.112), hw(0.092)), hw(0.005), hw(0.005), hw(0.005), hw(0.005), pupil)
                .emit(frame, true, target);
            let gx = cx - side * hw(0.005);
            TaperedPrism::new(4, Vec3::new(gx, hs(0.110), hw(0.094)), Vec3::new(gx, hs(0.114), hw(0.094)), hw(0.0026), hw(0.003), hw(0.0026), hw(0.003), glint)
                .emit(frame, true, target);
            let lash_w = if self.feminine { 0.020 } else { 0.018 };
            TaperedPrism::new(6, Vec3::new(cx + side * hw(0.001), hs(0.115), hw(0.088)), Vec3::new(cx + side * hw(0.002), hs(0.120), hw(0.090)), hw(lash_w), hw(0.009), hw(lash_w * 0.94), hw(0.009), lash_col)
                .emit(frame, true, target);

            let brow_h = if self.feminine { 0.0075 } else { 0.0115 };
            let inner = Vec3::new(side * hw(0.017), hs(if self.feminine { 0.127 } else { 0.125 }), hw(0.094));
            let arch = Vec3::new(side * hw(0.039), hs(if self.feminine { 0.135 } else { 0.132 }), hw(0.092));
            let outer = Vec3::new(side * hw(0.056), hs(0.127), hw(0.085));
            let up_off = Vec3::new(0.0, hs(brow_h), 0.0);
            if side > 0.0 {
                push_visage_quad(frame, [arch, inner, inner + up_off, arch + up_off], self.brow_tint, target);
                push_visage_quad(frame, [outer, arch, arch + up_off, outer + up_off * 0.72], self.brow_tint, target);
            } else {
                push_visage_quad(frame, [inner, arch, arch + up_off, inner + up_off], self.brow_tint, target);
                push_visage_quad(frame, [arch, outer, outer + up_off * 0.72, arch + up_off], self.brow_tint, target);
            }
        }
    }
}

/// Core `HeadSculpture` emitting the neck, sculpted cranium, nose bridge, eyes, and two-tone lips.
#[derive(Clone, Copy, Debug)]
pub struct BareVisage {
    skin: Vec3,
    brow_tint: Vec3,
    feminine: bool,
}

impl BareVisage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(skin: Vec3, brow_tint: Vec3, feminine: bool) -> Self {
        Self { skin, brow_tint, feminine }
    }
}

impl HeadSculpture for BareVisage {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let neck = pose.neck_frame();
        let shade_skin = self.skin * Vec3::new(0.87, 0.83, 0.81);
        TaperedPrism::hexagonal(
            Vec3::new(0.0, hs(-0.15), hw(-0.022)),
            Vec3::new(0.0, hs(0.025), hw(-0.004)),
            pose.crown_girth(if self.feminine { 0.046 } else { 0.055 }),
            pose.crown_girth(if self.feminine { 0.042 } else { 0.049 }),
            shade_skin,
        )
        .emit(&neck, true, target);

        let frame = pose.crown_frame();
        CraniumContours::new(self.skin, self.feminine).emit(pose, &frame, target);
        NasalBridge::new(self.skin, self.feminine).emit(pose, &frame, target);
        OcularGaze::new(self.skin, self.brow_tint, self.feminine).emit(pose, &frame, target);

        let upper_lip = if self.feminine {
            self.skin * Vec3::new(0.82, 0.49, 0.52)
        } else {
            self.skin * Vec3::new(0.75, 0.63, 0.61)
        };
        let lower_lip = if self.feminine {
            self.skin * Vec3::new(0.91, 0.58, 0.60)
        } else {
            self.skin * Vec3::new(0.83, 0.71, 0.69)
        };
        let lip_w = if self.feminine { 0.021 } else { 0.024 };
        TaperedPrism::new(6, Vec3::new(0.0, hs(0.032), hw(0.086)), Vec3::new(0.0, hs(0.040), hw(0.088)), hw(lip_w), hw(0.009), hw(lip_w * 0.84), hw(0.010), upper_lip)
            .emit(&frame, true, target);
        TaperedPrism::new(6, Vec3::new(0.0, hs(0.022), hw(0.085)), Vec3::new(0.0, hs(0.032), hw(0.087)), hw(lip_w * 0.88), hw(0.009), hw(lip_w * 0.96), hw(0.010), lower_lip)
            .emit(&frame, true, target);
    }
}
