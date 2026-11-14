use std::f32::consts::PI;
use std::sync::Mutex;
use glam::Vec3;
use super::{AcousticWave, SoundStage};

/// Dry unattenuated mono waveform source ready for 3D stereo distance and azimuth projection.
pub trait MonoSource {
    fn dry_sample(&self, seconds: f32) -> f32;
}

/// 3D spatial acoustic stage computing distance falloff, stereo azimuth panning, ITD micro-delay, and 16-bit stereo RIFF WAVE encoding.
#[derive(Clone, Copy, Debug)]
pub struct SpatialStage {
    sample_rate: u32,
    distance_gain: f32,
    stereo_pan: f32,
    centisec: u32,
}

impl SpatialStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32, distance_gain: f32, stereo_pan: f32, centisec: u32) -> Self {
        Self {
            sample_rate,
            distance_gain,
            stereo_pan,
            centisec,
        }
    }

    /// Secondary constructor projecting a 3D acoustic emitter relative to listener position, gaze direction, and surface normal.
    pub fn projected(
        source_pos: Vec3,
        listener_pos: Vec3,
        view_dir: Vec3,
        up: Vec3,
        range: (f32, f32, f32, f32),
        centisec: u32,
    ) -> Self {
        let (max_dist, knee, min_gain, max_gain) = range;
        let delta = source_pos - listener_pos;
        let dist = delta.length();
        let falloff = (1.0 - dist / max_dist).clamp(0.0, 1.0);
        let distance_gain = (falloff * falloff * (knee / (knee + dist * dist))).clamp(min_gain, max_gain);
        let right = view_dir.cross(up).normalize_or_zero();
        let dir = if dist > 1e-4 { delta / dist } else { Vec3::ZERO };
        let stereo_pan = dir.dot(right).clamp(-0.92, 0.92);
        Self::new(24000, distance_gain, stereo_pan, centisec)
    }

    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Left and right stereo channel gains computed from distance attenuation and azimuth angle.
    pub fn channel_gains(&self) -> (f32, f32) {
        let angle = (self.stereo_pan.clamp(-1.0, 1.0) + 1.0) * (PI * 0.25);
        let (sin_a, cos_a) = angle.sin_cos();
        (self.distance_gain * cos_a, self.distance_gain * sin_a)
    }

    /// Attenuates a dry mono sample by the 3D distance falloff factor.
    pub fn attenuate(&self, dry: f32) -> f32 {
        (dry * self.distance_gain).clamp(-1.0, 1.0)
    }

    /// Encodes a 16-bit 2-channel stereo RIFF WAVE into `wav`, reusing existing capacity when available.
    pub fn encode_stereo_into<S: MonoSource>(&self, source: &S, itd_scale: f32, wav: &mut Vec<u8>) {
        let rate = self.sample_rate;
        let total_samples = (rate * self.centisec) / 100;
        let data_bytes = total_samples * 4;
        wav.clear();
        wav.reserve(44 + data_bytes as usize);
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
        let itd = self.stereo_pan * itd_scale;
        let inv_rate = 1.0 / (rate as f32);
        for idx in 0..total_samples {
            let t = (idx as f32) * inv_rate;
            let raw_l = source.dry_sample((t + itd.max(0.0)).max(0.0));
            let raw_r = source.dry_sample((t + (-itd).max(0.0)).max(0.0));
            let pcm_l = (raw_l * left_gain * 31500.0).round().clamp(-32767.0, 32767.0) as i16;
            let pcm_r = (raw_r * right_gain * 31500.0).round().clamp(-32767.0, 32767.0) as i16;
            wav.extend_from_slice(&pcm_l.to_le_bytes());
            wav.extend_from_slice(&pcm_r.to_le_bytes());
        }
    }

    /// Allocates and encodes a 16-bit 2-channel stereo RIFF WAVE byte vector.
    pub fn stereo_riff_wave<S: MonoSource>(&self, source: &S, itd_scale: f32) -> Vec<u8> {
        let mut wav = Vec::new();
        self.encode_stereo_into(source, itd_scale, &mut wav);
        wav
    }

    /// Encodes into a shared reusable scratch buffer and dispatches playback when `SoundStage` is live.
    pub fn thump_stereo<W: AcousticWave + MonoSource>(&self, wave: &W, itd_scale: f32, filename: &str) {
        if !SoundStage::active() {
            let _ = wave.amplitude_at(0.05);
            return;
        }
        static STEREO_SCRATCH: Mutex<Vec<u8>> = Mutex::new(Vec::new());
        if let Ok(mut buf) = STEREO_SCRATCH.lock() {
            self.encode_stereo_into(wave, itd_scale, &mut buf);
            wave.play_wav(&buf, filename, true);
        }
    }
}
