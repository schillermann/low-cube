use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use super::deck::{HullDriveDemand, SoundDeck};
use super::rain_patter::ShowerDemand;
use super::sink::{WaveSink, CHUNK_SAMPLES};
use super::tape::TapeSample;
use super::voice::BurstVoice;

const IDLE_KEEPALIVE_CHUNKS: u16 = 150;
const PREBUFFER_CHUNKS: usize = 3;

/// Non-blocking message delivered from 120 Hz gameplay ticks to the real-time audio stream thread.
enum AudioOrder {
    Drive(HullDriveDemand),
    Shower(ShowerDemand),
    TapeBurst(TapeSample, f32, (f32, f32)),
    RiffBurst(Vec<u8>),
}

/// Non-blocking acoustic dispatch queue feeding the real-time polyphonic `SoundDeck` stream.
#[derive(Clone, Copy, Debug)]
pub(super) struct SoundOutbox {
    armed: bool,
}

impl SoundOutbox {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(armed: bool) -> Self {
        Self { armed }
    }

    fn apply_order(self, deck: &mut SoundDeck, order: AudioOrder) {
        let _ = self.armed;
        match order {
            AudioOrder::Drive(demand) => deck.update_drive(demand),
            AudioOrder::Shower(demand) => deck.update_shower(demand),
            AudioOrder::TapeBurst(tape, pitch, gains) => {
                deck.push_burst(BurstVoice::from_tape(tape, pitch, gains));
            }
            AudioOrder::RiffBurst(wav) => {
                if let Some(voice) = BurstVoice::from_riff(&wav) {
                    deck.push_burst(voice);
                }
            }
        }
    }

    fn stream_loop(self, rx: Receiver<AudioOrder>) {
        let mut deck = SoundDeck::studio();
        let mut sink = WaveSink::open();
        let mut chunk = [0i16; CHUNK_SAMPLES];
        let step = Duration::from_millis(20);
        let mut next_deadline = Instant::now();
        let mut keepalive: u16 = 0;

        loop {
            if !deck.active() && keepalive == 0 {
                let Ok(first) = rx.recv() else { break };
                self.apply_order(&mut deck, first);
                while let Ok(more) = rx.try_recv() {
                    self.apply_order(&mut deck, more);
                }
                if !deck.active() {
                    continue;
                }
                keepalive = IDLE_KEEPALIVE_CHUNKS;
                // Pre-fill 60 ms cushion so OS audio ringbuffer never underruns on scheduler jitter
                for _ in 0..PREBUFFER_CHUNKS {
                    while let Ok(more) = rx.try_recv() {
                        self.apply_order(&mut deck, more);
                    }
                    deck.mix_stereo_into(&mut chunk);
                    let _ = sink.push_chunk(&chunk);
                }
                next_deadline = Instant::now() + step;
                continue;
            }

            while let Ok(order) = rx.try_recv() {
                self.apply_order(&mut deck, order);
            }
            if deck.active() {
                keepalive = IDLE_KEEPALIVE_CHUNKS;
            } else {
                keepalive = keepalive.saturating_sub(1);
            }
            deck.mix_stereo_into(&mut chunk);
            let hw_paced = sink.push_chunk(&chunk);
            if !hw_paced {
                let now = Instant::now();
                if next_deadline > now {
                    std::thread::sleep(next_deadline - now);
                } else if now - next_deadline > Duration::from_millis(60) {
                    next_deadline = now;
                }
                next_deadline += step;
            }
        }
    }

    fn channel(self) -> &'static SyncSender<AudioOrder> {
        static STREAM_TX: OnceLock<SyncSender<AudioOrder>> = OnceLock::new();
        STREAM_TX.get_or_init(|| {
            let (tx, rx) = mpsc::sync_channel(128);
            let _ = std::thread::Builder::new()
                .name("low-cube-acoustics".to_owned())
                .spawn(move || self.stream_loop(rx));
            tx
        })
    }

    pub(super) fn submit(self, bytes: &[u8], filename: &str, overwrite: bool) {
        if !self.armed {
            let _ = (bytes.len(), filename.len(), overwrite);
            return;
        }
        let _ = self.channel().try_send(AudioOrder::RiffBurst(bytes.to_vec()));
    }

    pub(super) fn submit_tape(self, tape: TapeSample, pitch: f32, gains: (f32, f32)) {
        if !self.armed {
            let _ = (tape.frame_count(), pitch, gains);
            return;
        }
        let _ = self.channel().try_send(AudioOrder::TapeBurst(tape, pitch, gains));
    }

    pub(super) fn submit_drive(self, demand: HullDriveDemand) {
        if !self.armed {
            let _ = demand;
            return;
        }
        let _ = self.channel().try_send(AudioOrder::Drive(demand));
    }

    pub(super) fn submit_shower(self, demand: ShowerDemand) {
        if !self.armed {
            let _ = demand;
            return;
        }
        let _ = self.channel().try_send(AudioOrder::Shower(demand));
    }
}
