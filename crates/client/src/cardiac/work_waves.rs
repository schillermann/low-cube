use std::f32::consts::PI;
#[cfg(not(test))]
use std::sync::OnceLock;
use super::AcousticWave;

/// Earthy low-mid soil scrape and dull palm-thud waveform emitted when digging with bare hands.
#[derive(Clone, Copy, Debug)]
pub struct PalmScrape {
    sample_rate: u32,
}

impl PalmScrape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Secondary constructor calibrated at 16000 Hz mono for warm, earthy hand-digging scrapes.
    pub const fn earthy() -> Self {
        Self::new(16000)
    }

    fn soil_grain(&self, t: f32) -> f32 {
        let idx = ((t.max(0.0) * (self.sample_rate as f32)) as u32).wrapping_add(17);
        let h = idx.wrapping_mul(747796405).wrapping_add(2891336453);
        let w = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277803737);
        (((w ^ (w >> 16)) as i32) as f32) / (i32::MAX as f32)
    }
}

impl AcousticWave for PalmScrape {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        if !(0.0..0.16).contains(&t) {
            return 0.0;
        }
        let dt = 1.0 / (self.sample_rate as f32);
        let muffled_grain = 0.50 * self.soil_grain(t) + 0.35 * self.soil_grain(t - dt) + 0.15 * self.soil_grain(t - 2.0 * dt);
        let palm_thud = (-28.0 * t).exp() * (2.0 * PI * (92.0 * t - 140.0 * t * t)).sin() * 0.68;
        let scoop_rub = (t / 0.16 * PI).sin() * (-14.0 * t).exp() * muffled_grain * 0.45;
        (palm_thud + scoop_rub).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        #[cfg(not(test))]
        {
            static PALM_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = PALM_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_palm_dig.wav");
        }
        #[cfg(test)]
        {
            let _ = self.amplitude_at(0.02);
        }
    }
}

/// Metallic iron blade ring and crisp gravel crunch waveform emitted when digging with a spade.
#[derive(Clone, Copy, Debug)]
pub struct SpadeStrike {
    sample_rate: u32,
}

impl SpadeStrike {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Secondary constructor calibrated at 22050 Hz mono for metallic iron-spade strikes.
    pub const fn forged() -> Self {
        Self::new(22050)
    }

    fn gravel_grit(&self, t: f32) -> f32 {
        let idx = ((t.max(0.0) * (self.sample_rate as f32)) as u32).wrapping_add(53);
        let h = idx.wrapping_mul(1664525).wrapping_add(1013904223);
        let w = (h ^ (h >> 13)).wrapping_mul(1274126177);
        (((w ^ (w >> 16)) as i32) as f32) / (i32::MAX as f32)
    }
}

impl AcousticWave for SpadeStrike {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        if !(0.0..0.20).contains(&t) {
            return 0.0;
        }
        let dt = 1.0 / (self.sample_rate as f32);
        let crisp = self.gravel_grit(t) - 0.55 * self.gravel_grit(t - dt);
        let steel_ring = (-22.0 * t).exp()
            * (0.46 * (2.0 * PI * 690.0 * t).sin()
                + 0.34 * (2.0 * PI * 1460.0 * t).sin()
                + 0.20 * (2.0 * PI * 2310.0 * t).sin());
        let gravel_bite = (-36.0 * t).exp() * crisp * 0.58;
        (steel_ring + gravel_bite).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        #[cfg(not(test))]
        {
            static SPADE_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = SPADE_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_spade_dig.wav");
        }
        #[cfg(test)]
        {
            let _ = self.amplitude_at(0.02);
        }
    }
}

/// Multi-cylinder combustion chug and exhaust rumble waveform emitted while driving a vehicle.
#[derive(Clone, Copy, Debug)]
pub struct EngineRumble {
    sample_rate: u32,
}

impl EngineRumble {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Secondary constructor calibrated at 16000 Hz mono for low-end combustion engine rumble.
    pub const fn combustion() -> Self {
        Self::new(16000)
    }
}

impl AcousticWave for EngineRumble {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        if !(0.0..0.36).contains(&t) {
            return 0.0;
        }
        let env = (t / 0.36 * PI).sin() * (-3.2 * t).exp();
        let chug = 0.55 + 0.45 * (2.0 * PI * 14.0 * t).sin();
        let cyl = 0.56 * (2.0 * PI * 42.0 * t).sin()
            + 0.29 * (2.0 * PI * 84.0 * t).sin()
            + 0.15 * (2.0 * PI * 126.0 * t).cos();
        (env * chug * cyl).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        #[cfg(not(test))]
        {
            static ENGINE_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = ENGINE_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_car_drive.wav");
        }
        #[cfg(test)]
        {
            let _ = self.amplitude_at(0.05);
        }
    }
}
