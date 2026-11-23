use super::hull_chorus::{HullWaveTrace, PropulsionCue};
use super::outbox::SoundOutbox;
use super::tape::TapeSample;
use super::{AcousticWave, MonoSource, SoundStage};

/// Real NASA in-cabin spacecraft thrust recording for Epstein-class `FusionTorch` propulsion.
#[derive(Clone, Copy, Debug)]
pub struct FusionHum {
    tape: TapeSample,
    ignition: bool,
}

impl FusionHum {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tape: TapeSample, ignition: bool) -> Self {
        Self { tape, ignition }
    }

    /// Secondary constructor backed by the embedded NASA in-cabin engine ignition kick.
    pub const fn torch() -> Self {
        Self::new(TapeSample::epstein_ignite(), true)
    }

    /// Secondary constructor backed by the embedded seamless NASA in-cabin hull rumble loop.
    pub const fn sustained() -> Self {
        Self::new(TapeSample::epstein_loop(), false)
    }
}

impl MonoSource for FusionHum {
    fn dry_sample(&self, seconds: f32) -> f32 {
        self.tape.sample_at_sec(seconds)
    }
}

impl AcousticWave for FusionHum {
    fn sample_rate(&self) -> u32 {
        self.tape.rate()
    }

    fn amplitude_at(&self, seconds: f32) -> f32 {
        self.dry_sample(seconds)
    }

    fn thump(&self) {
        if self.ignition {
            HullWaveTrace::sink().note(PropulsionCue::FusionIgnition);
            SoundOutbox::new(SoundStage::active()).submit_tape(self.tape, 1.0, (0.86, 0.86));
        } else {
            HullWaveTrace::sink().note(PropulsionCue::FusionSustained);
            let _ = self.amplitude_at(0.10);
        }
    }
}
