use std::f32::consts::PI;
use std::sync::OnceLock;
use super::{AcousticWave, SoundStage};

/// Superconducting magnetic-confinement hum and ionized plasma lance crackle of a `FusionTorch` drive.
#[derive(Clone, Copy, Debug)]
pub struct FusionHum {
    sample_rate: u32,
}

impl FusionHum {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Secondary constructor calibrated at 16000 Hz mono for Epstein-style fusion reactor harmonics.
    pub const fn torch() -> Self {
        Self::new(16000)
    }

    fn plasma_crackle(&self, t: f32) -> f32 {
        let idx = ((t.max(0.0) * (self.sample_rate as f32)) as u32).wrapping_add(431);
        let h = idx.wrapping_mul(0x85EB_CA6B).wrapping_add(0xC2B2_AE35);
        let w = (h ^ (h >> 15)).wrapping_mul(0x27D4_EB2F);
        (((w ^ (w >> 13)) as i32) as f32) / (i32::MAX as f32)
    }
}

impl AcousticWave for FusionHum {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        if !(0.0..0.48).contains(&t) {
            return 0.0;
        }
        let env = (t / 0.48 * PI).sin() * (-1.45 * t).exp();
        let coil_hum = 0.46 * (2.0 * PI * 58.0 * t).sin()
            + 0.30 * (2.0 * PI * 116.0 * t).sin()
            + 0.18 * (2.0 * PI * 232.0 * t + 0.4 * (2.0 * PI * 14.0 * t).sin()).sin();
        let dt = 1.0 / (self.sample_rate as f32);
        let lance_hiss = 0.55 * self.plasma_crackle(t) - 0.35 * self.plasma_crackle(t - dt);
        (env * (0.68 * coil_hum + 0.32 * lance_hiss)).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        if SoundStage::active() {
            static FUSION_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = FUSION_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_fusion_hum.wav");
        } else {
            let _ = self.amplitude_at(0.08);
        }
    }
}
