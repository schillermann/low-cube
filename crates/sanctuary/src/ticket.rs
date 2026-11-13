use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use crate::wormhole::{CrockfordStamp, PortalCode};

/// Shareable invite ticket (`LC1-...`) pinning the host X25519 key and direct P2P UDP socket.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortalTicket {
    host_key: [u8; 32],
    direct: SocketAddr,
}

impl PortalTicket {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(host_key: [u8; 32], direct: SocketAddr) -> Self {
        Self { host_key, direct }
    }

    pub const fn host_key(&self) -> [u8; 32] {
        self.host_key
    }

    pub const fn direct(&self) -> SocketAddr {
        self.direct
    }

    pub fn push_v4(addr: SocketAddr, out: &mut Vec<u8>) {
        let v4 = match addr.ip() {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => Ipv4Addr::LOCALHOST,
        };
        out.extend_from_slice(&v4.octets());
        out.extend_from_slice(&addr.port().to_le_bytes());
    }

    pub fn read_v4(bytes: &[u8]) -> Option<SocketAddr> {
        if bytes.len() < 6 {
            return None;
        }
        let ip = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
        let port = u16::from_le_bytes([bytes[4], bytes[5]]);
        if port == 0 {
            return None;
        }
        Some(SocketAddr::V4(SocketAddrV4::new(ip, port)))
    }

    /// Serializes this ticket into `38` raw binary bytes (`32B` X25519 public key + `6B` IPv4:Port).
    pub fn encode_bytes(&self, out: &mut Vec<u8>) {
        out.clear();
        out.reserve(38);
        out.extend_from_slice(&self.host_key);
        Self::push_v4(self.direct, out);
    }

    pub fn decode_bytes(raw: &[u8]) -> Option<Self> {
        if raw.len() < 38 {
            return None;
        }
        let mut host_key = [0u8; 32];
        host_key.copy_from_slice(&raw[0..32]);
        let direct = Self::read_v4(&raw[32..38])?;
        Some(Self::new(host_key, direct))
    }

    /// Formats this invite ticket as a compact copy-pasteable `LC1-...` uppercase hex code.
    pub fn encode(&self) -> String {
        let mut raw = Vec::with_capacity(38);
        self.encode_bytes(&mut raw);
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        let mut text = String::with_capacity(4 + raw.len() * 2);
        text.push_str("LC1-");
        for b in raw {
            text.push(HEX[(b >> 4) as usize] as char);
            text.push(HEX[(b & 0x0F) as usize] as char);
        }
        text
    }

    fn hex_nibble(ch: u8) -> Option<u8> {
        match ch {
            b'0'..=b'9' => Some(ch - b'0'),
            b'a'..=b'f' => Some(ch - b'a' + 10),
            b'A'..=b'F' => Some(ch - b'A' + 10),
            _ => None,
        }
    }

    pub fn decode(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        let body = trimmed.strip_prefix("LC1-").or_else(|| trimmed.strip_prefix("lc1-"))?;
        let bytes_str = body.as_bytes();
        if bytes_str.len() % 2 != 0 {
            return None;
        }
        let mut raw = Vec::with_capacity(bytes_str.len() / 2);
        let mut idx = 0;
        while idx < bytes_str.len() {
            let hi = Self::hex_nibble(bytes_str[idx])?;
            let lo = Self::hex_nibble(bytes_str[idx + 1])?;
            raw.push((hi << 4) | lo);
            idx += 2;
        }
        Self::decode_bytes(&raw)
    }
}

/// Non-blocking UI validation state avoiding per-frame DNS resolution while typing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinPreview {
    Empty,
    CodeReady(PortalCode),
    CodeTypo,
    TicketOrAddressReady,
    Malformed,
}

/// Polymorphic P2P destination parsed from a 12-char `PortalCode`, `LC1-...` ticket, or `host[:port]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinTarget {
    Ticket(PortalTicket),
    Direct(SocketAddr),
    Wormhole(PortalCode),
}

impl JoinTarget {
    /// Pure lexical preview without network or DNS calls (safe to invoke every frame at 60 FPS).
    pub fn preview(spec: &str) -> JoinPreview {
        let trimmed = spec.trim();
        if trimmed.is_empty() {
            return JoinPreview::Empty;
        }
        if trimmed.len() >= 4 && trimmed[..4].eq_ignore_ascii_case("lc1-") {
            return if PortalTicket::decode(trimmed).is_some() {
                JoinPreview::TicketOrAddressReady
            } else {
                JoinPreview::Malformed
            };
        }
        if let Some(code) = PortalCode::parse(trimmed) {
            return JoinPreview::CodeReady(code);
        }
        if CrockfordStamp::resembles_twelve_char_code(trimmed) {
            return JoinPreview::CodeTypo;
        }
        let plausible_host = trimmed.eq_ignore_ascii_case("localhost")
            || trimmed.contains('.')
            || trimmed.contains(':');
        if plausible_host && !trimmed.contains(' ') {
            JoinPreview::TicketOrAddressReady
        } else {
            JoinPreview::Malformed
        }
    }

    pub fn parse(spec: &str) -> Option<Self> {
        let trimmed = spec.trim();
        let body = trimmed
            .rsplit_once("--join")
            .map_or(trimmed, |(_, rhs)| rhs.trim());
        if body.is_empty() {
            return None;
        }
        if let Some(code) = PortalCode::parse(body) {
            return Some(Self::Wormhole(code));
        }
        if CrockfordStamp::resembles_twelve_char_code(body) {
            return None;
        }
        let token = body.split_whitespace().last()?;
        if token.is_empty() {
            return None;
        }
        if let Some(ticket) = PortalTicket::decode(token) {
            return Some(Self::Ticket(ticket));
        }
        let candidate = if token.contains(':') {
            token.to_string()
        } else if token.contains('.') || token.eq_ignore_ascii_case("localhost") {
            format!("{token}:27777")
        } else {
            return None;
        };
        let mut resolved = candidate.to_socket_addrs().ok()?;
        let addr = resolved
            .find(SocketAddr::is_ipv4)
            .or_else(|| candidate.to_socket_addrs().ok()?.next())?;
        Some(Self::Direct(addr))
    }
}
