use glam::Vec3;
use world::{MeshTarget, Quad};

/// Sculpted 3D low-poly house sparrow (*Passer domesticus*) figure with articulated wings and pecking posture.
#[derive(Clone, Copy, Debug)]
pub struct SparrowFigure {
    position: Vec3,
    axes: (Vec3, Vec3, Vec3),
    posture: (f32, f32, bool),
}

impl SparrowFigure {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(position: Vec3, axes: (Vec3, Vec3, Vec3), posture: (f32, f32, bool)) -> Self {
        Self { position, axes, posture }
    }

    /// Secondary constructor aligning the sparrow figure with surface up, forward heading, body pitch, and wing state.
    pub fn aligned(position: Vec3, up: Vec3, forward: Vec3, pitch_down: f32, wing_flap: f32, airborne: bool) -> Self {
        let n_up = up.normalize_or_zero();
        let flat_fwd = (forward - n_up * forward.dot(n_up)).normalize_or_zero();
        let fwd = if flat_fwd.length_squared() > 0.25 {
            flat_fwd
        } else {
            let fallback = if n_up.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
            n_up.cross(fallback).normalize_or_zero()
        };
        let right = fwd.cross(n_up).normalize_or_zero();
        Self::new(position, (right, n_up, fwd), (pitch_down, wing_flap, airborne))
    }

    fn body_point(&self, x: f32, y: f32, z: f32) -> Vec3 {
        let (right, up, fwd) = self.axes;
        let (pitch, _, _) = self.posture;
        let (sp, cp) = pitch.sin_cos();
        let py = y - 0.085;
        let ry = 0.085 + py * cp - z * sp;
        let rz = py * sp + z * cp;
        self.position + right * x + up * ry + fwd * rz
    }

    fn push_oriented_quad(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, fallback_n: Vec3, color: Vec3, target: &mut dyn MeshTarget) {
        let cross = (v1 - v0).cross(v3 - v0);
        let normal = if cross.length_squared() > 1e-8 {
            cross.normalize()
        } else {
            fallback_n.normalize_or_zero()
        };
        target.push_quad(Quad::new(v0, v1, v2, v3, normal, color));
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_tapered_prism(
        &self,
        z0: f32,
        y0: f32,
        wx0: f32,
        hy0: f32,
        z1: f32,
        y1: f32,
        wx1: f32,
        hy1: f32,
        top_col: Vec3,
        side_col: Vec3,
        belly_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let (right, up, fwd) = self.axes;
        let b_tl = self.body_point(-wx0, y0 + hy0, z0);
        let b_tr = self.body_point(wx0, y0 + hy0, z0);
        let b_bl = self.body_point(-wx0, y0 - hy0, z0);
        let b_br = self.body_point(wx0, y0 - hy0, z0);
        let f_tl = self.body_point(-wx1, y1 + hy1, z1);
        let f_tr = self.body_point(wx1, y1 + hy1, z1);
        let f_bl = self.body_point(-wx1, y1 - hy1, z1);
        let f_br = self.body_point(wx1, y1 - hy1, z1);

        Self::push_oriented_quad(b_tl, b_tr, f_tr, f_tl, up, top_col, target);
        Self::push_oriented_quad(f_bl, f_br, b_br, b_bl, -up, belly_col, target);
        Self::push_oriented_quad(b_bl, b_tl, f_tl, f_bl, -right, side_col, target);
        Self::push_oriented_quad(f_br, f_tr, b_tr, b_br, right, side_col, target);
        Self::push_oriented_quad(b_br, b_tr, b_tl, b_bl, -fwd, side_col, target);
        Self::push_oriented_quad(f_bl, f_tl, f_tr, f_br, fwd, top_col, target);
    }

    fn emit_wing(&self, side: f32, chestnut: Vec3, wing_bar: Vec3, target: &mut dyn MeshTarget) {
        let (right, up, _) = self.axes;
        let (_, flap, airborne) = self.posture;
        if airborne {
            let (sf, cf) = flap.sin_cos();
            let span = 0.22;
            let tip_x = side * (0.055 + span * cf);
            let tip_y = 0.115 + span * sf;
            let mid_x = side * (0.055 + span * 0.52 * cf);
            let mid_y = 0.115 + span * 0.52 * sf;

            let r_f = self.body_point(side * 0.052, 0.118, 0.045);
            let r_b = self.body_point(side * 0.048, 0.110, -0.055);
            let m_f = self.body_point(mid_x, mid_y + 0.008, 0.038);
            let m_b = self.body_point(mid_x, mid_y, -0.048);
            let t_f = self.body_point(tip_x, tip_y, 0.015);
            let t_b = self.body_point(tip_x * 0.92, tip_y - 0.006, -0.035);

            Self::push_oriented_quad(r_f, m_f, m_b, r_b, up, chestnut, target);
            Self::push_oriented_quad(r_b, m_b, m_f, r_f, -up, wing_bar * 0.88, target);
            Self::push_oriented_quad(m_f, t_f, t_b, m_b, up, wing_bar, target);
            Self::push_oriented_quad(m_b, t_b, t_f, m_f, -up, chestnut * 0.92, target);
        } else {
            let flutter = flap.sin().abs() * 0.014;
            let sx0 = side * 0.054;
            let sx1 = side * (0.072 + flutter);
            let w_f = self.body_point(sx0, 0.118, 0.042);
            let w_b = self.body_point(sx0 * 0.85, 0.095, -0.075);
            let w_top = self.body_point(sx1, 0.126 + flutter, -0.010);
            let w_low = self.body_point(sx1, 0.078, -0.020);
            Self::push_oriented_quad(w_f, w_top, w_b, w_low, right * side, chestnut, target);
            Self::push_oriented_quad(w_low, w_b, w_top, w_f, -right * side, wing_bar, target);
        }
    }

    fn emit_legs(&self, leg_col: Vec3, target: &mut dyn MeshTarget) {
        let (right, up, fwd) = self.axes;
        let (_, _, airborne) = self.posture;
        if airborne {
            return;
        }
        for side in [-1.0_f32, 1.0_f32] {
            let x = side * 0.028;
            let hip = self.position + right * x + up * 0.078 - fwd * 0.008;
            let foot = self.position + right * x + up * 0.004 + fwd * 0.012;
            let toe = self.position + right * x + up * 0.004 + fwd * 0.044;
            let dx = right * 0.007;
            Self::push_oriented_quad(hip - dx, hip + dx, foot + dx, foot - dx, fwd, leg_col, target);
            Self::push_oriented_quad(foot - dx, foot + dx, toe + dx, toe - dx, up, leg_col, target);
        }
    }

    /// Emits the complete faceted 3D sparrow mesh into `target`.
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        let chestnut = Vec3::new(0.48, 0.31, 0.19);
        let flank_brown = Vec3::new(0.56, 0.42, 0.30);
        let buff_belly = Vec3::new(0.76, 0.72, 0.65);
        let crown_grey = Vec3::new(0.42, 0.36, 0.31);
        let cheek_pale = Vec3::new(0.84, 0.80, 0.73);
        let dark_bib = Vec3::new(0.16, 0.14, 0.13);
        let wing_bar = Vec3::new(0.82, 0.74, 0.62);
        let beak_amber = Vec3::new(0.24, 0.20, 0.16);
        let tail_brown = Vec3::new(0.32, 0.22, 0.15);
        let leg_ochre = Vec3::new(0.52, 0.36, 0.24);

        // Plump torso (rump to breast)
        self.emit_tapered_prism(-0.072, 0.095, 0.046, 0.036, 0.058, 0.105, 0.054, 0.044, chestnut, flank_brown, buff_belly, target);
        // Head, grey-brown crown, pale cheeks, and black throat bib
        self.emit_tapered_prism(0.048, 0.125, 0.042, 0.034, 0.118, 0.138, 0.034, 0.028, crown_grey, cheek_pale, dark_bib, target);
        // Conical seed-cracking beak
        self.emit_tapered_prism(0.118, 0.136, 0.018, 0.015, 0.166, 0.130, 0.004, 0.004, beak_amber, beak_amber, beak_amber, target);
        // Notched tail fan
        self.emit_tapered_prism(-0.165, 0.118, 0.036, 0.008, -0.068, 0.098, 0.026, 0.014, tail_brown, tail_brown, tail_brown, target);

        // Articulated left and right wings
        self.emit_wing(-1.0, chestnut, wing_bar, target);
        self.emit_wing(1.0, chestnut, wing_bar, target);
        // Slender legs and toes when grounded or perched
        self.emit_legs(leg_ochre, target);
    }
}
