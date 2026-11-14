use glam::Vec3;
use world::{MeshTarget, Quad};

/// Sculpted 3D low-poly European wild boar (*Sus scrofa*) with bristled dorsal crest, snout disc, and curved ivory tusks.
#[derive(Clone, Copy, Debug)]
pub struct BoarFigure {
    position: Vec3,
    axes: (Vec3, Vec3, Vec3),
    kinematics: (f32, f32, f32),
    hit_flash: f32,
}

impl BoarFigure {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        position: Vec3,
        axes: (Vec3, Vec3, Vec3),
        kinematics: (f32, f32, f32),
        hit_flash: f32,
    ) -> Self {
        Self {
            position,
            axes,
            kinematics,
            hit_flash,
        }
    }

    /// Secondary constructor aligning the wild boar with surface up, forward heading, snout pitch, stride phase, bristle lift, and hit flash.
    pub fn aligned(
        position: Vec3,
        up: Vec3,
        forward: Vec3,
        head_pitch: f32,
        stride_phase: f32,
        bristle_lift: f32,
        hit_flash: f32,
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
        Self::new(position, (right, n_up, fwd), (head_pitch, stride_phase, bristle_lift), hit_flash)
    }

    fn tint(&self, col: Vec3) -> Vec3 {
        let f = self.hit_flash.clamp(0.0, 1.0);
        col * (1.0 - 0.55 * f) + Vec3::new(0.95, 0.18, 0.14) * (0.55 * f)
    }

    fn torso_point(&self, x: f32, y: f32, z: f32) -> Vec3 {
        let (right, up, fwd) = self.axes;
        self.position + right * x + up * y + fwd * z
    }

    fn head_point(&self, x: f32, dy: f32, dz: f32) -> Vec3 {
        let (head_pitch, _, _) = self.kinematics;
        let (sp, cp) = head_pitch.sin_cos();
        let y = 0.44 + dy * cp - dz * sp;
        let z = 0.34 + dy * sp + dz * cp;
        self.torso_point(x, y, z)
    }

    fn push_oriented_quad(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, fallback_n: Vec3, color: Vec3, target: &mut dyn MeshTarget) {
        let cross = (v1 - v0).cross(v3 - v0);
        let normal = if cross.length_squared() > 1e-8 { cross.normalize() } else { fallback_n.normalize_or_zero() };
        target.push_quad(Quad::new(v0, v1, v2, v3, normal, color));
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_box(
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

    fn head_ring(&self, dz: f32, dy: f32, wx: f32, hy: f32) -> [Vec3; 4] {
        [
            self.head_point(-wx, dy + hy, dz),
            self.head_point(wx, dy + hy, dz),
            self.head_point(-wx, dy - hy, dz),
            self.head_point(wx, dy - hy, dz),
        ]
    }

    fn emit_tusks_and_ears(&self, coat: Vec3, bristle: Vec3, ivory: Vec3, target: &mut dyn MeshTarget) {
        for side in [-1.0_f32, 1.0_f32] {
            let ear_base = [
                self.head_point(side * 0.088 - 0.016, 0.14, 0.03),
                self.head_point(side * 0.088 + 0.016, 0.14, 0.03),
                self.head_point(side * 0.084 - 0.014, 0.10, 0.01),
                self.head_point(side * 0.084 + 0.014, 0.10, 0.01),
            ];
            let ear_tip = [
                self.head_point(side * 0.155 - 0.010, 0.23, 0.09),
                self.head_point(side * 0.155 + 0.010, 0.23, 0.09),
                self.head_point(side * 0.148 - 0.010, 0.20, 0.07),
                self.head_point(side * 0.148 + 0.010, 0.20, 0.07),
            ];
            self.emit_box(ear_base, ear_tip, bristle, coat, coat, target);

            let tusk_root = [
                self.head_point(side * 0.068 - 0.011, -0.045, 0.24),
                self.head_point(side * 0.068 + 0.011, -0.045, 0.24),
                self.head_point(side * 0.068 - 0.011, -0.068, 0.23),
                self.head_point(side * 0.068 + 0.011, -0.068, 0.23),
            ];
            let tusk_mid = [
                self.head_point(side * 0.118 - 0.008, 0.005, 0.29),
                self.head_point(side * 0.118 + 0.008, 0.005, 0.29),
                self.head_point(side * 0.118 - 0.008, -0.014, 0.28),
                self.head_point(side * 0.118 + 0.008, -0.014, 0.28),
            ];
            let tusk_tip = [
                self.head_point(side * 0.136 - 0.004, 0.075, 0.27),
                self.head_point(side * 0.136 + 0.004, 0.075, 0.27),
                self.head_point(side * 0.136 - 0.004, 0.062, 0.26),
                self.head_point(side * 0.136 + 0.004, 0.062, 0.26),
            ];
            self.emit_box(tusk_root, tusk_mid, ivory, ivory, ivory, target);
            self.emit_box(tusk_mid, tusk_tip, ivory, ivory, ivory, target);
        }
    }

    fn emit_legs(&self, flank: Vec3, shank: Vec3, hoof: Vec3, target: &mut dyn MeshTarget) {
        let (_, stride_phase, _) = self.kinematics;
        for (hip_x, hip_z, phase_off) in [
            (-0.135_f32, 0.23_f32, 0.0_f32),
            (0.135, 0.23, std::f32::consts::PI),
            (-0.118, -0.26, std::f32::consts::PI),
            (0.118, -0.26, 0.0),
        ] {
            let cyc = stride_phase + phase_off;
            let swing = cyc.sin() * 0.095;
            let lift = cyc.cos().max(0.0) * 0.055;
            let top_leg = [
                self.torso_point(hip_x - 0.042, 0.34, hip_z - 0.042),
                self.torso_point(hip_x + 0.042, 0.34, hip_z - 0.042),
                self.torso_point(hip_x - 0.042, 0.34, hip_z + 0.042),
                self.torso_point(hip_x + 0.042, 0.34, hip_z + 0.042),
            ];
            let knee_z = hip_z + swing * 0.55;
            let knee = [
                self.torso_point(hip_x - 0.030, 0.16 + lift * 0.5, knee_z - 0.030),
                self.torso_point(hip_x + 0.030, 0.16 + lift * 0.5, knee_z - 0.030),
                self.torso_point(hip_x - 0.030, 0.16 + lift * 0.5, knee_z + 0.030),
                self.torso_point(hip_x + 0.030, 0.16 + lift * 0.5, knee_z + 0.030),
            ];
            let ankle_z = hip_z + swing;
            let ankle = [
                self.torso_point(hip_x - 0.024, 0.045 + lift, ankle_z - 0.024),
                self.torso_point(hip_x + 0.024, 0.045 + lift, ankle_z - 0.024),
                self.torso_point(hip_x - 0.024, 0.045 + lift, ankle_z + 0.024),
                self.torso_point(hip_x + 0.024, 0.045 + lift, ankle_z + 0.024),
            ];
            let sole = [
                self.torso_point(hip_x - 0.026, lift, ankle_z - 0.026),
                self.torso_point(hip_x + 0.026, lift, ankle_z - 0.026),
                self.torso_point(hip_x - 0.026, lift, ankle_z + 0.030),
                self.torso_point(hip_x + 0.026, lift, ankle_z + 0.030),
            ];
            self.emit_box(top_leg, knee, flank, flank, shank, target);
            self.emit_box(knee, ankle, shank, shank, shank, target);
            self.emit_box(ankle, sole, hoof, hoof, hoof, target);
        }
    }

    /// Emits the complete 3D wild boar mesh into `target`.
    pub fn emit(&self, target: &mut dyn MeshTarget) {
        let (_, _, bristle_lift) = self.kinematics;
        let coat = self.tint(Vec3::new(0.24, 0.18, 0.14));
        let grizzle = self.tint(Vec3::new(0.35, 0.29, 0.24));
        let bristle = self.tint(Vec3::new(0.11, 0.09, 0.08));
        let snout_disc = self.tint(Vec3::new(0.38, 0.25, 0.24));
        let hoof = self.tint(Vec3::new(0.09, 0.08, 0.08));
        let ivory = Vec3::new(0.93, 0.90, 0.78);

        let rump = self.torso_ring(-0.42, 0.42, 0.11, 0.11);
        let hip = self.torso_ring(-0.22, 0.46, 0.165, 0.155);
        let rib = self.torso_ring(0.06, 0.49, 0.205, 0.195);
        let shoulder = self.torso_ring(0.32, 0.50, 0.195, 0.210);
        self.emit_box(rump, hip, bristle, coat, coat, target);
        self.emit_box(hip, rib, bristle, grizzle, coat, target);
        self.emit_box(rib, shoulder, bristle, coat, coat, target);

        let crest_h = 0.055 + 0.040 * bristle_lift.clamp(0.0, 1.0);
        let crest_back = self.torso_ring(-0.18, 0.63 + crest_h * 0.5, 0.028, crest_h * 0.5);
        let crest_front = self.torso_ring(0.34, 0.71 + crest_h * 0.6, 0.034, crest_h * 0.6);
        self.emit_box(crest_back, crest_front, bristle, bristle, bristle, target);

        let tail_root = self.torso_ring(-0.42, 0.47, 0.022, 0.022);
        let tail_tip = self.torso_ring(-0.53, 0.34, 0.015, 0.015);
        self.emit_box(tail_root, tail_tip, bristle, bristle, bristle, target);

        let skull = self.head_ring(-0.02, 0.04, 0.145, 0.145);
        let jowl = self.head_ring(0.18, -0.01, 0.105, 0.100);
        let snout = self.head_ring(0.35, -0.05, 0.062, 0.058);
        let disc = self.head_ring(0.40, -0.05, 0.058, 0.054);
        self.emit_box(skull, jowl, bristle, grizzle, coat, target);
        self.emit_box(jowl, snout, coat, coat, coat, target);
        self.emit_box(snout, disc, snout_disc, snout_disc, snout_disc, target);

        self.emit_tusks_and_ears(coat, bristle, ivory, target);
        self.emit_legs(coat, bristle, hoof, target);
    }
}
