use std::f32::consts::TAU;
use glam::Vec3;

/// Compact `128 × 64` (`32 KB` `Rgba8Unorm`) sky lookup texture width.
pub const SKY_LUT_WIDTH: u32 = 128;

/// Compact `128 × 64` (`32 KB` `Rgba8Unorm`) sky lookup texture height.
pub const SKY_LUT_HEIGHT: u32 = 64;

/// `2 Hz` (`0.5 s`) Atmosphere Sky-LUT refresh interval replacing per-pixel Rayleigh raymarching.
pub const SKY_LUT_INTERVAL_SECS: f32 = 0.5;

const LUT_BYTE_LEN: usize = (SKY_LUT_WIDTH as usize) * (SKY_LUT_HEIGHT as usize) * 4;

/// Focused contract evaluating sky scatter RGB + transmittance along a unit view ray `ray_dir`.
pub trait SkyLutMedium {
    fn sample_sky_ray(&self, ray_dir: Vec3) -> [f32; 4];
}

/// `128 × 64` Atmosphere Sky-LUT refreshed at `2 Hz` (`0.5 s` cadence, `4` private fields).
pub struct AtmosphereSkyLut {
    texels: Vec<u8>,
    last_refresh_secs: f32,
    signature: [u32; 4],
    primed: bool,
}

impl AtmosphereSkyLut {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(texels: Vec<u8>, last_refresh_secs: f32, signature: [u32; 4], primed: bool) -> Self {
        Self { texels, last_refresh_secs, signature, primed }
    }

    pub fn cold() -> Self {
        Self::new(vec![0u8; LUT_BYTE_LEN], -10.0, [0; 4], false)
    }

    pub fn texel_bytes(&self) -> &[u8] {
        &self.texels
    }

    pub const fn dimensions(&self) -> (u32, u32) {
        let _ = self;
        (SKY_LUT_WIDTH, SKY_LUT_HEIGHT)
    }

    /// Recomputes the `128 × 64` LUT only when unprimed, `>= 0.5 s` (`2 Hz`) has elapsed, or planet signature shifted.
    pub fn refresh_at_2hz(
        &mut self,
        elapsed_secs: f32,
        medium_signature: [u32; 4],
        up_axis: Vec3,
        medium: &dyn SkyLutMedium,
    ) -> bool {
        let sig_changed = self.signature != medium_signature;
        let due = !self.primed
            || sig_changed
            || (elapsed_secs - self.last_refresh_secs).abs() >= SKY_LUT_INTERVAL_SECS;
        if !due {
            return false;
        }
        self.primed = true;
        self.last_refresh_secs = elapsed_secs;
        self.signature = medium_signature;
        self.bake_spherical(up_axis, medium);
        true
    }

    fn bake_spherical(&mut self, up_axis: Vec3, medium: &dyn SkyLutMedium) {
        let up = up_axis.try_normalize().unwrap_or(Vec3::Y);
        let ref_axis = if up.y.abs() > 0.92 { Vec3::X } else { Vec3::Y };
        let tangent = ref_axis.cross(up).normalize_or_zero();
        let bitangent = up.cross(tangent).normalize_or_zero();
        for row in 0..SKY_LUT_HEIGHT {
            // Non-linear horizon concentration (`v` near 0.5 packs extra precision along sunset rim)
            let v = ((row as f32) + 0.5) / (SKY_LUT_HEIGHT as f32);
            let signed = v * 2.0 - 1.0;
            let sin_el = (signed * signed.abs()).clamp(-1.0, 1.0);
            let cos_el = (1.0 - sin_el * sin_el).max(0.0).sqrt();
            for col in 0..SKY_LUT_WIDTH {
                let u = ((col as f32) + 0.5) / (SKY_LUT_WIDTH as f32);
                let az = u * TAU;
                let (sin_az, cos_az) = az.sin_cos();
                let dir = (tangent * (cos_el * cos_az) + bitangent * (cos_el * sin_az) + up * sin_el)
                    .normalize_or_zero();
                let rgba = medium.sample_sky_ray(dir);
                let idx = ((row * SKY_LUT_WIDTH + col) as usize) * 4;
                self.texels[idx] = (rgba[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                self.texels[idx + 1] = (rgba[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                self.texels[idx + 2] = (rgba[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                self.texels[idx + 3] = (rgba[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
        }
    }
}
