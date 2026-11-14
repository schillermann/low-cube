use std::f32::consts::PI;
use glam::Vec3;
use super::AcousticWave;

/// Hoarse laryngeal roe deer alarm bark ("Schrecken") with 3D spatial distance falloff and stereo azimuth.
#[derive(Clone, Copy, Debug)]
pub struct RoeBark {
    sample_rate: u32,
    distance_gain: f32,
    stereo_pan: f32,
    pitch_scale: f32,
}

impl RoeBark {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32, distance_gain: f32, stereo_pan: f32, pitch_scale: f32) -> Self {
        Self {
            sample_rate,
            distance_gain,
            stereo_pan,
            pitch_scale,
        }
    }

    /// Secondary constructor projecting a 3D roe deer alarm bark relative to listener position and gaze.
    pub fn spatial(deer_pos: Vec3, listener_pos: Vec3, view_dir: Vec3, up: Vec3, seed: u32) -> Self {
        let delta = deer_pos - listener_pos;
        let dist = delta.length();
        let falloff = (1.0 - dist / 44.0).clamp(0.0, 1.0);
        let distance_gain = (falloff * falloff * (24.0 / (24.0 + dist * dist))).clamp(0.04, 0.90);
        let right = view_dir.cross(up).normalize_or_zero();
        let dir = if dist > 1e-4 { delta / dist } else { Vec3::ZERO };
        let stereo_pan = dir.dot(right).clamp(-0.92, 0.92);
        let pitch_scale = 0.88 + (((seed % 25) as f32) * 0.009);
        Self::new(24000, distance_gain, stereo_pan, pitch_scale)
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
        let total_samples = (rate * 48) / 100;
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
            let raw_l = self.larynx_wave((t + itd.max(0.0)).max(0.0));
            let raw_r = self.larynx_wave((t + (-itd).max(0.0)).max(0.0));
            let pcm_l = (raw_l * left_gain * 31500.0).round().clamp(-32767.0, 32767.0) as i16;
            let pcm_r = (raw_r * right_gain * 31500.0).round().clamp(-32767.0, 32767.0) as i16;
            wav.extend_from_slice(&pcm_l.to_le_bytes());
            wav.extend_from_slice(&pcm_r.to_le_bytes());
        }
        wav
    }

    fn glottal_rasp(t: f32, rate: u32) -> f32 {
        let idx = ((t.max(0.0) * (rate as f32)) as u32).wrapping_add(17);
        let h = idx.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
        let w = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277_803_737);
        (((w ^ (w >> 16)) as i32) as f32) / (i32::MAX as f32)
    }

    fn bark_pulse(&self, t: f32, start: f32, dur: f32, f_base: f32) -> f32 {
        if t < start || t >= start + dur {
            return 0.0;
        }
        let dt = t - start;
        let u = dt / dur;
        let attack = (u * 8.5).clamp(0.0, 1.0);
        let env = attack * (1.0 - u).powf(0.65) * (-2.6 * u).exp();
        let freq = f_base * (1.08 - 0.24 * u);
        let phase = 2.0 * PI * freq * dt;
        let glottal = 0.54 * phase.sin() + 0.34 * (2.0 * phase + 0.3).sin() + 0.20 * (3.0 * phase).cos();
        let throat_formant = (2.0 * PI * (620.0 * self.pitch_scale) * dt).sin() * 0.22;
        let rasp = Self::glottal_rasp(t, self.sample_rate) * (0.30 + 0.22 * phase.cos().abs());
        env * (glottal + throat_formant + rasp)
    }

    fn larynx_wave(&self, t: f32) -> f32 {
        let p = self.pitch_scale;
        let primary = self.bark_pulse(t, 0.015, 0.165, 295.0 * p) * 0.95;
        let echo_bark = self.bark_pulse(t, 0.255, 0.125, 268.0 * p) * 0.68;
        (primary + echo_bark).clamp(-1.0, 1.0)
    }
}

impl AcousticWave for RoeBark {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        (self.larynx_wave(t) * self.distance_gain).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        #[cfg(not(test))]
        {
            let bytes = self.stereo_riff_wave();
            #[cfg(target_os = "linux")]
            {
                let path = std::env::temp_dir().join("low_cube_roe_bark_v1_spatial.wav");
                let _ = std::fs::write(&path, &bytes);
            }
            self.play_cached_wav(&bytes, "low_cube_roe_bark_v1_spatial.wav");
        }
        #[cfg(test)]
        {
            let _ = self.amplitude_at(0.06);
        }
    }
}
