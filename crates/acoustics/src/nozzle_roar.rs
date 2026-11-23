use super::hull_chorus::{HullWaveTrace, PropulsionCue};
use super::outbox::SoundOutbox;
use super::tape::TapeSample;
use super::{AcousticWave, MonoSource, SoundStage};

/// Real NASA rocket-bell combustion roar and chamber light-off recording (`RocketNozzle`).
#[derive(Clone, Copy, Debug)]
pub struct NozzleRoar {
    tape: TapeSample,
    ignition: bool,
}

impl NozzleRoar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tape: TapeSample, ignition: bool) -> Self {
        Self { tape, ignition }
    }

    /// Secondary constructor backed by the embedded NASA SSME chamber light-off recording.
    pub const fn ignition() -> Self {
        Self::new(TapeSample::bell_ignite(), true)
    }

    /// Secondary constructor backed by the embedded seamless NASA rocket-bell roar loop.
    pub const fn sustained() -> Self {
        Self::new(TapeSample::bell_loop(), false)
    }
}

impl MonoSource for NozzleRoar {
    fn dry_sample(&self, seconds: f32) -> f32 {
        self.tape.sample_at_sec(seconds)
    }
}

impl AcousticWave for NozzleRoar {
    fn sample_rate(&self) -> u32 {
        self.tape.rate()
    }

    fn amplitude_at(&self, seconds: f32) -> f32 {
        self.dry_sample(seconds)
    }

    fn thump(&self) {
        if self.ignition {
            HullWaveTrace::sink().note(PropulsionCue::BellIgnition);
            SoundOutbox::new(SoundStage::active()).submit_tape(self.tape, 1.0, (0.88, 0.88));
        } else {
            HullWaveTrace::sink().note(PropulsionCue::BellSustained);
            let _ = self.amplitude_at(0.10);
        }
    }
}
