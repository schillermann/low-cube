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

    pub fn publish(self) {
        SoundOutbox::new(SoundStage::active()).submit_drive(self);
    }

    pub(super) fn apply_to(self, loops: &mut [LoopTrack; 3]) {
        let ep_gain = if self.epstein_throttle > 0.02 {
            (0.38 + 0.56 * self.epstein_throttle).clamp(0.0, 0.94)
        } else {
            0.0
        };
        let ep_rate = 0.96 + 0.08 * self.epstein_throttle.clamp(0.0, 1.0);
        loops[0].command(ep_gain, ep_rate, 0.0);

        let bell_gain = if self.bell_throttle > 0.02 {
            (0.36 + 0.58 * self.bell_throttle).clamp(0.0, 0.94)
        } else {
            0.0
        };
        let bell_rate = 0.97 + 0.07 * self.bell_throttle.clamp(0.0, 1.0);
        loops[1].command(bell_gain, bell_rate, 0.0);

        let rcs_gain = if self.rcs_throttle > 0.02 {
            (0.28 + 0.44 * self.rcs_throttle).clamp(0.0, 0.72)
        } else {
            0.0
        };
        loops[2].command(rcs_gain, 1.0, self.rcs_pan);
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
        self.drive_lease = LEASE_CHUNKS;
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
            for track in &mut self.loops {
                acc = track.mix_frame(acc);
            }
            let mut idx = 0;
            while idx < self.bursts.len() {
                if let Some(next_acc) = self.bursts[idx].mix_frame(acc) {
                    acc = next_acc;
                    idx += 1;
                } else {
                    self.bursts.swap_remove(idx);
                }
            }
            let soft_l = (acc.0 / (1.0 + 0.28 * acc.0.abs())).clamp(-1.0, 1.0);
            let soft_r = (acc.1 / (1.0 + 0.28 * acc.1.abs())).clamp(-1.0, 1.0);
            frame[0] = (soft_l * 31800.0).round() as i16;
            frame[1] = (soft_r * 31800.0).round() as i16;
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
        deck.push_burst(BurstVoice::from_tape(TapeSample::rcs_burst(), 1.0, (0.85, 0.35)));
        assert!(deck.active());

        let mut left_sq = 0.0_f64;
        let mut right_sq = 0.0_f64;
        for _ in 0..4 {
            deck.update_drive(HullDriveDemand::new(1.0, 0.0, 0.85, -0.55));
            deck.mix_stereo_into(&mut chunk);
            for pair in chunk.chunks_exact(2) {
                left_sq += f64::from(pair[0]) * f64::from(pair[0]);
                right_sq += f64::from(pair[1]) * f64::from(pair[1]);
            }
        }
        assert!(left_sq > right_sq * 1.10, "Port RCS + Epstein stream must lean toward left channel");

        // Stop refreshing HullDriveDemand (simulating dismount or releasing controls): dead-man lease fades to silence
        for _ in 0..80 {
            deck.mix_stereo_into(&mut chunk);
        }
        let tail_peak = chunk.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
        assert_eq!(tail_peak, 0, "Expired HullDriveDemand lease must fade continuous thruster loops cleanly to zero");
        assert!(!deck.active());
    }
}
