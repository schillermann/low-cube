use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use crate::ticket::PortalTicket;

struct PortalRoom {
    host_public: SocketAddr,
    host_lan: SocketAddr,
    guests: Vec<SocketAddr>,
    age_secs: f32,
}

impl PortalRoom {
    fn new(host_public: SocketAddr, host_lan: SocketAddr, guests: Vec<SocketAddr>, age_secs: f32) -> Self {
        Self { host_public, host_lan, guests, age_secs }
    }

    fn slot_for(&mut self, guest: SocketAddr) -> u16 {
        if let Some(pos) = self.guests.iter().position(|&addr| addr == guest) {
            (pos + 1) as u16
        } else if self.guests.len() < 512 {
            self.guests.push(guest);
            self.guests.len() as u16
        } else {
            1
        }
    }

    fn guest_at(&self, slot: u16) -> Option<SocketAddr> {
        let idx = slot.checked_sub(1)? as usize;
        self.guests.get(idx).copied()
    }
}

/// Zero-router-config UDP NAT hole-punching rendezvous and end-to-end encrypted relay fallback.
pub struct RelayBeacon {
    socket: UdpSocket,
    rooms: HashMap<[u8; 8], PortalRoom>,
}

impl RelayBeacon {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(socket: UdpSocket, rooms: HashMap<[u8; 8], PortalRoom>) -> Self {
        Self { socket, rooms }
    }

    /// Secondary constructor binding a non-blocking UDP socket on `bind_addr`.
    pub fn bind(bind_addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self::new(socket, HashMap::new()))
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    fn send_punch(socket: &UdpSocket, recipient: SocketAddr, peer: SocketAddr) {
        let mut msg = Vec::with_capacity(10);
        msg.extend_from_slice(b"LBPU");
        PortalTicket::push_v4(peer, &mut msg);
        let _ = socket.send_to(&msg, recipient);
    }

    /// Pumps non-blocking rendezvous registrations, hole-punch notifications, and encrypted forwards.
    pub fn step(&mut self, dt: f32) {
        for room in self.rooms.values_mut() {
            room.age_secs += dt;
        }
        self.rooms.retain(|_, room| room.age_secs < 45.0);

        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            let pkt = &buf[..len];
            if pkt.len() >= 18 && &pkt[0..4] == b"LBHO" {
                let mut portal_id = [0u8; 8];
                portal_id.copy_from_slice(&pkt[4..12]);
                let lan = PortalTicket::read_v4(&pkt[12..18]).unwrap_or(src);
                if let Some(room) = self.rooms.get_mut(&portal_id) {
                    room.host_public = src;
                    room.host_lan = lan;
                    room.age_secs = 0.0;
                } else {
                    self.rooms.insert(portal_id, PortalRoom::new(src, lan, Vec::new(), 0.0));
                }
                let mut reply = Vec::with_capacity(10);
                reply.extend_from_slice(b"LBRF");
                PortalTicket::push_v4(src, &mut reply);
                let _ = self.socket.send_to(&reply, src);
            } else if pkt.len() >= 12 && &pkt[0..4] == b"LBRV" {
                let mut portal_id = [0u8; 8];
                portal_id.copy_from_slice(&pkt[4..12]);
                if let Some(room) = self.rooms.get_mut(&portal_id) {
                    let _ = room.slot_for(src);
                    Self::send_punch(&self.socket, room.host_public, src);
                    Self::send_punch(&self.socket, src, room.host_public);
                    if room.host_lan != room.host_public {
                        Self::send_punch(&self.socket, src, room.host_lan);
                    }
                }
            } else if pkt.len() > 14 && &pkt[0..4] == b"LBFW" {
                let mut portal_id = [0u8; 8];
                portal_id.copy_from_slice(&pkt[4..12]);
                let slot = u16::from_le_bytes([pkt[12], pkt[13]]);
                let inner = &pkt[14..];
                if let Some(room) = self.rooms.get_mut(&portal_id) {
                    if slot == 0 {
                        let assigned = room.slot_for(src);
                        let mut fwd = Vec::with_capacity(14 + inner.len());
                        fwd.extend_from_slice(b"LBFW");
                        fwd.extend_from_slice(&portal_id);
                        fwd.extend_from_slice(&assigned.to_le_bytes());
                        fwd.extend_from_slice(inner);
                        let _ = self.socket.send_to(&fwd, room.host_public);
                    } else if src == room.host_public {
                        if let Some(guest_addr) = room.guest_at(slot) {
                            let mut fwd = Vec::with_capacity(14 + inner.len());
                            fwd.extend_from_slice(b"LBFW");
                            fwd.extend_from_slice(&portal_id);
                            fwd.extend_from_slice(&0u16.to_le_bytes());
                            fwd.extend_from_slice(inner);
                            let _ = self.socket.send_to(&fwd, guest_addr);
                        }
                    }
                }
            }
        }
    }
}
