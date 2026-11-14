use std::f32::consts::PI;
use glam::Vec3;
use super::AcousticWave;

/// Rhythmic three-bound cloven-hoof turf thuds and tall meadow grass rustling emitted by a fleeing roe deer.
#[derive(Clone, Copy, Debug)]
pub struct RoeHoofbeats {
    sample_rate: u32,
    distance_gain: f32,
    stereo_pan: f32,
    pace_scale: f32,
}

impl RoeHoofbeats {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32, distance_gain: f32, stereo_pan: f32, pace_scale: f32) -> Self {
        Self {
            sample_rate,
            distance_gain,
            stereo_pan,
            pace_scale,
        }
    }

    /// Secondary constructor projecting 3D receding hoofbeats and grass swish relative to listener gaze.
    pub fn spatial(deer_pos: Vec3, listener_pos: Vec3, view_dir: Vec3, up: Vec3, seed: u32) -> Self {
        let delta = deer_pos - listener_pos;
        let dist = delta.length();
        let falloff = (1.0 - dist / 40.0).clamp(0.0, 1.0);
        let distance_gain = (falloff * falloff * (22.0 / (22.0 + dist * dist))).clamp(0.04, 0.92);
        let right = view_dir.cross(up).normalize_or_zero();
        let dir = if dist > 1e-4 { delta / dist } else { Vec3::ZERO };
        let stereo_pan = dir.dot(right).clamp(-0.92, 0.92);
        let pace_scale = 0.94 + (((seed % 19) as f32) * 0.007);
        Self::new(24000, distance_gain, stereo_pan, pace_scale)
    }

    /// Left and right stereo channel gains computed from distance attenuation and azimuth angle.
    pub fn channel_gains(&self) -> (f32, f32) {
        let angle = (self.stereo_pan.clamp(-1.0, 1.0) + 1.0) * (PI * 0.25);
        let (sin_a, cos_a) = angle.sin_cos();
        (self.distance_gain * cos_a, self.distance_gain * sin_a)
    }

    /// Encodes a 16-bit 2-channel stereo RIFF WAVE with interaural level and micro-delay cues.
    pub fn stereo_riff_wave(&self) -> Vec<u8> {
        let rate = self.sample_rate;
        let total_samples = (rate * 62) / 100;
        let data_bytes = total_samples * 4;
        let mut wav = Vec::with_capacity(44 + data_bytes as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_bytes).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&rate.to_le_bytes());
        wav.extend_from_slice(&(rate * 4).to_le_bytes());
        wav.extend_from_slice(&4u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_bytes.to_le_bytes());
        let (left_gain, right_gain) = self.channel_gains();
        let itd = self.stereo_pan * 0.00034;
        let inv_rate = 1.0 / (rate as f32);
        for idx in 0..total_samples {
            let t = (idx as f32) * inv_rate;
            let raw_l = self.meadow_gallop_wave((t + itd.max(0.0)).max(0.0));
            let raw_r = self.meadow_gallop_wave((t + (-itd).max(0.0)).max(0.0));
            let pcm_l = (raw_l * left_gain * 31500.0).round().clamp(-32767.0, 32767.0) as i16;
            let pcm_r = (raw_r * right_gain * 31500.0).round().clamp(-32767.0, 32767.0) as i16;
            wav.extend_from_slice(&pcm_l.to_le_bytes());
            wav.extend_from_slice(&pcm_r.to_le_bytes());
        }
        wav
    }

    fn turf_grain(t: f32, rate: u32) -> f32 {
        let idx = ((t.max(0.0) * (rate as f32)) as u32).wrapping_add(31);
        let h = idx.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
        let w = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277_803_737);
        (((w ^ (w >> 16)) as i32) as f32) / (i32::MAX as f32)
    }

    fn hoof_thud(&self, dt: f32, pitch: f32, step_dt: f32, t: f32) -> f32 {
        if !(0.0..0.058).contains(&dt) {
            return 0.0;
        }
        let g0 = Self::turf_grain(t, self.sample_rate);
        let g1 = Self::turf_grain(t - step_dt, self.sample_rate);
        let g2 = Self::turf_grain(t - 2.0 * step_dt, self.sample_rate);
        let muffled_earth = 0.50 * g0 + 0.35 * g1 + 0.15 * g2;
        let sod_drum = (-46.0 * dt).exp() * (2.0 * PI * (104.0 * pitch * dt - 215.0 * dt * dt)).sin() * 0.76;
        let hoof_tap = (-92.0 * dt).exp() * muffled_earth * 0.34;
        sod_drum + hoof_tap
    }

    fn bound_stride(&self, t: f32, start: f32, gain: f32) -> f32 {
        let window = 0.148;
        if t < start || t >= start + window {
            return 0.0;
        }
        let step_dt = 1.0 / (self.sample_rate as f32);
        let p = self.pace_scale;
        let fore = self.hoof_thud(t - start, p * 1.06, step_dt, t) * 0.78;
        let hind_delay = 0.046 / p;
        let hind = self.hoof_thud(t - (start + hind_delay), p * 0.94, step_dt, t) * 0.96;

        let u = (t - start) / window;
        let swish_env = (u * PI).sin().powi(2) * (-1.7 * u).exp();
        let raw_a = Self::turf_grain(t, self.sample_rate) - 0.68 * Self::turf_grain(t - step_dt, self.sample_rate);
        let raw_b = Self::turf_grain(t - step_dt, self.sample_rate) - 0.68 * Self::turf_grain(t - 2.0 * step_dt, self.sample_rate);
        let silky_grass = 0.58 * raw_a + 0.42 * raw_b;
        let grass_rustle = swish_env * silky_grass * 0.34;

        gain * (fore + hind + grass_rustle)
    }

    fn meadow_gallop_wave(&self, t: f32) -> f32 {
        let first_bound = self.bound_stride(t, 0.012, 1.00);
        let second_bound = self.bound_stride(t, 0.225, 0.74);
        let third_bound = self.bound_stride(t, 0.435, 0.48);
        (first_bound + second_bound + third_bound).clamp(-1.0, 1.0)
    }
}

impl AcousticWave for RoeHoofbeats {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        (self.meadow_gallop_wave(t) * self.distance_gain).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        #[cfg(not(test))]
        {
            let bytes = self.stereo_riff_wave();
            #[cfg(target_os = "linux")]
            {
                let path = std::env::temp_dir().join("low_cube_roe_hoofbeats_spatial.wav");
                let _ = std::fs::write(&path, &bytes);
            }
            self.play_cached_wav(&bytes, "low_cube_roe_hoofbeats_spatial.wav");
        }
        #[cfg(test)]
        {
            let _ = self.amplitude_at(0.03);
        }
    }
}
