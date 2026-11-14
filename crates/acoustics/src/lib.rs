mod birdsong;
mod boar_snort;
mod dig_decorators;
mod hearty_bite;
mod hoofbeats;
mod roe_bark;
mod shutter;
mod spatial;
mod work_waves;

use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
pub use self::birdsong::SparrowChirp;
pub use self::boar_snort::BoarSnort;
pub use self::dig_decorators::{CadencedWave, CrumbledWave, DigWaveTrace, InflectedWave};
pub use self::hearty_bite::HeartyBite;
pub use self::hoofbeats::RoeHoofbeats;
pub use self::roe_bark::RoeBark;
pub use self::shutter::ShutterClick;
pub use self::spatial::{MonoSource, SpatialStage};
pub use self::work_waves::{EngineRumble, PalmScrape, SpadeStrike};

static LIVE_AUDIO: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
#[link(name = "winmm")]
extern "system" {
    fn PlaySoundA(psz_sound: *const u8, hmod: *mut core::ffi::c_void, fdw_sound: u32) -> i32;
}

/// Hardware acoustic output gate keeping automated test suites silent across all workspace crates.
#[derive(Clone, Copy, Debug)]
pub struct SoundStage {
    armed: bool,
}

impl SoundStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(armed: bool) -> Self {
        Self { armed }
    }

    /// Secondary constructor arming live OS audio output for interactive sessions.
    pub const fn live() -> Self {
        Self::new(true)
    }

    /// Applies this stage state to the process acoustic output gate.
    pub fn activate(self) {
        LIVE_AUDIO.store(self.armed, Ordering::Relaxed);
    }

    pub(crate) fn active() -> bool {
        LIVE_AUDIO.load(Ordering::Relaxed)
    }
}

/// Interface for synthesized acoustic waveforms emitted during vital posture and world events.
pub trait AcousticWave: Send + Sync {
    fn sample_rate(&self) -> u32;
    fn amplitude_at(&self, seconds: f32) -> f32;
    fn thump(&self);

    fn riff_wave(&self) -> Vec<u8> {
        let rate = self.sample_rate();
        let total_samples = (rate * 55) / 100;
        let data_bytes = total_samples * 2;
        let mut wav = Vec::with_capacity(44 + data_bytes as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_bytes).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&rate.to_le_bytes());
        wav.extend_from_slice(&(rate * 2).to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_bytes.to_le_bytes());
        let inv_rate = 1.0 / (rate as f32);
        for idx in 0..total_samples {
            let amp = self.amplitude_at((idx as f32) * inv_rate);
            let pcm = (amp * 31500.0).round() as i16;
            wav.extend_from_slice(&pcm.to_le_bytes());
        }
        wav
    }

    fn play_cached_wav(&self, bytes: &[u8], filename: &str) {
        self.play_wav(bytes, filename, false);
    }

    fn play_wav(&self, bytes: &[u8], filename: &str, overwrite: bool) {
        if !SoundStage::active() {
            let _ = (bytes.len(), filename.len(), overwrite);
            return;
        }
        #[cfg(target_os = "windows")]
        unsafe {
            use std::sync::Mutex;
            static LIVE_WAV: Mutex<Vec<u8>> = Mutex::new(Vec::new());
            let _ = (filename, overwrite);
            if let Ok(mut buf) = LIVE_WAV.lock() {
                let _ = PlaySoundA(core::ptr::null(), core::ptr::null_mut(), 0);
                buf.clear();
                buf.extend_from_slice(bytes);
                let _ = PlaySoundA(buf.as_ptr(), core::ptr::null_mut(), 0x0001 | 0x0002 | 0x0004);
            }
        }
        #[cfg(target_os = "linux")]
        {
            use std::process::{Child, Command, Stdio};
            use std::sync::atomic::AtomicU8;
            use std::sync::Mutex;
            static BACKEND: AtomicU8 = AtomicU8::new(0);
            static LIVE_PROCS: Mutex<Vec<Child>> = Mutex::new(Vec::new());
            let path = std::env::temp_dir().join(filename);
            if overwrite || !path.exists() {
                let _ = std::fs::write(&path, bytes);
            }
            let spawn_kind = |k: u8| -> std::io::Result<Child> {
                let mut cmd = match k {
                    1 => Command::new("paplay"),
                    2 => Command::new("pw-play"),
                    _ => {
                        let mut c = Command::new("aplay");
                        c.arg("-q");
                        c
                    }
                };
                cmd.arg(&path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()
            };
            let child = match BACKEND.load(Ordering::Relaxed) {
                k @ 1..=3 => spawn_kind(k).ok(),
                4 => None,
                _ => [1u8, 2, 3]
                    .into_iter()
                    .find_map(|k| spawn_kind(k).ok().inspect(|_| BACKEND.store(k, Ordering::Relaxed)))
                    .or_else(|| {
                        BACKEND.store(4, Ordering::Relaxed);
                        None
                    }),
            };
            if let Ok(mut procs) = LIVE_PROCS.lock() {
                procs.retain_mut(|p| p.try_wait().ok().flatten().is_none());
                if let Some(c) = child {
                    procs.push(c);
                }
            }
        }
    }
}

/// Acoustic low-frequency cardiac double-thump ("lub-dub") waveform emitted during heart-attack death.
#[derive(Clone, Copy, Debug)]
pub struct CardiacPulse {
    sample_rate: u32,
}

impl CardiacPulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Secondary constructor calibrated at 8000 Hz mono for resonant sub-bass chest thumps.
    pub const fn studio() -> Self {
        Self::new(8000)
    }
}

impl AcousticWave for CardiacPulse {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        let lub = if (0.0..0.16).contains(&t) {
            let u = t / 0.16;
            let env = (u * PI).sin() * (-11.0 * t).exp();
            let phase = 2.0 * PI * (64.0 * t - 88.0 * t * t);
            env * (0.75 * phase.sin() + 0.25 * (0.5 * phase).sin())
        } else {
            0.0
        };
        let dub = if (0.20..0.35).contains(&t) {
            let dt = t - 0.20;
            let u = dt / 0.15;
            let env = 0.85 * (u * PI).sin() * (-13.0 * dt).exp();
            let phase = 2.0 * PI * (54.0 * dt - 74.0 * dt * dt);
            env * (0.70 * phase.sin() + 0.30 * (0.5 * phase).sin())
        } else {
            0.0
        };
        (lub + dub).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        if SoundStage::active() {
            static WAVE_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = WAVE_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_cardiac_pulse.wav");
        } else {
            let _ = self.amplitude_at(0.05);
        }
    }
}

/// Acoustic decorator overlaying a subtle clinical pulse-monitor beep atop an underlying cardiac waveform.
#[derive(Clone, Copy, Debug)]
pub struct MonitoredPulse<W: AcousticWave = CardiacPulse> {
    origin: W,
    frequency_hz: f32,
    gain: f32,
}

impl<W: AcousticWave> MonitoredPulse<W> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: W, frequency_hz: f32, gain: f32) -> Self {
        Self {
            origin,
            frequency_hz,
            gain,
        }
    }
}

impl MonitoredPulse<CardiacPulse> {
    /// Secondary constructor decorating a studio cardiac pulse with a C6 (1046.5 Hz) clinical monitor beep.
    pub const fn clinical() -> Self {
        Self::new(CardiacPulse::studio(), 1046.5, 0.22)
    }
}

impl<W: AcousticWave> AcousticWave for MonitoredPulse<W> {
    fn sample_rate(&self) -> u32 {
        self.origin.sample_rate()
    }

    fn amplitude_at(&self, t: f32) -> f32 {
        let base = self.origin.amplitude_at(t);
        let beep = if (0.01..0.14).contains(&t) {
            let u = (t - 0.01) / 0.13;
            let env = (u * PI).sin().powi(2) * (-4.5 * u).exp();
            let tone = 2.0 * PI * self.frequency_hz * t;
            self.gain * env * (0.88 * tone.sin() + 0.12 * (2.0 * tone).sin())
        } else {
            0.0
        };
        (base + beep).clamp(-1.0, 1.0)
    }

    fn thump(&self) {
        if SoundStage::active() {
            static MONITOR_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = MONITOR_CACHE.get_or_init(|| self.riff_wave());
            self.play_cached_wav(bytes, "low_cube_monitored_pulse.wav");
        } else {
            let _ = self.amplitude_at(0.05);
        }
    }
}
