use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
#[link(name = "winmm")]
extern "system" {
    fn PlaySoundA(psz_sound: *const u8, hmod: *mut core::ffi::c_void, fdw_sound: u32) -> i32;
}

/// Immutable PCM RIFF payload queued for asynchronous hardware audio playback.
struct SoundJob {
    bytes: Vec<u8>,
    filename: String,
    overwrite: bool,
}

impl SoundJob {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(bytes: Vec<u8>, filename: String, overwrite: bool) -> Self {
        Self { bytes, filename, overwrite }
    }

    fn play_blocking(self) {
        #[cfg(target_os = "windows")]
        unsafe {
            use std::sync::Mutex;
            static LIVE_WAV: Mutex<Vec<u8>> = Mutex::new(Vec::new());
            let _ = (&self.filename, self.overwrite);
            if let Ok(mut buf) = LIVE_WAV.lock() {
                let _ = PlaySoundA(core::ptr::null(), core::ptr::null_mut(), 0);
                buf.clear();
                buf.extend_from_slice(&self.bytes);
                let _ = PlaySoundA(buf.as_ptr(), core::ptr::null_mut(), 0x0001 | 0x0002 | 0x0004);
            }
        }
        #[cfg(target_os = "linux")]
        {
            use std::process::{Child, Command, Stdio};
            use std::sync::atomic::{AtomicU8, Ordering};
            use std::sync::Mutex;
            static BACKEND: AtomicU8 = AtomicU8::new(0);
            static LIVE_PROCS: Mutex<Vec<Child>> = Mutex::new(Vec::new());
            let path = std::env::temp_dir().join(&self.filename);
            if self.overwrite || !path.exists() {
                let _ = std::fs::write(&path, &self.bytes);
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
                while procs.len() >= 4 {
                    let mut oldest = procs.remove(0);
                    let _ = oldest.kill();
                    let _ = oldest.wait();
                }
                if let Some(c) = child {
                    procs.push(c);
                }
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            let _ = (self.bytes.len(), self.filename.len(), self.overwrite);
        }
    }
}

/// Non-blocking acoustic dispatch queue decoupling 120 Hz simulation ticks from OS audio syscalls.
#[derive(Clone, Copy, Debug)]
pub(super) struct SoundOutbox {
    armed: bool,
}

impl SoundOutbox {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(armed: bool) -> Self {
        Self { armed }
    }

    fn drain_loop(rx: Receiver<SoundJob>) {
        while let Ok(mut job) = rx.recv() {
            while let Ok(newer) = rx.try_recv() {
                job = newer;
            }
            job.play_blocking();
        }
    }

    pub(super) fn submit(self, bytes: &[u8], filename: &str, overwrite: bool) {
        if !self.armed {
            let _ = (bytes.len(), filename.len(), overwrite);
            return;
        }
        static QUEUE: OnceLock<SyncSender<SoundJob>> = OnceLock::new();
        let tx = QUEUE.get_or_init(|| {
            let (tx, rx) = mpsc::sync_channel(2);
            let _ = std::thread::Builder::new()
                .name("low-cube-acoustics".to_owned())
                .spawn(move || Self::drain_loop(rx));
            tx
        });
        let _ = tx.try_send(SoundJob::new(bytes.to_vec(), filename.to_owned(), overwrite));
    }
}
