use glam::Vec3;
use world::{MeshTarget, Quad};

/// Sculpted 3D low-poly European roe deer (*Capreolus capreolus*) with white rump mirror, ears, hooves, and optional buck antlers.
#[derive(Clone, Copy, Debug)]
pub struct RoeFigure {
    position: Vec3,
    axes: (Vec3, Vec3, Vec3),
    kinematics: (f32, f32, f32),
    antlered: bool,
}

impl RoeFigure {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        position: Vec3,
        axes: (Vec3, Vec3, Vec3),
        kinematics: (f32, f32, f32),
        antlered: bool,
    ) -> Self {
        Self {
            position,
            axes,
            kinematics,
            antlered,
        }
    }

    /// Secondary constructor aligning the roe deer figure with surface up, forward heading, neck pitch, gait phase, and antler state.
    pub fn aligned(
        position: Vec3,
        up: Vec3,
        forward: Vec3,
        neck_pitch: f32,
        stride_phase: f32,
        bound_lift: f32,
        antlered: bool,
    ) -> Self {
        let n_up = up.normalize_or_zero();
        let flat_fwd = (forward - n_up * forward.dot(n_up)).normalize_or_zero();
        let fwd = if flat_fwd.length_squared() > 0.25 {
            flat_fwd
        } else {
            let fallback = if n_up.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
            n_up.cross(fallback).normalize_or_zero()
        };
        let right = fwd.cross(n_up).normalize_or_zero();
        Self::new(position, (right, n_up, fwd), (neck_pitch, stride_phase, bound_lift), antlered)
    }

    fn torso_point(&self, x: f32, y: f32, z: f32) -> Vec3 {
        let (right, up, fwd) = self.axes;
        let (_, _, bound_lift) = self.kinematics;
        let body_tilt = (bound_lift * 0.35).clamp(0.0, 0.24);
        let (st, ct) = body_tilt.sin_cos();
        let py = y - 0.54;
        let ry = 0.54 + py * ct + z * st;
        let rz = z * ct - py * st;
        self.position + right * x + up * ry + fwd * rz
    }

    fn neck_point(&self, x: f32, dy: f32, dz: f32) -> Vec3 {
        let (neck_pitch, _, _) = self.kinematics;
        let (sp, cp) = neck_pitch.sin_cos();
        let y = 0.56 + dy * cp - dz * sp;
        let z = 0.28 + dy * sp + dz * cp;
        self.torso_point(x, y, z)
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
    fn emit_box_segment(
        &self,
        b: [Vec3; 4],
        f: [Vec3; 4],
        top_col: Vec3,
        side_col: Vec3,
        belly_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let (right, up, fwd) = self.axes;
        Self::push_oriented_quad(b[0], b[1], f[1], f[0], up, top_col, target);
        Self::push_oriented_quad(f[2], f[3], b[3], b[2], -up, belly_col, target);
        Self::push_oriented_quad(b[2], b[0], f[0], f[2], -right, side_col, target);
        Self::push_oriented_quad(f[3], f[1], b[1], b[3], right, side_col, target);
        Self::push_oriented_quad(b[3], b[1], b[0], b[2], -fwd, side_col, target);
        Self::push_oriented_quad(f[2], f[0], f[1], f[3], fwd, top_col, target);
    }

    fn torso_ring(&self, z: f32, y: f32, wx: f32, hy: f32) -> [Vec3; 4] {
        [
            self.torso_point(-wx, y + hy, z),
            self.torso_point(wx, y + hy, z),
            self.torso_point(-wx, y - hy, z),
            self.torso_point(wx, y - hy, z),
        ]
    }

    fn neck_ring(&self, dz: f32, dy: f32, wx: f32, hy: f32) -> [Vec3; 4] {
        [
            self.neck_point(-wx, dy + hy, dz),
            self.neck_point(wx, dy + hy, dz),
            self.neck_point(-wx, dy - hy, dz),
            self.neck_point(wx, dy - hy, dz),
        ]
    }

    fn emit_ears_and_antlers(&self, coat: Vec3, inner_ear: Vec3, antler_col: Vec3, tine_col: Vec3, target: &mut dyn MeshTarget) {
        for side in [-1.0_f32, 1.0_f32] {
            let ear_root = [
                self.neck_point(side * 0.042 - 0.014, 0.39, 0.12),
                self.neck_point(side * 0.042 + 0.014, 0.39, 0.12),
                self.neck_point(side * 0.042 - 0.012, 0.36, 0.10),
                self.neck_point(side * 0.042 + 0.012, 0.36, 0.10),
            ];
            let ear_tip = [
                self.neck_point(side * 0.118 - 0.008, 0.50, 0.07),
                self.neck_point(side * 0.118 + 0.008, 0.50, 0.07),
                self.neck_point(side * 0.112 - 0.008, 0.47, 0.06),
                self.neck_point(side * 0.112 + 0.008, 0.47, 0.06),
            ];
            self.emit_box_segment(ear_root, ear_tip, coat, coat, inner_ear, target);

            if self.antlered {
                let pedicle = [
                    self.neck_point(side * 0.028 - 0.010, 0.40, 0.16),
                    self.neck_point(side * 0.028 + 0.010, 0.40, 0.16),
                    self.neck_point(side * 0.028 - 0.010, 0.38, 0.15),
                    self.neck_point(side * 0.028 + 0.010, 0.38, 0.15),
                ];
                let mid_beam = [
                    self.neck_point(side * 0.056 - 0.008, 0.50, 0.15),
                    self.neck_point(side * 0.056 + 0.008, 0.50, 0.15),
                    self.neck_point(side * 0.056 - 0.008, 0.48, 0.14),
                    self.neck_point(side * 0.056 + 0.008, 0.48, 0.14),
                ];
                let crown_tine = [
                    self.neck_point(side * 0.074 - 0.005, 0.59, 0.12),
                    self.neck_point(side * 0.074 + 0.005, 0.59, 0.12),
                    self.neck_point(side * 0.074 - 0.005, 0.58, 0.11),
                    self.neck_point(side * 0.074 + 0.005, 0.58, 0.11),
                ];
                let brow_tine = [
                    self.neck_point(side * 0.052 - 0.005, 0.54, 0.21),
                    self.neck_point(side * 0.052 + 0.005, 0.54, 0.21),
                    self.neck_point(side * 0.052 - 0.005, 0.53, 0.20),
                    self.neck_point(side * 0.052 + 0.005, 0.53, 0.20),
                ];
                self.emit_box_segment(pedicle, mid_beam, antler_col, antler_col, antler_col, target);
                self.emit_box_segment(mid_beam, crown_tine, tine_col, antler_col, antler_col, target);
                self.emit_box_segment(mid_beam, brow_tine, tine_col, tine_col, antler_col, target);
            }
        }
    }

    fn emit_legs(&self, upper_col: Vec3, lower_col: Vec3, hoof_col: Vec3, target: &mut dyn MeshTarget) {
        let (_, stride_phase, bound_lift) = self.kinematics;
        let limbs = [
            (-0.095_f32, 0.24_f32, 0.0_f32, true),
            (0.095, 0.24, std::f32::consts::PI, true),
            (-0.100, -0.28, std::f32::consts::PI, false),
            (0.100, -0.28, 0.0, false),
        ];
        for (lx, lz, offset, forelimb) in limbs {
            let swing = (stride_phase + offset).sin() * 0.13;
            let (knee_dz, hoof_dz, lift) = if bound_lift > 0.02 {
                let reach = if forelimb { 0.15 } else { -0.17 };
                (reach * 0.55, reach, 0.09)
            } else {
                (swing * 0.45, swing, ((stride_phase + offset).cos().max(0.0)) * 0.035 * swing.abs().signum())
            };
            let hip = self.torso_ring(lz, 0.48, 0.030, 0.030);
            let hip_shifted = hip.map(|p| p + self.axes.0 * lx);
            let knee = self.torso_ring(lz + knee_dz, 0.25 + lift * 0.5, 0.020, 0.020).map(|p| p + self.axes.0 * lx);
            let fetlock = self.torso_ring(lz + hoof_dz, 0.045 + lift, 0.015, 0.015).map(|p| p + self.axes.0 * lx);
            let hoof = self.torso_ring(lz + hoof_dz + 0.014, 0.006 + lift, 0.017, 0.008).map(|p| p + self.axes.0 * lx);
            self.emit_box_segment(hip_shifted, knee, upper_col, upper_col, lower_col, target);
            self.emit_box_segment(knee, fetlock, lower_col, lower_col, lower_col, target);
            self.emit_box_segment(fetlock, hoof, hoof_col, hoof_col, hoof_col, target);
        }
    }

    /// Emits the complete faceted 3D European roe deer mesh into `target`.
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        let russet_back = Vec3::new(0.64, 0.35, 0.18);
        let tawny_flank = Vec3::new(0.73, 0.44, 0.24);
        let cream_belly = Vec3::new(0.86, 0.80, 0.70);
        let rump_mirror = Vec3::new(0.95, 0.94, 0.90);
        let muzzle_dark = Vec3::new(0.16, 0.14, 0.13);
        let inner_ear = Vec3::new(0.88, 0.78, 0.72);
        let antler_beam = Vec3::new(0.36, 0.28, 0.20);
        let antler_tine = Vec3::new(0.82, 0.77, 0.66);
        let cannon_tan = Vec3::new(0.66, 0.46, 0.29);
        let hoof_slate = Vec3::new(0.15, 0.14, 0.13);

        // Snow-white rump mirror ("Spiegel") -> haunches -> slender waist -> deep chest
        let mirror = self.torso_ring(-0.42, 0.56, 0.095, 0.075);
        let haunch = self.torso_ring(-0.33, 0.56, 0.132, 0.112);
        let waist = self.torso_ring(-0.04, 0.55, 0.122, 0.105);
        let chest = self.torso_ring(0.31, 0.55, 0.136, 0.120);
        self.emit_box_segment(mirror, haunch, russet_back, rump_mirror, rump_mirror, target);
        self.emit_box_segment(haunch, waist, russet_back, tawny_flank, cream_belly, target);
        self.emit_box_segment(waist, chest, russet_back, tawny_flank, cream_belly, target);

        // Slender articulated neck, roe cranium, and dark nose muzzle
        let neck_base = self.neck_ring(-0.02, 0.01, 0.072, 0.074);
        let neck_top = self.neck_ring(0.10, 0.34, 0.048, 0.048);
        let skull = self.neck_ring(0.19, 0.36, 0.052, 0.044);
        let muzzle = self.neck_ring(0.34, 0.31, 0.026, 0.024);
        let nose_tip = self.neck_ring(0.38, 0.30, 0.018, 0.016);
        self.emit_box_segment(neck_base, neck_top, russet_back, tawny_flank, cream_belly, target);
        self.emit_box_segment(neck_top, skull, russet_back, tawny_flank, cream_belly, target);
        self.emit_box_segment(skull, muzzle, tawny_flank, tawny_flank, cream_belly, target);
        self.emit_box_segment(muzzle, nose_tip, muzzle_dark, muzzle_dark, muzzle_dark, target);

        self.emit_ears_and_antlers(tawny_flank, inner_ear, antler_beam, antler_tine, target);
        self.emit_legs(tawny_flank, cannon_tan, hoof_slate, target);
    }
}
