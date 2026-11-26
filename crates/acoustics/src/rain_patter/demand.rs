use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, Ordering};
use super::super::outbox::SoundOutbox;
use super::super::voice::LoopTrack;
use super::super::SoundStage;

static SHOWER_LIT: AtomicBool = AtomicBool::new(false);

thread_local! {
    static RAIN_RECORDING: Cell<bool> = const { Cell::new(false) };
    static RAIN_CUES: RefCell<Vec<PrecipitationCue>> = const { RefCell::new(Vec::new()) };
}

/// Distinct atmospheric precipitation acoustic signatures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrecipitationCue {
    DrySilence,
    WoodlandRain,
    CabinRain,
    PolarSnowfall,
}

/// Deterministic acoustic trace capturing precipitation cues emitted within a scope (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct RainWaveTrace {
    expected_min: usize,
}

impl RainWaveTrace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(expected_min: usize) -> Self {
        Self { expected_min }
    }

    pub(super) const fn sink() -> Self {
        Self::new(0)
    }

    pub(super) fn note(self, cue: PrecipitationCue) {
        let _ = self.expected_min;
        RAIN_RECORDING.with(|active| {
            if active.get() {
                RAIN_CUES.with(|buf| buf.borrow_mut().push(cue));
            }
        });
    }

    pub fn capture<F: FnOnce()>(self, action: F) -> Vec<PrecipitationCue> {
        RAIN_CUES.with(|buf| buf.borrow_mut().clear());
        RAIN_RECORDING.with(|active| active.set(true));
        action();
        RAIN_RECORDING.with(|active| active.set(false));
        let recorded = RAIN_CUES.with(|buf| buf.borrow().clone());
        assert!(
            recorded.len() >= self.expected_min,
            "expected at least {} precipitation cues, got {}",
            self.expected_min,
            recorded.len()
        );
        recorded
    }
}

/// Live 120 Hz precipitation loop command driving continuous woodland rain or polar snowfall (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct ShowerDemand {
    intensity: f32,
    frozen: bool,
    sheltered: bool,
}

impl ShowerDemand {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(intensity: f32, frozen: bool, sheltered: bool) -> Self {
        Self { intensity, frozen, sheltered }
    }

    pub const fn calm() -> Self {
        Self::new(0.0, false, false)
    }

    pub fn audible(self) -> bool {
        self.intensity > 0.04
    }

    pub fn cue(self) -> PrecipitationCue {
        if !self.audible() {
            PrecipitationCue::DrySilence
        } else if self.frozen {
            PrecipitationCue::PolarSnowfall
        } else if self.sheltered {
            PrecipitationCue::CabinRain
        } else {
            PrecipitationCue::WoodlandRain
        }
    }

    pub fn publish(self) {
        if self.audible() {
            RainWaveTrace::sink().note(self.cue());
            if SoundStage::active() {
                SHOWER_LIT.store(true, Ordering::Relaxed);
                SoundOutbox::new(true).submit_shower(self);
            }
        } else if SoundStage::active() && SHOWER_LIT.swap(false, Ordering::Relaxed) {
            SoundOutbox::new(true).submit_shower(Self::calm());
        }
    }

    pub(crate) fn apply_to(self, rain_loop: &mut LoopTrack, snow_loop: &mut LoopTrack) {
        if !self.audible() {
            rain_loop.fade_out();
            snow_loop.fade_out();
            return;
        }
        let shelter_k = if self.sheltered { 0.48 } else { 1.0 };
        if self.frozen {
            rain_loop.fade_out();
            let gain = ((0.06 + 0.14 * self.intensity) * shelter_k).clamp(0.0, 0.24);
            let rate = if self.sheltered { 0.84 } else { 0.96 + 0.06 * self.intensity.clamp(0.0, 1.0) };
            snow_loop.command(gain, rate, 0.0);
        } else {
            snow_loop.fade_out();
            let gain = ((0.10 + 0.26 * self.intensity) * shelter_k).clamp(0.0, 0.36);
            let rate = if self.sheltered { 0.82 } else { 0.96 + 0.08 * self.intensity.clamp(0.0, 1.0) };
            rain_loop.command(gain, rate, 0.0);
        }
    }
}
