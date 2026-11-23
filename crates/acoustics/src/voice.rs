use std::f32::consts::PI;
use std::sync::Arc;
use super::tape::TapeSample;

pub(super) const STREAM_RATE: u32 = 24000;

/// Continuous looping real-audio track with sample-accurate attack/release envelope and stereo pan.
#[derive(Clone, Copy, Debug)]
pub(super) struct LoopTrack {
    tape: TapeSample,
    cursor: f64,
    envelope: (f32, f32),
    spatial: (f32, f32),
}

impl LoopTrack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        tape: TapeSample,
        cursor: f64,
        envelope: (f32, f32),
        spatial: (f32, f32),
    ) -> Self {
        Self { tape, cursor, envelope, spatial }
    }

    pub(super) const fn idle(tape: TapeSample) -> Self {
        Self::new(tape, 0.0, (0.0, 0.0), (1.0, 0.0))
    }

    pub(super) fn command(&mut self, target_gain: f32, pitch_rate: f32, stereo_pan: f32) {
        self.envelope.1 = target_gain.clamp(0.0, 1.0);
        self.spatial = (pitch_rate.clamp(0.75, 1.35), stereo_pan.clamp(-0.85, 0.85));
    }

    pub(super) fn fade_out(&mut self) {
        self.envelope.1 = 0.0;
    }

    pub(super) fn audible(&self) -> bool {
        self.envelope.0 > 0.0005 || self.envelope.1 > 0.0005
    }

    pub(super) fn mix_frame(&mut self, pair: (f32, f32)) -> (f32, f32) {
        let (mut cur, target) = self.envelope;
        let slew = if target > cur { 0.0022 } else { 0.00034 };
        cur += (target - cur) * slew;
        if cur < 0.0003 && target == 0.0 {
            self.envelope.0 = 0.0;
            return pair;
        }
        self.envelope.0 = cur;
        let (dry, wrapped) = self.tape.sample_looped(self.cursor);
        let rate_scale = f64::from(self.tape.rate()) / f64::from(STREAM_RATE);
        self.cursor = wrapped + f64::from(self.spatial.0) * rate_scale;
        let angle = (self.spatial.1 + 1.0) * (PI * 0.25);
        let (sin_a, cos_a) = angle.sin_cos();
        (pair.0 + dry * cur * cos_a, pair.1 + dry * cur * sin_a)
    }
}

/// Polyphonic one-shot stereo or mono voice mixed into the real-time `SoundDeck` stream.
#[derive(Clone, Debug)]
pub(super) struct BurstVoice {
    pcm_stereo: Arc<[i16]>,
    cursor: f64,
    step: f64,
    gains: (f32, f32),
}

impl BurstVoice {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        pcm_stereo: Arc<[i16]>,
        cursor: f64,
        step: f64,
        gains: (f32, f32),
    ) -> Self {
        Self { pcm_stereo, cursor, step, gains }
    }

    pub(super) fn from_tape(tape: TapeSample, pitch_rate: f32, gains: (f32, f32)) -> Self {
        let count = tape.frame_count();
        let mut stereo = Vec::with_capacity(count * 2);
        for idx in 0..count {
            let s = (tape.sample_linear(idx as f64).unwrap_or(0.0) * 32000.0).round() as i16;
            stereo.push(s);
            stereo.push(s);
        }
        let step = (f64::from(tape.rate()) / f64::from(STREAM_RATE)) * f64::from(pitch_rate);
        Self::new(stereo.into(), 0.0, step.clamp(0.4, 2.5), gains)
    }

    pub(super) fn from_riff(wav: &[u8]) -> Option<Self> {
        if wav.len() < 44 || &wav[0..4] != b"RIFF" || &wav[8..12] != b"WAVE" {
            return None;
        }
        let mut pos = 12;
        let mut channels = 1u16;
        let mut rate = STREAM_RATE;
        let mut bits = 16u16;
        let mut data_slice: Option<&[u8]> = None;
        while pos + 8 <= wav.len() {
            let id = &wav[pos..pos + 4];
            let sz = u32::from_le_bytes(wav[pos + 4..pos + 8].try_into().ok()?) as usize;
            pos += 8;
            let end = (pos + sz).min(wav.len());
            if id == b"fmt " && end >= pos + 16 {
                channels = u16::from_le_bytes(wav[pos + 2..pos + 4].try_into().ok()?);
                rate = u32::from_le_bytes(wav[pos + 4..pos + 8].try_into().ok()?);
                bits = u16::from_le_bytes(wav[pos + 14..pos + 16].try_into().ok()?);
            } else if id == b"data" {
                data_slice = Some(&wav[pos..end]);
                break;
            }
            pos = end + (sz & 1);
        }
        let raw = data_slice?;
        if bits != 16 || channels == 0 {
            return None;
        }
        let ch = channels as usize;
        let frames = raw.len() / (2 * ch);
        let mut stereo = Vec::with_capacity(frames * 2);
        for f in 0..frames {
            let base = f * 2 * ch;
            let l = i16::from_le_bytes([raw[base], raw[base + 1]]);
            let r = if ch >= 2 {
                i16::from_le_bytes([raw[base + 2], raw[base + 3]])
            } else {
                l
            };
            stereo.push(l);
            stereo.push(r);
        }
        let step = (f64::from(rate) / f64::from(STREAM_RATE)).clamp(0.25, 3.0);
        Some(Self::new(stereo.into(), 0.0, step, (1.0, 1.0)))
    }

    pub(super) fn mix_frame(&mut self, pair: (f32, f32)) -> Option<(f32, f32)> {
        let frames = self.pcm_stereo.len() / 2;
        let idx = self.cursor.floor() as usize;
        if idx >= frames {
            return None;
        }
        let frac = (self.cursor - (idx as f64)) as f32;
        let l0 = f32::from(self.pcm_stereo[idx * 2]) / 32768.0;
        let r0 = f32::from(self.pcm_stereo[idx * 2 + 1]) / 32768.0;
        let (l1, r1) = if idx + 1 < frames {
            (
                f32::from(self.pcm_stereo[(idx + 1) * 2]) / 32768.0,
                f32::from(self.pcm_stereo[(idx + 1) * 2 + 1]) / 32768.0,
            )
        } else {
            (0.0, 0.0)
        };
        self.cursor += self.step;
        let l = (l0 + (l1 - l0) * frac) * self.gains.0;
        let r = (r0 + (r1 - r0) * frac) * self.gains.1;
        Some((pair.0 + l, pair.1 + r))
    }
}
