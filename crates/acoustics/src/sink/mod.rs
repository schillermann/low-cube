#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod imp {
    use super::CHUNK_SAMPLES;

    pub(super) struct PlatformSink {
        armed: bool,
    }

    impl PlatformSink {
        const fn new(armed: bool) -> Self {
            Self { armed }
        }
        pub(super) fn open() -> Self {
            Self::new(false)
        }
        pub(super) fn write_chunk(&mut self, samples: &[i16; CHUNK_SAMPLES]) -> bool {
            let _ = (self.armed, samples.len());
            false
        }
    }
}

pub(super) const CHUNK_SAMPLES: usize = 960;

/// Continuous real-time 24 kHz 16-bit stereo OS hardware sink.
pub(super) struct WaveSink {
    inner: imp::PlatformSink,
}

impl WaveSink {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(inner: imp::PlatformSink) -> Self {
        Self { inner }
    }

    pub(super) fn open() -> Self {
        Self::new(imp::PlatformSink::open())
    }

    /// Writes 20 ms of stereo PCM; returns `true` when hardware-paced (Win32 `waveOut`), `false` when timer-paced.
    pub(super) fn push_chunk(&mut self, samples: &[i16; CHUNK_SAMPLES]) -> bool {
        self.inner.write_chunk(samples)
    }
}
