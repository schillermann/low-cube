use std::f32::consts::PI;
use std::sync::OnceLock;
use super::{AcousticWave, SoundStage};

/// Savory two-bite chew waveform emitted when eating wild boar provisions.
#[derive(Clone, Copy, Debug)]
pub struct HeartyBite {
    sample_rate: u32,
}

impl HeartyBite {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Secondary constructor calibrated at 22050 Hz mono for warm savory bite transients.
    pub const fn savory() -> Self {
        Self::new(22050)
    }

    fn chew_grain(t: f32, rate: u32) -> f32 {
        let idx = ((t.max(0.0) * (rate as f32) * 0.35) as u32).wrapping_add(53);
        let h = idx.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
        let w = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277_803_737);
        (((w ^ (w >> 16)) as i32) as f32) / (i32::MAX as f32)
    }

    fn bite_pulse(&self, t: f32, start: f32, dur: f32, pitch: f32) -> f32 {
        if t < start || t >= start + dur {
            return 0.0;
        }
        let dt = t - start;
        let u = dt / dur;
        let env = (u * 11.0).clamp(0.0, 1.0) * (1.0 - u).powf(0.8) * (-4.2 * u).exp();
        let jaw = (2.0 * PI * (pitch * (1.12 - 0.32 * u)) * dt).sin() * 0.58;
        let body = (2.0 * PI * (pitch * 2.05) * dt).sin() * 0.24;
        let crisp = Self::chew_grain(t, self.sample_rate) * 0.28 * (-6.0 * u).exp();
        env * (jaw + body + crisp)
    }
}

impl AcousticWave for HeartyBite {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        let first = self.bite_pulse(t, 0.015, 0.135, 148.0) * 0.92;
        let second = self.bite_pulse(t, 0.205, 0.125, 132.0) * 0.78;
        (first + second).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        if SoundStage::active() {
            static BITE_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = BITE_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_hearty_bite.wav");
        } else {
            let _ = self.amplitude_at(0.05);
        }
    }
}
