use std::f32::consts::PI;
use glam::Vec3;
use super::{AcousticWave, MonoSource, SpatialStage};

/// Spatialized roe deer bounding hoofbeat and meadow-grass swish waveform.
#[derive(Clone, Copy, Debug)]
pub struct RoeHoofbeats {
    stage: SpatialStage,
    pace_scale: f32,
}

impl RoeHoofbeats {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn from_stage(stage: SpatialStage, pace_scale: f32) -> Self {
        Self { stage, pace_scale }
    }

    /// Secondary constructor preserving explicit rate, distance gain, pan, and pace parameters.
    pub const fn new(sample_rate: u32, distance_gain: f32, stereo_pan: f32, pace_scale: f32) -> Self {
        Self::from_stage(SpatialStage::new(sample_rate, distance_gain, stereo_pan, 62), pace_scale)
    }

    /// Secondary constructor projecting 3D bounding hoofbeats relative to listener position and gaze.
    pub fn spatial(deer_pos: Vec3, listener_pos: Vec3, view_dir: Vec3, up: Vec3, seed: u32) -> Self {
        let stage = SpatialStage::projected(deer_pos, listener_pos, view_dir, up, (40.0, 22.0, 0.04, 0.88), 62);
        let pace_scale = 0.94 + (((seed % 19) as f32) * 0.007);
        Self::from_stage(stage, pace_scale)
    }

    pub fn channel_gains(&self) -> (f32, f32) {
        self.stage.channel_gains()
    }

    pub fn stereo_riff_wave(&self) -> Vec<u8> {
        self.stage.stereo_riff_wave(self, 0.00034)
    }

    fn turf_grain(t: f32, rate: u32) -> f32 {
        let idx = ((t.max(0.0) * (rate as f32)) as u32).wrapping_add(31);
        let h = idx.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
        let w = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277_803_737);
        (((w ^ (w >> 16)) as i32) as f32) / (i32::MAX as f32)
    }

    fn hoof_thud(&self, dt: f32, pitch: f32, step_dt: f32, t: f32) -> f32 {
        if !(0.0..0.056).contains(&dt) {
            return 0.0;
        }
        let u = dt / 0.056;
        let env = (u * 14.0).clamp(0.0, 1.0) * (-34.0 * dt).exp();
        let phase = 2.0 * PI * (128.0 * pitch * dt - 640.0 * dt * dt);
        let sod_drum = env * (0.76 * phase.sin() + 0.24 * (1.85 * phase).sin());
        let rate = self.stage.sample_rate();
        let tap = Self::turf_grain(t, rate) - 0.55 * Self::turf_grain(t - step_dt, rate);
        let hoof_tap = (-95.0 * dt).exp() * tap * 0.42;
        sod_drum + hoof_tap
    }

    fn bound_stride(&self, t: f32, start: f32, gain: f32) -> f32 {
        let window = 0.148;
        if t < start || t >= start + window {
            return 0.0;
        }
        let rate = self.stage.sample_rate();
        let step_dt = 1.0 / (rate as f32);
        let p = self.pace_scale;
        let fore = self.hoof_thud(t - start, p * 1.06, step_dt, t) * 0.78;
        let hind_delay = 0.046 / p;
        let hind = self.hoof_thud(t - (start + hind_delay), p * 0.94, step_dt, t) * 0.96;
        let u = (t - start) / window;
        let swish_env = (u * PI).sin().powi(2) * (-1.7 * u).exp();
        let raw_a = Self::turf_grain(t, rate) - 0.68 * Self::turf_grain(t - step_dt, rate);
        let raw_b = Self::turf_grain(t - step_dt, rate) - 0.68 * Self::turf_grain(t - 2.0 * step_dt, rate);
        let grass_rustle = swish_env * (0.58 * raw_a + 0.42 * raw_b) * 0.34;
        gain * (fore + hind + grass_rustle)
    }
}

impl MonoSource for RoeHoofbeats {
    fn dry_sample(&self, t: f32) -> f32 {
        let first_bound = self.bound_stride(t, 0.012, 1.00);
        let second_bound = self.bound_stride(t, 0.225, 0.74);
        let third_bound = self.bound_stride(t, 0.435, 0.48);
        (first_bound + second_bound + third_bound).clamp(-1.0, 1.0)
    }
}

impl AcousticWave for RoeHoofbeats {
    fn sample_rate(&self) -> u32 {
        self.stage.sample_rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        self.stage.attenuate(self.dry_sample(t))
    }

    fn thump(&self) {
        self.stage.thump_stereo(self, 0.00034, "low_cube_roe_hoofbeats_spatial.wav");
    }
}
