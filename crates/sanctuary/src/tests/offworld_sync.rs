use glam::{Quat, Vec3};
use world::{
    BlockMedium, BlockMold, BodyId, CelestialSystem, CompanionPortal, CompanionPresence,
    MoldArchetype, PlanetaryWorld, PresenceAttire, PresenceKinematics, SolarProportions,
    WorldEarthworks, WorldFact,
};
use crate::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};

#[test]
fn test_offworld_fact_codec_roundtrip_and_nesting_guard() {
    let inner = WorldFact::Harvested {
        eye: Vec3::new(0.25, 9.4, 0.25),
        direction: Vec3::NEG_Y,
    };
    let scoped = WorldFact::OffWorld { body: BodyId::MARS, fact: Box::new(inner) };
    let mut bytes = Vec::new();
    scoped.inscribe(&mut bytes);
    let (decoded, used) = WorldFact::decode(&bytes).expect("decode OffWorld");
    assert_eq!(used, bytes.len());
    assert_eq!(decoded, scoped);

    let mut nested = vec![15u8];
    nested.extend_from_slice(&BodyId::EUROPA.raw_code().to_le_bytes());
    nested.extend_from_slice(&bytes);
    assert!(WorldFact::decode(&nested).is_none(), "Nested OffWorld wire frames must be rejected");
}

#[test]
fn test_coop_mars_and_europa_earthworks_sync_across_moving_orbits_and_snapshots() {
    let prop = SolarProportions::compressed(16, 0.002);
    let host_world = CelestialSystem::barren_heliosphere(prop, 0.5);
    let host = SanctuaryHost::bind_cosmos("127.0.0.1:0", host_world, prop).expect("bind cosmos host");
    let addr = host.local_addr().expect("host addr");
    let mut hosted = HostedPlanet::spawning(host);

    let mars_c0 = hosted.body_center(BodyId::MARS).expect("Mars center");
    let mars_top0 = hosted.constrain_to_surface(mars_c0 + Vec3::new(0.25, 12.0, 0.25), 0.0);
    let local_probe = mars_top0 - mars_c0;
    let dug = hosted.harvest_surface(mars_top0 + Vec3::Y * 1.4, Vec3::NEG_Y);
    assert!(dug.is_some(), "Host must harvest Mars surface block");
    let mars_after_host = hosted.constrain_to_surface(mars_c0 + local_probe, 0.0);
    assert!(
        mars_after_host.y < mars_top0.y - 0.20,
        "before={} after={}",
        mars_top0.y,
        mars_after_host.y
    );

    // Return host to Earth so orbital epoch advances Mars to a new inertial coordinate
    let _ = hosted.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0);
    for _ in 0..8 {
        let _ = hosted.step_surface(2.5);
    }

    let mut guest = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 0.5)
        .expect("guest connect cosmos");
    let mars_c1 = guest.body_center(BodyId::MARS).expect("Guest Mars center");
    guest.sync_presence(CompanionPresence::new(
        0,
        PresenceKinematics::new(mars_c1 + local_probe, Quat::IDENTITY, Quat::IDENTITY, (1.4, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::astronaut(),
    ));
    for _ in 0..8 {
        let _ = guest.step_surface(0.03);
        let _ = hosted.step_surface(0.03);
    }

    let mars_c1 = guest.body_center(BodyId::MARS).expect("Guest Mars center");
    let guest_mars_top = guest.constrain_to_surface(mars_c1 + local_probe, 0.0);
    let expected_local_y = mars_after_host.y - mars_c0.y;
    assert!(
        ((guest_mars_top.y - mars_c1.y) - expected_local_y).abs() < 0.05,
        "Late-joining guest must receive Host's Mars excavation in body-local coordinates across orbital motion"
    );

    // Guest flies to Europa and builds a quarried stone block while Host remains on Earth
    let europa_c = guest.body_center(BodyId::EUROPA).expect("Europa center");
    let europa_stand = guest.constrain_to_surface(europa_c + Vec3::new(0.25, 24.0, 0.25), 0.0);
    guest.sync_presence(CompanionPresence::new(
        0,
        PresenceKinematics::new(europa_stand, Quat::IDENTITY, Quat::IDENTITY, (1.4, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::astronaut(),
    ));
    for _ in 0..6 {
        let _ = guest.pump_portal(0.06);
        let _ = hosted.pump_portal(0.06);
    }
    let europa_harvest = guest.constrain_to_surface(europa_c + Vec3::new(1.25, 24.0, 0.25), 0.0);
    assert!(guest.harvest_surface(europa_harvest + Vec3::Y * 1.4, Vec3::NEG_Y).is_some());
    for _ in 0..6 {
        let _ = guest.pump_portal(0.06);
        let _ = hosted.pump_portal(0.06);
    }
    let mold = BlockMold::with_medium(
        MoldArchetype::FullCube,
        world::CardinalTurn::Forward,
        BlockMedium::QuarriedStone,
    );
    let europa_build_spot = guest.constrain_to_surface(europa_c + Vec3::new(-0.75, 24.0, 0.25), 0.0);
    assert!(guest.place_molded_surface_block(
        europa_build_spot + Vec3::Y * 1.6,
        Vec3::NEG_Y,
        europa_stand,
        mold,
    ));
    for _ in 0..8 {
        let _ = guest.pump_portal(0.06);
        let _ = hosted.pump_portal(0.06);
    }

    let host_europa_c = hosted.body_center(BodyId::EUROPA).expect("Host Europa center");
    let host_europa_built = hosted.constrain_to_surface(host_europa_c + Vec3::new(-0.75, 24.0, 0.25), 0.0);
    assert!(
        (host_europa_built.y - host_europa_c.y) > (europa_build_spot.y - europa_c.y) + 0.35,
        "Guest's Europa block build must synchronize to Host without evicting Host's prior Earth view"
    );
}
