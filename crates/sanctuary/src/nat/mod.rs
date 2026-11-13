mod router;
mod stun;

use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::thread;
use std::time::Duration;

pub use router::GatewayLease;
pub use stun::StunProbe;

/// Serverless cross-platform NAT aperture combining RFC 5389 STUN and local router UPnP/NAT-PMP.
#[derive(Clone, Copy, Debug)]
pub struct NatAperture {
    stun_budget_ms: u64,
}

impl NatAperture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(stun_budget_ms: u64) -> Self {
        Self { stun_budget_ms }
    }

    pub const fn interactive() -> Self {
        Self::new(260)
    }

    /// Resolves the outbound local IPv4 address for sockets bound to `0.0.0.0`.
    pub fn local_ipv4(bound: SocketAddr) -> Ipv4Addr {
        if let IpAddr::V4(v4) = bound.ip() {
            if !v4.is_unspecified() {
                return v4;
            }
        }
        if let Ok(probe) = UdpSocket::bind("0.0.0.0:0") {
            let _ = probe.connect("198.51.100.1:9");
            if let Ok(local) = probe.local_addr() {
                if let IpAddr::V4(v4) = local.ip() {
                    if !v4.is_unspecified() {
                        return v4;
                    }
                }
            }
        }
        Ipv4Addr::LOCALHOST
    }

    /// Discovers the host's external endpoint via STUN/UPnP and spawns background router port mapping.
    pub fn open_host(&self, socket: &UdpSocket) -> SocketAddr {
        let bound = socket.local_addr().expect("host UDP socket bound");
        let port = bound.port();
        let lan_ip = Self::local_ipv4(bound);
        if lan_ip.is_loopback() {
            return SocketAddr::V4(SocketAddrV4::new(lan_ip, port));
        }

        let lease = GatewayLease::new(lan_ip, port);
        thread::spawn(move || lease.request_port_mapping());

        let probe = StunProbe::fresh(Duration::from_millis(self.stun_budget_ms));
        if let Some(mapped) = probe.discover(socket) {
            return mapped;
        }
        if let Some(wan_ip) = lease.query_external_ip(Duration::from_millis(140)) {
            return SocketAddr::V4(SocketAddrV4::new(wan_ip, port));
        }
        SocketAddr::V4(SocketAddrV4::new(lan_ip, port))
    }

    /// Discovers the guest's external endpoint on its live `UdpSocket` for mutual FRITZ!Box hole-punching.
    pub fn probe_guest(&self, socket: &UdpSocket, remote_hint: SocketAddr) -> SocketAddr {
        let bound = socket.local_addr().expect("guest UDP socket bound");
        let port = bound.port();
        if remote_hint.ip().is_loopback() {
            return SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
        }
        let lan_ip = Self::local_ipv4(bound);
        if lan_ip.is_loopback() {
            return SocketAddr::V4(SocketAddrV4::new(lan_ip, port));
        }
        let probe = StunProbe::fresh(Duration::from_millis(self.stun_budget_ms));
        if let Some(mapped) = probe.discover(socket) {
            return mapped;
        }
        let lease = GatewayLease::new(lan_ip, port);
        if let Some(wan_ip) = lease.query_external_ip(Duration::from_millis(120)) {
            return SocketAddr::V4(SocketAddrV4::new(wan_ip, port));
        }
        SocketAddr::V4(SocketAddrV4::new(lan_ip, port))
    }
}
