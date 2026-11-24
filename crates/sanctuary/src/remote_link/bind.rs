use std::net::{SocketAddr, UdpSocket};
use glam::{Quat, Vec3};
use world::{CompanionPresence, PilotCraft, PresenceAttire, PresenceKinematics};
use crate::glide::SmoothRoster;
use crate::link_state::{LinkPhase, LinkSync};
use crate::nat::NatAperture;
use crate::passport::{FixedPassport, KnockPassport, PassportVault};
use crate::seal::HandshakeIdentity;
use crate::ticket::JoinTarget;
use crate::wormhole::WormholePledge;
use super::RemoteLink;

impl RemoteLink {
    pub fn bind(target: JoinTarget) -> std::io::Result<Self> {
        Self::bind_with_vault(target, Box::new(FixedPassport::unassigned()))
    }

    pub fn bind_with_identity(target: JoinTarget, keys: HandshakeIdentity) -> std::io::Result<Self> {
        Self::bind_with_vault(target, Box::new(FixedPassport::pinned(&keys)))
    }

    pub fn bind_with_vault(target: JoinTarget, vault: Box<dyn PassportVault>) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        let _ = socket.set_broadcast(true);
        let guest_endpoint = match target {
            JoinTarget::Wormhole(code) => NatAperture::interactive().probe_guest(&socket, code.endpoint()),
            JoinTarget::Ticket(_) | JoinTarget::Direct(_) => socket.local_addr()?,
        };
        Ok(Self::bind_socket_with_vault(socket, guest_endpoint, target, vault))
    }

    pub fn bind_on_socket(socket: UdpSocket, guest_endpoint: SocketAddr, target: JoinTarget) -> Self {
        Self::bind_socket_with_vault(socket, guest_endpoint, target, Box::new(FixedPassport::unassigned()))
    }

    pub fn bind_socket_with_identity(
        socket: UdpSocket,
        guest_endpoint: SocketAddr,
        target: JoinTarget,
        keys: HandshakeIdentity,
    ) -> Self {
        Self::bind_socket_with_vault(socket, guest_endpoint, target, Box::new(FixedPassport::pinned(&keys)))
    }

    pub fn bind_socket_with_vault(
        socket: UdpSocket,
        guest_endpoint: SocketAddr,
        target: JoinTarget,
        vault: Box<dyn PassportVault>,
    ) -> Self {
        let _ = socket.set_nonblocking(true);
        let _ = socket.set_broadcast(true);
        let kin = PresenceKinematics::new(Vec3::new(0.0, 32.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
        let initial_pres = CompanionPresence::new(0, kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire());
        let passport = KnockPassport::from_vault(target, vault);
        let phase = match target {
            JoinTarget::Wormhole(code) => {
                let pledge = WormholePledge::begin(code, guest_endpoint);
                pledge.transmit_hello(&socket);
                LinkPhase::SeekingWormhole { pledge, passport, retry_clock: 0.0 }
            }
            JoinTarget::Ticket(_) | JoinTarget::Direct(_) => {
                passport.emit_knock(target, &socket);
                LinkPhase::Knocking { passport, retry_clock: 0.0 }
            }
        };
        let sync = LinkSync::new(0, 1.0, SmoothRoster::empty(), (initial_pres, PilotCraft::Pedestrian));
        Self::new(socket, target, phase, sync)
    }
}
