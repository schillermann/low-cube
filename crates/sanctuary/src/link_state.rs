use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use world::{CompanionPresence, PlanetaryWorld, WorldFact};
use crate::chronicle_log::DirtySlices;
use crate::envelope::{CrackNotice, SanctuaryEnvelope};
use crate::seal::{HandshakeIdentity, SessionSeal};
use crate::ticket::{JoinTarget, PortalTicket};
use crate::wormhole::{PunchCode, WormholePledge};

/// Polymorphic P2P session establishment phase without nullable cipher fields.
pub enum LinkPhase {
    SeekingWormhole {
        pledge: WormholePledge,
        keys: HandshakeIdentity,
        retry_clock: f32,
    },
    Knocking {
        keys: HandshakeIdentity,
        retry_clock: f32,
    },
    Sealed {
        companion_id: u32,
        seal: SessionSeal,
        peer: SocketAddr,
    },
}

impl LinkPhase {
    pub fn emit_knock(target: JoinTarget, socket: &UdpSocket, keys: &HandshakeIdentity) {
        let mut knock = Vec::with_capacity(36);
        knock.extend_from_slice(b"LKNK");
        knock.extend_from_slice(&keys.public_bytes());
        match target {
            JoinTarget::Direct(addr) => {
                let _ = socket.send_to(&knock, addr);
            }
            JoinTarget::Ticket(ticket) => {
                let _ = socket.send_to(&knock, ticket.direct());
            }
            JoinTarget::Wormhole(_) => {}
        }
    }

    pub fn try_resolve_wormhole(&self, src: SocketAddr, pkt: &[u8]) -> Option<PortalTicket> {
        if let Self::SeekingWormhole { ref pledge, .. } = *self {
            pledge.unseal_reply(src, pkt)
        } else {
            None
        }
    }

    pub fn punch_code(&self) -> Option<PunchCode> {
        if let Self::SeekingWormhole { ref pledge, .. } = *self {
            Some(pledge.punch_code())
        } else {
            None
        }
    }
}

/// Synchronized chronicle cursor, companion list, and outbound avatar presence.
pub struct LinkSync {
    ack_seq: u32,
    pulse_clock: f32,
    companions: Vec<CompanionPresence>,
    local_presence: CompanionPresence,
}

impl LinkSync {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        ack_seq: u32,
        pulse_clock: f32,
        companions: Vec<CompanionPresence>,
        local_presence: CompanionPresence,
    ) -> Self {
        Self { ack_seq, pulse_clock, companions, local_presence }
    }

    pub fn update_presence(&mut self, presence: CompanionPresence, cid: u32) {
        self.local_presence = presence.with_slot(cid);
    }

    pub fn assign_slot(&mut self, cid: u32) {
        self.local_presence = self.local_presence.with_slot(cid);
    }

    pub fn copy_companions(&self, out: &mut Vec<CompanionPresence>) {
        out.clear();
        out.extend_from_slice(&self.companions);
    }

    pub fn step_pulse(&mut self, dt: f32, cid: u32, seal: &mut SessionSeal, peer: SocketAddr, socket: &UdpSocket) {
        self.pulse_clock += dt;
        if self.pulse_clock >= 0.05 {
            self.pulse_clock = 0.0;
            let (mut payload, mut frame) = (Vec::new(), Vec::new());
            let pulse = SanctuaryEnvelope::GuestPulse {
                ack_seq: self.ack_seq,
                presence: self.local_presence,
            };
            pulse.inscribe(&mut payload);
            seal.seal(cid, &payload, &mut frame);
            let _ = socket.send_to(&frame, peer);
        }
    }

    pub fn absorb_envelope(
        &mut self,
        env: SanctuaryEnvelope,
        world: &mut dyn PlanetaryWorld,
        predicted: &mut VecDeque<WorldFact>,
        slices: &DirtySlices,
    ) -> bool {
        let faces = world.face_count();
        match env {
            SanctuaryEnvelope::HostChronicle { base_seq, facts } => {
                if base_seq > self.ack_seq {
                    return false;
                }
                let skip = (self.ack_seq - base_seq) as usize;
                let mut changed = false;
                for fact in facts.into_iter().skip(skip) {
                    self.ack_seq += 1;
                    if let Some(pos) = predicted.iter().position(|p| *p == fact) {
                        let _ = predicted.remove(pos);
                    } else if fact.enact_on(world) {
                        slices.mark_fact(&fact, faces);
                        changed = true;
                    }
                }
                changed
            }
            SanctuaryEnvelope::HostPulse { crack, companions, .. } => {
                self.companions = companions;
                if let CrackNotice::Active { eye, direction, stage } = crack {
                    if world.loosen_surface(eye, direction, stage).is_some() {
                        slices.mark_fact(&WorldFact::Loosened { eye, direction, stage }, faces);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}
