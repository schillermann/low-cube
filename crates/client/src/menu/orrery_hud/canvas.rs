use std::f32::consts::TAU;
use glam::{Vec2, Vec3};
use world::{MeshTarget, Quad};

/// Tilted 3D holographic viewport projecting Steuerkonsolen-Orrery wireframes into the Nose-Camera HUD.
#[derive(Clone, Copy, Debug)]
pub(super) struct OrreryCanvas {
    center: (f32, f32),
    scale: f32,
    inv_aspect: f32,
}

impl OrreryCanvas {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(center: (f32, f32), scale: f32, inv_aspect: f32) -> Self {
        Self { center, scale, inv_aspect }
    }

    pub(super) fn project(self, local: Vec3) -> Vec2 {
        let (sin_p, cos_p) = 0.44_f32.sin_cos();
        let rx = local.x;
        let ry = local.y * cos_p - local.z * sin_p;
        let rz = local.y * sin_p + local.z * cos_p;
        let persp = 2.45 / (2.45 - rz).clamp(1.15, 3.80);
        Vec2::new(
            self.center.0 + rx * persp * self.scale * self.inv_aspect,
            self.center.1 + ry * persp * self.scale,
        )
    }

    pub(super) fn emit_beam_2d(
        self,
        a: Vec2,
        b: Vec2,
        half_w: f32,
        color: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        let ux = self.inv_aspect.max(0.25);
        let metric = Vec2::new((b.x - a.x) / ux, b.y - a.y);
        let len = metric.length();
        if len <= 1e-5 {
            return;
        }
        let dir = metric / len;
        let perp = Vec2::new(-dir.y * half_w * ux, dir.x * half_w);
        let n = if glow < 0.0 { Vec3::new(0.0, 1.0, 1.0) } else { Vec3::Z };
        target.push_quad(Quad::glowing(
            Vec3::new(a.x - perp.x, a.y - perp.y, 0.0),
            Vec3::new(b.x - perp.x, b.y - perp.y, 0.0),
            Vec3::new(b.x + perp.x, b.y + perp.y, 0.0),
            Vec3::new(a.x + perp.x, a.y + perp.y, 0.0),
            n,
            color,
            glow,
        ));
    }

    pub(super) fn emit_strut_3d(
        self,
        a: Vec3,
        b: Vec3,
        half_w: f32,
        color: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        self.emit_beam_2d(self.project(a), self.project(b), half_w, color, glow, target);
    }

    pub(super) fn emit_holo_panel(
        self,
        pts: [Vec3; 4],
        color: Vec3,
        alpha: f32,
        target: &mut dyn MeshTarget,
    ) {
        let [p0, p1, p2, p3] = pts.map(|p| self.project(p));
        target.push_quad(Quad::glowing(
            Vec3::new(p0.x, p0.y, 0.0),
            Vec3::new(p1.x, p1.y, 0.0),
            Vec3::new(p2.x, p2.y, 0.0),
            Vec3::new(p3.x, p3.y, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            color,
            -alpha.clamp(0.02, 0.95),
        ));
    }

    pub(super) fn emit_projector_base(self, target: &mut dyn MeshTarget) {
        let ring_y = -0.50;
        let teal = Vec3::new(0.16, 0.92, 0.80);
        let dim_teal = Vec3::new(0.10, 0.62, 0.68);
        let segs = 8usize;
        for i in 0..segs {
            let a0 = (i as f32) * TAU / (segs as f32) + 0.3927;
            let a1 = ((i + 1) as f32) * TAU / (segs as f32) + 0.3927;
            let (s0, c0) = a0.sin_cos();
            let (s1, c1) = a1.sin_cos();
            let outer0 = Vec3::new(c0 * 0.86, ring_y, s0 * 0.86);
            let outer1 = Vec3::new(c1 * 0.86, ring_y, s1 * 0.86);
            let inner0 = Vec3::new(c0 * 0.72, ring_y + 0.015, s0 * 0.72);
            let inner1 = Vec3::new(c1 * 0.72, ring_y + 0.015, s1 * 0.72);
            self.emit_holo_panel([inner0, outer0, outer1, inner1], dim_teal, 0.16, target);
            let cone0 = Vec3::new(c0 * 0.94, ring_y + 0.42, s0 * 0.94);
            let cone1 = Vec3::new(c1 * 0.94, ring_y + 0.42, s1 * 0.94);
            self.emit_holo_panel([outer0, outer1, cone1, cone0], dim_teal, 0.05, target);
            self.emit_strut_3d(outer0, outer1, 0.0030, teal, 0.96, target);
            self.emit_strut_3d(inner0, inner1, 0.0018, dim_teal, 0.88, target);
        }
    }

    pub(super) fn emit_wire_cube(
        self,
        center: Vec3,
        half: f32,
        edge_col: Vec3,
        fill_alpha: f32,
        target: &mut dyn MeshTarget,
    ) {
        let c = [
            Vec3::new(-half, -half, -half),
            Vec3::new(half, -half, -half),
            Vec3::new(half, -half, half),
            Vec3::new(-half, -half, half),
            Vec3::new(-half, half, -half),
            Vec3::new(half, half, -half),
            Vec3::new(half, half, half),
            Vec3::new(-half, half, half),
        ]
        .map(|o| center + o);
        for face in [[c[3], c[2], c[6], c[7]], [c[4], c[5], c[6], c[7]], [c[1], c[2], c[6], c[5]]] {
            self.emit_holo_panel(face, edge_col, fill_alpha, target);
        }
        let strut_w = (half * 0.20).clamp(0.0008, 0.0022);
        for (i, j) in [
            (0, 1), (1, 2), (2, 3), (3, 0),
            (4, 5), (5, 6), (6, 7), (7, 4),
            (0, 4), (1, 5), (2, 6), (3, 7),
        ] {
            self.emit_strut_3d(c[i], c[j], strut_w, edge_col, 0.96, target);
        }
    }

    pub(super) fn emit_gravity_shell(self, center: Vec3, radius: f32, dashes: usize, col: Vec3, target: &mut dyn MeshTarget) {
        let total = dashes * 2;
        for i in (0..total).step_by(2) {
            let a0 = (i as f32) * TAU / (total as f32);
            let a1 = ((i + 1) as f32) * TAU / (total as f32);
            let p0 = center + Vec3::new(a0.cos() * radius, 0.0, a0.sin() * radius);
            let p1 = center + Vec3::new(a1.cos() * radius, 0.0, a1.sin() * radius);
            self.emit_strut_3d(p0, p1, 0.0014, col, -0.42, target);
        }
    }

    pub(super) fn emit_pyramid_frame(
        self,
        pos: Vec3,
        nose_dir: Vec3,
        flight_dir: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let fwd = if nose_dir.length_squared() > 0.1 { nose_dir.normalize() } else { Vec3::X };
        let ref_up = if fwd.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
        let side = fwd.cross(ref_up).normalize_or_zero();
        let dorsal = side.cross(fwd).normalize_or_zero();
        let edge_gold = Vec3::new(1.00, 0.88, 0.28);
        let base_gold = Vec3::new(0.96, 0.74, 0.20);
        let magenta = Vec3::new(0.98, 0.36, 0.80);

        let tip = pos + fwd * 0.088;
        let tail = pos - fwd * 0.042;
        let b0 = tail + dorsal * 0.036;
        let b1 = tail - dorsal * 0.020 + side * 0.034;
        let b2 = tail - dorsal * 0.020 - side * 0.034;

        for face in [[b0, b1, tip, tip], [b1, b2, tip, tip], [b2, b0, tip, tip], [b0, b2, b1, b1]] {
            self.emit_holo_panel(face, edge_gold, 0.16, target);
        }
        for rear in [b0, b1, b2] {
            self.emit_strut_3d(tip, rear, 0.0023, edge_gold, 0.98, target);
        }
        for (a, b) in [(b0, b1), (b1, b2), (b2, b0)] {
            self.emit_strut_3d(a, b, 0.0020, base_gold, 0.96, target);
        }

        let vec_dir = if flight_dir.length_squared() > 0.1 { flight_dir.normalize() } else { fwd };
        if vec_dir.dot(fwd) < 0.992 {
            self.emit_strut_3d(tip, tip + vec_dir * 0.105, 0.0020, magenta, 0.98, target);
        }
    }
}
