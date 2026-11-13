use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use curve25519_dalek::scalar::Scalar;
use sha2::{Digest, Sha256, Sha512};
use super::crockford::CrockfordStamp;

/// Guest-to-host 12-character NAT hole-punch reply code (`XXXX-XXXX-XXXX`) bound to a `PortalCode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PunchCode {
    ip: Ipv4Addr,
    port: u16,
    tag12: u16,
}

impl PunchCode {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ip: Ipv4Addr, port: u16, tag12: u16) -> Self {
        Self { ip, port, tag12: tag12 & 0x0FFF }
    }

    pub const fn endpoint(&self) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(self.ip, self.port))
    }

    pub fn encode(&self) -> String {
        let ip_bits = u32::from(self.ip) as u64;
        let hi8 = ((self.tag12 >> 4) & 0xFF) as u64;
        let lo4 = (self.tag12 & 0x0F) as u8;
        let core56 = (ip_bits << 24) | ((self.port as u64) << 8) | hi8;
        CrockfordStamp::punch().encode60(core56, lo4)
    }
}

/// Serverless 12-character P2P portal code (`XXXX-XXXX-XXXX`) encoding IPv4, UDP port, and SPAKE2 secret.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortalCode {
    ip: Ipv4Addr,
    port: u16,
    secret: u8,
}

impl PortalCode {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ip: Ipv4Addr, port: u16, secret: u8) -> Self {
        Self { ip, port, secret }
    }

    pub fn draw_secret() -> u8 {
        let mut raw = [0u8; 1];
        getrandom::fill(&mut raw).expect("OS CSPRNG entropy unavailable");
        if raw[0] == 0 { 1 } else { raw[0] }
    }

    pub const fn slot_port_for_secret(secret: u8) -> u16 {
        27800 + ((secret as u16) % 99) + 1
    }

    pub fn from_endpoint(endpoint: SocketAddr, secret: u8) -> Self {
        let v4 = match endpoint.ip() {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => Ipv4Addr::LOCALHOST,
        };
        let bounded = if secret == 0 { 1 } else { secret };
        Self::new(v4, endpoint.port().max(1), bounded)
    }

    pub const fn endpoint(&self) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(self.ip, self.port))
    }

    pub const fn lan_port(&self) -> u16 {
        Self::slot_port_for_secret(self.secret)
    }

    fn compute_tag4(ip: Ipv4Addr, port: u16, secret: u8) -> u8 {
        let mut hasher = Sha256::new();
        hasher.update(b"LowCube-PortalCode-Tag4-v2:");
        hasher.update(ip.octets());
        hasher.update(port.to_le_bytes());
        hasher.update([secret]);
        let digest = hasher.finalize();
        digest[0] & 0x0F
    }

    fn compute_punch_tag12(&self, guest_ip: Ipv4Addr, guest_port: u16) -> u16 {
        let mut hasher = Sha256::new();
        hasher.update(b"LowCube-PunchCode-Tag12-v2:");
        hasher.update(self.ip.octets());
        hasher.update(self.port.to_le_bytes());
        hasher.update([self.secret]);
        hasher.update(guest_ip.octets());
        hasher.update(guest_port.to_le_bytes());
        let digest = hasher.finalize();
        u16::from_le_bytes([digest[0], digest[1]]) & 0x0FFF
    }

    pub fn encode(&self) -> String {
        let ip_bits = u32::from(self.ip) as u64;
        let core56 = (ip_bits << 24) | ((self.port as u64) << 8) | (self.secret as u64);
        let tag4 = Self::compute_tag4(self.ip, self.port, self.secret);
        CrockfordStamp::host().encode60(core56, tag4)
    }

    /// Parses a 12-character code (`K7M9-P2X4-R8W3` or `k7m9p2x4r8w3`) and verifies its 4-bit checksum.
    pub fn parse(input: &str) -> Option<Self> {
        let (core56, tag4) = CrockfordStamp::host().decode60(input)?;
        let ip = Ipv4Addr::from((core56 >> 24) as u32);
        let port = ((core56 >> 8) & 0xFFFF) as u16;
        let secret = (core56 & 0xFF) as u8;
        if port == 0 || secret == 0 || ip.is_unspecified() {
            return None;
        }
        if Self::compute_tag4(ip, port, secret) != tag4 {
            return None;
        }
        Some(Self::new(ip, port, secret))
    }

    /// Mints the guest's 12-character FRITZ!Box / NAT counter-punch reply code bound to this portal.
    pub fn mint_punch(&self, guest_endpoint: SocketAddr) -> PunchCode {
        let v4 = match guest_endpoint.ip() {
            IpAddr::V4(ip) if !ip.is_unspecified() => ip,
            _ => Ipv4Addr::LOCALHOST,
        };
        let port = guest_endpoint.port().max(1);
        let tag12 = self.compute_punch_tag12(v4, port);
        PunchCode::new(v4, port, tag12)
    }

    /// Verifies a guest's 12-character punch code (`99.975%` typo rejection via 12-bit SHA-256 tag).
    pub fn verify_punch(&self, input: &str) -> Option<SocketAddr> {
        let (core56, lo4) = CrockfordStamp::punch().decode60(input)?;
        let guest_ip = Ipv4Addr::from((core56 >> 24) as u32);
        let guest_port = ((core56 >> 8) & 0xFFFF) as u16;
        let hi8 = (core56 & 0xFF) as u16;
        let tag12 = (hi8 << 4) | ((lo4 & 0x0F) as u16);
        if guest_port == 0 || guest_ip.is_unspecified() {
            return None;
        }
        if self.compute_punch_tag12(guest_ip, guest_port) != tag12 {
            return None;
        }
        Some(SocketAddr::V4(SocketAddrV4::new(guest_ip, guest_port)))
    }

    /// Derives the Ristretto255 SPAKE2 password scalar `w` from this code's secret and endpoint.
    pub fn password_scalar(&self) -> Scalar {
        let mut hasher = Sha512::new();
        hasher.update(b"LowCube-SPAKE2-Password-v2:");
        hasher.update(self.ip.octets());
        hasher.update(self.port.to_le_bytes());
        hasher.update([self.secret]);
        let wide: [u8; 64] = hasher.finalize().into();
        Scalar::from_bytes_mod_order_wide(&wide)
    }
}
