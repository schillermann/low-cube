use glam::Vec3;
use crate::{MeshTarget, Quad};

/// Cockpit and walker cloud-immersion fog veil surrounding `eye` when inside a 3D cloud (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct CloudVeil {
    eye: Vec3,
    density_and_dark: (f32, f32),
}

impl CloudVeil {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(eye: Vec3, density_and_dark: (f32, f32)) -> Self {
        Self { eye, density_and_dark }
    }

    pub fn emit(self, up: Vec3, target: &mut dyn MeshTarget) {
        let (density, dark) = self.density_and_dark;
        if density <= 0.02 {
            return;
        }
        let col = Vec3::new(0.96, 0.98, 1.00).lerp(Vec3::new(0.40, 0.46, 0.56), dark);
        let alpha = (density * 0.46).clamp(0.02, 0.50);
        let glow = -8.0 - alpha;
        let r = 0.065;
        let c = self.eye;
        let [x, y, z] = [Vec3::X * r, Vec3::Y * r, Vec3::Z * r];
        let [b0, b1, b2, b3] = [c - x - y - z, c + x - y - z, c + x - y + z, c - x - y + z];
        let [p0, p1, p2, p3] = [c - x + y - z, c + x + y - z, c + x + y + z, c - x + y + z];
        let n = up.normalize_or_zero();
        target.push_quad(Quad::glowing(p0, p3, p2, p1, n, col, glow));
        target.push_quad(Quad::glowing(b0, b1, b2, b3, n, col * 0.94, glow));
        target.push_quad(Quad::glowing(b0, p0, p1, b1, n, col * 0.97, glow));
        target.push_quad(Quad::glowing(p3, b3, b2, p2, n, col * 0.97, glow));
        target.push_quad(Quad::glowing(b3, p3, p0, b0, n, col * 0.97, glow));
        target.push_quad(Quad::glowing(b1, p1, p2, b2, n, col * 0.97, glow));
    }
}

/// Zero-allocation toroidal near-camera raindrops and soft Arctic snowflakes (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct LocalDownpour {
    eye: Vec3,
    axes: (Vec3, Vec3, Vec3),
    weather: (f32, f32, bool),
}

impl LocalDownpour {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(eye: Vec3, axes: (Vec3, Vec3, Vec3), weather: (f32, f32, bool)) -> Self {
        Self { eye, axes, weather }
    }

    fn streak_noise(self, index: u32, salt: u32) -> f32 {
        let _ = self.eye;
        let mut h = index.wrapping_mul(0x9E37_79B9).wrapping_add(salt);
        h ^= h >> 16;
        h = h.wrapping_mul(0x7FEB_352D);
        h ^= h >> 15;
        (h >> 8) as f32 / 16_777_215.0
    }

    pub fn emit(self, target: &mut dyn MeshTarget) {
        let (intensity, epoch, frozen) = self.weather;
        if intensity <= 0.04 {
            return;
        }
        let (tang, bitang, up) = self.axes;
        let count = ((14.0 + 34.0 * intensity) as u32).clamp(10, 48);
        let box_w = 12.0_f32;
        let box_h = 10.0_f32;
        let base_u = self.eye.dot(tang);
        let base_v = self.eye.dot(bitang);
        let base_y = self.eye.dot(up);

        for i in 0..count {
            let rx = self.streak_noise(i, 0x1357_9BDF) * box_w;
            let rz = self.streak_noise(i, 0x2468_ACE0) * box_w;
            let ry = self.streak_noise(i, 0xFEDC_BA98) * box_h;
            let fall_speed = if frozen { 2.0 } else { 14.2 };
            let wind_u = if frozen {
                (epoch * 1.4 + (i as f32) * 0.9).sin() * 0.45 + epoch * 0.60
            } else {
                epoch * 1.5
            };
            let du = (rx + wind_u - base_u).rem_euclid(box_w) - box_w * 0.5;
            let dv = (rz + epoch * 0.45 - base_v).rem_euclid(box_w) - box_w * 0.5;
            let dy = (ry - epoch * fall_speed - base_y).rem_euclid(box_h) - box_h * 0.44;
            if du * du + dv * dv < 0.25 {
                continue;
            }
            let pos = self.eye + tang * du + bitang * dv + up * dy;

            if frozen {
                let fwd = (self.eye - pos).normalize_or_zero();
                let ref_ax = if fwd.dot(tang).abs() < 0.88 { tang } else { bitang };
                let right_s = ref_ax.cross(fwd).normalize_or_zero();
                let up_s = fwd.cross(right_s).normalize_or_zero();
                let r = 0.032;
                let col = Vec3::new(0.94, 0.97, 1.00);
                let glow = -6.0 - (0.34 + 0.36 * intensity).clamp(0.28, 0.72);
                target.push_quad(Quad::glowing(
                    pos - right_s * r - up_s * r,
                    pos + right_s * r - up_s * r,
                    pos + right_s * r + up_s * r,
                    pos - right_s * r + up_s * r,
                    up,
                    col,
                    glow,
                ));
            } else {
                let side = (tang * (-dv) + bitang * du).normalize_or_zero();
                let half_w = 0.014;
                let half_len = 0.28;
                let slant = tang * 0.04;
                let col = Vec3::new(0.70, 0.80, 0.94);
                let glow = -6.0 - (0.20 + 0.30 * intensity).clamp(0.16, 0.52);
                target.push_quad(Quad::glowing(
                    pos - side * half_w - up * half_len - slant,
                    pos + side * half_w - up * half_len - slant,
                    pos + side * half_w + up * half_len + slant,
                    pos - side * half_w + up * half_len + slant,
                    up,
                    col,
                    glow,
                ));
            }
        }
    }
}
