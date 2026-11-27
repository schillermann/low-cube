use std::f32::consts::PI;
use std::sync::OnceLock;
use super::outbox::SoundOutbox;
use super::tape::TapeSample;
use super::{AcousticWave, SoundStage};

/// Real woodland turf, humus, and hand-scooping earth recording emitted when digging with bare hands.
#[derive(Clone, Copy, Debug)]
pub struct PalmScrape {
    tape: TapeSample,
}

impl PalmScrape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tape: TapeSample) -> Self {
        Self { tape }
    }

    /// Secondary constructor selecting the baseline woodland loam hand-digging recording.
    pub const fn earthy() -> Self {
        Self::new(TapeSample::palm_dig(0))
    }

    /// Secondary constructor cycling across 4 distinct real bare-hand soil scoop recordings.
    pub const fn stroke(variant: usize) -> Self {
        Self::new(TapeSample::palm_dig(variant))
    }
}

impl AcousticWave for PalmScrape {
    fn sample_rate(&self) -> u32 {
        self.tape.rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        self.tape.sample_at_sec(t)
    }

    fn thump(&self) {
        if SoundStage::active() {
            SoundOutbox::new(true).submit_tape(self.tape, 1.0, (0.68, 0.68));
        } else {
            let _ = self.amplitude_at(0.04);
        }
    }
}

/// Titanium EVA rig-wrench ring and crisp regolith crunch waveform emitted when excavating with a rig tool.
#[derive(Clone, Copy, Debug)]
pub struct RigStrike {
    sample_rate: u32,
}

impl RigStrike {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Secondary constructor calibrated at 22050 Hz mono for titanium EVA rig-tool strikes.
    pub const fn titanium() -> Self {
        Self::new(22050)
    }

    fn gravel_grit(&self, t: f32) -> f32 {
        let idx = ((t.max(0.0) * (self.sample_rate as f32)) as u32).wrapping_add(53);
        let h = idx.wrapping_mul(1664525).wrapping_add(1013904223);
        let w = (h ^ (h >> 13)).wrapping_mul(1274126177);
        (((w ^ (w >> 16)) as i32) as f32) / (i32::MAX as f32)
    }
}

impl AcousticWave for RigStrike {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        if !(0.0..0.20).contains(&t) {
            return 0.0;
        }
        let dt = 1.0 / (self.sample_rate as f32);
        let crisp = self.gravel_grit(t) - 0.55 * self.gravel_grit(t - dt);
        let alloy_ring = (-22.0 * t).exp()
            * (0.46 * (2.0 * PI * 690.0 * t).sin()
                + 0.34 * (2.0 * PI * 1460.0 * t).sin()
                + 0.20 * (2.0 * PI * 2310.0 * t).sin());
        let gravel_bite = (-36.0 * t).exp() * crisp * 0.58;
        (alloy_ring + gravel_bite).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        if SoundStage::active() {
            static RIG_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = RIG_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_rig_dig.wav");
        } else {
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
        if SoundStage::active() {
            static ENGINE_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = ENGINE_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_car_drive.wav");
        } else {
            let _ = self.amplitude_at(0.05);
        }
    }
}
