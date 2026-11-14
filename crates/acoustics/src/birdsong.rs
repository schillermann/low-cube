use std::f32::consts::PI;
use glam::Vec3;
use super::{AcousticWave, MonoSource, SpatialStage};

/// Melodic three-syllable songbird syrinx chirp ("tschilp-tirili-tschilp") with 3D spatial distance and stereo azimuth.
#[derive(Clone, Copy, Debug)]
pub struct SparrowChirp {
    stage: SpatialStage,
    pitch_scale: f32,
}

impl SparrowChirp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn from_stage(stage: SpatialStage, pitch_scale: f32) -> Self {
        Self { stage, pitch_scale }
    }

    /// Secondary constructor preserving explicit rate, distance gain, pan, and pitch parameters.
    pub const fn new(sample_rate: u32, distance_gain: f32, stereo_pan: f32, pitch_scale: f32) -> Self {
        Self::from_stage(SpatialStage::new(sample_rate, distance_gain, stereo_pan, 48), pitch_scale)
    }

    /// Secondary constructor projecting a 3D bird vocalization relative to listener position, view direction, and surface up.
    pub fn spatial(bird_pos: Vec3, listener_pos: Vec3, view_dir: Vec3, up: Vec3, seed: u32) -> Self {
        let stage = SpatialStage::projected(bird_pos, listener_pos, view_dir, up, (32.0, 14.0, 0.03, 0.85), 48);
        let pitch_scale = 0.90 + (((seed % 29) as f32) * 0.0075);
        Self::from_stage(stage, pitch_scale)
    }

    pub fn channel_gains(&self) -> (f32, f32) {
        self.stage.channel_gains()
    }

    pub fn stereo_riff_wave(&self) -> Vec<u8> {
        self.stage.stereo_riff_wave(self, 0.00032)
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

impl MonoSource for SparrowChirp {
    fn dry_sample(&self, t: f32) -> f32 {
        let p = self.pitch_scale;
        let first = Self::syrinx_syllable(t, 0.01, 0.095, 2750.0 * p, 980.0 * p, 54.0) * 0.78;
        let second = Self::syrinx_syllable(t, 0.14, 0.115, 3050.0 * p, 1220.0 * p, 72.0) * 0.92;
        let third = Self::syrinx_syllable(t, 0.29, 0.105, 3420.0 * p, -640.0 * p, 46.0) * 0.74;
        (first + second + third).clamp(-1.0, 1.0)
    }
}

impl AcousticWave for SparrowChirp {
    fn sample_rate(&self) -> u32 {
        self.stage.sample_rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        self.stage.attenuate(self.dry_sample(t))
    }

    fn thump(&self) {
        self.stage.thump_stereo(self, 0.00032, "low_cube_sparrow_chirp_spatial.wav");
    }
}
