mod wormhole;

use std::net::UdpSocket;
use glam::{Quat, Vec3};
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, CompanionPresence, CubePlanet, Face,
    FacetedReliefFace, FoldedFace, HollowVeins, LivingVeins, NullFace, PlanetaryWorld,
    PresenceKinematics, WorldFact,
};
use crate::{JoinTarget, PortalTicket, RemotePlanet, SanctuaryEnvelope, SanctuaryHost};

fn build_ore_test_planet(authoritative: bool) -> CubePlanet {
    let half = 16.0;
    let faces: Vec<Box<dyn Face>> = world::ClimateZone::all()
        .into_iter()
        .map(|zone| -> Box<dyn Face> {
            let veins: Box<dyn world::SubterraneanVeins> = if authoritative {
                Box::new(LivingVeins::with_secret(0xDEAD_BEEF_CAFE_F00D, 0x1234_5678_9ABC_DEF0))
            } else {
                Box::new(HollowVeins::untouched())
            };
            let face = FacetedReliefFace::in_zone_with_veins(zone, half, 64, veins);
            Box::new(FoldedFace::in_zone(Box::new(face), zone, half))
        })
        .collect();
    CubePlanet::new(faces, Box::new(NullFace))
}

fn pump_pair(host: &mut SanctuaryHost, guest_a: &mut RemotePlanet, guest_b: &mut RemotePlanet, rounds: usize) {
    for _ in 0..rounds {
        let _ = guest_a.step_surface(0.06);
        let _ = guest_b.step_surface(0.06);
        let _ = host.tick(0.06);
    }
}

#[test]
fn test_encrypted_multiplayer_companions_dig_and_build_together_on_shared_cube() {
    let world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::beveled(16, 0.5));
    let mut host = SanctuaryHost::bind("127.0.0.1:0", world, 16, false).expect("bind host");
    let ticket = host.ticket();
    let encoded = ticket.encode();
    let parsed = JoinTarget::parse(&encoded).expect("parse LC1 ticket");

    let mut alice = RemotePlanet::dial(parsed, 16, false, 0.5).expect("dial alice");
    let mut bob = RemotePlanet::dial(JoinTarget::Direct(host.local_addr().expect("addr")), 16, false, 0.5).expect("dial bob");

    pump_pair(&mut host, &mut alice, &mut bob, 8);
    assert!(alice.connected() && bob.connected(), "Both guests must complete X25519+ChaCha20 handshake");

    let spawn = alice.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0);
    let alice_kin = PresenceKinematics::new(spawn, Quat::IDENTITY, Quat::IDENTITY, (0.9, 0.0));
    let bob_kin = PresenceKinematics::new(spawn + Vec3::X * 1.5, Quat::IDENTITY, Quat::IDENTITY, (0.4, 0.0));
    alice.sync_presence(CompanionPresence::new(0, alice_kin, (1.0, 0.5, 0.0, 0.0), ([3, 1, 2, 0, 0, 0, 0, 0, 0], [0; 4])));
    bob.sync_presence(CompanionPresence::new(0, bob_kin, (0.2, 0.0, 0.0, 0.0), ([5, 2, 1, 0, 0, 0, 0, 0, 0], [0; 4])));
    pump_pair(&mut host, &mut alice, &mut bob, 4);

    let mut seen_by_bob = Vec::new();
    bob.active_companions(&mut seen_by_bob);
    assert_eq!(seen_by_bob.len(), 1, "Bob must see Alice's companion avatar on the same cube face");

    let eye = spawn + Vec3::Y * 1.55;
    let before_alice = alice.constrain_to_surface(spawn, 0.0).y;
    assert!(alice.harvest_surface(eye, Vec3::NEG_Y).is_some());
    let after_predict = alice.constrain_to_surface(spawn, 0.0).y;
    assert!((before_alice - after_predict - 0.5).abs() < 0.05, "Alice digs exactly 1 layer locally");

    pump_pair(&mut host, &mut alice, &mut bob, 6);
    let after_echo_alice = alice.constrain_to_surface(spawn, 0.0).y;
    let after_sync_bob = bob.constrain_to_surface(spawn, 0.0).y;
    assert!((after_echo_alice - after_predict).abs() < 0.02, "Chronicle echo must never double-dig Alice's pit");
    assert!((after_sync_bob - after_predict).abs() < 0.02, "Bob's cube must reflect Alice's excavated shaft");

    let place_eye = Vec3::new(1.25, spawn.y + 1.55, 0.25);
    alice.sync_presence(CompanionPresence::new(0, PresenceKinematics::new(Vec3::new(2.25, spawn.y, 0.25), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)), (0.0, 0.0, 0.0, 0.0), ([0; 9], [0; 4])));
    pump_pair(&mut host, &mut alice, &mut bob, 2);
    assert!(alice.place_molded_surface_block(place_eye, Vec3::NEG_Y, Vec3::new(2.25, spawn.y, 0.25), BlockMold::cube().in_medium(BlockMedium::QuarriedStone)));
    pump_pair(&mut host, &mut alice, &mut bob, 6);
    let bob_pillar = bob.constrain_to_surface(Vec3::new(1.25, 20.0, 0.25), 0.0).y;
    assert!(bob_pillar > spawn.y + 0.3, "Bob must stand on the stone block placed by Alice");
}

#[test]
fn test_zero_knowledge_xray_shield_withholds_hidden_ores_until_adjacent_block_excavated() {
    let mut host = SanctuaryHost::bind(
        "127.0.0.1:0",
        Box::new(build_ore_test_planet(true)),
        16,
        false,
    )
    .expect("bind ore host");
    let addr = host.local_addr().expect("addr");
    let mut guest = RemotePlanet::new(
        Box::new(build_ore_test_planet(false)),
        crate::RemoteLink::bind(JoinTarget::Direct(addr)).expect("link"),
        std::collections::VecDeque::new(),
        crate::DirtySlices::clean(),
    );
    let mut observer = RemotePlanet::new(
        Box::new(build_ore_test_planet(false)),
        crate::RemoteLink::bind(JoinTarget::Direct(addr)).expect("link2"),
        std::collections::VecDeque::new(),
        crate::DirtySlices::clean(),
    );
    pump_pair(&mut host, &mut guest, &mut observer, 6);

    let mut disclosed_ores = 0usize;
    for idx in 0..8 {
        let x = 0.25 + (idx as f32) * 1.5;
        for _ in 0..8 {
            let cur_ground = guest.constrain_to_surface(Vec3::new(x, 28.0, 0.25), 0.0);
            guest.sync_presence(CompanionPresence::new(0, PresenceKinematics::new(cur_ground, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)), (0.0, 0.0, 0.0, 0.0), ([0; 9], [0; 4])));
            pump_pair(&mut host, &mut guest, &mut observer, 1);
            let _ = guest.harvest_surface(cur_ground + Vec3::Y * 1.2, Vec3::NEG_Y);
            pump_pair(&mut host, &mut guest, &mut observer, 5);
        }
    }
    for idx in 0..8 {
        let x = 0.25 + (idx as f32) * 1.5;
        let pit_floor = observer.constrain_to_surface(Vec3::new(x, 28.0, 0.25), 0.0);
        for h in [0.30, 0.80, 1.30, 1.80] {
            for dir in [Vec3::NEG_Y, Vec3::Z, Vec3::NEG_Z, Vec3::X, Vec3::NEG_X] {
                if let Some(med) = observer.loosen_surface(pit_floor + Vec3::Y * h, dir, 1) {
                    if med.mineral_ore() {
                        disclosed_ores += 1;
                    }
                }
            }
        }
    }
    assert!(disclosed_ores > 0, "Excavating shafts must dynamically disclose adjacent mineral seams to clients");
}

#[test]
fn test_sanctuary_ward_rejects_remote_telekinesis_speed_mining_and_block_duping() {
    let world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::beveled(16, 0.5));
    let mut host = SanctuaryHost::bind("127.0.0.1:0", world, 16, false).expect("bind host");
    let ticket = host.ticket();

    let sock = UdpSocket::bind("127.0.0.1:0").expect("bind cheater");
    sock.set_nonblocking(true).expect("nb");
    let cheater_keys = crate::HandshakeIdentity::generate();
    let mut knock = Vec::from(&b"LKNK"[..]);
    knock.extend_from_slice(&cheater_keys.public_bytes());
    sock.send_to(&knock, ticket.direct()).expect("send knock");
    let _ = host.tick(0.05);

    let mut buf = [0u8; 1500];
    let _ = sock.recv_from(&mut buf).expect("recv welcome");
    let cid = u32::from_le_bytes(buf[36..40].try_into().expect("cid"));
    let mut seal = cheater_keys.establish(ticket.host_key(), true);

    let stand = host.world().constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0);
    let (mut payload, mut frame) = (Vec::new(), Vec::new());
    let pulse = SanctuaryEnvelope::GuestPulse {
        ack_seq: 0,
        presence: CompanionPresence::new(cid, PresenceKinematics::new(stand, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)), (0.0, 0.0, 0.0, 0.0), ([0; 9], [0; 4])),
    };
    pulse.inscribe(&mut payload);
    seal.seal(cid, &payload, &mut frame);
    sock.send_to(&frame, ticket.direct()).expect("pulse");

    let far_spot = host.world().constrain_to_surface(Vec3::new(-6.25, 16.0, -6.25), 0.0);
    SanctuaryEnvelope::GuestPetition {
        fact: WorldFact::Harvested { eye: far_spot + Vec3::Y * 1.5, direction: Vec3::NEG_Y },
    }
    .inscribe(&mut payload);
    seal.seal(cid, &payload, &mut frame);
    sock.send_to(&frame, ticket.direct()).expect("telekinesis");
    let _ = host.tick(0.01);
    let far_after = host.world().constrain_to_surface(Vec3::new(-6.25, 16.0, -6.25), 0.0);
    assert!((far_after.y - far_spot.y).abs() < 1e-4, "Out-of-reach telekinesis mining must be rejected");

    for _ in 0..12 {
        let cur = host.world().constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0);
        SanctuaryEnvelope::GuestPetition {
            fact: WorldFact::Harvested { eye: cur + Vec3::Y * 1.4, direction: Vec3::NEG_Y },
        }
        .inscribe(&mut payload);
        seal.seal(cid, &payload, &mut frame);
        sock.send_to(&frame, ticket.direct()).expect("flood");
    }
    let _ = host.tick(0.001);
    let after_flood = host.world().constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0);
    assert!(stand.y - after_flood.y <= 1.05, "Packet-flood speed-mining bot must be rate-limited to <= 2 layers");
}

#[test]
fn test_sanctuary_plan_and_join_target_support_zero_flag_player_defaults() {
    use crate::{SanctuaryEndpoint, SanctuaryPlan, SanctuaryToml};

    let args = vec!["low-cube-server".to_string()];
    let default_plan = SanctuaryPlan::assembled(&args, &SanctuaryToml::new(String::new()));
    assert_eq!(default_plan, SanctuaryPlan::new(SanctuaryEndpoint::new(27777), 128));

    let toml = SanctuaryToml::new("[world]\nsize = 2\n[server]\nport = 28888\n".to_string());
    let configured = SanctuaryPlan::assembled(&args, &toml);
    assert_eq!(configured, SanctuaryPlan::new(SanctuaryEndpoint::new(28888), 256));

    let default_port_ip: std::net::SocketAddr = "127.0.0.1:27777".parse().expect("default ip");
    assert_eq!(JoinTarget::parse("127.0.0.1"), Some(JoinTarget::Direct(default_port_ip)));
    let ticket = PortalTicket::new([7u8; 32], default_port_ip);
    let pasted_line = format!("  Invite Ticket: --join {}  ", ticket.encode());
    assert_eq!(JoinTarget::parse(&pasted_line), Some(JoinTarget::Ticket(ticket)));
}
