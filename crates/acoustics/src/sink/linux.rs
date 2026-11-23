use std::io::Write;
use std::process::{Child, Command, Stdio};
use super::CHUNK_SAMPLES;

enum PipeStream {
    Connected(Child),
    Disconnected,
}

pub(super) struct PlatformSink {
    pipe: PipeStream,
    backend_idx: u8,
    byte_scratch: Vec<u8>,
}

impl PlatformSink {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(pipe: PipeStream, backend_idx: u8, byte_scratch: Vec<u8>) -> Self {
        Self { pipe, backend_idx, byte_scratch }
    }

    pub(super) fn open() -> Self {
        let mut sink = Self::new(PipeStream::Disconnected, 0, Vec::with_capacity(CHUNK_SAMPLES * 2));
        sink.try_connect();
        sink
    }

    fn spawn_backend(&self, kind: u8) -> PipeStream {
        let _ = self.backend_idx;
        let mut cmd = match kind {
            0 => {
                let mut c = Command::new("pw-cat");
                c.args(["--playback", "--raw", "--format", "s16", "--rate", "24000", "--channels", "2", "--latency", "60ms", "-"]);
                c
            }
            1 => {
                let mut c = Command::new("pacat");
                c.args(["--playback", "--raw", "--format=s16le", "--rate=24000", "--channels=2", "--latency-msec=60"]);
                c
            }
            2 => {
                let mut c = Command::new("aplay");
                c.args(["-q", "-t", "raw", "-f", "S16_LE", "-r", "24000", "-c", "2", "--buffer-size=4800", "--period-size=960", "-"]);
                c
            }
            _ => return PipeStream::Disconnected,
        };
        match cmd.stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
            Ok(child) => PipeStream::Connected(child),
            Err(_) => PipeStream::Disconnected,
        }
    }

    fn try_connect(&mut self) {
        while self.backend_idx < 3 {
            let cur = self.backend_idx;
            self.backend_idx += 1;
            let candidate = self.spawn_backend(cur);
            if matches!(candidate, PipeStream::Connected(_)) {
                self.pipe = candidate;
                return;
            }
        }
    }

    pub(super) fn write_chunk(&mut self, samples: &[i16; CHUNK_SAMPLES]) -> bool {
        self.byte_scratch.clear();
        for &s in samples {
            self.byte_scratch.extend_from_slice(&s.to_le_bytes());
        }
        while matches!(self.pipe, PipeStream::Connected(_)) {
            let ok = match &mut self.pipe {
                PipeStream::Connected(child) => {
                    let alive = child.try_wait().ok().flatten().is_none();
                    alive
                        && child
                            .stdin
                            .as_mut()
                            .is_some_and(|stdin| stdin.write_all(&self.byte_scratch).is_ok())
                }
                PipeStream::Disconnected => false,
            };
            if ok {
                return false;
            }
            if let PipeStream::Connected(mut dead) =
                core::mem::replace(&mut self.pipe, PipeStream::Disconnected)
            {
                let _ = dead.kill();
                let _ = dead.wait();
            }
            self.try_connect();
        }
        false
    }
}
