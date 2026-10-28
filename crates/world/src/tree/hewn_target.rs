use std::f32::consts::PI;
use glam::Vec3;
use crate::{MeshTarget, Quad, SurfaceFrame};

#[derive(Clone, Copy, Debug)]
struct TimberAnchor {
    frame: SurfaceFrame,
    u: f32,
    v: f32,
    elevation: f32,
}

#[derive(Clone, Copy, Debug)]
struct NotchCleave {
    notch_ratio: f32,
    shudder: f32,
    cleave_u: f32,
    cleave_v: f32,
}

/// Decorator sculpting progressive heartwood axe notches, trunk/crown strike shudder, woodchips, and falling leaves.
pub struct HewnTimberTarget<'a> {
    inner: &'a mut dyn MeshTarget,
    anchor: TimberAnchor,
    notch: NotchCleave,
    foliage_color: Vec3,
}

impl<'a> HewnTimberTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        inner: &'a mut dyn MeshTarget,
        frame: SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        notch_ratio: f32,
        shudder: f32,
        cleave_u: f32,
        cleave_v: f32,
        foliage_color: Vec3,
    ) -> Self {
        Self {
            inner,
            anchor: TimberAnchor { frame, u, v, elevation },
            notch: NotchCleave { notch_ratio, shudder, cleave_u, cleave_v },
            foliage_color,
        }
    }

    fn displace(&self, point: Vec3) -> Vec3 {
        let (a, n) = (self.anchor, self.notch);
        if n.shudder <= 0.0 {
            return point;
        }
        let rel_h = (a.frame.elevation_of(point) - a.elevation).max(0.0);
        let cleave_vec = a.frame.tangent() * n.cleave_u + a.frame.bitangent() * n.cleave_v;
        let side_vec = a.frame.tangent() * (-n.cleave_v) + a.frame.bitangent() * n.cleave_u;
        let wave = (n.shudder * PI * 6.0).sin() * n.shudder * n.shudder;
        let trunk_sway = (rel_h * 0.042).min(0.095) * wave;
        let phase = point.x * 3.7 + point.z * 5.3;
        let rustle = ((n.shudder * 8.0 + phase * 0.25) * PI).sin() * n.shudder;
        let crown_flutter = (rel_h - 1.0).clamp(0.0, 2.5) * 0.028 * rustle;
        point
            + cleave_vec * (trunk_sway + crown_flutter)
            + side_vec * (crown_flutter * 0.65)
            + a.frame.normal() * (crown_flutter * 0.25)
    }

    fn push_shuddered(&mut self, v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, normal: Vec3, color: Vec3) {
        let (p0, p1, p2, p3) = (self.displace(v0), self.displace(v1), self.displace(v2), self.displace(v3));
        self.inner.push_quad(Quad::new(p0, p1, p2, p3, normal, color));
    }

    /// Emits flying 3D woodchip splinters from the chest-height axe notch and fluttering canopy leaves.
    pub fn emit_cleave_burst(&mut self) {
        let (a, n) = (self.anchor, self.notch);
        if n.shudder <= 0.0 {
            return;
        }
        let progress = (1.0 - n.shudder).clamp(0.02, 0.98);
        let (tang, bitang, up) = (a.frame.tangent(), a.frame.bitangent(), a.frame.normal());
        let cleave_dir = (tang * n.cleave_u + bitang * n.cleave_v).normalize_or_zero();
        let out_dir = if cleave_dir.length_squared() > 0.1 { -cleave_dir } else { -bitang };
        let side_dir = out_dir.cross(up).normalize_or_zero();
        let notch_origin = a.frame.point_at(a.u, a.v, a.elevation + 1.08) + out_dir * 0.24;

        let chips = [
            (-0.85, 0.65, 0.46, 0.34, 0.00),
            (0.82, 0.70, 0.52, 0.38, 0.04),
            (-0.42, 0.92, 0.40, 0.28, 0.08),
            (0.48, 0.88, 0.48, 0.32, 0.02),
            (-0.95, 0.32, 0.44, 0.30, 0.06),
            (0.94, 0.36, 0.50, 0.36, 0.10),
        ];
        for (i, &(sx, fwd, reach, apex, delay)) in chips.iter().enumerate() {
            let tk = ((progress - delay) / (1.0 - delay)).clamp(0.0, 1.0);
            if tk <= 0.0 || tk >= 1.0 {
                continue;
            }
            let dist = 0.04 + reach * tk;
            let lift = (apex * tk - 1.15 * tk * tk).max(-0.98);
            let center = notch_origin + (side_dir * sx + out_dir * fwd).normalize_or_zero() * dist + up * lift;
            let spin = tk * 9.0 + (i as f32) * 1.1;
            let axis_a = (side_dir * spin.cos() + up * spin.sin()) * 0.038;
            let axis_b = (out_dir * spin.sin() - up * spin.cos()) * 0.022;
            let col = if i % 2 == 0 { Vec3::new(0.88, 0.74, 0.48) } else { Vec3::new(0.46, 0.31, 0.19) };
            let norm = axis_a.cross(axis_b).normalize_or_zero();
            self.inner.push_quad(Quad::new(
                center - axis_a - axis_b,
                center + axis_a - axis_b,
                center + axis_a + axis_b,
                center - axis_a + axis_b,
                if norm.length_squared() > 0.1 { norm } else { up },
                col,
            ));
        }

        for i in 0..8 {
            let fi = i as f32;
            let ang = fi * (PI * 0.25) + 0.35;
            let rad = 0.42 + ((i % 3) as f32) * 0.26;
            let h0 = 2.10 + ((i % 4) as f32) * 0.24;
            let h = (h0 - 1.65 * progress).max(0.08);
            let sway = 0.18 * (progress * 7.5 + fi * 1.3).sin();
            let drift = 0.14 * (progress * 6.0 + fi * 2.1).cos();
            let center = a.frame.point_at(
                a.u + rad * ang.cos() + sway,
                a.v + rad * ang.sin() + drift,
                a.elevation + h,
            );
            let tilt = (progress * 7.5 + fi).cos() * 0.034;
            let span_u = tang * 0.048 + up * tilt;
            let span_v = bitang * 0.048 - up * tilt;
            let shade = 0.90 + ((i % 3) as f32) * 0.08;
            let leaf_n = (up + tang * (tilt * 8.0)).normalize_or_zero();
            self.inner.push_quad(Quad::new(
                center - span_u,
                center - span_v,
                center + span_u,
                center + span_v,
                leaf_n,
                (self.foliage_color * shade).clamp(Vec3::ZERO, Vec3::ONE),
            ));
        }
    }
}

impl MeshTarget for HewnTimberTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let (a, n) = (self.anchor, self.notch);
        quad.transfer(|v0, v1, v2, v3, normal, color| {
            let base_rel = a.frame.elevation_of(v0) - a.elevation;
            let top_rel = a.frame.elevation_of(v2) - a.elevation;
            if top_rel > 1.35 && normal.dot(a.frame.normal()) > -0.8 {
                self.foliage_color = color;
            }
            let cleave_vec = a.frame.tangent() * n.cleave_u + a.frame.bitangent() * n.cleave_v;
            let notch_dir = if cleave_vec.length_squared() > 0.1 { -cleave_vec } else { -a.frame.bitangent() };
            if n.notch_ratio > 0.0 && base_rel.abs() < 0.15 && top_rel >= 0.85 {
                let t_mid = (1.12 / top_rel).clamp(0.44, 0.62);
                let t_low = (t_mid - 0.07 - 0.13 * n.notch_ratio).max(0.18);
                let t_high = (t_mid + 0.09 + 0.16 * n.notch_ratio).min(0.86);
                let (l_low, l_mid, l_high) = (v0.lerp(v3, t_low), v0.lerp(v3, t_mid), v0.lerp(v3, t_high));
                let (r_low, r_mid, r_high) = (v1.lerp(v2, t_low), v1.lerp(v2, t_mid), v1.lerp(v2, t_high));
                let axis_mid = a.frame.point_at(a.u, a.v, a.elevation + top_rel * t_mid);
                let align_l = (l_mid - axis_mid).normalize_or_zero().dot(notch_dir).max(0.0);
                let align_r = (r_mid - axis_mid).normalize_or_zero().dot(notch_dir).max(0.0);
                if align_l > 0.05 || align_r > 0.05 {
                    let reach = (0.32 + 0.56 * n.notch_ratio).clamp(0.0, 0.88);
                    let c_left = l_mid.lerp(axis_mid, (align_l * reach).max(0.18 * n.notch_ratio));
                    let c_right = r_mid.lerp(axis_mid, (align_r * reach).max(0.18 * n.notch_ratio));
                    let (hw_floor, hw_roof, hw_cheek) = (
                        Vec3::new(0.88, 0.74, 0.48),
                        Vec3::new(0.72, 0.56, 0.33),
                        Vec3::new(0.64, 0.49, 0.28),
                    );
                    let up_n = (normal + a.frame.normal() * 0.95).normalize_or_zero();
                    let dn_n = (normal - a.frame.normal() * 0.95).normalize_or_zero();
                    self.push_shuddered(v0, v1, r_low, l_low, normal, color);
                    self.push_shuddered(l_high, r_high, v2, v3, normal, color);
                    self.push_shuddered(l_low, r_low, c_right, c_left, up_n, hw_floor);
                    self.push_shuddered(c_left, c_right, r_high, l_high, dn_n, hw_roof);
                    self.push_shuddered(l_low, c_left, l_high, l_high, normal, hw_cheek);
                    self.push_shuddered(r_low, r_high, c_right, r_low, normal, hw_cheek);
                    return;
                }
            }
            self.push_shuddered(v0, v1, v2, v3, normal, color);
        });
    }
}
