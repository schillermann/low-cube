use std::f32::consts::PI;
#[cfg(not(test))]
use std::sync::OnceLock;

#[cfg(all(target_os = "windows", not(test)))]
#[link(name = "winmm")]
extern "system" {
    fn PlaySoundA(psz_sound: *const u8, hmod: *mut core::ffi::c_void, fdw_sound: u32) -> i32;
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

    fn sample_amplitude(&self, t: f32) -> f32 {
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

    /// Synthesizes a 16-bit mono PCM RIFF WAVE byte stream containing one booming "lub-dub" heartbeat.
    pub fn riff_wave(&self) -> Vec<u8> {
        let total_samples = (self.sample_rate * 55) / 100;
        let data_bytes = total_samples * 2;
        let mut wav = Vec::with_capacity(44 + data_bytes as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_bytes).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&self.sample_rate.to_le_bytes());
        wav.extend_from_slice(&(self.sample_rate * 2).to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_bytes.to_le_bytes());
        let inv_rate = 1.0 / (self.sample_rate as f32);
        for idx in 0..total_samples {
            let amp = self.sample_amplitude((idx as f32) * inv_rate);
            let pcm = (amp * 31500.0).round() as i16;
            wav.extend_from_slice(&pcm.to_le_bytes());
        }
        wav
    }

    /// Triggers asynchronous playback of the booming heartbeat sound without blocking the simulation loop.
    pub fn thump(&self) {
        #[cfg(not(test))]
        {
            static WAVE_CACHE: OnceLock<Vec<u8>> = OnceLock::new();
            let bytes = WAVE_CACHE.get_or_init(|| self.riff_wave());
            #[cfg(target_os = "windows")]
            unsafe {
                let _ = PlaySoundA(bytes.as_ptr(), core::ptr::null_mut(), 0x0001 | 0x0002 | 0x0004);
            }
            #[cfg(target_os = "linux")]
            {
                use std::process::{Command, Stdio};
                static PATH_CACHE: OnceLock<std::path::PathBuf> = OnceLock::new();
                let path = PATH_CACHE.get_or_init(|| {
                    let file = std::env::temp_dir().join("low_cube_cardiac_pulse.wav");
                    let _ = std::fs::write(&file, bytes);
                    file
                });
                if Command::new("paplay").arg(path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().is_err()
                    && Command::new("pw-play").arg(path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().is_err()
                {
                    let _ = Command::new("aplay").arg("-q").arg(path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
                }
            }
        }
        #[cfg(test)]
        {
            let _ = self.sample_amplitude(0.05);
        }
    }
}
