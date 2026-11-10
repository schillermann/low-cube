use std::f32::consts::PI;
use glam::Vec3;
use super::AcousticWave;

/// Melodic three-syllable songbird syrinx chirp ("tschilp-tirili-tschilp") with 3D spatial distance and stereo azimuth.
#[derive(Clone, Copy, Debug)]
pub struct SparrowChirp {
    sample_rate: u32,
    distance_gain: f32,
    stereo_pan: f32,
    pitch_scale: f32,
}

impl SparrowChirp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32, distance_gain: f32, stereo_pan: f32, pitch_scale: f32) -> Self {
        Self {
            sample_rate,
            distance_gain,
            stereo_pan,
            pitch_scale,
        }
    }

    /// Secondary constructor projecting a 3D bird vocalization relative to listener position, view direction, and surface up.
    pub fn spatial(bird_pos: Vec3, listener_pos: Vec3, view_dir: Vec3, up: Vec3, seed: u32) -> Self {
        let delta = bird_pos - listener_pos;
        let dist = delta.length();
        let falloff = (1.0 - dist / 32.0).clamp(0.0, 1.0);
        let distance_gain = (falloff * falloff * (14.0 / (14.0 + dist * dist))).clamp(0.03, 0.85);
        let right = view_dir.cross(up).normalize_or_zero();
        let dir = if dist > 1e-4 { delta / dist } else { Vec3::ZERO };
        let stereo_pan = dir.dot(right).clamp(-0.92, 0.92);
        let pitch_scale = 0.90 + (((seed % 29) as f32) * 0.0075);
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
        let itd = self.stereo_pan * 0.00032;
        let inv_rate = 1.0 / (rate as f32);
        for idx in 0..total_samples {
            let t = (idx as f32) * inv_rate;
            let raw_l = self.syrinx_wave((t + itd.max(0.0)).max(0.0));
            let raw_r = self.syrinx_wave((t + (-itd).max(0.0)).max(0.0));
            let pcm_l = (raw_l * left_gain * 31500.0).round().clamp(-32767.0, 32767.0) as i16;
            let pcm_r = (raw_r * right_gain * 31500.0).round().clamp(-32767.0, 32767.0) as i16;
            wav.extend_from_slice(&pcm_l.to_le_bytes());
            wav.extend_from_slice(&pcm_r.to_le_bytes());
        }
        wav
    }

    fn syrinx_wave(&self, t: f32) -> f32 {
        let p = self.pitch_scale;
        let first = Self::syrinx_syllable(t, 0.01, 0.095, 2750.0 * p, 980.0 * p, 54.0) * 0.78;
        let second = Self::syrinx_syllable(t, 0.14, 0.115, 3050.0 * p, 1220.0 * p, 72.0) * 0.92;
        let third = Self::syrinx_syllable(t, 0.29, 0.105, 3420.0 * p, -640.0 * p, 46.0) * 0.74;
        (first + second + third).clamp(-1.0, 1.0)
    }

    fn syrinx_syllable(t: f32, start: f32, dur: f32, f_base: f32, f_sweep: f32, trill_hz: f32) -> f32 {
        if t < start || t >= start + dur {
            return 0.0;
        }
        let dt = t - start;
        let u = dt / dur;
        let env = (u * PI).sin().powi(2) * (-3.2 * u).exp();
        let trill = (2.0 * PI * trill_hz * dt).sin() * 140.0;
        let freq = f_base + f_sweep * (u * PI).sin() + trill;
        let phase = 2.0 * PI * freq * dt;
        let fundamental = phase.sin();
        let overtone = (2.0 * phase + 0.4).sin() * 0.28;
        let shimmer = (3.0 * phase).cos() * 0.09;
        env * (0.72 * fundamental + overtone + shimmer)
    }
}

impl AcousticWave for SparrowChirp {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        (self.syrinx_wave(t) * self.distance_gain).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        #[cfg(not(test))]
        {
            let bytes = self.stereo_riff_wave();
            #[cfg(target_os = "linux")]
            {
                let path = std::env::temp_dir().join("low_cube_sparrow_chirp_spatial.wav");
                let _ = std::fs::write(&path, &bytes);
            }
            self.play_cached_wav(&bytes, "low_cube_sparrow_chirp_spatial.wav");
        }
        #[cfg(test)]
        {
            let _ = self.amplitude_at(0.05);
        }
    }
}
