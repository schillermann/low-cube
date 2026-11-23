use std::cell::{Cell, RefCell};
use super::outbox::SoundOutbox;
use super::tape::TapeSample;
use super::{AcousticWave, MonoSource, SoundStage, SpatialStage};

thread_local! {
    static HULL_RECORDING: Cell<bool> = const { Cell::new(false) };
    static HULL_CUES: RefCell<Vec<PropulsionCue>> = const { RefCell::new(Vec::new()) };
}

/// Distinct hull-conducted spacecraft propulsion acoustic signatures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropulsionCue {
    FusionIgnition,
    FusionSustained,
    BellIgnition,
    BellSustained,
    AttitudeValve,
    AttitudeVent,
    HullCombined,
}

/// Deterministic acoustic trace capturing spacecraft thruster cues emitted within a scope.
#[derive(Clone, Copy, Debug)]
pub struct HullWaveTrace {
    expected_min: usize,
}

impl HullWaveTrace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(expected_min: usize) -> Self {
        Self { expected_min }
    }

    pub(super) const fn sink() -> Self {
        Self::new(0)
    }

    pub(super) fn note(self, cue: PropulsionCue) {
        let _ = self.expected_min;
        HULL_RECORDING.with(|active| {
            if active.get() {
                HULL_CUES.with(|buf| buf.borrow_mut().push(cue));
            }
        });
    }

    pub fn capture<F: FnOnce()>(self, action: F) -> Vec<PropulsionCue> {
        HULL_CUES.with(|buf| buf.borrow_mut().clear());
        HULL_RECORDING.with(|active| active.set(true));
        action();
        HULL_RECORDING.with(|active| active.set(false));
        let recorded = HULL_CUES.with(|buf| buf.borrow().clone());
        assert!(
            recorded.len() >= self.expected_min,
            "expected at least {} propulsion cues, got {}",
            self.expected_min,
            recorded.len()
        );
        recorded
    }
}

/// Hull-conducted stereo decorator superposing main-drive structural roar with RCS attitude-pod steam bursts.
#[derive(Clone, Copy, Debug)]
pub struct HullChorus<M, V> {
    stage: SpatialStage,
    drive: M,
    attitude: V,
    mix_balance: (f32, f32),
}

impl<M, V> HullChorus<M, V> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        stage: SpatialStage,
        drive: M,
        attitude: V,
        mix_balance: (f32, f32),
    ) -> Self {
        Self { stage, drive, attitude, mix_balance }
    }

    /// Secondary constructor balancing full hull-conducted main drive with stereo-leaned RCS bursts.
    pub fn cockpit(drive: M, attitude: V, stereo_pan: f32) -> Self {
        let pan = (stereo_pan * 0.36).clamp(-0.32, 0.32);
        Self::new(SpatialStage::new(24000, 0.94, pan, 42), drive, attitude, (0.78, 0.64))
    }
}

impl<M: MonoSource, V: MonoSource> MonoSource for HullChorus<M, V> {
    fn dry_sample(&self, t: f32) -> f32 {
        let (drive_gain, rcs_gain) = self.mix_balance;
        let sum = self.drive.dry_sample(t) * drive_gain + self.attitude.dry_sample(t) * rcs_gain;
        (sum / (1.0 + 0.38 * sum.abs())).clamp(-1.0, 1.0)
    }
}

impl<M: MonoSource + Send + Sync, V: MonoSource + Send + Sync> AcousticWave for HullChorus<M, V> {
    fn sample_rate(&self) -> u32 {
        self.stage.sample_rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        self.stage.attenuate(self.dry_sample(t))
    }

    fn thump(&self) {
        HullWaveTrace::sink().note(PropulsionCue::HullCombined);
        let gains = self.stage.channel_gains();
        SoundOutbox::new(SoundStage::active()).submit_tape(TapeSample::rcs_burst(), 1.0, gains);
    }
}
