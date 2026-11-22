use glam::{Mat4, Vec2, Vec3};
use world::{CockpitAxes, JetTone, MeshTarget, Quad};
use crate::menu::glyph::PixelStamp;

pub(super) const CARD_GLASS: Vec3 = Vec3::new(0.014, 0.056, 0.034);
pub(super) const CARD_RIM: Vec3 = Vec3::new(0.16, 0.84, 0.42);
pub(super) const CYAN_WIRE: Vec3 = Vec3::new(0.22, 0.92, 1.00);
pub(super) const AMBER_WIRE: Vec3 = Vec3::new(1.00, 0.68, 0.20);
pub(super) const STEER_PEAK: Vec3 = Vec3::new(0.44, 1.00, 0.52);
pub(super) const STEER_HALO: Vec3 = Vec3::new(0.14, 0.88, 0.28);

/// Camera-synchronized 3D perspective stage projecting `HullWireframe`, `ExhaustJet`, and `STEER` vectors.
#[derive(Clone, Copy, Debug)]
pub(super) struct CameraStage {
    ship_axes: CockpitAxes,
    cam_basis: (Vec3, Vec3, Vec3),
    center: Vec2,
    scale_ux: (f32, f32),
}

impl CameraStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        ship_axes: CockpitAxes,
        cam_basis: (Vec3, Vec3, Vec3),
        center: Vec2,
        scale_ux: (f32, f32),
    ) -> Self {
        Self { ship_axes, cam_basis, center, scale_ux }
    }

    pub(super) fn from_view_proj(
        ship_axes: CockpitAxes,
        view_proj: Mat4,
        center: Vec2,
        scale: f32,
        inv_aspect: f32,
    ) -> Self {
        let c0 = view_proj.x_axis;
        let c1 = view_proj.y_axis;
        let c2 = view_proj.z_axis;
        let raw_r = Vec3::new(c0.x, c1.x, c2.x);
        let raw_u = Vec3::new(c0.y, c1.y, c2.y);
        let basis = if raw_r.length_squared() > 1e-6 && raw_u.length_squared() > 1e-6 {
            let r = raw_r.normalize();
            let u = (raw_u - r * raw_u.dot(r)).normalize_or_zero();
            let u_clean = if u.length_squared() > 0.5 { u } else { Vec3::Y };
            let b = r.cross(u_clean).normalize_or_zero();
            (r, u_clean, b)
        } else {
            (Vec3::X, Vec3::Y, Vec3::Z)
        };
        Self::new(ship_axes, basis, center, (scale, inv_aspect.max(0.25)))
    }

    fn to_tilted_cam(self, ship_local: Vec3) -> Vec3 {
        let w = self.ship_axes.to_world(ship_local);
        let (r, u, b) = self.cam_basis;
        let cv = Vec3::new(w.dot(r), w.dot(u), w.dot(b));
        let (sy, cy) = (-0.32_f32).sin_cos();
        let (sp, cp) = 0.36_f32.sin_cos();
        let x1 = cv.x * cy - cv.z * sy;
        let z1 = cv.x * sy + cv.z * cy;
        let y2 = cv.y * cp - z1 * sp;
        let z2 = cv.y * sp + z1 * cp;
        Vec3::new(x1, y2, z2)
    }

    pub(super) fn project(self, ship_local: Vec3) -> Vec2 {
        let v = self.to_tilted_cam(ship_local);
        let (scale, ux) = self.scale_ux;
        let persp = 2.65 / (2.65 - v.z).clamp(1.25, 4.20);
        Vec2::new(
            self.center.x + v.x * persp * scale * ux,
            self.center.y + v.y * persp * scale,
        )
    }

    pub(super) fn emit_beam(
        self,
        a: Vec2,
        b: Vec2,
        half_w: f32,
        col: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        let ux = self.scale_ux.1;
        let metric = Vec2::new((b.x - a.x) / ux, b.y - a.y);
        let len = metric.length();
        if len <= 1e-5 {
            return;
        }
        let dir = metric / len;
        let perp = Vec2::new(-dir.y * half_w * ux, dir.x * half_w);
        let norm = if glow < 0.0 { Vec3::new(0.0, 1.0, 1.0) } else { Vec3::Z };
        target.push_quad(Quad::glowing(
            Vec3::new(a.x - perp.x, a.y - perp.y, 0.0),
            Vec3::new(b.x - perp.x, b.y - perp.y, 0.0),
            Vec3::new(b.x + perp.x, b.y + perp.y, 0.0),
            Vec3::new(a.x + perp.x, a.y + perp.y, 0.0),
            norm,
            col,
            glow,
        ));
    }

    pub(super) fn emit_box(self, center: Vec3, half: Vec3, col: Vec3, width: f32, target: &mut dyn MeshTarget) {
        let lo = center - half;
        let hi = center + half;
        let c = [
            Vec3::new(lo.x, lo.y, lo.z),
            Vec3::new(hi.x, lo.y, lo.z),
            Vec3::new(hi.x, lo.y, hi.z),
            Vec3::new(lo.x, lo.y, hi.z),
            Vec3::new(lo.x, hi.y, lo.z),
            Vec3::new(hi.x, hi.y, lo.z),
            Vec3::new(hi.x, hi.y, hi.z),
            Vec3::new(lo.x, hi.y, hi.z),
        ]
        .map(|p| self.project(p));
        for (i, j) in [
            (0, 1), (1, 2), (2, 3), (3, 0),
            (4, 5), (5, 6), (6, 7), (7, 4),
            (0, 4), (1, 5), (2, 6), (3, 7),
        ] {
            self.emit_beam(c[i], c[j], width, col, 0.96, target);
        }
    }

    pub(super) fn emit_nose_pyramid(self, center: Vec3, half: f32, target: &mut dyn MeshTarget) {
        let base_y = center.y - half;
        let apex = self.project(Vec3::new(center.x, center.y + half * 1.45, center.z));
        let b = [
            Vec3::new(center.x - half, base_y, center.z - half),
            Vec3::new(center.x + half, base_y, center.z - half),
            Vec3::new(center.x + half, base_y, center.z + half),
            Vec3::new(center.x - half, base_y, center.z + half),
        ]
        .map(|p| self.project(p));
        for i in 0..4 {
            self.emit_beam(b[i], b[(i + 1) % 4], 0.0020, CYAN_WIRE, 0.96, target);
            self.emit_beam(b[i], apex, 0.0022, CYAN_WIRE, 0.98, target);
        }
    }

    pub(super) fn emit_bell_skirt(self, center: Vec3, half: f32, col: Vec3, target: &mut dyn MeshTarget) {
        let t_h = half * 0.48;
        let l_h = half * 0.88;
        let top = center.y + half * 0.35;
        let bot = center.y - half;
        let t = [
            Vec3::new(center.x - t_h, top, center.z - t_h),
            Vec3::new(center.x + t_h, top, center.z - t_h),
            Vec3::new(center.x + t_h, top, center.z + t_h),
            Vec3::new(center.x - t_h, top, center.z + t_h),
        ]
        .map(|p| self.project(p));
        let b = [
            Vec3::new(center.x - l_h, bot, center.z - l_h),
            Vec3::new(center.x + l_h, bot, center.z - l_h),
            Vec3::new(center.x + l_h, bot, center.z + l_h),
            Vec3::new(center.x - l_h, bot, center.z + l_h),
        ]
        .map(|p| self.project(p));
        for i in 0..4 {
            self.emit_beam(t[i], b[i], 0.0020, col, 0.96, target);
            self.emit_beam(b[i], b[(i + 1) % 4], 0.0022, col, 0.96, target);
        }
    }

    pub(super) fn emit_jet(
        self,
        tone: JetTone,
        mouth_local: Vec3,
        dir_local: Vec3,
        throttle: f32,
        bounds: (f32, f32, f32, f32),
        target: &mut dyn MeshTarget,
    ) {
        if throttle <= 0.02 {
            return;
        }
        let ux = self.scale_ux.1;
        let t = throttle.clamp(0.0, 1.0);
        let reach = match tone {
            JetTone::VernierCyan => 0.36 + 0.42 * t,
            JetTone::BellAmber | JetTone::FusionBlue => 0.44 + 0.52 * t,
        };
        let p0 = self.project(mouth_local);
        let raw_tip = self.project(mouth_local + dir_local * reach);
        let tip = Vec2::new(
            raw_tip.x.clamp(bounds.0 + 0.015 * ux, bounds.2 - 0.015 * ux),
            raw_tip.y.clamp(bounds.1 + 0.015, bounds.3 - 0.015),
        );
        let m = Vec2::new((tip.x - p0.x) / ux, tip.y - p0.y);
        let len = m.length();
        if len <= 0.008 {
            return;
        }
        let u = m / len;
        let base_w = match tone {
            JetTone::VernierCyan => 0.018 + 0.010 * t,
            JetTone::BellAmber | JetTone::FusionBlue => 0.024 + 0.012 * t,
        };
        let side = Vec2::new(-u.y * ux, u.x);
        let (sheath, core) = match tone {
            JetTone::VernierCyan => (CYAN_WIRE, Vec3::new(0.86, 1.00, 1.00)),
            JetTone::BellAmber => (AMBER_WIRE, Vec3::new(1.00, 0.95, 0.72)),
            JetTone::FusionBlue => (Vec3::new(0.36, 0.86, 1.00), Vec3::new(0.94, 0.99, 1.00)),
        };
        target.push_quad(Quad::glowing(
            Vec3::new(p0.x - side.x * base_w, p0.y - side.y * base_w, 0.0),
            Vec3::new(p0.x + side.x * base_w, p0.y + side.y * base_w, 0.0),
            Vec3::new(tip.x + side.x * base_w * 0.12, tip.y + side.y * base_w * 0.12, 0.0),
            Vec3::new(tip.x - side.x * base_w * 0.12, tip.y - side.y * base_w * 0.12, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            sheath,
            -0.78,
        ));
        let ctip = p0.lerp(tip, 0.64);
        let cw = base_w * 0.45;
        target.push_quad(Quad::glowing(
            Vec3::new(p0.x - side.x * cw, p0.y - side.y * cw, 0.0),
            Vec3::new(p0.x + side.x * cw, p0.y + side.y * cw, 0.0),
            Vec3::new(ctip.x, ctip.y, 0.0),
            Vec3::new(ctip.x, ctip.y, 0.0),
            Vec3::Z,
            core,
            0.99,
        ));
    }

    pub(super) fn emit_steer_vector(
        self,
        anchor_local: Vec3,
        steer_local: Vec3,
        bounds: (f32, f32, f32, f32),
        target: &mut dyn MeshTarget,
    ) {
        let mag = steer_local.length();
        if mag <= 0.03 {
            return;
        }
        let ux = self.scale_ux.1;
        let unit = steer_local / mag;
        let reach = 0.62 + 0.68 * mag.clamp(0.0, 1.0);
        let a = self.project(anchor_local);
        let raw_b = self.project(anchor_local + unit * reach);
        let b = Vec2::new(
            raw_b.x.clamp(bounds.0 + 0.06 * ux, bounds.2 - 0.06 * ux),
            raw_b.y.clamp(bounds.1 + 0.06, bounds.3 - 0.06),
        );
        let m = Vec2::new((b.x - a.x) / ux, b.y - a.y);
        let len = m.length();
        if len <= 0.012 {
            return;
        }
        let dir = m / len;
        let s = Vec2::new(-dir.y * ux, dir.x);
        let f = Vec2::new(dir.x * ux, dir.y);
        self.emit_beam(a, b, 0.0082, STEER_HALO, -0.32, target);
        self.emit_beam(a, b, 0.0034, STEER_PEAK, 0.98, target);
        let w0 = b - f * 0.026 + s * 0.022;
        let w1 = b - f * 0.026 - s * 0.022;
        self.emit_beam(w0, b, 0.0032, STEER_PEAK, 0.98, target);
        self.emit_beam(w1, b, 0.0032, STEER_PEAK, 0.98, target);

        let caption = if dir.x < -0.25 { "< STEER" } else { "STEER >" };
        let lx = (b.x + dir.x * 0.04 * ux).clamp(bounds.0 + 0.14 * ux, bounds.2 - 0.14 * ux);
        let ly = (b.y + 0.045).clamp(bounds.1 + 0.045, bounds.3 - 0.042);
        PixelStamp::new(0.0048 * ux, 0.0072, STEER_PEAK).emit_centered(caption, lx, ly, target);
    }
}
