use std::f32::consts::PI;
#[cfg(not(test))]
use std::sync::{Mutex, OnceLock};
use super::{AcousticWave, PalmScrape, SpadeStrike};

const DIG_VARIATIONS: usize = 8;

#[cfg(test)]
thread_local! {
    static TRACE_RECORDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static TRACE_VARIANTS: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Test-only acoustic trace capturing emitted digging stroke variations without OS playback.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub struct DigWaveTrace {
    expected_min: usize,
}

#[cfg(test)]
impl DigWaveTrace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(expected_min: usize) -> Self {
        Self { expected_min }
    }

    pub fn record_variants<F: FnOnce()>(self, action: F) -> Vec<u8> {
        TRACE_VARIANTS.with(|v| v.borrow_mut().clear());
        TRACE_RECORDING.with(|r| r.set(true));
        action();
        TRACE_RECORDING.with(|r| r.set(false));
        let out = TRACE_VARIANTS.with(|v| v.borrow().clone());
        assert!(out.len() >= self.expected_min, "expected at least {} dig waves, got {}", self.expected_min, out.len());
        out
    }
}

/// Acoustic decorator warping pitch, modal frequencies, and stroke duration of an underlying waveform.
#[derive(Clone, Copy, Debug)]
pub struct InflectedWave<W: AcousticWave> {
    origin: W,
    pitch_ratio: f32,
    gain: f32,
}

impl<W: AcousticWave> InflectedWave<W> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: W, pitch_ratio: f32, gain: f32) -> Self {
        Self { origin, pitch_ratio, gain }
    }
}

impl<W: AcousticWave> AcousticWave for InflectedWave<W> {
    fn sample_rate(&self) -> u32 {
        self.origin.sample_rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        (self.origin.amplitude_at(t * self.pitch_ratio) * self.gain).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        let _ = self.amplitude_at(0.02);
        self.origin.thump();
    }
}

/// Acoustic decorator overlaying delayed soil-clod and pebble crumble particles atop an underlying waveform.
#[derive(Clone, Copy, Debug)]
pub struct CrumbledWave<W: AcousticWave> {
    origin: W,
    delay_sec: f32,
    crumb_gain: f32,
    grain_seed: u32,
}

impl<W: AcousticWave> CrumbledWave<W> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: W, delay_sec: f32, crumb_gain: f32, grain_seed: u32) -> Self {
        Self { origin, delay_sec, crumb_gain, grain_seed }
    }

    fn clod_grain(&self, t: f32, rate: u32) -> f32 {
        let idx = ((t.max(0.0) * (rate as f32)) as u32).wrapping_add(self.grain_seed);
        let h = idx.wrapping_mul(747796405).wrapping_add(2891336453);
        let w = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277803737);
        (((w ^ (w >> 16)) as i32) as f32) / (i32::MAX as f32)
    }
}

impl<W: AcousticWave> AcousticWave for CrumbledWave<W> {
    fn sample_rate(&self) -> u32 {
        self.origin.sample_rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        let base = self.origin.amplitude_at(t);
        if self.crumb_gain <= 0.0 || !(self.delay_sec..self.delay_sec + 0.135).contains(&t) {
            return base;
        }
        let rate = self.origin.sample_rate();
        let dt = 1.0 / (rate as f32);
        let (g0, g1) = (self.clod_grain(t, rate), self.clod_grain(t - dt, rate));
        let grit = 0.65 * (g0 - 0.45 * g1) + 0.35 * (0.5 * g0 + 0.5 * g1);
        let u = (t - self.delay_sec) / 0.135;
        let env = (u * PI).sin().powi(2) * (-5.5 * u).exp();
        (base + env * grit * self.crumb_gain).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        let _ = self.amplitude_at(0.02);
        self.origin.thump();
    }
}

/// Immutable contour recipe wrapping an origin waveform in `CrumbledWave<InflectedWave<W>>`.
#[derive(Clone, Copy, Debug)]
struct StrokeContour {
    pitch_ratio: f32,
    gain: f32,
    crumb_delay: f32,
    crumb_gain: f32,
}

impl StrokeContour {
    const fn new(pitch_ratio: f32, gain: f32, crumb_delay: f32, crumb_gain: f32) -> Self {
        Self { pitch_ratio, gain, crumb_delay, crumb_gain }
    }

    const fn decorate<W: AcousticWave + Copy>(self, origin: W, grain_seed: u32) -> CrumbledWave<InflectedWave<W>> {
        CrumbledWave::new(
            InflectedWave::new(origin, self.pitch_ratio, self.gain),
            self.crumb_delay,
            self.crumb_gain,
            grain_seed,
        )
    }
}

/// Acoustic decorator cycling an underlying excavation waveform across 8 inflected and crumbled contours.
#[derive(Clone, Copy, Debug)]
pub struct CadencedWave<W: AcousticWave + Copy> {
    origin: W,
    cadence: u32,
    slots: &'static [&'static str; DIG_VARIATIONS],
}

impl<W: AcousticWave + Copy> CadencedWave<W> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: W, cadence: u32, slots: &'static [&'static str; DIG_VARIATIONS]) -> Self {
        Self { origin, cadence, slots }
    }

    const fn composed(&self) -> CrumbledWave<InflectedWave<W>> {
        const CONTOURS: [StrokeContour; DIG_VARIATIONS] = [
            StrokeContour::new(1.00, 1.00, 0.000, 0.00),
            StrokeContour::new(0.80, 0.96, 0.090, 0.25),
            StrokeContour::new(1.22, 0.92, 0.015, 0.12),
            StrokeContour::new(0.89, 0.98, 0.070, 0.20),
            StrokeContour::new(1.11, 0.94, 0.050, 0.16),
            StrokeContour::new(0.74, 0.95, 0.100, 0.28),
            StrokeContour::new(1.28, 0.90, 0.012, 0.15),
            StrokeContour::new(0.69, 1.00, 0.080, 0.34),
        ];
        let idx = (self.cadence as usize) % DIG_VARIATIONS;
        let seed = (idx as u32).wrapping_mul(7919).wrapping_add(313);
        CONTOURS[idx].decorate(self.origin, seed)
    }
}

impl CadencedWave<PalmScrape> {
    /// Secondary constructor decorating bare-hand `PalmScrape::earthy()` with progressive cadence variation.
    pub const fn palm(cadence: u32) -> Self {
        const PALM_SLOTS: [&str; DIG_VARIATIONS] = [
            "low_cube_palm_dig_v3_0.wav",
            "low_cube_palm_dig_v3_1.wav",
            "low_cube_palm_dig_v3_2.wav",
            "low_cube_palm_dig_v3_3.wav",
            "low_cube_palm_dig_v3_4.wav",
            "low_cube_palm_dig_v3_5.wav",
            "low_cube_palm_dig_v3_6.wav",
            "low_cube_palm_dig_v3_7.wav",
        ];
        Self::new(PalmScrape::earthy(), cadence, &PALM_SLOTS)
    }
}

impl CadencedWave<SpadeStrike> {
    /// Secondary constructor decorating iron-blade `SpadeStrike::forged()` with progressive cadence variation.
    pub const fn spade(cadence: u32) -> Self {
        const SPADE_SLOTS: [&str; DIG_VARIATIONS] = [
            "low_cube_spade_dig_v3_0.wav",
            "low_cube_spade_dig_v3_1.wav",
            "low_cube_spade_dig_v3_2.wav",
            "low_cube_spade_dig_v3_3.wav",
            "low_cube_spade_dig_v3_4.wav",
            "low_cube_spade_dig_v3_5.wav",
            "low_cube_spade_dig_v3_6.wav",
            "low_cube_spade_dig_v3_7.wav",
        ];
        Self::new(SpadeStrike::forged(), cadence, &SPADE_SLOTS)
    }
}

impl<W: AcousticWave + Copy> AcousticWave for CadencedWave<W> {
    fn sample_rate(&self) -> u32 {
        self.composed().sample_rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        self.composed().amplitude_at(t)
    }

    fn thump(&self) {
        #[cfg(not(test))]
        {
            static WAVE_BANK: OnceLock<Mutex<Vec<(&'static str, &'static [u8])>>> = OnceLock::new();
            let idx = (self.cadence as usize) % DIG_VARIATIONS;
            let filename = self.slots[idx];
            let mutex = WAVE_BANK.get_or_init(|| Mutex::new(Vec::with_capacity(16)));
            let mut guard = mutex.lock().expect("dig wave cache poisoned");
            let bytes: &'static [u8] = if let Some(&(_, cached)) = guard.iter().find(|&&(name, _)| name == filename) {
                cached
            } else {
                let leaked: &'static [u8] = Box::leak(self.riff_wave().into_boxed_slice());
                guard.push((filename, leaked));
                leaked
            };
            drop(guard);
            self.play_cached_wav(bytes, filename);
        }
        #[cfg(test)]
        {
            let _ = self.amplitude_at(0.02);
            let idx = ((self.cadence as usize) % DIG_VARIATIONS) as u8;
            let tag_offset = if self.slots[0].contains("spade") { 100u8 } else { 0u8 };
            TRACE_RECORDING.with(|r| {
                if r.get() {
                    TRACE_VARIANTS.with(|v| v.borrow_mut().push(tag_offset + idx));
                }
            });
        }
    }
}
