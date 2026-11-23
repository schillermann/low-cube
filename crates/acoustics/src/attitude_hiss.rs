use super::hull_chorus::{HullWaveTrace, PropulsionCue};
use super::outbox::SoundOutbox;
use super::tape::TapeSample;
use super::{AcousticWave, MonoSource, SoundStage, SpatialStage};

/// Real foley solenoid valve snap and pressurized gas jet recording ("teakettle" RCS pod).
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

    /// Secondary constructor backed by the embedded real switch-snap + compressed-gas burst recording.
    pub fn valve_burst(throttle: f32, stereo_pan: f32, cadence: u32) -> Self {
        const INFLECTIONS: [f32; 6] = [1.00, 0.95, 1.05, 0.97, 1.03, 0.93];
        let pitch = INFLECTIONS[(cadence as usize) % INFLECTIONS.len()];
        let gain = (0.48 + 0.46 * throttle.clamp(0.0, 1.0)).clamp(0.0, 0.95);
        let pan = stereo_pan.clamp(-0.68, 0.68);
        Self::new(SpatialStage::new(24000, gain, pan, 27), TapeSample::rcs_burst(), pitch, true)
    }

    /// Secondary constructor backed by the embedded real mechanical solenoid shutoff recording.
    pub fn valve_shutoff(stereo_pan: f32) -> Self {
        let pan = stereo_pan.clamp(-0.68, 0.68);
        Self::new(SpatialStage::new(24000, 0.62, pan, 10), TapeSample::rcs_shutoff(), 1.0, false)
    }

    /// Secondary constructor backed by the embedded seamless compressed-gas vent loop.
    pub fn sustained_vent(throttle: f32, stereo_pan: f32) -> Self {
        let gain = (0.40 + 0.46 * throttle.clamp(0.0, 1.0)).clamp(0.0, 0.88);
        let pan = stereo_pan.clamp(-0.68, 0.68);
        Self::new(SpatialStage::new(24000, gain, pan, 40), TapeSample::rcs_loop(), 1.0, false)
    }

    pub fn channel_gains(&self) -> (f32, f32) {
        self.stage.channel_gains()
    }

    pub fn stereo_riff_wave(&self) -> Vec<u8> {
        self.stage.stereo_riff_wave(self, 0.00032)
    }

    pub fn thump_shutoff(&self) {
        SoundOutbox::new(SoundStage::active()).submit_tape(
            self.tape,
            self.pitch_scale,
            self.stage.channel_gains(),
        );
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
        if self.valve_transient {
            HullWaveTrace::sink().note(PropulsionCue::AttitudeValve);
            SoundOutbox::new(SoundStage::active()).submit_tape(
                self.tape,
                self.pitch_scale,
                self.stage.channel_gains(),
            );
        } else {
            HullWaveTrace::sink().note(PropulsionCue::AttitudeVent);
            let _ = self.amplitude_at(0.05);
        }
    }
}
