use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const STUN_COOKIE: u32 = 0x2112_A442;
const PUBLIC_STUN_HOSTS: &[&str] = &[
    "stun.cloudflare.com:3478",
    "stun.t-online.de:3478",
    "stun.l.google.com:19302",
    "stun.nextcloud.com:443",
];

/// Pure-Rust RFC 5389 STUN binding probe operating directly on the active game `UdpSocket`.
#[derive(Clone, Copy, Debug)]
pub struct StunProbe {
    tx_id: [u8; 12],
    timeout: Duration,
}

impl StunProbe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tx_id: [u8; 12], timeout: Duration) -> Self {
        Self { tx_id, timeout }
    }

    pub fn fresh(timeout: Duration) -> Self {
        let mut tx_id = [0u8; 12];
        getrandom::fill(&mut tx_id).expect("OS CSPRNG entropy unavailable");
        Self::new(tx_id, timeout)
    }

    pub fn request_packet(&self) -> [u8; 20] {
        let mut pkt = [0u8; 20];
        pkt[0..2].copy_from_slice(&0x0001u16.to_be_bytes());
        pkt[2..4].copy_from_slice(&0u16.to_be_bytes());
        pkt[4..8].copy_from_slice(&STUN_COOKIE.to_be_bytes());
        pkt[8..20].copy_from_slice(&self.tx_id);
        pkt
    }

    pub fn parse_response(&self, pkt: &[u8]) -> Option<SocketAddr> {
        if pkt.len() < 20 || u16::from_be_bytes([pkt[0], pkt[1]]) != 0x0101 {
            return None;
        }
        if u32::from_be_bytes([pkt[4], pkt[5], pkt[6], pkt[7]]) != STUN_COOKIE {
            return None;
        }
        if pkt[8..20] != self.tx_id {
            return None;
        }
        let msg_len = (u16::from_be_bytes([pkt[2], pkt[3]]) as usize).min(pkt.len() - 20);
        let attrs = &pkt[20..20 + msg_len];
        let mut pos = 0usize;
        let mut plain_fallback: Option<SocketAddr> = None;
        while pos + 4 <= attrs.len() {
            let kind = u16::from_be_bytes([attrs[pos], attrs[pos + 1]]);
            let len = u16::from_be_bytes([attrs[pos + 2], attrs[pos + 3]]) as usize;
            pos += 4;
            if pos + len > attrs.len() {
                break;
            }
            let val = &attrs[pos..pos + len];
            if kind == 0x0020 && val.len() >= 8 && val[1] == 0x01 {
                let port = u16::from_be_bytes([val[2], val[3]]) ^ ((STUN_COOKIE >> 16) as u16);
                let cookie_bytes = STUN_COOKIE.to_be_bytes();
                let ip = Ipv4Addr::new(
                    val[4] ^ cookie_bytes[0],
                    val[5] ^ cookie_bytes[1],
                    val[6] ^ cookie_bytes[2],
                    val[7] ^ cookie_bytes[3],
                );
                if port != 0 && !ip.is_unspecified() {
                    return Some(SocketAddr::V4(SocketAddrV4::new(ip, port)));
                }
            } else if kind == 0x0001 && val.len() >= 8 && val[1] == 0x01 {
                let port = u16::from_be_bytes([val[2], val[3]]);
                let ip = Ipv4Addr::new(val[4], val[5], val[6], val[7]);
                if port != 0 && !ip.is_unspecified() {
                    plain_fallback = Some(SocketAddr::V4(SocketAddrV4::new(ip, port)));
                }
            }
            pos += (len + 3) & !3;
        }
        plain_fallback
    }

    pub fn discover_from(&self, socket: &UdpSocket, servers: &[SocketAddr]) -> Option<SocketAddr> {
        if servers.is_empty() {
            return None;
        }
        let req = self.request_packet();
        for &srv in servers {
            let _ = socket.send_to(&req, srv);
        }
        let deadline = Instant::now() + self.timeout;
        let mut buf = [0u8; 512];
        while Instant::now() < deadline {
            while let Ok((len, _)) = socket.recv_from(&mut buf) {
                if let Some(mapped) = self.parse_response(&buf[..len]) {
                    return Some(mapped);
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
        None
    }

    pub fn discover(&self, socket: &UdpSocket) -> Option<SocketAddr> {
        let (tx, rx) = mpsc::channel();
        for &host in PUBLIC_STUN_HOSTS {
            let sender = tx.clone();
            thread::spawn(move || {
                if let Ok(addrs) = host.to_socket_addrs() {
                    for addr in addrs.filter(SocketAddr::is_ipv4) {
                        let _ = sender.send(addr);
                        break;
                    }
                }
            });
        }
        drop(tx);
        let dns_deadline = Instant::now() + Duration::from_millis(160);
        let mut resolved = Vec::with_capacity(PUBLIC_STUN_HOSTS.len());
        while resolved.len() < PUBLIC_STUN_HOSTS.len() {
            let now = Instant::now();
            if now >= dns_deadline {
                break;
            }
            match rx.recv_timeout(dns_deadline - now) {
                Ok(addr) => resolved.push(addr),
                Err(_) => break,
            }
        }
        self.discover_from(socket, &resolved)
    }
}
