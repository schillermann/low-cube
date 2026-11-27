use std::collections::HashSet;
use std::net::UdpSocket;
use glam::{Quat, Vec3};
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, CompanionPortal, CompanionPresence, CraftBlueprint,
    CraftIdentity, CraftPose, DrivenCraft, ExposedStratum, PlanetaryWorld, PresenceAttire,
    PresenceKinematics, ProximityUtterance, VehicleAxes, WorldEarthworks, WorldFact,
};
use crate::seal_batch::{OutboundFrame, ParallelSeals, PulseDelivery, SequentialDelivery};
use crate::{
    ChronicleLog, ChronicleStore, CompactedChronicle, DiskVaultChronicle, GuestWard,
    HandshakeIdentity, HostedPlanet, JoinTarget, MtuBudgetAudience, ProximityAudience,
    PulseAudience, RemoteLink, RemotePlanet, SanctuaryHost, SmoothRoster, WardVitals,
    WorldAudience, INTIMATE_NEIGHBOR_CAP, NEARBY_SIGHT_METERS, PACKET_COMPANION_CAP,
};

fn companion_at(slot: u32, pos: Vec3) -> CompanionPresence {
    let kin = PresenceKinematics::new(pos, Quat::IDENTITY, Quat::IDENTITY, (1.6, 0.0));
    CompanionPresence::new(slot, kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::astronaut())
}

#[test]
fn test_pulse_audience_decorator_chain_culls_distant_players_and_rotates_hotspots() {
    let mut roster = Vec::new();
    roster.push(companion_at(1, Vec3::new(0.0, 512.0, 0.0)));
    for i in 0..60u32 {
        let dist = 2.0 + (i as f32) * 0.6;
        roster.push(companion_at(10 + i, Vec3::new(dist, 512.0, 0.0)));
    }
    roster.push(companion_at(500, Vec3::new(900.0, 512.0, 0.0)));
    roster.push(companion_at(501, Vec3::new(12_000_000.0, 0.0, 0.0)));

    let crafts: Vec<DrivenCraft> = Vec::new();
    let audience = MtuBudgetAudience::new(
        ProximityAudience::new(WorldAudience::new(&roster, &crafts), NEARBY_SIGHT_METERS),
        INTIMATE_NEIGHBOR_CAP,
        PACKET_COMPANION_CAP,
    );

    let mut cursor = 0usize;
    let mut seen_slots = HashSet::new();
    for _ in 0..8 {
        let mut out_c = Vec::new();
        let mut out_v = Vec::new();
        audience.select_for(1, Vec3::new(0.0, 512.0, 0.0), &mut cursor, &mut out_c, &mut out_v);
        assert_eq!(out_c.len(), PACKET_COMPANION_CAP, "Hotspot packet must stay capped at MTU budget");
        assert!(!out_c.iter().any(|c| c.slot() >= 500), "Distant and off-planet players must be 0 Hz");
        for immediate in 10..18u32 {
            assert!(
                out_c.iter().any(|c| c.same_slot(immediate)),
                "Closest {INTIMATE_NEIGHBOR_CAP} neighbors must appear in every pulse"
            );
        }
        for c in out_c {
            seen_slots.insert(c.slot());
        }
    }
    assert_eq!(seen_slots.len(), 60, "Round-Robin must visit all 60 local hotspot neighbors without starvation");
}

#[test]
fn test_smooth_roster_sight_lease_prevents_hotspot_flicker_and_expires_departures() {
    let mut roster = SmoothRoster::empty();
    let first_batch: Vec<CompanionPresence> = (2..2 + PACKET_COMPANION_CAP as u32)
        .map(|s| companion_at(s, Vec3::new(s as f32, 64.0, 0.0)))
        .collect();
    roster.reconcile_snapshot(&first_batch, &[]);
    roster.advance_tick(0.05);

    let second_batch: Vec<CompanionPresence> = (10..10 + PACKET_COMPANION_CAP as u32)
        .map(|s| companion_at(s, Vec3::new(s as f32, 64.0, 0.0)))
        .collect();
    roster.reconcile_snapshot(&second_batch, &[]);
    let mut visible = Vec::new();
    roster.copy_companions(&mut visible);
    assert!(
        visible.len() > PACKET_COMPANION_CAP,
        "Rotated neighbors must remain visible under 0.85s Sight-Lease instead of flickering"
    );

    roster.advance_tick(1.0);
    roster.reconcile_snapshot(&second_batch, &[]);
    roster.copy_companions(&mut visible);
    assert_eq!(visible.len(), PACKET_COMPANION_CAP, "Unrefreshed neighbors must expire cleanly after lease window");
}

#[test]
fn test_compacted_and_disk_vault_chronicle_decorators_deduplicate_and_fold_wal() {
    let wal_path = std::env::temp_dir().join(format!("sanctuary_wal_{}.bin", std::process::id()));
    let _ = std::fs::remove_file(&wal_path);

    let cid = CraftIdentity::new(7);
    let part = world::VehiclePart::new(
        (0, 0),
        0.0,
        BlockMold::atom_from_stamp([0, 0, BlockMedium::FlightConsole.code()]),
    );
    let bp = CraftBlueprint::new(cid, vec![part]);
    let stratum = ExposedStratum::new((2, 3, -4), BlockMedium::GoldOre);
    {
        let mut world = BiomedCubeWorld::of_size(4);
        let base = CompactedChronicle::wrapping(ChronicleLog::empty());
        let mut vault = DiskVaultChronicle::replay_and_open(&wal_path, base, &mut world).expect("open wal");
        vault.record(WorldFact::StratumDisclosed { face: 0, stratum });
        vault.record(WorldFact::StratumDisclosed { face: 0, stratum });
        vault.record(WorldFact::CraftCommissioned { blueprint: bp.clone() });
        vault.record(WorldFact::CraftCommissioned { blueprint: bp.clone() });
        let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
        for x in [1.0f32, 2.0, 3.0] {
            vault.record(WorldFact::HelmClaimed { craft: cid, pilot_slot: 2, eye: Vec3::Y * 4.0, direction: Vec3::NEG_Y });
            let pose = CraftPose::parked(Vec3::new(x, 2.5, 0.0), axes, Vec3::Z);
            vault.record(WorldFact::HelmReleased { craft: cid, pose });
        }
        assert_eq!(vault.total(), 8, "Duplicate stratum and blueprint commissions must be compacted immediately");
    }
    let mut rebooted_world = BiomedCubeWorld::of_size(4);
    let rebooted = DiskVaultChronicle::replay_and_open(
        &wal_path,
        CompactedChronicle::wrapping(ChronicleLog::empty()),
        &mut rebooted_world,
    )
    .expect("replay wal");
    assert_eq!(rebooted.total(), 3, "WAL replay must fold 3 claim+park cycles down to the single latest parked pose");
    let _ = std::fs::remove_file(&wal_path);
}

#[test]
fn test_admin_block_ward_and_spawn_stone_round_robin_plus_proximity_chat() {
    let base_world = Box::new(BiomedCubeWorld::of_size(32));
    let raw_host = SanctuaryHost::bind("127.0.0.1:0", base_world, 32).expect("bind host");
    let addr = raw_host.local_addr().expect("addr");
    let mut host_planet = HostedPlanet::new(raw_host);

    let spawn_mold = BlockMold::cube().in_medium(BlockMedium::SpawnStone);
    let placed_a = host_planet.place_molded_surface_block(
        Vec3::new(3.25, 19.0, 0.25),
        Vec3::NEG_Y,
        Vec3::new(1.25, 17.0, 0.25),
        spawn_mold,
    );
    let placed_b = host_planet.place_molded_surface_block(
        Vec3::new(-3.25, 19.0, 0.25),
        Vec3::NEG_Y,
        Vec3::new(-1.25, 17.0, 0.25),
        spawn_mold,
    );
    assert!(placed_a && placed_b, "Admin host must be allowed to place SpawnStone plinths");

    let mut link_a = RemoteLink::bind(JoinTarget::Direct(addr)).expect("guest A");
    let mut link_b = RemoteLink::bind(JoinTarget::Direct(addr)).expect("guest B");
    for _ in 0..20 {
        let _ = host_planet.step_surface(0.02);
        let _ = link_a.step_handshake(0.02);
        let _ = link_b.step_handshake(0.02);
        if link_a.connected() && link_b.connected() {
            break;
        }
    }
    assert!(link_a.connected() && link_b.connected());
    let mut guest_a = RemotePlanet::from_link(link_a, 32, 1.0);
    let mut guest_b = RemotePlanet::from_link(link_b, 32, 1.0);
    let sp_a = guest_a.assigned_spawn_point().expect("assigned SpawnStone A");
    let sp_b = guest_b.assigned_spawn_point().expect("assigned SpawnStone B");
    assert!((sp_a.x - sp_b.x).abs() > 4.0, "Joining guests must distribute across distinct Admin SpawnStones");

    guest_a.sync_presence(companion_at(2, Vec3::new(3.25, 17.5, 1.25)));
    guest_b.sync_presence(companion_at(3, Vec3::new(500.0, 17.5, 0.25)));
    host_planet.sync_presence(companion_at(1, Vec3::new(2.0, 17.5, 0.25)));
    for _ in 0..8 {
        let _ = guest_a.step_surface(0.02);
        let _ = guest_b.step_surface(0.02);
        let _ = host_planet.step_surface(0.02);
    }

    assert!(
        !guest_a.place_molded_surface_block(
            Vec3::new(4.25, 19.0, 1.25),
            Vec3::NEG_Y,
            Vec3::new(3.25, 17.5, 1.25),
            spawn_mold,
        ) || {
            for _ in 0..6 {
                let _ = guest_a.step_surface(0.02);
                let _ = host_planet.step_surface(0.02);
            }
            let mut pts = Vec::new();
            host_planet.collect_spawn_points(&mut pts);
            pts.len() == 2
        },
        "AdminBlockWard must reject guest SpawnStone placement on authoritative server"
    );

    let _ = guest_a.harvest_surface(Vec3::new(3.25, 19.0, 0.25), Vec3::NEG_Y);
    for _ in 0..6 {
        let _ = guest_a.step_surface(0.02);
        let _ = host_planet.step_surface(0.02);
    }
    let mut beacons_after = Vec::new();
    host_planet.collect_spawn_points(&mut beacons_after);
    assert_eq!(beacons_after.len(), 2, "AdminBlockWard must protect Admin SpawnStones from guest harvesting");

    guest_a.utter_nearby("Hallo am Spawn");
    for _ in 0..6 {
        let _ = guest_a.step_surface(0.02);
        let _ = host_planet.step_surface(0.02);
        let _ = guest_b.step_surface(0.02);
    }
    let mut host_heard: Vec<ProximityUtterance> = Vec::new();
    let mut far_heard: Vec<ProximityUtterance> = Vec::new();
    host_planet.drain_utterances(&mut host_heard);
    guest_b.drain_utterances(&mut far_heard);
    assert_eq!(host_heard.len(), 1, "Nearby player within earshot must receive proximity chat");
    assert!(far_heard.is_empty(), "Far-away player (> 96 m) must receive zero proximity chat");
}

#[test]
fn test_parallel_seals_decorator_encrypts_large_cohorts_across_threads() {
    let tx_sock = UdpSocket::bind("127.0.0.1:0").expect("tx");
    let host_keys = HandshakeIdentity::generate();
    let mut wards = Vec::new();
    let mut rx_socks = Vec::new();
    let mut guest_seals = Vec::new();
    for i in 0..16u32 {
        let rx = UdpSocket::bind("127.0.0.1:0").expect("rx");
        rx.set_nonblocking(true).expect("nb");
        let addr = rx.local_addr().expect("addr");
        let g_keys = HandshakeIdentity::generate();
        let h_seal = host_keys.establish(g_keys.public_bytes(), false);
        let g_seal = g_keys.establish(host_keys.public_bytes(), true);
        wards.push((i + 2, GuestWard::new(h_seal, addr, companion_at(i + 2, Vec3::ZERO), WardVitals::fresh())));
        rx_socks.push(rx);
        guest_seals.push(g_seal);
    }
    let mut frames: Vec<OutboundFrame<'_>> = wards
        .iter_mut()
        .map(|(cid, w)| OutboundFrame::prepare(*cid, w, vec![b"pulse".to_vec()]))
        .collect();
    ParallelSeals::hardware(SequentialDelivery::new()).deliver(&tx_sock, &mut frames);
    let mut buf = [0u8; 256];
    for (rx, g_seal) in rx_socks.iter().zip(guest_seals.iter_mut()) {
        let (len, _) = rx.recv_from(&mut buf).expect("sealed datagram");
        let (_, plain) = g_seal.open_frame(&buf[..len]).expect("authentic AEAD tag");
        assert_eq!(plain, b"pulse");
    }
}
