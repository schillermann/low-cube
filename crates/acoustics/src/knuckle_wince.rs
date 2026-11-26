use std::f32::consts::PI;
use std::sync::OnceLock;
use super::{AcousticWave, DigWaveTrace, SoundStage};

const WINCE_VARIANTS: usize = 4;

/// Vocal tract formant triad shaping glottal harmonics into human vowels (`3` fields).
#[derive(Clone, Copy, Debug)]
struct VowelTract {
    f1: f32,
    f2: f32,
    f3: f32,
}

impl VowelTract {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(f1: f32, f2: f32, f3: f32) -> Self {
        Self { f1, f2, f3 }
    }

    const fn open_ah() -> Self {
        Self::new(790.0, 1310.0, 2680.0)
    }

    fn glide(self, target: Self, raw_blend: f32) -> Self {
        let blend = 0.5 * (1.0 - (PI * raw_blend.clamp(0.0, 1.0)).cos());
        Self::new(
            self.f1 + (target.f1 - self.f1) * blend,
            self.f2 + (target.f2 - self.f2) * blend,
            self.f3 + (target.f3 - self.f3) * blend,
        )
    }

    fn aua_contour(self, u: f32) -> Self {
        let rounded_oo = Self::new(340.0, 720.0, 2380.0);
        let trailing_ah = Self::new(720.0, 1240.0, 2560.0);
        if u < 0.54 {
            self.glide(rounded_oo, (u - 0.18) / 0.34)
        } else {
            rounded_oo.glide(trailing_ah, (u - 0.54) / 0.18)
        }
    }

    fn transfer_gain(self, freq: f32) -> f32 {
        let g1 = (58.0 * 58.0) / ((freq - self.f1).powi(2) + 58.0 * 58.0);
        let g2 = (78.0 * 78.0) / ((freq - self.f2).powi(2) + 78.0 * 78.0);
        let g3 = (115.0 * 115.0) / ((freq - self.f3).powi(2) + 115.0 * 115.0);
        g1 + 0.74 * g2 + 0.28 * g3
    }
}

/// Knuckle-on-stone slap and vocal "Aua!" pain exclamation emitted when striking hard rock bare-handed (`3` fields).
#[derive(Clone, Copy, Debug)]
pub struct KnuckleWince {
    sample_rate: u32,
    pitch_scale: f32,
    variant: u8,
}

impl KnuckleWince {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32, pitch_scale: f32, variant: u8) -> Self {
        Self {
            sample_rate,
            pitch_scale,
            variant,
        }
    }

    /// Secondary constructor calibrated at 24 kHz for a natural human "Aua!" exclamation.
    pub const fn vocal() -> Self {
        Self::new(24000, 1.0, 0)
    }

    /// Secondary constructor cycling across 4 natural vocal pitch inflections on repeated bare-hand strikes.
    pub const fn inflected(cadence: u32) -> Self {
        const SCALES: [f32; WINCE_VARIANTS] = [1.00, 0.94, 1.06, 0.97];
        let idx = (cadence as usize) % WINCE_VARIANTS;
        Self::new(24000, SCALES[idx], idx as u8)
    }

    fn knuckle_grit(&self, t: f32) -> f32 {
        let seed = u32::from(self.variant).wrapping_mul(977).wrapping_add(71);
        let idx = ((t.max(0.0) * (self.sample_rate as f32)) as u32).wrapping_add(seed);
        let h = idx.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let w = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
        (((w ^ (w >> 16)) as i32) as f32) / (i32::MAX as f32)
    }

    fn stone_slap(&self, t: f32) -> f32 {
        if !(0.0..0.032).contains(&t) {
            return 0.0;
        }
        let dt = 1.0 / (self.sample_rate as f32);
        let snap = self.knuckle_grit(t) - 0.62 * self.knuckle_grit(t - dt);
        let bone_thud = (-95.0 * t).exp() * (2.0 * PI * (235.0 * t - 1400.0 * t * t)).sin() * 0.42;
        let rock_tap = (-145.0 * t).exp() * ((2.0 * PI * 1140.0 * t).sin() * 0.22 + snap * 0.38);
        bone_thud + rock_tap
    }

    fn vocal_cry(&self, t: f32) -> f32 {
        let onset = 0.024_f32;
        let span = 0.400_f32;
        if !(onset..onset + span).contains(&t) {
            return 0.0;
        }
        let tau = t - onset;
        let u = tau / span;
        let attack = ((u / 0.042).clamp(0.0, 1.0) * (PI * 0.5)).sin();
        let dip_dist = (u - 0.53) / 0.062;
        let syllable_contour = 1.0 - 0.62 * (-dip_dist * dip_dist).exp();
        let release = (1.0 - ((u - 0.64) / 0.36).clamp(0.0, 1.0)).powf(1.65);
        let env = attack * syllable_contour * (1.0 - 0.18 * u) * release;

        let chest_hz = 216.0 * self.pitch_scale;
        let wm = 3.0 * PI / span;
        let wv = 2.0 * PI * 5.8;
        let inst_f0 =
            chest_hz * (1.18 - 0.34 * u + 0.12 * (wm * tau).sin() + 0.012 * (wv * tau).sin());
        let cycles = chest_hz
            * (1.18 * tau - 0.17 * tau * tau / span
                + 0.12 * (1.0 - (wm * tau).cos()) / wm
                + 0.012 * (1.0 - (wv * tau).cos()) / wv);
        let tract = VowelTract::open_ah().aua_contour(u);

        let mut voiced = 0.0_f32;
        for k in 1..=18 {
            let kf = k as f32;
            let freq = kf * inst_f0;
            if freq < 4600.0 {
                let resonance = tract.transfer_gain(freq) + 0.14 / kf.powf(1.35);
                let tilt = 1.0 / kf.powf(0.82);
                let phase = 2.0 * PI * kf * cycles + kf * 0.37;
                voiced += resonance * tilt * phase.sin();
            }
        }
        let breath = self.knuckle_grit(t) * ((-24.0 * u).exp() * 0.08 + 0.03);
        env * (voiced * 0.62 + breath)
    }
}

impl AcousticWave for KnuckleWince {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        (self.stone_slap(t) + self.vocal_cry(t)).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        let idx = (self.variant as usize) % WINCE_VARIANTS;
        DigWaveTrace::new(0).note_variant(200 + (idx as u8));
        if SoundStage::active() {
            const SLOTS: [&str; WINCE_VARIANTS] = [
                "low_cube_knuckle_wince_0.wav",
                "low_cube_knuckle_wince_1.wav",
                "low_cube_knuckle_wince_2.wav",
                "low_cube_knuckle_wince_3.wav",
            ];
            static WINCE_CACHE: [OnceLock<Vec<u8>>; WINCE_VARIANTS] = [
                OnceLock::new(),
                OnceLock::new(),
                OnceLock::new(),
                OnceLock::new(),
            ];
            let bytes = WINCE_CACHE[idx].get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, SLOTS[idx]);
        } else {
            let _ = self.amplitude_at(0.08);
        }
    }
}
