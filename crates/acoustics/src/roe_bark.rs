use std::f32::consts::PI;
use glam::Vec3;
use super::{AcousticWave, MonoSource, SpatialStage};

/// Hoarse laryngeal roe deer alarm bark ("Schrecken") with 3D spatial distance falloff and stereo azimuth.
#[derive(Clone, Copy, Debug)]
pub struct RoeBark {
    stage: SpatialStage,
    pitch_scale: f32,
}

impl RoeBark {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn from_stage(stage: SpatialStage, pitch_scale: f32) -> Self {
        Self { stage, pitch_scale }
    }

    /// Secondary constructor preserving explicit rate, distance gain, pan, and pitch parameters.
    pub const fn new(sample_rate: u32, distance_gain: f32, stereo_pan: f32, pitch_scale: f32) -> Self {
        Self::from_stage(SpatialStage::new(sample_rate, distance_gain, stereo_pan, 48), pitch_scale)
    }

    /// Secondary constructor projecting a 3D roe deer alarm bark relative to listener position and gaze.
    pub fn spatial(deer_pos: Vec3, listener_pos: Vec3, view_dir: Vec3, up: Vec3, seed: u32) -> Self {
        let stage = SpatialStage::projected(deer_pos, listener_pos, view_dir, up, (44.0, 24.0, 0.04, 0.90), 48);
        let pitch_scale = 0.88 + (((seed % 25) as f32) * 0.009);
        Self::from_stage(stage, pitch_scale)
    }

    pub fn channel_gains(&self) -> (f32, f32) {
        self.stage.channel_gains()
    }

    pub fn stereo_riff_wave(&self) -> Vec<u8> {
        self.stage.stereo_riff_wave(self, 0.00034)
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
        let rasp = Self::glottal_rasp(t, self.stage.sample_rate()) * (0.30 + 0.22 * phase.cos().abs());
        env * (glottal + throat_formant + rasp)
    }
}

impl MonoSource for RoeBark {
    fn dry_sample(&self, t: f32) -> f32 {
        let p = self.pitch_scale;
        let primary = self.bark_pulse(t, 0.015, 0.165, 295.0 * p) * 0.95;
        let echo_bark = self.bark_pulse(t, 0.255, 0.125, 268.0 * p) * 0.68;
        (primary + echo_bark).clamp(-1.0, 1.0)
    }
}

impl AcousticWave for RoeBark {
    fn sample_rate(&self) -> u32 {
        self.stage.sample_rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        self.stage.attenuate(self.dry_sample(t))
    }

    fn thump(&self) {
        self.stage.thump_stereo(self, 0.00034, "low_cube_roe_bark_v1_spatial.wav");
    }
}
