use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};

/// Polymorphic relay route state eliminating nullable socket fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeaconRoute {
    Unassisted,
    Assisted(SocketAddr),
}

impl BeaconRoute {
    pub fn address(&self) -> Option<SocketAddr> {
        match *self {
            Self::Unassisted => None,
            Self::Assisted(addr) => Some(addr),
        }
    }
}

/// Shareable invite code (`LC1-...`) pinning the host X25519 key, direct socket, and optional NAT relay beacon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortalTicket {
    host_key: [u8; 32],
    direct: SocketAddr,
    beacon: BeaconRoute,
}

impl PortalTicket {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(host_key: [u8; 32], direct: SocketAddr, beacon: BeaconRoute) -> Self {
        Self { host_key, direct, beacon }
    }

    pub const fn host_key(&self) -> [u8; 32] {
        self.host_key
    }

    pub const fn direct(&self) -> SocketAddr {
        self.direct
    }

    pub const fn beacon(&self) -> BeaconRoute {
        self.beacon
    }

    pub fn portal_id(&self) -> [u8; 8] {
        let mut id = [0u8; 8];
        id.copy_from_slice(&self.host_key[0..8]);
        id
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
        Some(SocketAddr::V4(SocketAddrV4::new(ip, port)))
    }

    /// Formats this invite ticket as a compact copy-pasteable `LC1-...` uppercase hex code.
    pub fn encode(&self) -> String {
        let mut raw = Vec::with_capacity(45);
        raw.extend_from_slice(&self.host_key);
        Self::push_v4(self.direct, &mut raw);
        match self.beacon {
            BeaconRoute::Unassisted => raw.push(0),
            BeaconRoute::Assisted(addr) => {
                raw.push(1);
                Self::push_v4(addr, &mut raw);
            }
        }
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
        if raw.len() < 39 {
            return None;
        }
        let mut host_key = [0u8; 32];
        host_key.copy_from_slice(&raw[0..32]);
        let direct = Self::read_v4(&raw[32..38])?;
        let beacon = if raw[38] == 1 && raw.len() >= 45 {
            BeaconRoute::Assisted(Self::read_v4(&raw[39..45])?)
        } else {
            BeaconRoute::Unassisted
        };
        Some(Self::new(host_key, direct, beacon))
    }

    /// Resolves a shareable local IPv4 address for outbound UDP sockets bound to `0.0.0.0`.
    pub fn shareable_local_endpoint(bound: SocketAddr) -> SocketAddr {
        if !bound.ip().is_unspecified() {
            return bound;
        }
        if let Ok(probe) = UdpSocket::bind("0.0.0.0:0") {
            let _ = probe.connect("192.0.2.1:9");
            if let Ok(local) = probe.local_addr() {
                if !local.ip().is_unspecified() {
                    return SocketAddr::new(local.ip(), bound.port());
                }
            }
        }
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), bound.port())
    }
}

/// Polymorphic destination parsed from `--join <LC1-ticket | host:port>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinTarget {
    Ticket(PortalTicket),
    Direct(SocketAddr),
}

impl JoinTarget {
    pub fn parse(spec: &str) -> Option<Self> {
        let trimmed = spec.trim();
        let token = trimmed
            .rsplit_once("--join")
            .map_or(trimmed, |(_, rhs)| rhs.trim())
            .split_whitespace()
            .last()?;
        if token.is_empty() {
            return None;
        }
        if let Some(ticket) = PortalTicket::decode(token) {
            return Some(Self::Ticket(ticket));
        }
        let candidate = if token.contains(':') {
            token.to_string()
        } else {
            format!("{token}:27777")
        };
        let mut resolved = candidate.to_socket_addrs().ok()?;
        let addr = resolved
            .find(SocketAddr::is_ipv4)
            .or_else(|| candidate.to_socket_addrs().ok()?.next())?;
        Some(Self::Direct(addr))
    }
}
