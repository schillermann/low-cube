use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{CompanionPresence, CraftPose, DrivenCraft, PilotCraft, PlanetaryWorld};
use crate::glide::CompanionGlide;
use crate::seal::SessionSeal;

const MAX_HEAD_REACH_FROM_BODY: f32 = 5.2;
const MAX_CRAFT_PARK_REACH: f32 = 12.0;
const MAX_DIG_BURST_TOKENS: f32 = 2.0;
const DIG_REFILL_PER_SEC: f32 = 5.0;
const INITIAL_STARTER_BLOCKS: u32 = 64;

/// Authoritative per-guest anti-cheat vitals: chronicle ack, dig rate tokens, timeout, and block tally.
pub struct WardVitals {
    ack_seq: u32,
    dig_tokens: f32,
    silence_secs: f32,
    earned_blocks: u32,
}

impl WardVitals {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ack_seq: u32, dig_tokens: f32, silence_secs: f32, earned_blocks: u32) -> Self {
        Self { ack_seq, dig_tokens, silence_secs, earned_blocks }
    }

    pub const fn fresh() -> Self {
        Self::new(0, MAX_DIG_BURST_TOKENS, 0.0, INITIAL_STARTER_BLOCKS)
    }

    pub fn advance(&mut self, dt: f32) {
        self.silence_secs += dt;
        self.dig_tokens = (self.dig_tokens + dt * DIG_REFILL_PER_SEC).min(MAX_DIG_BURST_TOKENS);
    }

    pub fn alive(&self) -> bool {
        self.silence_secs < 12.0
    }
}

/// Authoritative P2P companion session sealing UDP frames and vetting reach, dig cadence, and block credits.
pub struct GuestWard {
    seal: SessionSeal,
    peer: SocketAddr,
    glide: CompanionGlide,
    vitals: WardVitals,
}

impl GuestWard {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_glide(
        seal: SessionSeal,
        peer: SocketAddr,
        glide: CompanionGlide,
        vitals: WardVitals,
    ) -> Self {
        Self { seal, peer, glide, vitals }
    }

    /// Secondary constructor anchoring an admitted companion awaiting its first live pulse.
    pub fn new(
        seal: SessionSeal,
        peer: SocketAddr,
        presence: CompanionPresence,
        vitals: WardVitals,
    ) -> Self {
        Self::with_glide(seal, peer, CompanionGlide::awaiting_first(presence), vitals)
    }

    pub fn step(&mut self, dt: f32) {
        self.vitals.advance(dt);
        self.glide.advance_tick(dt);
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) {
        self.glide.blend_subtick(subtick_secs);
    }

    pub fn alive(&self) -> bool {
        self.vitals.alive()
    }

    pub const fn ack_seq(&self) -> u32 {
        self.vitals.ack_seq
    }

    pub const fn presence(&self) -> CompanionPresence {
        self.glide.authoritative_presence()
    }

    pub fn smooth_presence(&self) -> CompanionPresence {
        self.glide.smooth_presence()
    }

    pub fn push_authoritative_craft(&self, out: &mut Vec<DrivenCraft>) {
        self.glide.authoritative_craft().push_active(out);
    }

    pub fn push_smooth_craft(&self, out: &mut Vec<DrivenCraft>) {
        self.glide.smooth_craft().push_active(out);
    }

    pub fn into_pilot_craft(self) -> PilotCraft {
        self.glide.into_authoritative_craft()
    }

    pub fn open_datagram(&mut self, src: SocketAddr, frame: &[u8]) -> Option<Vec<u8>> {
        let (_, body) = self.seal.open_frame(frame)?;
        self.peer = src;
        self.vitals.silence_secs = 0.0;
        Some(body)
    }

    pub fn absorb_pulse(
        &mut self,
        companion_id: u32,
        ack_seq: u32,
        presence: CompanionPresence,
        craft: PilotCraft,
        world: &dyn PlanetaryWorld,
    ) {
        self.vitals.ack_seq = self.vitals.ack_seq.max(ack_seq);
        let rec_pres = self.glide.authoritative_presence().reconciled(companion_id, presence, world);
        let rec_craft = self.glide.authoritative_craft().reconciled(companion_id, craft, world);
        self.glide.absorb(rec_pres, rec_craft);
    }

    pub fn eye_within_reach(&self, eye: Vec3, direction: Vec3) -> bool {
        if !eye.is_finite() || !direction.is_finite() || direction.length_squared() < 1e-4 {
            return false;
        }
        self.presence().within_reach(eye, MAX_HEAD_REACH_FROM_BODY)
    }

    pub fn pose_within_reach(&self, pose: CraftPose) -> bool {
        pose.origin().is_finite() && self.presence().within_reach(pose.origin(), MAX_CRAFT_PARK_REACH)
    }

    pub fn consume_harvest_token(&mut self) -> bool {
        if self.vitals.dig_tokens >= 1.0 {
            self.vitals.dig_tokens -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn credit_blocks(&mut self, count: u32) {
        self.vitals.earned_blocks = self.vitals.earned_blocks.saturating_add(count);
    }

    pub fn spend_block(&mut self) -> bool {
        if self.vitals.earned_blocks > 0 {
            self.vitals.earned_blocks -= 1;
            true
        } else {
            false
        }
    }

    pub fn send_sealed(&mut self, socket: &UdpSocket, companion_id: u32, payload: &[u8], scratch: &mut Vec<u8>) {
        self.seal.seal(companion_id, payload, scratch);
        let _ = socket.send_to(scratch, self.peer);
    }
}
