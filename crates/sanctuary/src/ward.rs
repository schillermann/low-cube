use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{
    CompanionPresence, CraftPose, DrivenCraft, GoreStamp, PilotCraft, PlanetaryWorld,
};
use crate::glide::CompanionGlide;
use crate::seal::SessionSeal;

const MAX_HEAD_REACH_FROM_BODY: f32 = 5.2;
const MAX_CRAFT_PARK_REACH: f32 = 12.0;
const MAX_DIG_BURST_TOKENS: f32 = 2.0;
const DIG_REFILL_PER_SEC: f32 = 5.0;
const INITIAL_STARTER_BLOCKS: u32 = 64;

/// Authoritative per-guest anti-cheat vitals: chronicle ack, block tally, dig cadence, gore stamp, and AoI cursor.
pub struct WardVitals {
    ledger: (u32, u32),
    cadence: (f32, f32),
    gore: GoreStamp,
    sight_cursor: usize,
}

impl WardVitals {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ledger: (u32, u32), cadence: (f32, f32), gore: GoreStamp, sight_cursor: usize) -> Self {
        Self { ledger, cadence, gore, sight_cursor }
    }

    pub const fn fresh() -> Self {
        Self::new((0, INITIAL_STARTER_BLOCKS), (MAX_DIG_BURST_TOKENS, 0.0), GoreStamp::calm(), 0)
    }

    pub fn advance(&mut self, dt: f32) {
        let next_tokens = (self.cadence.0 + dt * DIG_REFILL_PER_SEC).min(MAX_DIG_BURST_TOKENS);
        self.cadence = (next_tokens, self.cadence.1 + dt);
    }

    pub fn alive(&self) -> bool {
        self.cadence.1 < 12.0
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
    }

    pub fn alive(&self) -> bool {
        self.vitals.alive()
    }

    pub const fn ack_seq(&self) -> u32 {
        self.vitals.ledger.0
    }

    pub const fn gore_stamp(&self) -> GoreStamp {
        self.vitals.gore
    }

    pub fn record_gore(&mut self, impulse: Vec3) {
        self.vitals.gore = self.vitals.gore.advanced(impulse);
    }

    pub fn presence(&self) -> CompanionPresence {
        self.glide.authoritative_presence()
    }

    pub fn anchor_position(&self) -> Vec3 {
        self.presence().transfer(|_, kin, _, _| kin.transfer(|pos, _, _, _, _| pos))
    }

    pub fn sight_cursor_mut(&mut self) -> &mut usize {
        &mut self.vitals.sight_cursor
    }

    pub fn presence_at(&self, render_secs: f64) -> CompanionPresence {
        self.glide.presence_at(render_secs)
    }

    pub fn craft_at(&self, render_secs: f64) -> PilotCraft {
        self.glide.craft_at(render_secs)
    }

    pub fn pedestrian_observer(&self, world: &dyn PlanetaryWorld) -> Option<(Vec3, bool)> {
        if !self.glide.live() || matches!(self.glide.authoritative_craft(), PilotCraft::Driving(_)) {
            return None;
        }
        self.presence().transfer(|_, kin, _, _| {
            kin.transfer(|pos, _, _, eye_h, _| {
                world.woodland_habitat(pos).then_some((pos, eye_h > 0.05 && eye_h < 0.62))
            })
        })
    }

    pub fn push_authoritative_craft(&self, out: &mut Vec<DrivenCraft>) {
        self.glide.authoritative_craft().push_active(out);
    }

    pub fn push_authoritative_craft_with_speed(&self, out: &mut Vec<(DrivenCraft, f32)>) {
        self.glide.push_craft_with_speed(out);
    }

    pub fn push_craft_at(&self, render_secs: f64, out: &mut Vec<DrivenCraft>) {
        self.glide.craft_at(render_secs).push_active(out);
    }

    pub fn into_pilot_craft(self) -> PilotCraft {
        self.glide.into_authoritative_craft()
    }

    pub fn open_datagram(&mut self, src: SocketAddr, frame: &[u8]) -> Option<Vec<u8>> {
        let (_, body) = self.seal.open_frame(frame)?;
        self.peer = src;
        self.vitals.cadence.1 = 0.0;
        Some(body)
    }

    pub fn absorb_pulse(
        &mut self,
        companion_id: u32,
        ack_seq: u32,
        now: f64,
        presence: CompanionPresence,
        craft: PilotCraft,
        world: &dyn PlanetaryWorld,
    ) {
        self.vitals.ledger.0 = self.vitals.ledger.0.max(ack_seq);
        let rec_pres = self.glide.authoritative_presence().reconciled(companion_id, presence, world);
        let rec_craft = self.glide.authoritative_craft().reconciled(companion_id, craft, world);
        self.glide = self.glide.absorbed(now, rec_pres, rec_craft);
    }

    pub fn absorb_standby_pulse(
        &mut self,
        companion_id: u32,
        ack_seq: u32,
        now: f64,
        presence: CompanionPresence,
        craft: PilotCraft,
        max_extent: f32,
    ) {
        self.vitals.ledger.0 = self.vitals.ledger.0.max(ack_seq);
        let valid_pos = presence.transfer(|_, kin, _, _| {
            kin.transfer(|p, _, _, _, _| p.is_finite() && p.abs().max_element() <= max_extent)
        });
        let rec_pres = if valid_pos {
            presence.with_slot(companion_id)
        } else {
            self.glide.authoritative_presence().with_slot(companion_id)
        };
        let rec_craft = match craft {
            PilotCraft::Pedestrian => PilotCraft::Pedestrian,
            PilotCraft::Driving(dc) if dc.pose().origin().is_finite() && dc.identity().assigned() => {
                PilotCraft::Driving(dc.with_slot(companion_id))
            }
            PilotCraft::Driving(_) => self.glide.authoritative_craft().with_slot(companion_id),
        };
        self.glide = self.glide.absorbed(now, rec_pres, rec_craft);
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
        if self.vitals.cadence.0 >= 1.0 {
            self.vitals.cadence.0 -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn credit_blocks(&mut self, count: u32) {
        self.vitals.ledger.1 = self.vitals.ledger.1.saturating_add(count);
    }

    pub fn spend_block(&mut self) -> bool {
        self.spend_blocks(1)
    }

    pub fn spend_blocks(&mut self, count: u32) -> bool {
        if count > 0 && self.vitals.ledger.1 >= count {
            self.vitals.ledger.1 -= count;
            true
        } else {
            false
        }
    }

    pub fn send_sealed(&mut self, socket: &UdpSocket, companion_id: u32, payload: &[u8], scratch: &mut Vec<u8>) {
        self.seal.seal(companion_id, payload, scratch);
        let _ = socket.send_to(scratch, self.peer);
    }

    pub fn seal_outbound(&mut self, companion_id: u32, payload: &[u8], out: &mut Vec<u8>) -> SocketAddr {
        self.seal.seal(companion_id, payload, out);
        self.peer
    }
}
