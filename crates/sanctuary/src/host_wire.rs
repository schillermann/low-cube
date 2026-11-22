use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{CraftIdentity, CraftPose, PlanetaryWorld, WorldFact};
use crate::aura::HostIdentity;
use crate::host_state::HostState;
use crate::mailbox::PortalBeat;
use crate::ticket::PortalTicket;

/// Encapsulated P2P UDP host wire and session state implementing `PortalBeat` without `PlanetaryWorld` locks.
pub struct HostWire {
    socket: UdpSocket,
    identity: HostIdentity,
    state: HostState,
    deferred: VecDeque<(u32, WorldFact)>,
}

impl HostWire {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        socket: UdpSocket,
        identity: HostIdentity,
        state: HostState,
        deferred: VecDeque<(u32, WorldFact)>,
    ) -> Self {
        Self { socket, identity, state, deferred }
    }

    pub const fn socket(&self) -> &UdpSocket { &self.socket }
    pub const fn identity(&self) -> &HostIdentity { &self.identity }
    pub const fn state(&self) -> &HostState { &self.state }
    pub fn state_mut(&mut self) -> &mut HostState { &mut self.state }

    pub fn ticket(&self) -> PortalTicket {
        PortalTicket::new(self.identity.keys().public_bytes(), self.identity.endpoint())
    }

    pub fn punch_peer(&mut self, peer: SocketAddr) {
        if let Ok(sock) = self.socket.try_clone() {
            self.state.punch_peer(&sock, peer);
        }
    }

    pub fn present_vessels(&self, sink: &mut dyn world::VesselSink) {
        let mut crafts = Vec::new();
        self.state.guest_crafts(&mut crafts);
        self.state.fleet().present_vessels(&crafts, sink);
    }

    pub fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        let mut crafts = Vec::new();
        self.state.guest_crafts(&mut crafts);
        self.state.fleet().find_passenger_seat(&crafts, eye, direction)
    }

    pub fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        let mut crafts = Vec::new();
        self.state.guest_crafts(&mut crafts);
        crafts.into_iter().find(|c| c.identity() == craft).map(|c| c.pose().world_seat_at(local_seat))
    }

    pub fn resolve_deck_pose(&self, craft: CraftIdentity) -> Option<(CraftPose, f32)> {
        let mut crafts = Vec::new();
        self.state.guest_crafts(&mut crafts);
        self.state.fleet().resolve_deck_pose(&crafts, craft)
    }

    pub fn constrain_on_craft(&self, craft: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        let mut crafts = Vec::new();
        self.state.guest_crafts(&mut crafts);
        self.state.fleet().constrain_craft_by_id(&crafts, craft, position, player_height)
    }

    fn answer_knock(&mut self, peer: SocketAddr, guest_pub: [u8; 32]) {
        let cid = self.state.admit_guest(&self.identity, peer, guest_pub);
        let mut reply = Vec::with_capacity(60);
        reply.extend_from_slice(b"LWEL");
        reply.extend_from_slice(&self.identity.keys().public_bytes());
        reply.extend_from_slice(&cid.to_le_bytes());
        reply.extend_from_slice(&self.identity.blocks_per_edge().to_le_bytes());
        self.identity.proportions().inscribe_tail(&mut reply);
        let _ = self.socket.send_to(&reply, peer);
    }

    fn answer_handshake_frame(&mut self, src: SocketAddr, pkt: &[u8]) -> bool {
        if pkt.len() == 37 && &pkt[0..4] == b"LPK1" {
            let ticket = self.ticket();
            self.state.answer_wormhole(&self.socket, src, pkt, ticket);
            return true;
        }
        if pkt.len() >= 36 && &pkt[0..4] == b"LKNK" {
            let mut pub_key = [0u8; 32];
            pub_key.copy_from_slice(&pkt[4..36]);
            self.answer_knock(src, pub_key);
            return true;
        }
        false
    }

    pub fn pump_authoritative(&mut self, dt: f32, world: &mut dyn PlanetaryWorld) -> (bool, usize) {
        let mut mutated = false;
        while let Some((cid, fact)) = self.deferred.pop_front() {
            if self.state.enact_guest_petition(cid, fact, world) {
                mutated = true;
            }
        }
        let (pulse_ready, parked) = self.state.step_clocks(&self.socket, dt, world);
        if parked {
            mutated = true;
        }
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            let pkt = &buf[..len];
            if self.answer_handshake_frame(src, pkt) || pkt.len() < 30 || &pkt[0..2] != b"LS" {
                continue;
            }
            if let Ok(cid_bytes) = pkt[2..6].try_into() {
                let cid = u32::from_le_bytes(cid_bytes);
                if self.state.process_companion_datagram(cid, src, pkt, world) {
                    mutated = true;
                }
            }
        }
        if pulse_ready {
            self.state.broadcast(&self.socket);
        }
        (mutated, self.state.active_craft_count())
    }
}

impl PortalBeat for HostWire {
    fn step_standby(&mut self, dt: f32) -> usize {
        let max_extent = self.identity.proportions().heliosphere_span();
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            let pkt = &buf[..len];
            if self.answer_handshake_frame(src, pkt) || pkt.len() < 30 || &pkt[0..2] != b"LS" {
                continue;
            }
            if let Ok(cid_bytes) = pkt[2..6].try_into() {
                let cid = u32::from_le_bytes(cid_bytes);
                self.state.process_standby_datagram(cid, src, pkt, max_extent, &mut self.deferred);
            }
        }
        self.state.step_standby_clocks(&self.socket, dt, &mut self.deferred);
        self.state.active_craft_count()
    }
}
