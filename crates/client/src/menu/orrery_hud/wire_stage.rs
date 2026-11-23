use glam::{Mat4, Vec2, Vec3};
use world::{CockpitAxes, JetTone, MeshTarget, Quad};

pub(super) const CARD_GLASS: Vec3 = Vec3::new(0.014, 0.056, 0.034);
pub(super) const CARD_RIM: Vec3 = Vec3::new(0.16, 0.84, 0.42);
pub(super) const GHOST_HULL: Vec3 = Vec3::new(0.14, 0.58, 0.52);
pub(super) const HULL_NEAR: Vec3 = Vec3::new(0.24, 0.88, 0.74);
pub(super) const IDLE_POD: Vec3 = Vec3::new(0.18, 0.74, 0.86);
pub(super) const CYAN_WIRE: Vec3 = Vec3::new(0.20, 0.94, 1.00);
pub(super) const AMBER_WIRE: Vec3 = Vec3::new(1.00, 0.64, 0.14);
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
        let (sy, cy) = (-0.52_f32).sin_cos();
        let (sp, cp) = 0.48_f32.sin_cos();
        let x1 = cv.x * cy - cv.z * sy;
        let z1 = cv.x * sy + cv.z * cy;
        let y2 = cv.y * cp - z1 * sp;
        let z2 = cv.y * sp + z1 * cp;
        Vec3::new(x1, y2, z2)
    }

    pub(super) fn cam_depth(self, ship_local: Vec3) -> f32 {
        self.to_tilted_cam(ship_local).z
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

    pub(super) fn emit_hull_edge(self, a: Vec3, b: Vec3, target: &mut dyn MeshTarget) {
        let mid_z = self.cam_depth((a + b) * 0.5);
        let (col, glow, width) = if mid_z >= 0.0 {
            (HULL_NEAR, -0.68, 0.0018)
        } else {
            (GHOST_HULL, -0.36, 0.0014)
        };
        self.emit_beam(self.project(a), self.project(b), width, col, glow, target);
    }

    pub(super) fn emit_pod_rosette(
        self,
        center: Vec3,
        radius: f32,
        active: bool,
        target: &mut dyn MeshTarget,
    ) {
        let near = self.cam_depth(center) >= 0.0;
        let (col, glow, w) = if active {
            (AMBER_WIRE, 0.99, 0.0028)
        } else if near {
            (CYAN_WIRE, 0.92, 0.0021)
        } else {
            (IDLE_POD, -0.68, 0.0018)
        };
        let pts = [
            Vec3::new(center.x - radius, center.y, center.z),
            Vec3::new(center.x, center.y, center.z + radius),
            Vec3::new(center.x + radius, center.y, center.z),
            Vec3::new(center.x, center.y, center.z - radius),
            Vec3::new(center.x, center.y + radius, center.z),
            Vec3::new(center.x, center.y - radius, center.z),
        ]
        .map(|p| self.project(p));
        for i in 0..4 {
            self.emit_beam(pts[i], pts[(i + 1) % 4], w, col, glow, target);
            self.emit_beam(pts[i], pts[4], w * 0.88, col, glow, target);
            self.emit_beam(pts[i], pts[5], w * 0.88, col, glow, target);
        }
    }

    pub(super) fn emit_bell_skirt(
        self,
        center: Vec3,
        half: Vec3,
        (col, glow): (Vec3, f32),
        target: &mut dyn MeshTarget,
    ) {
        let t_h = half.x * 0.48;
        let l_h = half.x * 0.92;
        let top = center.y + half.y;
        let bot = center.y - half.y;
        let w = if glow > 0.0 { 0.0026 } else { 0.0017 };
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
            self.emit_beam(t[i], t[(i + 1) % 4], w * 0.85, col, glow, target);
            self.emit_beam(t[i], b[i], w, col, glow, target);
            self.emit_beam(b[i], b[(i + 1) % 4], w, col, glow, target);
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
            JetTone::VernierCyan => 0.56 + 0.54 * t,
            JetTone::BellAmber | JetTone::FusionBlue => 0.64 + 0.62 * t,
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
            JetTone::VernierCyan => 0.022 + 0.012 * t,
            JetTone::BellAmber | JetTone::FusionBlue => 0.028 + 0.014 * t,
        };
        let side = Vec2::new(-u.y * ux, u.x);
        let (sheath, core, spark) = match tone {
            JetTone::VernierCyan => (CYAN_WIRE, Vec3::new(0.92, 1.00, 1.00), Vec3::new(1.00, 0.90, 0.48)),
            JetTone::BellAmber => (AMBER_WIRE, Vec3::new(1.00, 0.96, 0.76), Vec3::new(1.00, 0.98, 0.86)),
            JetTone::FusionBlue => (Vec3::new(0.36, 0.86, 1.00), Vec3::new(0.95, 0.99, 1.00), Vec3::ONE),
        };
        let sp = 0.011 + 0.004 * t;
        target.push_quad(Quad::glowing(
            Vec3::new(p0.x, p0.y + sp, 0.0),
            Vec3::new(p0.x + sp * ux, p0.y, 0.0),
            Vec3::new(p0.x, p0.y - sp, 0.0),
            Vec3::new(p0.x - sp * ux, p0.y, 0.0),
            Vec3::Z,
            spark,
            0.99,
        ));
        target.push_quad(Quad::glowing(
            Vec3::new(p0.x - side.x * base_w, p0.y - side.y * base_w, 0.0),
            Vec3::new(p0.x + side.x * base_w, p0.y + side.y * base_w, 0.0),
            Vec3::new(tip.x + side.x * base_w * 0.10, tip.y + side.y * base_w * 0.10, 0.0),
            Vec3::new(tip.x - side.x * base_w * 0.10, tip.y - side.y * base_w * 0.10, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            sheath,
            -0.88,
        ));
        let ctip = p0.lerp(tip, 0.70);
        let cw = base_w * 0.48;
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
}
