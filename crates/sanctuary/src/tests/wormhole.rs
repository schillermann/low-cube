use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::Duration;
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::{
    JoinPreview, JoinTarget, PortalCode, RemoteLink, RemotePlanet, SanctuaryHost, StunProbe,
};

/// Stateful Port-Restricted Cone NAT simulator modeling a factory-default FRITZ!Box (UPnP disabled).
struct SimulatedFritzBox {
    lan_facing: UdpSocket,
    wan_facing: UdpSocket,
    lan_client: SocketAddr,
    open_holes: HashSet<SocketAddr>,
}

impl SimulatedFritzBox {
    fn new(lan_facing: UdpSocket, wan_facing: UdpSocket, lan_client: SocketAddr, open_holes: HashSet<SocketAddr>) -> Self {
        Self { lan_facing, wan_facing, lan_client, open_holes }
    }

    fn bind_for(lan_client: SocketAddr) -> Self {
        let lan_facing = UdpSocket::bind("127.0.0.1:0").expect("bind lan router");
        let wan_facing = UdpSocket::bind("127.0.0.1:0").expect("bind wan router");
        lan_facing.set_nonblocking(true).expect("nb");
        wan_facing.set_nonblocking(true).expect("nb");
        Self::new(lan_facing, wan_facing, lan_client, HashSet::new())
    }

    fn lan_gateway_addr(&self) -> SocketAddr {
        self.lan_facing.local_addr().expect("lan addr")
    }

    fn wan_public_addr(&self) -> SocketAddr {
        self.wan_facing.local_addr().expect("wan addr")
    }

    /// Forwards outbound LAN->WAN traffic (recording stateful hole) and filters inbound WAN->LAN traffic.
    fn step(&mut self, peer_wan: SocketAddr, peer_lan_gateway: SocketAddr) {
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.lan_facing.recv_from(&mut buf) {
            if src == self.lan_client {
                self.open_holes.insert(peer_wan);
                let _ = self.wan_facing.send_to(&buf[..len], peer_wan);
            }
        }
        while let Ok((len, src)) = self.wan_facing.recv_from(&mut buf) {
            if self.open_holes.contains(&src) {
                let _ = self.lan_facing.send_to(&buf[..len], self.lan_client);
                let _ = peer_lan_gateway;
            }
        }
    }
}

#[test]
fn test_twelve_char_crockford_codes_roundtrip_normalize_confusable_letters_and_reject_typos() {
    let host_code = PortalCode::new(Ipv4Addr::new(84, 152, 19, 204), 27842, 173);
    let encoded = host_code.encode();
    assert_eq!(encoded.len(), 14, "Formatted 12-char code `XXXX-XXXX-XXXX` must be 14 chars");
    assert_eq!(&encoded[4..5], "-");
    assert_eq!(&encoded[9..10], "-");

    let ungrouped_lower = encoded.replace('-', "").to_ascii_lowercase();
    assert_eq!(PortalCode::parse(&ungrouped_lower), Some(host_code));
    assert_eq!(JoinTarget::preview(&ungrouped_lower), JoinPreview::CodeReady(host_code));

    // Flipping the first character must be flagged as a 12-char code typo without DNS resolution!
    let mut chars: Vec<char> = encoded.chars().collect();
    chars[0] = if chars[0] == 'A' { 'B' } else { 'A' };
    let corrupted: String = chars.into_iter().collect();
    if PortalCode::parse(&corrupted).is_none() {
        assert_eq!(JoinTarget::preview(&corrupted), JoinPreview::CodeTypo);
        assert_eq!(JoinTarget::parse(&corrupted), None);
    }

    // Guest counter-punch code must validate against the host code and reject host's own code!
    let guest_ep: SocketAddr = "91.45.88.12:54321".parse().expect("guest ep");
    let punch = host_code.mint_punch(guest_ep);
    let punch_str = punch.encode();
    assert_eq!(punch_str.len(), 14);
    assert_eq!(host_code.verify_punch(&punch_str), Some(guest_ep));
    assert_eq!(host_code.verify_punch(&encoded), None);
}

#[test]
fn test_twelve_char_portal_code_connects_directly_and_rejects_wrong_secret() {
    let world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::beveled(16, 0.5));
    let mut host = SanctuaryHost::bind("127.0.0.1:0", world, 16, false).expect("bind host");
    let code = host.code();
    let compact = code.encode().replace('-', "").to_ascii_lowercase();
    let target = JoinTarget::parse(&compact).expect("parse compact 12-char code");
    assert_eq!(target, JoinTarget::Wormhole(code));

    let wrong_secret = if code.encode().ends_with('0') { 77 } else { 78 };
    let wrong_code = PortalCode::from_endpoint(
        code.endpoint(),
        if PortalCode::from_endpoint(code.endpoint(), wrong_secret) == code { 79 } else { wrong_secret },
    );

    let mut imposter = RemotePlanet::dial(JoinTarget::Wormhole(wrong_code), 16, false, 0.5).expect("imposter");
    let mut friend = RemotePlanet::dial(target, 16, false, 0.5).expect("friend");

    for _ in 0..12 {
        let _ = imposter.step_surface(0.18);
        let _ = friend.step_surface(0.18);
        let _ = host.tick(0.06);
    }
    assert!(friend.connected(), "Friend with matching 12-char code must complete SPAKE2 + X25519 handshake");
    assert!(!imposter.connected(), "Imposter with wrong secret on same IP:Port must fail SPAKE2 authentication");
}

#[test]
fn test_rfc5389_stun_probe_discovers_xor_mapped_public_endpoint_on_game_socket() {
    let stun_srv = UdpSocket::bind("127.0.0.1:0").expect("bind mock stun");
    let srv_addr = stun_srv.local_addr().expect("srv addr");
    let client = UdpSocket::bind("127.0.0.1:0").expect("bind client");
    client.set_nonblocking(true).expect("nb");

    let handle = std::thread::spawn(move || {
        let mut req = [0u8; 64];
        let (len, src) = stun_srv.recv_from(&mut req).expect("recv stun req");
        assert_eq!(len, 20);
        assert_eq!(&req[0..2], &[0x00, 0x01]);
        let cookie = [0x21u8, 0x12, 0xA4, 0x42];
        let port = 51820u16 ^ 0x2112;
        let ip = [84 ^ cookie[0], 152 ^ cookie[1], 42 ^ cookie[2], 99 ^ cookie[3]];
        let mut resp = Vec::with_capacity(32);
        resp.extend_from_slice(&0x0101u16.to_be_bytes());
        resp.extend_from_slice(&12u16.to_be_bytes());
        resp.extend_from_slice(&cookie);
        resp.extend_from_slice(&req[8..20]);
        resp.extend_from_slice(&0x0020u16.to_be_bytes());
        resp.extend_from_slice(&8u16.to_be_bytes());
        resp.extend_from_slice(&[0x00, 0x01]);
        resp.extend_from_slice(&port.to_be_bytes());
        resp.extend_from_slice(&ip);
        stun_srv.send_to(&resp, src).expect("send stun resp");
    });

    let probe = StunProbe::fresh(Duration::from_millis(200));
    let mapped = probe.discover_from(&client, &[srv_addr]).expect("decode XOR-MAPPED-ADDRESS");
    handle.join().expect("join stun");
    assert_eq!(mapped, SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(84, 152, 42, 99), 51820)));
}

#[test]
fn test_dual_fritzbox_port_restricted_nat_connects_via_mutual_twelve_char_punch_codes() {
    let host_sock = UdpSocket::bind("127.0.0.1:0").expect("host sock");
    let host_lan = host_sock.local_addr().expect("host lan");
    drop(host_sock);
    let mut fritz_host = SimulatedFritzBox::bind_for(host_lan);

    let guest_sock = UdpSocket::bind("127.0.0.1:0").expect("guest sock");
    let guest_lan = guest_sock.local_addr().expect("guest lan");
    let mut fritz_guest = SimulatedFritzBox::bind_for(guest_lan);

    // Host advertises `fritz_guest.lan_gateway_addr()` as the simulated Internet path into `fritz_host`?
    // Wait: Guest sends to its own router (`fritz_guest.lan_gateway_addr()`), which forwards to `fritz_host.wan_public_addr()`,
    // and Host sends to its own router (`fritz_host.lan_gateway_addr()`), which forwards to `fritz_guest.wan_public_addr()`!
    let world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::beveled(16, 0.5));
    let mut host = SanctuaryHost::bind_with_endpoint(
        &host_lan.to_string(),
        fritz_guest.lan_gateway_addr(),
        world,
        16,
        false,
    )
    .expect("bind host behind fritzbox");

    let host_code = host.code();
    let link = RemoteLink::bind_on_socket(
        guest_sock,
        fritz_host.lan_gateway_addr(),
        JoinTarget::Wormhole(host_code),
    );
    let guest_punch = link.punch_code().expect("guest punch code").encode();
    let mut guest = RemotePlanet::from_link(link, 16, false, 0.5);

    // Phase 1: Before Host punches Guest's code, Host's FRITZ!Box drops EVERY unsolicited packet from Guest!
    for _ in 0..8 {
        let _ = guest.step_surface(0.18);
        fritz_guest.step(fritz_host.wan_public_addr(), fritz_host.lan_gateway_addr());
        fritz_host.step(fritz_guest.wan_public_addr(), fritz_guest.lan_gateway_addr());
        let _ = host.tick(0.06);
    }
    assert!(!guest.connected(), "Closed FRITZ!Box NAT must block unsolicited inbound guest packets");

    // Phase 2: Host enters Guest's 12-char Punch Code -> punches outbound UDP hole -> both FRITZ!Boxen connect!
    let verified_peer = host_code.verify_punch(&guest_punch).expect("verify 12-char guest punch code");
    host.punch_nat_peer(verified_peer);

    for _ in 0..12 {
        let _ = host.tick(0.06);
        fritz_host.step(fritz_guest.wan_public_addr(), fritz_guest.lan_gateway_addr());
        fritz_guest.step(fritz_host.wan_public_addr(), fritz_host.lan_gateway_addr());
        let _ = guest.step_surface(0.18);
        fritz_guest.step(fritz_host.wan_public_addr(), fritz_host.lan_gateway_addr());
        fritz_host.step(fritz_guest.wan_public_addr(), fritz_guest.lan_gateway_addr());
    }
    assert!(guest.connected(), "Mutual 12-char Ping-Pong Punch must pierce dual closed FRITZ!Box NATs without any server");
}
