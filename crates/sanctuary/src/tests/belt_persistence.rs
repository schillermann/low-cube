use glam::{Quat, Vec3};
use world::{
    CelestialSystem, CompanionPortal, CompanionPresence, CosmicFrame, PlanetaryWorld,
    PresenceAttire, PresenceKinematics, SolarProportions, SurvivorWelcome,
};
use crate::{HandshakeIdentity, JoinTarget, RemotePlanet, SanctuaryHost};

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
            PresenceAttire::astronaut(),
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
