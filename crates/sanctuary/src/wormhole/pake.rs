use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use chacha20poly1305::aead::{AeadInPlace, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, Tag};
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_TABLE;
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use sha2::{Digest, Sha256, Sha512};
use crate::ticket::PortalTicket;
use super::phrase::{PortalCode, PunchCode};

const WIRE_VERSION: u8 = 2;

/// Canonical nothing-up-my-sleeve SPAKE2 Ristretto255 generators `M` and `N`.
struct SpakePoints {
    m_point: RistrettoPoint,
    n_point: RistrettoPoint,
}

impl SpakePoints {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(m_point: RistrettoPoint, n_point: RistrettoPoint) -> Self {
        Self { m_point, n_point }
    }

    fn canonical() -> Self {
        let m_hash: [u8; 64] = Sha512::digest(b"LowCube-SPAKE2-M-Point-Ristretto255-v2").into();
        let n_hash: [u8; 64] = Sha512::digest(b"LowCube-SPAKE2-N-Point-Ristretto255-v2").into();
        Self::new(
            RistrettoPoint::from_uniform_bytes(&m_hash),
            RistrettoPoint::from_uniform_bytes(&n_hash),
        )
    }

    fn transcript_cipher(
        code: PortalCode,
        x_star: &[u8; 32],
        y_star: &[u8; 32],
        shared: &RistrettoPoint,
    ) -> ChaCha20Poly1305 {
        let mut hasher = Sha256::new();
        hasher.update(b"LowCube-SPAKE2-Transcript-v2:");
        hasher.update(code.password_scalar().as_bytes());
        hasher.update(x_star);
        hasher.update(y_star);
        hasher.update(shared.compress().as_bytes());
        let key_bytes: [u8; 32] = hasher.finalize().into();
        ChaCha20Poly1305::new(Key::from_slice(&key_bytes))
    }

    fn random_scalar() -> Scalar {
        let mut raw = [0u8; 64];
        getrandom::fill(&mut raw).expect("OS CSPRNG entropy unavailable");
        Scalar::from_bytes_mod_order_wide(&raw)
    }
}

/// Guest-side ephemeral SPAKE2 pledge sending `LPK1` and minting FRITZ!Box counter-punch codes.
pub struct WormholePledge {
    code: PortalCode,
    punch: PunchCode,
    secret: Scalar,
    blinded_x: [u8; 32],
}

impl WormholePledge {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(code: PortalCode, punch: PunchCode, secret: Scalar, blinded_x: [u8; 32]) -> Self {
        Self { code, punch, secret, blinded_x }
    }

    pub fn begin(code: PortalCode, guest_endpoint: SocketAddr) -> Self {
        let secret = SpakePoints::random_scalar();
        let pts = SpakePoints::canonical();
        let w = code.password_scalar();
        let x_point = (&secret * RISTRETTO_BASEPOINT_TABLE) + (w * pts.m_point);
        let punch = code.mint_punch(guest_endpoint);
        Self::new(code, punch, secret, x_point.compress().to_bytes())
    }

    pub const fn code(&self) -> PortalCode {
        self.code
    }

    pub const fn punch_code(&self) -> PunchCode {
        self.punch
    }

    pub fn hello_packet(&self) -> [u8; 37] {
        let mut pkt = [0u8; 37];
        pkt[0..4].copy_from_slice(b"LPK1");
        pkt[4] = WIRE_VERSION;
        pkt[5..37].copy_from_slice(&self.blinded_x);
        pkt
    }

    fn subnet_broadcast_ip() -> Option<Ipv4Addr> {
        let probe = UdpSocket::bind("0.0.0.0:0").ok()?;
        probe.connect("198.51.100.1:9").ok()?;
        match probe.local_addr().ok()?.ip() {
            IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_unspecified() => {
                let oct = v4.octets();
                Some(Ipv4Addr::new(oct[0], oct[1], oct[2], 255))
            }
            _ => None,
        }
    }

    /// Emits `LPK1` directly to the decoded endpoint plus same-PC/LAN hairpin bypass targets.
    pub fn transmit_hello(&self, socket: &UdpSocket) {
        let pkt = self.hello_packet();
        let direct = self.code.endpoint();
        let _ = socket.send_to(&pkt, direct);
        if direct.ip().is_loopback() {
            return;
        }
        let sub_bcast = Self::subnet_broadcast_ip();
        for port in [direct.port(), self.code.lan_port(), 27777u16] {
            let _ = socket.send_to(&pkt, SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)));
            let _ = socket.send_to(&pkt, SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::BROADCAST, port)));
            if let Some(sub) = sub_bcast {
                let _ = socket.send_to(&pkt, SocketAddr::V4(SocketAddrV4::new(sub, port)));
            }
        }
    }

    /// Verifies `LPK2` via SPAKE2 and decrypts the authoritative `PortalTicket` (`None` on wrong code).
    pub fn unseal_reply(&self, src: SocketAddr, pkt: &[u8]) -> Option<PortalTicket> {
        if pkt.len() < 91 || &pkt[0..4] != b"LPK2" || pkt[4] != WIRE_VERSION {
            return None;
        }
        let y_star: [u8; 32] = pkt[5..37].try_into().ok()?;
        let y_point = CompressedRistretto(y_star).decompress()?;
        let pts = SpakePoints::canonical();
        let w = self.code.password_scalar();
        let shared = self.secret * (y_point - (w * pts.n_point));
        let aead = SpakePoints::transcript_cipher(self.code, &self.blinded_x, &y_star, &shared);

        let aad = &pkt[0..37];
        let tag = Tag::from_slice(&pkt[37..53]);
        let mut body = pkt[53..].to_vec();
        aead.decrypt_in_place_detached(Nonce::from_slice(&[0u8; 12]), aad, &mut body, tag).ok()?;
        let decoded = PortalTicket::decode_bytes(&body)?;
        Some(PortalTicket::new(decoded.host_key(), src))
    }
}

/// Host-side rate-limited SPAKE2 responder and mutual FRITZ!Box NAT hole-punch emitter.
pub struct WormholeGate {
    code: PortalCode,
    budget: f32,
    punch_peers: Vec<(SocketAddr, f32)>,
    punch_clock: f32,
}

impl WormholeGate {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(code: PortalCode, budget: f32, punch_peers: Vec<(SocketAddr, f32)>, punch_clock: f32) -> Self {
        Self { code, budget, punch_peers, punch_clock }
    }

    pub fn with_code(code: PortalCode) -> Self {
        Self::new(code, 8.0, Vec::new(), 0.0)
    }

    pub const fn code(&self) -> PortalCode {
        self.code
    }

    /// Fires an immediate UDP NAT punch packet at `peer` and schedules 8s of keep-open pulses.
    pub fn punch_peer(&mut self, socket: &UdpSocket, peer: SocketAddr) {
        let _ = socket.send_to(b"LCPN", peer);
        if let Some(entry) = self.punch_peers.iter_mut().find(|(addr, _)| *addr == peer) {
            entry.1 = 8.0;
        } else {
            if self.punch_peers.len() >= 8 {
                let _ = self.punch_peers.remove(0);
            }
            self.punch_peers.push((peer, 8.0));
        }
    }

    pub fn step(&mut self, socket: &UdpSocket, dt: f32) {
        self.budget = (self.budget + dt * 2.5).min(8.0);
        for (_, ttl) in &mut self.punch_peers {
            *ttl -= dt;
        }
        self.punch_peers.retain(|(_, ttl)| *ttl > 0.0);
        if !self.punch_peers.is_empty() {
            self.punch_clock += dt;
            if self.punch_clock >= 0.18 {
                self.punch_clock = 0.0;
                for &(peer, _) in &self.punch_peers {
                    let _ = socket.send_to(b"LCPN", peer);
                }
            }
        }
    }

    pub fn answer_hello(&mut self, socket: &UdpSocket, src: SocketAddr, pkt: &[u8], ticket: PortalTicket) {
        if pkt.len() != 37 || &pkt[0..4] != b"LPK1" || pkt[4] != WIRE_VERSION || self.budget < 1.0 {
            return;
        }
        let Ok(x_star) = pkt[5..37].try_into() else { return };
        let Some(x_point) = CompressedRistretto(x_star).decompress() else { return };
        self.budget -= 1.0;
        self.punch_peers.retain(|(addr, _)| *addr != src);

        let y_secret = SpakePoints::random_scalar();
        let pts = SpakePoints::canonical();
        let w = self.code.password_scalar();
        let y_point = (&y_secret * RISTRETTO_BASEPOINT_TABLE) + (w * pts.n_point);
        let y_star = y_point.compress().to_bytes();
        let shared = y_secret * (x_point - (w * pts.m_point));
        let aead = SpakePoints::transcript_cipher(self.code, &x_star, &y_star, &shared);

        let mut body = Vec::with_capacity(38);
        ticket.encode_bytes(&mut body);
        let mut reply = Vec::with_capacity(53 + body.len());
        reply.extend_from_slice(b"LPK2");
        reply.push(WIRE_VERSION);
        reply.extend_from_slice(&y_star);
        let aad: [u8; 37] = reply[0..37].try_into().expect("37-byte header");
        let tag = aead
            .encrypt_in_place_detached(Nonce::from_slice(&[0u8; 12]), &aad, &mut body)
            .expect("SPAKE2 AEAD seal");
        reply.extend_from_slice(tag.as_slice());
        reply.extend_from_slice(&body);
        let _ = socket.send_to(&reply, src);
    }
}
