mod arrival;
mod sync;
mod welcome;

use std::net::{SocketAddr, UdpSocket};
use world::Callsign;
use crate::passport::KnockPassport;
use crate::seal::SessionSeal;
use crate::ticket::{JoinTarget, PortalTicket};
use crate::wormhole::{PunchCode, WormholePledge};
pub use self::arrival::WelcomeSpawn;
pub use self::sync::LinkSync;
pub use self::welcome::WelcomeFrame;

/// Polymorphic P2P session establishment phase without nullable cipher fields.
pub enum LinkPhase {
    SeekingWormhole {
        pledge: WormholePledge,
        passport: KnockPassport,
        retry_clock: f32,
    },
    Knocking {
        passport: KnockPassport,
        retry_clock: f32,
    },
    Sealed {
        companion_id: u32,
        callsign: Callsign,
        seal: SessionSeal,
        peer: SocketAddr,
    },
}

impl LinkPhase {
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

    pub const fn assigned_callsign(&self) -> Callsign {
        match *self {
            Self::SeekingWormhole { .. } | Self::Knocking { .. } => Callsign::unnamed(),
            Self::Sealed { callsign, .. } => callsign,
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
            Self::Knocking { ref passport, ref mut retry_clock } => {
                *retry_clock += dt;
                if *retry_clock >= 0.16 {
                    *retry_clock = 0.0;
                    passport.emit_knock(target, socket);
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
