use std::f32::consts::PI;
use glam::Vec3;
use super::{AcousticWave, MonoSource, SpatialStage};

/// Guttural pharyngeal wild-boar grunt and angry charge roar with 3D spatial distance falloff and stereo azimuth.
#[derive(Clone, Copy, Debug)]
pub struct BoarSnort {
    stage: SpatialStage,
    pitch_scale: f32,
}

impl BoarSnort {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(stage: SpatialStage, pitch_scale: f32) -> Self {
        Self { stage, pitch_scale }
    }

    /// Secondary constructor projecting a 3D wild-boar vocalization relative to listener position and gaze.
    pub fn spatial(boar_pos: Vec3, listener_pos: Vec3, view_dir: Vec3, up: Vec3, seed: u32) -> Self {
        let stage = SpatialStage::projected(boar_pos, listener_pos, view_dir, up, (40.0, 26.0, 0.05, 0.92), 72);
        let pitch_scale = 0.90 + (((seed % 21) as f32) * 0.011);
        Self::new(stage, pitch_scale)
    }

    pub fn channel_gains(&self) -> (f32, f32) {
        self.stage.channel_gains()
    }

    pub fn stereo_riff_wave(&self) -> Vec<u8> {
        self.stage.stereo_riff_wave(self, 0.00034)
    }

    fn snout_air(t: f32) -> f32 {
        let a = (2.0 * PI * 431.0 * t + (2.0 * PI * 73.0 * t).sin() * 2.1).sin();
        let b = (2.0 * PI * 787.0 * t + (2.0 * PI * 113.0 * t).cos() * 1.7).sin();
        let c = (2.0 * PI * 1291.0 * t + (2.0 * PI * 167.0 * t).sin() * 1.4).cos();
        (a * 0.48 + b * 0.34 + c * 0.18).clamp(-1.0, 1.0)
    }

    fn glottal_grunt(&self, dt: f32, dur: f32, pulse: (f32, f32), formants: (f32, f32, f32)) -> f32 {
        let ((cyc, inst_hz), (f1, f2, squeeze)) = (pulse, formants);
        let u = dt / dur;
        let env = (u * PI).sin().powf(0.65) * (1.0 - 0.25 * u);
        let local = cyc.fract();
        let grain_dt = local / inst_hz;
        let grain_env = (-7.5 * local).exp() * (local * 26.0).clamp(0.0, 1.0);
        let p = self.pitch_scale;
        let throat = (2.0 * PI * (f1 * p) * grain_dt).sin() * 0.56;
        let nasal = (2.0 * PI * (f2 * p) * grain_dt).sin() * 0.34;
        let edge = (2.0 * PI * (f2 * 1.72 * p) * grain_dt).sin() * squeeze;
        let raw = grain_env * (throat + nasal + edge) + Self::snout_air(dt) * 0.10;
        env * (raw * 1.55).tanh()
    }
}

impl MonoSource for BoarSnort {
    fn dry_sample(&self, t: f32) -> f32 {
        let lead_grunt = if (0.015..0.235).contains(&t) {
            let dt = t - 0.015;
            let inst_hz = 31.0 - 5.0 * (dt / 0.22);
            let cyc = 31.0 * dt - 2.5 * dt * dt / 0.22;
            self.glottal_grunt(dt, 0.22, (cyc, inst_hz), (295.0, 620.0, 0.16)) * 0.86
        } else {
            0.0
        };
        let charge_roar = if (0.265..0.565).contains(&t) {
            let dt = t - 0.265;
            let ang = dt / 0.30 * PI;
            let arch = ang.sin();
            let inst_hz = 40.0 + 24.0 * arch;
            let cyc = 40.0 * dt + 24.0 * (0.30 / PI) * (1.0 - ang.cos());
            self.glottal_grunt(dt, 0.30, (cyc, inst_hz), (335.0 + 85.0 * arch, 740.0 + 190.0 * arch, 0.32)) * 0.96
        } else {
            0.0
        };
        let tail_grunt = if (0.585..0.705).contains(&t) {
            let dt = t - 0.585;
            self.glottal_grunt(dt, 0.12, (27.0 * dt, 27.0), (265.0, 560.0, 0.12)) * 0.64
        } else {
            0.0
        };
        (lead_grunt + charge_roar + tail_grunt).clamp(-1.0, 1.0)
    }
}

impl AcousticWave for BoarSnort {
    fn sample_rate(&self) -> u32 {
        self.stage.sample_rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        self.stage.attenuate(self.dry_sample(t))
    }

    fn thump(&self) {
        self.stage.thump_stereo(self, 0.00034, "low_cube_boar_grunt_v2_spatial.wav");
    }
}
