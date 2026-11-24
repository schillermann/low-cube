use std::time::{SystemTime, UNIX_EPOCH};
use glam::{Quat, Vec3};
use world::{
    BodyId, CelestialSystem, CompanionPortal, CompanionPresence, CosmicFrame, PilotCraft,
    PlanetaryWorld, PresenceAttire, PresenceKinematics, SatchelManifest, SolarProportions,
    SurvivorWelcome, WorldEarthworks,
};
use crate::{
    ChronicleLog, CompactedChronicle, DiskVaultChronicle, DiskVaultSurvivors, HandshakeIdentity,
    JoinTarget, MemorySurvivors, RemotePlanet, SanctuaryHost,
};

fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("low_cube_{label}_{nanos}"));
    std::fs::create_dir_all(&dir).expect("mkdir temp");
    dir
}

#[test]
fn test_persistent_x25519_keyfile_and_disk_vault_survivors_survive_offworld_server_restart() {
    let sandbox = unique_temp_dir("survivor_wal");
    let key_path = sandbox.join("alice.key");
    let wal_path = sandbox.join("sanctuary.wal");
    let surv_path = sandbox.join("sanctuary.wal.survivors");

    let keys_first = HandshakeIdentity::load_or_create(&key_path);
    let keys_reloaded = HandshakeIdentity::load_or_create(&key_path);
    assert_eq!(keys_first.public_bytes(), keys_reloaded.public_bytes());
    assert_eq!(keys_first.secret_bytes(), keys_reloaded.secret_bytes());

    let prop = SolarProportions::true_scale(16);
    let server_kit = SatchelManifest::parse_server_kit("gold_ore:2, iron_ore:3, spawn_stone:5", "server.toml");
    assert_eq!(server_kit.count_of(29), 0, "Admin SpawnStone must be stripped from guest starter kits");
    assert_eq!(server_kit.count_of(23), 2);
    assert_eq!(server_kit.count_of(22), 3);

    let mars_stand_local;
    {
        let mut world1 = CelestialSystem::barren_heliosphere(prop, 0.5);
        let chronicle1 = Box::new(
            DiskVaultChronicle::replay_and_open(
                &wal_path,
                CompactedChronicle::wrapping(ChronicleLog::empty()),
                world1.as_mut(),
            )
            .expect("open wal1"),
        );
        let survivors1 = Box::new(
            DiskVaultSurvivors::replay_and_open(&surv_path, MemorySurvivors::with_starter(server_kit))
                .expect("open survivors1"),
        );
        let mut host1 = SanctuaryHost::bind_cosmos_with_stores(
            "127.0.0.1:0",
            world1,
            prop,
            chronicle1,
            survivors1,
        )
        .expect("bind host1");
        let addr1 = host1.local_addr().expect("addr1");

        let mut link1 = crate::RemoteLink::bind_with_identity(JoinTarget::Direct(addr1), keys_first)
            .expect("bind link1");
        for _ in 0..8 {
            let _ = host1.tick(0.02);
            if link1.step_cosmos_handshake(0.02).is_some() {
                break;
            }
        }
        assert!(link1.connected());
        let mut alice1 = RemotePlanet::from_cosmos_link(link1, prop, 0.5);
        assert_eq!(alice1.assigned_survivor(), Some(SurvivorWelcome::FreshKit(server_kit)));

        alice1.anchor_to_frame(CosmicFrame::Body(BodyId::MARS), Vec3::new(0.25, 24.0, 0.25));
        mars_stand_local = alice1.constrain_to_surface(Vec3::new(2.25, 24.0, -1.25), 0.0);
        alice1.sync_presence(CompanionPresence::new(
            0,
            PresenceKinematics::new(mars_stand_local, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
            (0.0, 0.0, 0.0, 0.0),
            PresenceAttire::squire(),
        ));
        alice1.note_crafted_recipe(17);
        alice1.note_crafted_recipe(17);
        for _ in 0..6 {
            let _ = alice1.pump_portal(0.03);
            let _ = host1.tick(0.03);
        }
        let mine_spot = alice1.constrain_to_surface(Vec3::new(0.25, 24.0, 0.25), 0.0);
        assert!(alice1.harvest_surface(mine_spot + Vec3::Y * 1.4, Vec3::NEG_Y).is_some());
        for _ in 0..6 {
            let _ = alice1.pump_portal(0.03);
            let _ = host1.tick(0.03);
        }
    }

    let mut world2 = CelestialSystem::barren_heliosphere(prop, 0.5);
    world2.align_epoch(3_600.0);
    let chronicle2 = Box::new(
        DiskVaultChronicle::replay_and_open(
            &wal_path,
            CompactedChronicle::wrapping(ChronicleLog::empty()),
            world2.as_mut(),
        )
        .expect("open wal2"),
    );
    let survivors2 = Box::new(
        DiskVaultSurvivors::replay_and_open(&surv_path, MemorySurvivors::with_starter(server_kit))
            .expect("open survivors2"),
    );
    let mut host2 = SanctuaryHost::bind_cosmos_with_stores(
        "127.0.0.1:0",
        world2,
        prop,
        chronicle2,
        survivors2,
    )
    .expect("bind host2");
    let addr2 = host2.local_addr().expect("addr2");

    let mut link2 = crate::RemoteLink::bind_with_identity(
        JoinTarget::Direct(addr2),
        HandshakeIdentity::load_or_create(&key_path),
    )
    .expect("bind link2");
    for _ in 0..8 {
        let _ = host2.tick(0.02);
        if link2.step_cosmos_handshake(0.02).is_some() {
            break;
        }
    }
    assert!(link2.connected());
    let alice2 = RemotePlanet::from_cosmos_link(link2, prop, 0.5);
    assert_eq!(alice2.cosmic_frame(), CosmicFrame::Body(BodyId::MARS));

    let Some(SurvivorWelcome::Restored(restored)) = alice2.assigned_survivor() else {
        panic!("Expected Restored SurvivorRecord after WAL server restart");
    };
    assert_eq!(restored.frame(), CosmicFrame::Body(BodyId::MARS));
    assert!((restored.position() - mars_stand_local).length() < 0.02);
    assert_eq!(restored.manifest().count_of(28), 1, "Legitimate FusionTorch craft must persist");
    assert_eq!(restored.manifest().count_of(23), 0, "Consumed GoldOre must not duplicate");
    assert_eq!(restored.manifest().count_of(22), 0, "Consumed IronOre must not duplicate");

    let _ = std::fs::remove_dir_all(&sandbox);
}

#[test]
fn test_rapid_reconnect_before_timeout_preserves_survivor_checkpoint_and_renews_seal() {
    let prop = SolarProportions::true_scale(16);
    let mut host = SanctuaryHost::bind_cosmos(
        "127.0.0.1:0",
        CelestialSystem::barren_heliosphere(prop, 0.5),
        prop,
    )
    .expect("bind host");
    let addr = host.local_addr().expect("addr");
    let secret = HandshakeIdentity::generate().secret_bytes();

    let mut link_a = crate::RemoteLink::bind_with_identity(
        JoinTarget::Direct(addr),
        HandshakeIdentity::from_secret_bytes(secret),
    )
    .expect("link_a");
    for _ in 0..6 {
        let _ = host.tick(0.02);
        if link_a.step_cosmos_handshake(0.02).is_some() {
            break;
        }
    }
    let mut alice_a = RemotePlanet::from_cosmos_link(link_a, prop, 0.5);
    let target_spot = alice_a.constrain_to_surface(Vec3::new(-3.25, 20.0, 4.25), 0.0);
    alice_a.sync_presence(CompanionPresence::new(
        0,
        PresenceKinematics::new(target_spot, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::squire(),
    ));
    alice_a.sync_pilot_craft(PilotCraft::Pedestrian);
    for _ in 0..4 {
        let _ = alice_a.pump_portal(0.03);
        let _ = host.tick(0.03);
    }
    drop(alice_a);

    let mut link_b = crate::RemoteLink::bind_with_identity(
        JoinTarget::Direct(addr),
        HandshakeIdentity::from_secret_bytes(secret),
    )
    .expect("link_b");
    for _ in 0..6 {
        let _ = host.tick(0.02);
        if link_b.step_cosmos_handshake(0.02).is_some() {
            break;
        }
    }
    let mut alice_b = RemotePlanet::from_cosmos_link(link_b, prop, 0.5);
    let Some(SurvivorWelcome::Restored(rec)) = alice_b.assigned_survivor() else {
        panic!("Immediate reconnect before 12s timeout must restore live checkpoint");
    };
    assert!((rec.position() - target_spot).length() < 0.02);

    let dig_spot = alice_b.constrain_to_surface(Vec3::new(-2.75, 20.0, 4.25), 0.0);
    alice_b.sync_presence(CompanionPresence::new(
        0,
        PresenceKinematics::new(target_spot, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::squire(),
    ));
    for _ in 0..3 {
        let _ = alice_b.pump_portal(0.03);
        let _ = host.tick(0.03);
    }
    assert!(
        alice_b.harvest_surface(dig_spot + Vec3::Y * 1.4, Vec3::NEG_Y).is_some(),
        "Renewed SessionSeal must accept fresh tx_seq=1 packets immediately"
    );
}

#[test]
fn test_belt_asteroid_logout_restores_moving_voxel_crust_after_orbital_drift() {
    let prop = SolarProportions::compressed(64, 0.02);
    let mut host = SanctuaryHost::bind_cosmos(
        "127.0.0.1:0",
        CelestialSystem::barren_heliosphere(prop, 0.5),
        prop,
    )
    .expect("bind belt host");
    let addr = host.local_addr().expect("addr");
    let eph = host.world().celestial_atlas().ephemeris().expect("ephemeris");
    let sun = eph.snapshot_all(0.0)[0];
    let belt = world::AsteroidBelt::new(prop, 0.0);
    let au = prop.astronomical_unit();
    let (rock, cell) = (0..240)
        .find_map(|step| {
            let a = (step as f64) * 0.026;
            let probe = sun.center() + glam::DVec3::new(2.70 * au * a.cos(), 0.0, 2.70 * au * a.sin());
            belt.nearest_rock_with_cell(probe, sun.center(), sun.gm(), 420.0)
        })
        .expect("belt rock at ~2.7 AU");
    let belt_frame = CosmicFrame::BeltAsteroid { cell, id: rock.id() };
    let secret = HandshakeIdentity::generate().secret_bytes();

    let rock_stand;
    {
        let mut link1 = crate::RemoteLink::bind_with_identity(
            JoinTarget::Direct(addr),
            HandshakeIdentity::from_secret_bytes(secret),
        )
        .expect("link1");
        for _ in 0..6 {
            let _ = host.tick(0.02);
            if link1.step_cosmos_handshake(0.02).is_some() {
                break;
            }
        }
        let mut miner = RemotePlanet::from_cosmos_link(link1, prop, 0.5);
        miner.anchor_to_frame(belt_frame, Vec3::new(0.25, rock.half_size() + 2.0, 0.25));
        rock_stand = miner.constrain_to_surface(Vec3::new(0.25, rock.half_size() + 2.0, 0.25), 0.0);
        miner.sync_presence(CompanionPresence::new(
            0,
            PresenceKinematics::new(rock_stand, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
            (0.0, 0.0, 0.0, 0.0),
            PresenceAttire::squire(),
        ));
        for _ in 0..4 {
            let _ = miner.pump_portal(0.03);
            let _ = host.tick(0.03);
        }
    }

    host.align_epoch(7_200.0);
    let _ = host.tick(13.0);

    let mut link2 = crate::RemoteLink::bind_with_identity(
        JoinTarget::Direct(addr),
        HandshakeIdentity::from_secret_bytes(secret),
    )
    .expect("link2");
    for _ in 0..6 {
        let _ = host.tick(0.02);
        if link2.step_cosmos_handshake(0.02).is_some() {
            break;
        }
    }
    let miner2 = RemotePlanet::from_cosmos_link(link2, prop, 0.5);
    assert_eq!(miner2.cosmic_frame(), belt_frame);
    let Some(SurvivorWelcome::Restored(rec)) = miner2.assigned_survivor() else {
        panic!("Expected Restored BeltAsteroid record after 2-hour orbital drift");
    };
    assert_eq!(rec.frame(), belt_frame);
    let landed = miner2.constrain_to_surface(rec.position(), 0.0);
    assert!((landed - rock_stand).length() < 0.02, "Survivor must stand on exact moving BeltRock voxel");
    assert!(miner2.surface_contact(rec.position() + Vec3::Y * 1.0, 0.0).landable());
}
