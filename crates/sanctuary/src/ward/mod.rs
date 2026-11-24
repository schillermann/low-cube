mod vitals;

use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{
    Callsign, CompanionPresence, CosmicFrame, CraftPose, DrivenCraft, GoreStamp, GravityBubble,
    PilotCraft, PlanetaryWorld, SatchelManifest, VesselBerth,
};
use crate::glide::CompanionGlide;
use crate::seal::SessionSeal;
use crate::survivor_ledger::SurvivorStore;
pub use self::vitals::{WardCredential, WardVitals};

const MAX_HEAD_REACH_FROM_BODY: f32 = 5.2;
const MAX_CRAFT_PARK_REACH: f32 = 12.0;

/// Authoritative P2P companion session sealing UDP frames and vetting reach, Hill-bubble frame, berth, cadence, and credits.
pub struct GuestWard {
    seal: SessionSeal,
    locus: (SocketAddr, CosmicFrame),
    glide: CompanionGlide,
    vitals: WardVitals,
}

impl GuestWard {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_locus(
        seal: SessionSeal,
        locus: (SocketAddr, CosmicFrame),
        glide: CompanionGlide,
        vitals: WardVitals,
    ) -> Self {
        Self { seal, locus, glide, vitals }
    }

    pub fn with_glide(seal: SessionSeal, peer: SocketAddr, glide: CompanionGlide, vitals: WardVitals) -> Self {
        Self::with_locus(seal, (peer, CosmicFrame::earth()), glide, vitals)
    }

    /// Secondary constructor anchoring an admitted companion awaiting its first live pulse.
    pub fn new(seal: SessionSeal, peer: SocketAddr, presence: CompanionPresence, vitals: WardVitals) -> Self {
        Self::with_glide(seal, peer, CompanionGlide::awaiting_first(presence), vitals)
    }

    pub fn step(&mut self, dt: f32) { self.vitals.advance(dt); }
    pub fn alive(&self) -> bool { self.vitals.alive() }
    pub const fn live_presence(&self) -> bool { self.glide.live() }
    pub const fn established_traffic(&self) -> bool { self.seal.established_traffic() }
    pub const fn guest_pubkey(&self) -> [u8; 32] { self.vitals.guest_pubkey() }
    pub const fn callsign(&self) -> Callsign { self.vitals.callsign() }
    pub const fn manifest(&self) -> SatchelManifest { self.vitals.manifest() }
    pub fn manifest_mut(&mut self) -> &mut SatchelManifest { self.vitals.manifest_mut() }

    pub fn commit_checkpoint(
        &self,
        frame: CosmicFrame,
        position: Vec3,
        final_checkpoint: bool,
        store: &mut dyn SurvivorStore,
    ) {
        if self.seal.established_traffic() {
            self.vitals.commit_checkpoint(frame, position, final_checkpoint, store);
        }
    }
    pub const fn cosmic_frame(&self) -> CosmicFrame { self.locus.1 }
    pub const fn berth(&self) -> VesselBerth { self.glide.berth() }
    pub const fn ack_seq(&self) -> u32 { self.vitals.ack_seq() }
    pub const fn gore_stamp(&self) -> GoreStamp { self.vitals.gore_stamp() }
    pub fn record_gore(&mut self, impulse: Vec3) { self.vitals.record_gore(impulse); }
    pub fn presence(&self) -> CompanionPresence { self.glide.authoritative_presence() }
    pub fn anchor_position(&self) -> Vec3 { self.presence().transfer(|_, kin, _, _| kin.transfer(|pos, _, _, _, _| pos)) }
    pub fn sight_cursor_mut(&mut self) -> &mut usize { self.vitals.sight_cursor_mut() }
    pub fn presence_at(&self, render_secs: f64) -> CompanionPresence { self.glide.presence_at(render_secs) }
    pub fn presence_at_with_hull(&self, render_secs: f64, hull: Option<CraftPose>) -> CompanionPresence {
        self.glide.presence_at_with_hull(render_secs, hull)
    }
    pub fn craft_at(&self, render_secs: f64) -> PilotCraft { self.glide.craft_at(render_secs) }

    pub fn pedestrian_observer(&self, world: &dyn PlanetaryWorld) -> Option<(Vec3, bool)> {
        if !self.locus.1.earth_centered()
            || self.glide.berth().aboard_any()
            || !self.glide.live()
            || matches!(self.glide.authoritative_craft(), PilotCraft::Driving(_))
        {
            return None;
        }
        self.presence().transfer(|_, kin, _, _| {
            kin.transfer(|pos, _, _, eye_h, _| {
                world.woodland_habitat(pos).then_some((pos, eye_h > 0.05 && eye_h < 0.62))
            })
        })
    }

    pub fn push_authoritative_craft_with_speed(&self, out: &mut Vec<(CosmicFrame, DrivenCraft, f32)>) {
        let mut raw = Vec::with_capacity(1);
        self.glide.push_craft_with_speed(&mut raw);
        for (dc, speed) in raw {
            out.push((self.locus.1, dc, speed));
        }
    }

    pub fn into_pilot_craft(self) -> PilotCraft { self.glide.into_authoritative_craft() }

    pub fn open_datagram(&mut self, src: SocketAddr, frame: &[u8]) -> Option<Vec<u8>> {
        let (_, body) = self.seal.open_frame(frame)?;
        self.locus.0 = src;
        self.vitals.reset_heartbeat();
        Some(body)
    }

    pub fn absorb_pulse(
        &mut self,
        companion_id: u32,
        ack_seq: u32,
        now: f64,
        frame: CosmicFrame,
        berth: VesselBerth,
        presence: CompanionPresence,
        craft: PilotCraft,
        bubble: GravityBubble,
        world: &dyn PlanetaryWorld,
    ) {
        if berth.aboard_any() {
            self.absorb_standby_pulse(companion_id, ack_seq, now, frame, berth, presence, craft, bubble);
            return;
        }
        self.vitals.advance_ack(ack_seq);
        let frame_changed = self.locus.1 != frame;
        if frame == world.cosmic_frame() {
            let valid_in_world = presence.transfer(|_, kin, _, _| {
                kin.transfer(|pos, _, _, _, _| {
                    let clamped = world.confine_beneath_crust(pos);
                    clamped.is_finite() && (clamped - pos).length_squared() <= 36.0
                })
            });
            if !valid_in_world {
                return;
            }
            let rec_pres = presence.with_slot(companion_id);
            let rec_craft = self.glide.authoritative_craft().reconciled(companion_id, craft, world);
            self.manifest_mut().reconcile_from_presence(rec_pres);
            self.locus.1 = frame;
            self.glide = if frame_changed {
                CompanionGlide::anchored_with_berth(now, rec_pres, rec_craft, berth)
            } else {
                self.glide.absorbed_with_berth(now, rec_pres, rec_craft, berth)
            };
        } else {
            self.absorb_standby_pulse(companion_id, ack_seq, now, frame, berth, presence, craft, bubble);
        }
    }

    pub fn absorb_standby_pulse(
        &mut self,
        companion_id: u32,
        ack_seq: u32,
        now: f64,
        frame: CosmicFrame,
        berth: VesselBerth,
        presence: CompanionPresence,
        craft: PilotCraft,
        bubble: GravityBubble,
    ) {
        self.vitals.advance_ack(ack_seq);
        let valid_pos = presence.transfer(|_, kin, _, _| {
            kin.transfer(|p, _, _, _, _| {
                if berth.aboard_any() {
                    berth.valid_local_offset(p)
                } else {
                    bubble.valid_local_point(frame, p)
                }
            })
        });
        if !valid_pos {
            return;
        }
        let rec_pres = presence.with_slot(companion_id);
        self.manifest_mut().reconcile_from_presence(rec_pres);
        let rec_craft = match craft {
            PilotCraft::Pedestrian => PilotCraft::Pedestrian,
            PilotCraft::Driving(dc) if dc.identity().assigned() && bubble.valid_local_point(frame, dc.pose().origin()) => {
                PilotCraft::Driving(dc.with_slot(companion_id))
            }
            PilotCraft::Driving(_) => self.glide.authoritative_craft().with_slot(companion_id),
        };
        let frame_changed = self.locus.1 != frame && !berth.aboard_any();
        self.locus.1 = frame;
        self.glide = if frame_changed {
            CompanionGlide::anchored_with_berth(now, rec_pres, rec_craft, berth)
        } else {
            self.glide.absorbed_with_berth(now, rec_pres, rec_craft, berth)
        };
    }

    pub fn eye_within_reach(&self, eye: Vec3, direction: Vec3) -> bool {
        if self.glide.berth().aboard_any()
            || !eye.is_finite()
            || !direction.is_finite()
            || direction.length_squared() < 1e-4
        {
            return false;
        }
        self.presence().within_reach(eye, MAX_HEAD_REACH_FROM_BODY)
    }

    pub fn pose_within_reach(&self, pose: CraftPose) -> bool {
        if !pose.origin().is_finite() {
            return false;
        }
        if self.glide.berth().aboard_any() {
            return self.anchor_position().abs().max_element() <= MAX_CRAFT_PARK_REACH;
        }
        self.presence().within_reach(pose.origin(), MAX_CRAFT_PARK_REACH)
    }

    pub fn consume_harvest_token(&mut self) -> bool { self.vitals.consume_harvest_token() }
    pub fn credit_blocks(&mut self, count: u32) { self.vitals.credit_blocks(count); }
    pub fn spend_block(&mut self) -> bool { self.spend_blocks(1) }
    pub fn spend_blocks(&mut self, count: u32) -> bool { self.vitals.spend_blocks(count) }

    pub fn send_sealed(&mut self, socket: &UdpSocket, companion_id: u32, payload: &[u8], scratch: &mut Vec<u8>) {
        self.seal.seal(companion_id, payload, scratch);
        let _ = socket.send_to(scratch, self.locus.0);
    }

    pub fn seal_outbound(&mut self, companion_id: u32, payload: &[u8], out: &mut Vec<u8>) -> SocketAddr {
        self.seal.seal(companion_id, payload, out);
        self.locus.0
    }
}
