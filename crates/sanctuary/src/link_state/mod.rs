mod sync;

use std::net::{SocketAddr, UdpSocket};
use crate::seal::{HandshakeIdentity, SessionSeal};
use crate::ticket::{JoinTarget, PortalTicket};
use crate::wormhole::{PunchCode, WormholePledge};
pub use self::sync::LinkSync;

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

    pub fn step_retry(&mut self, dt: f32, target: JoinTarget, socket: &UdpSocket) {
        match *self {
            Self::SeekingWormhole { ref pledge, ref mut retry_clock, .. } => {
                *retry_clock += dt;
                if *retry_clock >= 0.16 {
                    *retry_clock = 0.0;
                    pledge.transmit_hello(socket);
                }
            }
            Self::Knocking { ref keys, ref mut retry_clock } => {
                *retry_clock += dt;
                if *retry_clock >= 0.16 {
                    *retry_clock = 0.0;
                    Self::emit_knock(target, socket, keys);
                }
            }
            Self::Sealed { .. } => {}
        }
    }

    pub fn unseal_envelope(&mut self, src: SocketAddr, pkt: &[u8]) -> Option<crate::envelope::SanctuaryEnvelope> {
        if let Self::Sealed { ref mut seal, ref mut peer, .. } = *self {
            seal.open_frame(pkt).and_then(|(_, plain)| {
                *peer = src;
                crate::envelope::SanctuaryEnvelope::decode(&plain)
            })
        } else {
            None
        }
    }
}
