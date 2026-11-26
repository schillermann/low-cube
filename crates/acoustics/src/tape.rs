use std::sync::OnceLock;
use super::rain_patter::{CanopyWash, MuffledCanopy, PatterDrops, WindSwell};
use super::MonoSource;

const EPSTEIN_LOOP_WAV: &[u8] = include_bytes!("../assets/epstein_loop.wav");
const EPSTEIN_IGNITE_WAV: &[u8] = include_bytes!("../assets/epstein_ignite.wav");
const BELL_LOOP_WAV: &[u8] = include_bytes!("../assets/bell_loop.wav");
const BELL_IGNITE_WAV: &[u8] = include_bytes!("../assets/bell_ignite.wav");
const RCS_LOOP_WAV: &[u8] = include_bytes!("../assets/rcs_loop.wav");
const RCS_BURST_WAV: &[u8] = include_bytes!("../assets/rcs_burst.wav");
const RCS_SHUTOFF_WAV: &[u8] = include_bytes!("../assets/rcs_shutoff.wav");
const PALM_DIG_WAVS: [&[u8]; 4] = [
    include_bytes!("../assets/palm_dig_0.wav"),
    include_bytes!("../assets/palm_dig_1.wav"),
    include_bytes!("../assets/palm_dig_2.wav"),
    include_bytes!("../assets/palm_dig_3.wav"),
];
const SOIL_CRUMBLE_WAV: &[u8] = include_bytes!("../assets/soil_crumble.wav");

/// Zero-allocation embedded 16-bit mono PCM studio recording bundled inside the executable.
#[derive(Clone, Copy, Debug)]
pub struct TapeSample {
    raw: &'static [u8],
    pcm_offset: usize,
    sample_rate: u32,
}

impl TapeSample {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(raw: &'static [u8], pcm_offset: usize, sample_rate: u32) -> Self {
        Self { raw, pcm_offset, sample_rate }
    }

    pub const fn epstein_loop() -> Self {
        Self::new(EPSTEIN_LOOP_WAV, 44, 24000)
    }

    pub const fn epstein_ignite() -> Self {
        Self::new(EPSTEIN_IGNITE_WAV, 44, 24000)
    }

    pub const fn bell_loop() -> Self {
        Self::new(BELL_LOOP_WAV, 44, 24000)
    }

    pub const fn bell_ignite() -> Self {
        Self::new(BELL_IGNITE_WAV, 44, 24000)
    }

    pub const fn rcs_loop() -> Self {
        Self::new(RCS_LOOP_WAV, 44, 24000)
    }

    pub const fn rcs_burst() -> Self {
        Self::new(RCS_BURST_WAV, 44, 24000)
    }

    pub const fn rcs_shutoff() -> Self {
        Self::new(RCS_SHUTOFF_WAV, 44, 24000)
    }

    pub const fn palm_dig(variant: usize) -> Self {
        Self::new(PALM_DIG_WAVS[variant % 4], 44, 24000)
    }

    pub const fn soil_crumble() -> Self {
        Self::new(SOIL_CRUMBLE_WAV, 44, 24000)
    }

    pub fn rain_loop() -> Self {
        static RAIN_PCM: OnceLock<&'static [u8]> = OnceLock::new();
        let raw = RAIN_PCM.get_or_init(|| {
            let chain = MuffledCanopy::woodland_rain(WindSwell::periodic(PatterDrops::foliage(
                CanopyWash::velvet(0x517C_C1B7),
            )));
            Self::bake_periodic_pcm(chain, 72_000, 24_000)
        });
        Self::new(raw, 0, 24_000)
    }

    pub fn snow_loop() -> Self {
        static SNOW_PCM: OnceLock<&'static [u8]> = OnceLock::new();
        let raw = SNOW_PCM.get_or_init(|| {
            let chain = MuffledCanopy::polar_snow(WindSwell::periodic(CanopyWash::velvet(0x9E37_79B9)));
            Self::bake_periodic_pcm(chain, 72_000, 24_000)
        });
        Self::new(raw, 0, 24_000)
    }

    fn bake_periodic_pcm<S: MonoSource>(source: S, frames: usize, rate: u32) -> &'static [u8] {
        let mut bytes = Vec::with_capacity(frames * 2);
        let inv = 1.0 / (rate as f32);
        for idx in 0..frames {
            let amp = source.dry_sample((idx as f32) * inv);
            let pcm = (amp * 28_000.0).round().clamp(-32_767.0, 32_767.0) as i16;
            bytes.extend_from_slice(&pcm.to_le_bytes());
        }
        Box::leak(bytes.into_boxed_slice())
    }

    pub const fn rate(&self) -> u32 {
        self.sample_rate
    }

    pub const fn frame_count(&self) -> usize {
        if self.raw.len() > self.pcm_offset {
            (self.raw.len() - self.pcm_offset) / 2
        } else {
            0
        }
    }

    fn frame_at(&self, index: usize) -> f32 {
        let byte_idx = self.pcm_offset + index * 2;
        if byte_idx + 1 >= self.raw.len() {
            return 0.0;
        }
        let s = i16::from_le_bytes([self.raw[byte_idx], self.raw[byte_idx + 1]]);
        f32::from(s) / 32768.0
    }

    pub fn sample_linear(&self, cursor: f64) -> Option<f32> {
        let len = self.frame_count();
        if cursor < 0.0 || len == 0 {
            return None;
        }
        let idx = cursor.floor() as usize;
        if idx >= len {
            return None;
        }
        let frac = (cursor - (idx as f64)) as f32;
        let a = self.frame_at(idx);
        let b = if idx + 1 < len { self.frame_at(idx + 1) } else { 0.0 };
        Some(a + (b - a) * frac)
    }

    pub fn sample_looped(&self, cursor: f64) -> (f32, f64) {
        let len = self.frame_count();
        if len == 0 {
            return (0.0, 0.0);
        }
        let span = len as f64;
        let wrapped = cursor.rem_euclid(span);
        let idx = wrapped.floor() as usize % len;
        let next = (idx + 1) % len;
        let frac = (wrapped - (idx as f64)) as f32;
        let val = self.frame_at(idx) + (self.frame_at(next) - self.frame_at(idx)) * frac;
        (val, wrapped)
    }

    pub fn sample_at_sec(&self, seconds: f32) -> f32 {
        if seconds < 0.0 {
            return 0.0;
        }
        self.sample_linear(f64::from(seconds) * f64::from(self.sample_rate)).unwrap_or(0.0)
    }
}
