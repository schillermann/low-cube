use std::f32::consts::PI;
use std::sync::{Mutex, OnceLock};
use super::tape::TapeSample;
use super::{AcousticWave, PalmScrape, SoundStage, SpadeStrike};

const DIG_VARIATIONS: usize = 8;

thread_local! {
    static TRACE_RECORDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static TRACE_VARIANTS: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Acoustic trace capturing emitted digging stroke variations when recording.
#[derive(Clone, Copy, Debug)]
pub struct DigWaveTrace {
    expected_min: usize,
}

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

    fn clod_sample(&self, local_t: f32) -> f32 {
        let seed_offset = ((self.grain_seed % 11) as f32) * 0.014;
        TapeSample::soil_crumble().sample_at_sec(seed_offset + local_t)
    }
}

impl<W: AcousticWave> AcousticWave for CrumbledWave<W> {
    fn sample_rate(&self) -> u32 {
        self.origin.sample_rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        let base = self.origin.amplitude_at(t);
        if self.crumb_gain <= 0.0 || !(self.delay_sec..self.delay_sec + 0.19).contains(&t) {
            return base;
        }
        let local_t = t - self.delay_sec;
        let u = local_t / 0.19;
        let env = (u * PI).sin();
        let clod = self.clod_sample(local_t);
        (base + env * clod * self.crumb_gain).tanh()
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
            StrokeContour::new(1.00, 0.88, 0.110, 0.24),
            StrokeContour::new(0.93, 0.86, 0.125, 0.30),
            StrokeContour::new(1.05, 0.84, 0.095, 0.20),
            StrokeContour::new(0.96, 0.88, 0.135, 0.28),
            StrokeContour::new(1.03, 0.85, 0.115, 0.25),
            StrokeContour::new(0.90, 0.87, 0.140, 0.34),
            StrokeContour::new(1.07, 0.83, 0.100, 0.22),
            StrokeContour::new(0.88, 0.89, 0.130, 0.36),
        ];
        let idx = (self.cadence as usize) % DIG_VARIATIONS;
        let seed = (idx as u32).wrapping_mul(7919).wrapping_add(313);
        CONTOURS[idx].decorate(self.origin, seed)
    }
}

impl CadencedWave<PalmScrape> {
    /// Secondary constructor cycling 4 real bare-hand soil recordings across 8 organic clod contours.
    pub const fn palm(cadence: u32) -> Self {
        const PALM_SLOTS: [&str; DIG_VARIATIONS] = [
            "low_cube_palm_dig_v4_0.wav",
            "low_cube_palm_dig_v4_1.wav",
            "low_cube_palm_dig_v4_2.wav",
            "low_cube_palm_dig_v4_3.wav",
            "low_cube_palm_dig_v4_4.wav",
            "low_cube_palm_dig_v4_5.wav",
            "low_cube_palm_dig_v4_6.wav",
            "low_cube_palm_dig_v4_7.wav",
        ];
        Self::new(PalmScrape::stroke(cadence as usize), cadence, &PALM_SLOTS)
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
        let idx = (self.cadence as usize) % DIG_VARIATIONS;
        let tag_offset = if self.slots[0].contains("spade") { 100u8 } else { 0u8 };
        TRACE_RECORDING.with(|r| {
            if r.get() {
                TRACE_VARIANTS.with(|v| v.borrow_mut().push(tag_offset + (idx as u8)));
            }
        });
        if SoundStage::active() {
            static WAVE_BANK: OnceLock<Mutex<Vec<(&'static str, &'static [u8])>>> = OnceLock::new();
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
        } else {
            let _ = self.amplitude_at(0.02);
        }
    }
}
