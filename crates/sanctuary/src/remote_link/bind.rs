use std::net::{SocketAddr, UdpSocket};
use glam::{Quat, Vec3};
use world::{CompanionPresence, PilotCraft, PresenceAttire, PresenceKinematics};
use crate::glide::SmoothRoster;
use crate::link_state::{LinkPhase, LinkSync};
use crate::nat::NatAperture;
use crate::passport::{KnockPassport, PassportVault};
use crate::ticket::JoinTarget;
use crate::wormhole::WormholePledge;
use super::RemoteLink;

/// Encapsulated client UDP socket bind and initial `LinkPhase` assembly (`<= 4` fields).
pub(super) struct LinkSocket {
    target: JoinTarget,
}

impl LinkSocket {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(target: JoinTarget) -> Self {
        Self { target }
    }

    pub(super) fn open_with_vault(self, vault: Box<dyn PassportVault>) -> std::io::Result<RemoteLink> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        let _ = socket.set_broadcast(true);
        let guest_endpoint = match self.target {
            JoinTarget::Wormhole(code) => NatAperture::interactive().probe_guest(&socket, code.endpoint()),
            JoinTarget::Ticket(_) | JoinTarget::Direct(_) => socket.local_addr()?,
        };
        Ok(self.assemble(socket, guest_endpoint, vault))
    }

    pub(super) fn assemble(
        self,
        socket: UdpSocket,
        guest_endpoint: SocketAddr,
        vault: Box<dyn PassportVault>,
    ) -> RemoteLink {
        let _ = socket.set_nonblocking(true);
        let _ = socket.set_broadcast(true);
        let kin = PresenceKinematics::new(Vec3::new(0.0, 32.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
        let initial_pres = CompanionPresence::new(0, kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire());
        let passport = KnockPassport::from_vault(self.target, vault);
        let phase = match self.target {
            JoinTarget::Wormhole(code) => {
                let pledge = WormholePledge::begin(code, guest_endpoint);
                pledge.transmit_hello(&socket);
                LinkPhase::SeekingWormhole { pledge, passport, retry_clock: 0.0 }
            }
            JoinTarget::Ticket(_) | JoinTarget::Direct(_) => {
                passport.emit_knock(self.target, &socket);
                LinkPhase::Knocking { passport, retry_clock: 0.0 }
            }
        };
        let sync = LinkSync::new(0, 1.0, SmoothRoster::empty(), (initial_pres, PilotCraft::Pedestrian));
        RemoteLink::new(socket, self.target, phase, sync)
    }
}
