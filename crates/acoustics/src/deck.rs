use super::outbox::SoundOutbox;
use super::tape::TapeSample;
use super::voice::{BurstVoice, LoopTrack};
use super::SoundStage;

const MAX_BURST_VOICES: usize = 10;
const LEASE_CHUNKS: u8 = 7;

/// Live 120 Hz spacecraft thruster loop command uniting Epstein, chemical bell, and stereo RCS levels.
#[derive(Clone, Copy, Debug)]
pub struct HullDriveDemand {
    epstein_throttle: f32,
    bell_throttle: f32,
    rcs_throttle: f32,
    rcs_pan: f32,
}

impl HullDriveDemand {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        epstein_throttle: f32,
        bell_throttle: f32,
        rcs_throttle: f32,
        rcs_pan: f32,
    ) -> Self {
        Self {
            epstein_throttle,
            bell_throttle,
            rcs_throttle,
            rcs_pan,
        }
    }

    pub const fn silent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn lit(self) -> bool {
        self.epstein_throttle > 0.02 || self.bell_throttle > 0.02 || self.rcs_throttle > 0.02
    }

    pub fn publish(self) {
        SoundOutbox::new(SoundStage::active()).submit_drive(self);
    }

    pub(super) fn apply_to(self, loops: &mut [LoopTrack; 3]) {
        let ep_gain = if self.epstein_throttle > 0.02 {
            (0.28 + 0.38 * self.epstein_throttle).clamp(0.0, 0.66)
        } else {
            0.0
        };
        let ep_rate = 0.96 + 0.08 * self.epstein_throttle.clamp(0.0, 1.0);
        loops[0].command(ep_gain, ep_rate, 0.0);

        let bell_gain = if self.bell_throttle > 0.02 {
            (0.26 + 0.40 * self.bell_throttle).clamp(0.0, 0.66)
        } else {
            0.0
        };
        let bell_rate = 0.97 + 0.07 * self.bell_throttle.clamp(0.0, 1.0);
        loops[1].command(bell_gain, bell_rate, 0.0);

        let rcs_gain = if self.rcs_throttle > 0.02 {
            (0.24 + 0.30 * self.rcs_throttle).clamp(0.0, 0.54)
        } else {
            0.0
        };
        loops[2].command(rcs_gain, 0.92, self.rcs_pan);
    }
}

/// Polyphonic stereo mixing deck uniting 3 continuous thruster loops, dead-man lease, and burst voices.
pub(super) struct SoundDeck {
    loops: [LoopTrack; 3],
    bursts: Vec<BurstVoice>,
    drive_lease: u8,
}

impl SoundDeck {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        loops: [LoopTrack; 3],
        bursts: Vec<BurstVoice>,
        drive_lease: u8,
    ) -> Self {
        Self { loops, bursts, drive_lease }
    }

    pub(super) fn studio() -> Self {
        Self::new(
            [
                LoopTrack::idle(TapeSample::epstein_loop()),
                LoopTrack::idle(TapeSample::bell_loop()),
                LoopTrack::idle(TapeSample::rcs_loop()),
            ],
            Vec::with_capacity(MAX_BURST_VOICES),
            0,
        )
    }

    pub(super) fn update_drive(&mut self, demand: HullDriveDemand) {
        demand.apply_to(&mut self.loops);
        self.drive_lease = if demand.lit() { LEASE_CHUNKS } else { 0 };
    }

    pub(super) fn push_burst(&mut self, voice: BurstVoice) {
        if self.bursts.len() >= MAX_BURST_VOICES {
            self.bursts.remove(0);
        }
        self.bursts.push(voice);
    }

    pub(super) fn active(&self) -> bool {
        !self.bursts.is_empty() || self.loops.iter().any(LoopTrack::audible)
    }

    pub(super) fn mix_stereo_into(&mut self, out_interleaved: &mut [i16]) {
        if self.drive_lease > 0 {
            self.drive_lease -= 1;
            if self.drive_lease == 0 {
                for track in &mut self.loops {
                    track.fade_out();
                }
            }
        }
        for frame in out_interleaved.chunks_exact_mut(2) {
            let mut acc = (0.0_f32, 0.0_f32);
            acc = self.loops[0].mix_frame(acc, false);
            acc = self.loops[1].mix_frame(acc, false);
            acc = self.loops[2].mix_frame(acc, true);
            let mut idx = 0;
            while idx < self.bursts.len() {
                if let Some(next_acc) = self.bursts[idx].mix_frame(acc) {
                    acc = next_acc;
                    idx += 1;
                } else {
                    self.bursts.swap_remove(idx);
                }
            }
            // True C^infinity soft-knee limiter bounded strictly within (-1.0, 1.0)
            let soft_l = acc.0.tanh();
            let soft_r = acc.1.tanh();
            frame[0] = (soft_l * 31500.0).round() as i16;
            frame[1] = (soft_r * 31500.0).round() as i16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_nasa_loops_stream_continuously_mix_polyphonically_and_fade_on_lease_expiry() {
        let mut deck = SoundDeck::studio();
        let mut chunk = [0i16; 960];

        assert!(!deck.active());
        deck.update_drive(HullDriveDemand::new(1.0, 0.0, 0.85, -0.55));
        deck.push_burst(BurstVoice::from_tape(TapeSample::epstein_ignite(), 1.0, (0.40, 0.40)));
        assert!(deck.active());

        let mut left_sq = 0.0_f64;
        let mut right_sq = 0.0_f64;
        for _ in 0..4 {
            deck.update_drive(HullDriveDemand::new(1.0, 0.0, 0.85, -0.55));
            deck.mix_stereo_into(&mut chunk);
            for pair in chunk.chunks_exact(2) {
                left_sq += f64::from(pair[0]) * f64::from(pair[0]);
                right_sq += f64::from(pair[1]) * f64::from(pair[1]);
                assert!(pair[0].unsigned_abs() <= 31500 && pair[1].unsigned_abs() <= 31500);
            }
        }
        assert!(left_sq > right_sq * 1.08, "Port RCS + Epstein stream must lean toward left channel");

        for _ in 0..80 {
            deck.mix_stereo_into(&mut chunk);
        }
        let tail_peak = chunk.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
        assert_eq!(tail_peak, 0, "Expired HullDriveDemand lease must fade continuous thruster loops cleanly to zero");
        assert!(!deck.active());
    }

    #[test]
    fn test_rapid_rcs_stereo_pan_reversals_glide_without_zipper_step_clicks() {
        let mut deck = SoundDeck::studio();
        let mut chunk_a = [0i16; 960];
        let mut chunk_b = [0i16; 960];

        deck.update_drive(HullDriveDemand::new(0.0, 0.0, 0.90, -0.75));
        for _ in 0..4 {
            deck.update_drive(HullDriveDemand::new(0.0, 0.0, 0.90, -0.75));
            deck.mix_stereo_into(&mut chunk_a);
        }
        deck.update_drive(HullDriveDemand::new(0.0, 0.0, 0.90, 0.75));
        deck.mix_stereo_into(&mut chunk_b);

        let seam_l = (i32::from(chunk_b[0]) - i32::from(chunk_a[958])).abs();
        let seam_r = (i32::from(chunk_b[1]) - i32::from(chunk_a[959])).abs();
        assert!(
            seam_l < 4500 && seam_r < 4500,
            "Abrupt RCS pan flip must glide sample-accurately without step click: dL={seam_l}, dR={seam_r}"
        );
    }
}
