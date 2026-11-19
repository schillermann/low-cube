use std::f32::consts::PI;
use std::sync::OnceLock;
use super::{AcousticWave, SoundStage};

/// Turbulent combustion chamber roar and supersonic exhaust crackle emitted by firing rocket nozzles.
#[derive(Clone, Copy, Debug)]
pub struct NozzleRoar {
    sample_rate: u32,
}

impl NozzleRoar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Secondary constructor calibrated at 16000 Hz mono for deep chamber rumble and jet roar.
    pub const fn ignition() -> Self {
        Self::new(16000)
    }

    fn jet_turbulence(&self, t: f32) -> f32 {
        let idx = ((t.max(0.0) * (self.sample_rate as f32)) as u32).wrapping_add(97);
        let h = idx.wrapping_mul(747796405).wrapping_add(2891336453);
        let w = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277803737);
        (((w ^ (w >> 16)) as i32) as f32) / (i32::MAX as f32)
    }
}

impl AcousticWave for NozzleRoar {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        if !(0.0..0.42).contains(&t) {
            return 0.0;
        }
        let dt = 1.0 / (self.sample_rate as f32);
        let env = (t / 0.42 * PI).sin() * (-2.4 * t).exp();
        let roar_noise = 0.45 * self.jet_turbulence(t)
            + 0.35 * self.jet_turbulence(t - dt)
            + 0.20 * self.jet_turbulence(t - 2.0 * dt);
        let sub_chamber = 0.52 * (2.0 * PI * 36.0 * t).sin()
            + 0.28 * (2.0 * PI * 72.0 * t).sin()
            + 0.14 * (2.0 * PI * 112.0 * t).cos();
        (env * (0.58 * sub_chamber + 0.54 * roar_noise)).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        if SoundStage::active() {
            static ROAR_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = ROAR_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_nozzle_roar.wav");
        } else {
            let _ = self.amplitude_at(0.06);
        }
    }
}
