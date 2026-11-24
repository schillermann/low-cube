use glam::Vec3;
use world::{Callsign, CosmicFrame, GoreStamp, SatchelManifest, SurvivorRecord};
use crate::survivor_ledger::SurvivorStore;

const MAX_DIG_BURST_TOKENS: f32 = 2.0;
const DIG_REFILL_PER_SEC: f32 = 5.0;
const INITIAL_STARTER_BLOCKS: u32 = 64;

/// Authoritative survivor credentials, server-resolved callsign, and satchel held by a `GuestWard` (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WardCredential {
    pubkey: [u8; 32],
    callsign: Callsign,
    manifest: SatchelManifest,
}

impl WardCredential {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(pubkey: [u8; 32], callsign: Callsign, manifest: SatchelManifest) -> Self {
        Self { pubkey, callsign, manifest }
    }

    pub const fn pubkey(&self) -> [u8; 32] {
        self.pubkey
    }

    pub const fn callsign(&self) -> Callsign {
        self.callsign
    }

    pub const fn manifest(&self) -> SatchelManifest {
        self.manifest
    }

    pub fn manifest_mut(&mut self) -> &mut SatchelManifest {
        &mut self.manifest
    }

    pub fn commit_to(
        &self,
        frame: CosmicFrame,
        position: Vec3,
        final_checkpoint: bool,
        store: &mut dyn SurvivorStore,
    ) {
        if self.pubkey != [0u8; 32] {
            let record = SurvivorRecord::with_callsign(frame, position, self.manifest, self.callsign);
            store.commit(self.pubkey, record, final_checkpoint);
        }
    }
}

/// Authoritative per-guest anti-cheat vitals and persistent survivor satchel (`<= 4` fields).
pub struct WardVitals {
    ledger: (u32, u32),
    cadence: (f32, f32),
    cursors: (GoreStamp, usize),
    credential: WardCredential,
}

impl WardVitals {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_credential(
        ledger: (u32, u32),
        cadence: (f32, f32),
        cursors: (GoreStamp, usize),
        credential: WardCredential,
    ) -> Self {
        Self { ledger, cadence, cursors, credential }
    }

    pub fn new(ledger: (u32, u32), cadence: (f32, f32), gore: GoreStamp, sight_cursor: usize) -> Self {
        Self::with_credential(
            ledger,
            cadence,
            (gore, sight_cursor),
            WardCredential::new([0u8; 32], Callsign::unnamed(), SatchelManifest::starter_kit()),
        )
    }

    pub fn fresh() -> Self {
        Self::for_survivor([0u8; 32], SatchelManifest::starter_kit())
    }

    pub const fn for_survivor(pubkey: [u8; 32], manifest: SatchelManifest) -> Self {
        Self::named_survivor(pubkey, Callsign::unnamed(), manifest)
    }

    pub const fn named_survivor(pubkey: [u8; 32], callsign: Callsign, manifest: SatchelManifest) -> Self {
        Self::with_credential(
            (0, INITIAL_STARTER_BLOCKS),
            (MAX_DIG_BURST_TOKENS, 0.0),
            (GoreStamp::calm(), 0),
            WardCredential::new(pubkey, callsign, manifest),
        )
    }

    pub fn advance(&mut self, dt: f32) {
        let next_tokens = (self.cadence.0 + dt * DIG_REFILL_PER_SEC).min(MAX_DIG_BURST_TOKENS);
        self.cadence = (next_tokens, self.cadence.1 + dt);
    }

    pub fn reset_heartbeat(&mut self) { self.cadence.1 = 0.0; }
    pub fn alive(&self) -> bool { self.cadence.1 < 12.0 }
    pub const fn ack_seq(&self) -> u32 { self.ledger.0 }

    pub fn advance_ack(&mut self, ack_seq: u32) {
        if ack_seq > self.ledger.0 {
            self.ledger.0 = ack_seq;
        }
    }

    pub fn consume_harvest_token(&mut self) -> bool {
        if self.cadence.0 >= 1.0 {
            self.cadence.0 -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn credit_blocks(&mut self, count: u32) { self.ledger.1 = self.ledger.1.saturating_add(count); }

    pub fn spend_blocks(&mut self, count: u32) -> bool {
        if count > 0 && self.ledger.1 >= count {
            self.ledger.1 -= count;
            true
        } else {
            false
        }
    }

    pub const fn gore_stamp(&self) -> GoreStamp { self.cursors.0 }
    pub fn record_gore(&mut self, impulse: Vec3) { self.cursors.0 = self.cursors.0.advanced(impulse); }
    pub fn sight_cursor_mut(&mut self) -> &mut usize { &mut self.cursors.1 }
    pub const fn guest_pubkey(&self) -> [u8; 32] { self.credential.pubkey() }
    pub const fn callsign(&self) -> Callsign { self.credential.callsign() }
    pub const fn manifest(&self) -> SatchelManifest { self.credential.manifest() }
    pub fn manifest_mut(&mut self) -> &mut SatchelManifest { self.credential.manifest_mut() }

    pub fn commit_checkpoint(
        &self,
        frame: CosmicFrame,
        position: Vec3,
        final_checkpoint: bool,
        store: &mut dyn SurvivorStore,
    ) {
        self.credential.commit_to(frame, position, final_checkpoint, store);
    }
}
