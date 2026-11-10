use std::f32::consts::PI;
#[cfg(not(test))]
use std::sync::OnceLock;
use super::AcousticWave;

/// Melodic three-syllable songbird syrinx chirp ("tschilp-tirili-tschilp") emitted by woodland sparrows.
#[derive(Clone, Copy, Debug)]
pub struct SparrowChirp {
    sample_rate: u32,
}

impl SparrowChirp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Secondary constructor calibrated at 24000 Hz mono for crisp high-frequency avian syrinx trills.
    pub const fn woodland() -> Self {
        Self::new(24000)
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
        let first = Self::syrinx_syllable(t, 0.01, 0.095, 2750.0, 980.0, 54.0) * 0.78;
        let second = Self::syrinx_syllable(t, 0.14, 0.115, 3050.0, 1220.0, 72.0) * 0.92;
        let third = Self::syrinx_syllable(t, 0.29, 0.105, 3420.0, -640.0, 46.0) * 0.74;
        (first + second + third).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        #[cfg(not(test))]
        {
            static CHIRP_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = CHIRP_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_sparrow_chirp.wav");
        }
        #[cfg(test)]
        {
            let _ = self.amplitude_at(0.05);
        }
    }
}
