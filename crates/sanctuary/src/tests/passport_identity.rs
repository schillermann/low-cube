use std::time::{SystemTime, UNIX_EPOCH};
use glam::{Quat, Vec3};
use world::{
    BodyId, Callsign, CelestialSystem, CompanionPortal, CompanionPresence, CosmicFrame,
    PlanetaryWorld, PresenceAttire, PresenceKinematics, ProximityUtterance, SatchelManifest,
    SolarProportions, SurvivorWelcome,
};
use crate::{
    ChronicleLog, CompactedChronicle, DiskPassports, DiskVaultChronicle, DiskVaultSurvivors,
    HandshakeIdentity, JoinTarget, MemoryPassports, MemorySurvivors, PassportClaim, PassportVault,
    RemoteLink, RemotePlanet, SanctuaryHost,
};

fn temp_sandbox(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("low_cube_{label}_{nanos}"));
    std::fs::create_dir_all(&dir).expect("mkdir sandbox");
    dir
}

#[test]
fn test_server_issues_identity_to_client_vault_and_restores_inventory_and_roster_toml() {
    let sandbox = temp_sandbox("server_passport");
    let passport_path = sandbox.join("passports.toml");
    let host_key_path = sandbox.join("world.wal.host.key");
    let wal_path = sandbox.join("world.wal");
    let surv_path = sandbox.join("world.wal.survivors");
    let roster_toml_path = sandbox.join("world.wal.survivors.toml");

    let prop = SolarProportions::true_scale(16);
    let starter = SatchelManifest::parse_server_kit("gold_ore:2, iron_ore:3", "server.toml");
    let mars_spot;

    {
        let host_keys = HandshakeIdentity::load_or_create(&host_key_path);
        let mut world1 = CelestialSystem::barren_heliosphere(prop, 0.5);
        let chronicle1 = Box::new(
            DiskVaultChronicle::replay_and_open(
                &wal_path,
                CompactedChronicle::wrapping(ChronicleLog::empty()),
                world1.as_mut(),
            )
            .expect("wal1"),
        );
        let survivors1 = Box::new(
            DiskVaultSurvivors::replay_and_open(&surv_path, MemorySurvivors::with_starter(starter))
                .expect("survivors1"),
        );
        let mut host1 = SanctuaryHost::bind_cosmos_with_identity(
            "127.0.0.1:0",
            host_keys,
            Callsign::host(),
            world1,
            prop,
            chronicle1,
            survivors1,
        )
        .expect("host1");
        let ticket1 = host1.ticket();

        let vault1 = Box::new(DiskPassports::open(&passport_path, Callsign::sanitized("Tom")));
        assert_eq!(vault1.claim_for(JoinTarget::Ticket(ticket1)), PassportClaim::Unassigned);

        let mut link1 = RemoteLink::bind_with_vault(JoinTarget::Ticket(ticket1), vault1).expect("link1");
        for _ in 0..8 {
            let _ = host1.tick(0.02);
            if link1.step_cosmos_handshake(0.02).is_some() {
                break;
            }
        }
        assert!(link1.connected());
        assert_eq!(link1.assigned_callsign().as_str(), "Tom");

        let reloaded_vault = DiskPassports::open(&passport_path, Callsign::unnamed());
        assert_eq!(reloaded_vault.stamp_count(), 1, "Client must persist server-issued passport");
        assert!(matches!(
            reloaded_vault.claim_for(JoinTarget::Ticket(ticket1)),
            PassportClaim::Issued(_)
        ));

        let mut tom1 = RemotePlanet::from_cosmos_link(link1, prop, 0.5);
        assert_eq!(tom1.assigned_survivor(), Some(SurvivorWelcome::FreshKit(starter)));

        tom1.anchor_to_frame(CosmicFrame::Body(BodyId::MARS), Vec3::new(0.25, 24.0, 0.25));
        mars_spot = tom1.constrain_to_surface(Vec3::new(1.75, 24.0, -0.75), 0.0);
        tom1.sync_presence(CompanionPresence::new(
            0,
            PresenceKinematics::new(mars_spot, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
            (0.0, 0.0, 0.0, 0.0),
            PresenceAttire::astronaut(),
        ));
        tom1.note_crafted_recipe(11);
        for _ in 0..6 {
            let _ = tom1.pump_portal(0.03);
            let _ = host1.tick(0.03);
        }
    }

    let roster_text = std::fs::read_to_string(&roster_toml_path).expect("roster toml");
    assert!(roster_text.contains("name = \"Tom\""), "Server roster TOML must map callsign to inventory");
    assert!(roster_text.contains("fusion_torch:1"), "Server roster TOML must list survivor items");

    let host_keys2 = HandshakeIdentity::load_or_create(&host_key_path);
    let mut world2 = CelestialSystem::barren_heliosphere(prop, 0.5);
    let chronicle2 = Box::new(
        DiskVaultChronicle::replay_and_open(
            &wal_path,
            CompactedChronicle::wrapping(ChronicleLog::empty()),
            world2.as_mut(),
        )
        .expect("wal2"),
    );
    let survivors2 = Box::new(
        DiskVaultSurvivors::replay_and_open(&surv_path, MemorySurvivors::with_starter(starter))
            .expect("survivors2"),
    );
    let mut host2 = SanctuaryHost::bind_cosmos_with_identity(
        "127.0.0.1:0",
        host_keys2,
        Callsign::host(),
        world2,
        prop,
        chronicle2,
        survivors2,
    )
    .expect("host2");
    let ticket2 = host2.ticket();

    let vault2 = Box::new(DiskPassports::open(&passport_path, Callsign::unnamed()));
    let mut link2 = RemoteLink::bind_with_vault(JoinTarget::Ticket(ticket2), vault2).expect("link2");
    for _ in 0..8 {
        let _ = host2.tick(0.02);
        if link2.step_cosmos_handshake(0.02).is_some() {
            break;
        }
    }
    assert!(link2.connected());
    assert_eq!(link2.assigned_callsign().as_str(), "Tom");
    let tom2 = RemotePlanet::from_cosmos_link(link2, prop, 0.5);
    let Some(SurvivorWelcome::Restored(restored)) = tom2.assigned_survivor() else {
        panic!("Server must restore survivor inventory when client presents server-issued passport");
    };
    assert_eq!(restored.callsign().as_str(), "Tom");
    assert_eq!(restored.frame(), CosmicFrame::Body(BodyId::MARS));
    assert!((restored.position() - mars_spot).length() < 0.02);
    assert_eq!(
        restored.manifest().count_of(world::RelicCatalog::from_medium(world::BlockMedium::FusionTorch)),
        1
    );

    let _ = std::fs::remove_dir_all(&sandbox);
}

#[test]
fn test_duplicate_callsigns_disambiguate_identities_and_format_proximity_chat() {
    let prop = SolarProportions::true_scale(16);
    let mut host = SanctuaryHost::bind_named_cosmos(
        "127.0.0.1:0",
        Callsign::sanitized("Alex"),
        CelestialSystem::barren_heliosphere(prop, 0.5),
        prop,
    )
    .expect("bind host");
    let ticket = JoinTarget::Ticket(host.ticket());

    let stand = host.world().constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0);
    host.sync_host_presence(CompanionPresence::new(
        1,
        PresenceKinematics::new(stand, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::astronaut(),
    ));

    let mut link_a = RemoteLink::bind_with_vault(
        ticket,
        Box::new(MemoryPassports::with_callsign(Callsign::sanitized("Tom"))),
    )
    .expect("link_a");
    for _ in 0..6 {
        let _ = host.tick(0.02);
        if link_a.step_cosmos_handshake(0.02).is_some() {
            break;
        }
    }
    assert_eq!(link_a.assigned_callsign().as_str(), "Tom");

    let mut link_b = RemoteLink::bind_with_vault(
        ticket,
        Box::new(MemoryPassports::with_callsign(Callsign::sanitized("Tom"))),
    )
    .expect("link_b");
    for _ in 0..6 {
        let _ = host.tick(0.02);
        if link_b.step_cosmos_handshake(0.02).is_some() {
            break;
        }
    }
    assert_eq!(link_b.assigned_callsign().as_str(), "Tom#2");

    let mut tom_a = RemotePlanet::from_cosmos_link(link_a, prop, 0.5);
    let mut tom_b = RemotePlanet::from_cosmos_link(link_b, prop, 0.5);
    let pres = CompanionPresence::new(
        0,
        PresenceKinematics::new(stand, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::astronaut(),
    );
    tom_a.sync_presence(pres);
    tom_b.sync_presence(pres);
    for _ in 0..4 {
        let _ = tom_a.pump_portal(0.03);
        let _ = tom_b.pump_portal(0.03);
        let _ = host.tick(0.03);
    }

    tom_a.utter_nearby("Ich bin der erste Tom");
    tom_b.utter_nearby("Ich bin der zweite Tom");
    host.utter_host_nearby("Willkommen an Bord");
    for _ in 0..5 {
        let _ = tom_a.pump_portal(0.03);
        let _ = tom_b.pump_portal(0.03);
        let _ = host.tick(0.03);
    }

    let mut host_heard: Vec<ProximityUtterance> = Vec::new();
    host.drain_host_utterances(&mut host_heard);
    let host_lines: Vec<String> = host_heard.iter().map(ProximityUtterance::formatted_line).collect();
    assert!(host_lines.contains(&"Tom: Ich bin der erste Tom".to_string()));
    assert!(host_lines.contains(&"Tom#2: Ich bin der zweite Tom".to_string()));

    let mut tom_a_heard: Vec<ProximityUtterance> = Vec::new();
    tom_a.drain_utterances(&mut tom_a_heard);
    let tom_a_lines: Vec<String> = tom_a_heard.iter().map(ProximityUtterance::formatted_line).collect();
    assert!(tom_a_lines.contains(&"Tom#2: Ich bin der zweite Tom".to_string()));
    assert!(tom_a_lines.contains(&"Alex: Willkommen an Bord".to_string()));
}

#[test]
fn test_passport_vault_isolates_identities_across_distinct_servers() {
    let sandbox = temp_sandbox("multi_realm");
    let passport_path = sandbox.join("passports.toml");
    let prop = SolarProportions::true_scale(16);

    let mut server_a = SanctuaryHost::bind_cosmos("127.0.0.1:0", CelestialSystem::barren_heliosphere(prop, 0.5), prop).expect("a");
    let mut server_b = SanctuaryHost::bind_cosmos("127.0.0.1:0", CelestialSystem::barren_heliosphere(prop, 0.5), prop).expect("b");
    let target_a = JoinTarget::Ticket(server_a.ticket());
    let target_b = JoinTarget::Ticket(server_b.ticket());

    let mut link_a = RemoteLink::bind_with_vault(target_a, Box::new(DiskPassports::open(&passport_path, Callsign::sanitized("Lisa")))).expect("la");
    for _ in 0..6 {
        let _ = server_a.tick(0.02);
        if link_a.step_cosmos_handshake(0.02).is_some() { break; }
    }
    let mut link_b = RemoteLink::bind_with_vault(target_b, Box::new(DiskPassports::open(&passport_path, Callsign::sanitized("Lisa")))).expect("lb");
    for _ in 0..6 {
        let _ = server_b.tick(0.02);
        if link_b.step_cosmos_handshake(0.02).is_some() { break; }
    }

    let book = DiskPassports::open(&passport_path, Callsign::unnamed());
    assert_eq!(book.stamp_count(), 2);
    assert_ne!(book.claim_for(target_a), book.claim_for(target_b), "Each server must issue its own distinct secret identity");

    let _ = std::fs::remove_dir_all(&sandbox);
}
