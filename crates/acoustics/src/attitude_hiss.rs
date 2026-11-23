use super::hull_chorus::{HullWaveTrace, PropulsionCue};
use super::tape::TapeSample;
use super::{AcousticWave, MonoSource, SpatialStage};

/// Real NASA laminar hull-conducted thruster recording ("teakettle" RCS attitude pod).
#[derive(Clone, Copy, Debug)]
pub struct AttitudeHiss {
    stage: SpatialStage,
    tape: TapeSample,
    pitch_scale: f32,
    valve_transient: bool,
}

impl AttitudeHiss {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        stage: SpatialStage,
        tape: TapeSample,
        pitch_scale: f32,
        valve_transient: bool,
    ) -> Self {
        Self { stage, tape, pitch_scale, valve_transient }
    }

    /// Secondary constructor backed by the embedded laminar hull-conducted RCS burst recording.
    pub fn valve_burst(throttle: f32, stereo_pan: f32, cadence: u32) -> Self {
        const INFLECTIONS: [f32; 6] = [0.94, 0.89, 0.97, 0.91, 0.95, 0.87];
        let pitch = INFLECTIONS[(cadence as usize) % INFLECTIONS.len()];
        let gain = (0.48 + 0.44 * throttle.clamp(0.0, 1.0)).clamp(0.0, 0.92);
        let pan = stereo_pan.clamp(-0.68, 0.68);
        Self::new(SpatialStage::new(24000, gain, pan, 26), TapeSample::rcs_burst(), pitch, true)
    }

    /// Secondary constructor backed by the embedded seamless laminar RCS vent loop.
    pub fn sustained_vent(throttle: f32, stereo_pan: f32) -> Self {
        let gain = (0.40 + 0.44 * throttle.clamp(0.0, 1.0)).clamp(0.0, 0.86);
        let pan = stereo_pan.clamp(-0.68, 0.68);
        Self::new(SpatialStage::new(24000, gain, pan, 40), TapeSample::rcs_loop(), 0.92, false)
    }

    pub fn channel_gains(&self) -> (f32, f32) {
        self.stage.channel_gains()
    }

    pub fn stereo_riff_wave(&self) -> Vec<u8> {
        self.stage.stereo_riff_wave(self, 0.00032)
    }
}

impl MonoSource for AttitudeHiss {
    fn dry_sample(&self, seconds: f32) -> f32 {
        self.tape.sample_at_sec(seconds * self.pitch_scale)
    }
}

impl AcousticWave for AttitudeHiss {
    fn sample_rate(&self) -> u32 {
        self.stage.sample_rate()
    }

    fn amplitude_at(&self, seconds: f32) -> f32 {
        self.stage.attenuate(self.dry_sample(seconds))
    }

    fn thump(&self) {
        let cue = if self.valve_transient {
            PropulsionCue::AttitudeValve
        } else {
            PropulsionCue::AttitudeVent
        };
        HullWaveTrace::sink().note(cue);
        let _ = self.amplitude_at(0.04);
    }
}
