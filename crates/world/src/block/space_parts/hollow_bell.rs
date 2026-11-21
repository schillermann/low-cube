use std::f32::consts::PI;
use glam::Vec3;
use crate::{MeshTarget, Quad};

/// Sculpted 3D hollow De Laval thruster bell with dark interior cavity and translucent supersonic jet plume.
pub struct HollowBell {
    origin: Vec3,
    basis: (Vec3, Vec3),
    envelope: (f32, f32),
    segments: usize,
}

impl HollowBell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: Vec3,
        basis: (Vec3, Vec3),
        envelope: (f32, f32),
        segments: usize,
    ) -> Self {
        Self { origin, basis, envelope, segments }
    }

    fn ortho_axes(&self) -> (Vec3, Vec3, Vec3) {
        let (axis, ref_up) = self.basis;
        let ex = axis.normalize_or_zero();
        let fallback = if ex.dot(ref_up).abs() > 0.92 {
            ex.any_orthonormal_vector()
        } else {
            ref_up
        };
        let sa = ex.cross(fallback).normalize_or_zero();
        let sb = ex.cross(sa).normalize_or_zero();
        (sa, sb, ex)
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_wall(
        &self,
        z0: f32,
        r0: f32,
        z1: f32,
        r1: f32,
        inward: bool,
        col: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        let (sa, sb, ex) = self.ortho_axes();
        let segs = self.segments.max(4);
        let step = (2.0 * PI) / (segs as f32);
        let (m0, m1) = (self.origin + ex * z0, self.origin + ex * z1);
        for i in 0..segs {
            let ((s0, c0), (s1, c1)) = (((i as f32) * step).sin_cos(), (((i + 1) as f32) * step).sin_cos());
            let (d0, d1) = (sa * c0 + sb * s0, sa * c1 + sb * s1);
            let [a0, a1] = [m0 + d0 * r0, m0 + d1 * r0];
            let [b0, b1] = [m1 + d0 * r1, m1 + d1 * r1];
            let out_norm = (a1 - a0).cross(b0 - a0).normalize_or_zero();
            if inward {
                target.push_quad(Quad::glowing(a1, a0, b0, b1, -out_norm, col, glow));
            } else {
                target.push_quad(Quad::glowing(a0, a1, b1, b0, out_norm, col, glow));
            }
        }
    }

    fn emit_ring(&self, z: f32, r_inner: f32, r_outer: f32, flip: bool, col: Vec3, glow: f32, target: &mut dyn MeshTarget) {
        let (sa, sb, ex) = self.ortho_axes();
        let segs = self.segments.max(4);
        let step = (2.0 * PI) / (segs as f32);
        let mid = self.origin + ex * z;
        let norm = if flip { -ex } else { ex };
        for i in 0..segs {
            let ((s0, c0), (s1, c1)) = (((i as f32) * step).sin_cos(), (((i + 1) as f32) * step).sin_cos());
            let (d0, d1) = (sa * c0 + sb * s0, sa * c1 + sb * s1);
            let [i0, i1] = [mid + d0 * r_inner, mid + d1 * r_inner];
            let [o0, o1] = [mid + d0 * r_outer, mid + d1 * r_outer];
            if flip {
                target.push_quad(Quad::glowing(i1, i0, o0, o1, norm, col, glow));
            } else {
                target.push_quad(Quad::glowing(i0, i1, o1, o0, norm, col, glow));
            }
        }
    }

    /// Emits the hollow outer nozzle wall, mouth lip ring, dark inner combustion liner, and deep throat injector.
    pub fn emit_hollow_shell(
        &self,
        throat_brass: Vec3,
        outer_graphite: Vec3,
        rim_metal: Vec3,
        active_burn: f32,
        target: &mut dyn MeshTarget,
    ) {
        let (len, mouth_r) = self.envelope;
        let throat_r = mouth_r * 0.42;
        let mid_z = len * 0.42;
        let mid_r = mouth_r * 0.78;
        let inner_mouth_r = mouth_r * 0.85;
        let inner_mid_r = mid_r * 0.84;
        let inner_throat_r = throat_r * 0.78;
        let cavity_depth = len * 0.10;
        let soot = Vec3::new(0.05, 0.06, 0.08);

        self.emit_wall(0.0, throat_r * 1.18, len * 0.20, throat_r, false, throat_brass, 0.0, target);
        self.emit_wall(len * 0.20, throat_r, mid_z, mid_r, false, outer_graphite, 0.0, target);
        self.emit_wall(mid_z, mid_r, len, mouth_r, false, outer_graphite, 0.0, target);
        self.emit_ring(len, inner_mouth_r, mouth_r, false, rim_metal, 0.0, target);

        let deep_lining = if active_burn > 0.03 { Vec3::new(0.32, 0.14, 0.06) } else { soot };
        let deep_glow = if active_burn > 0.03 { 0.28 * active_burn.min(1.0) } else { 0.0 };
        self.emit_wall(mid_z, inner_mid_r, len, inner_mouth_r, true, soot, 0.0, target);
        self.emit_wall(cavity_depth, inner_throat_r, mid_z, inner_mid_r, true, deep_lining, deep_glow, target);

        let (inj_col, inj_glow) = if active_burn > 0.03 {
            (Vec3::new(1.00, 0.82, 0.42), (0.88 * active_burn).clamp(0.45, 0.96))
        } else {
            (Vec3::new(0.04, 0.05, 0.07), 0.0)
        };
        self.emit_ring(cavity_depth, 0.0, inner_throat_r, false, inj_col, inj_glow, target);
    }

    /// Emits a 3-layer semi-transparent supersonic exhaust flame (`glow < -1.5`) only when `intensity > 0.03`.
    pub fn emit_translucent_jet(
        &self,
        intensity: f32,
        palette: (Vec3, Vec3, Vec3),
        reach_scale: f32,
        target: &mut dyn MeshTarget,
    ) {
        if intensity <= 0.03 {
            return;
        }
        let p = intensity.clamp(0.0, 1.2);
        let (len, mouth_r) = self.envelope;
        let reach = reach_scale * (0.55 + 0.45 * p);
        let (core_col, mid_col, veil_col) = palette;

        let core_glow = -(2.0 + (0.74 * p).clamp(0.20, 0.84));
        let mid_glow = -(2.0 + (0.46 * p).clamp(0.14, 0.56));
        let veil_glow = -(2.0 + (0.24 * p).clamp(0.08, 0.32));

        self.emit_wall(len * 0.22, mouth_r * 0.34, len + reach * 0.54, mouth_r * 0.07, false, core_col, core_glow, target);
        self.emit_wall(len * 0.50, mouth_r * 0.58, len + reach * 0.86, mouth_r * 0.14, false, mid_col, mid_glow, target);
        self.emit_wall(len * 0.84, mouth_r * 0.80, len + reach * 1.14, mouth_r * 0.22, false, veil_col, veil_glow, target);
    }
}
