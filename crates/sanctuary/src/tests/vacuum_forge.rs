use glam::{Quat, Vec3};
use world::{
    BodyId, CelestialSystem, CompanionPortal, CompanionPresence, CosmicFrame, PlanetaryWorld,
    PresenceAttire, PresenceKinematics, SatchelManifest, SolarProportions,
};
use crate::{
    ChronicleLog, CompactedChronicle, HandshakeIdentity, JoinTarget, MemorySurvivors, RemoteLink,
    RemotePlanet, SanctuaryHost,
};

#[test]
fn test_sanctuary_host_rejects_vacuum_titanium_in_atmosphere_and_accepts_on_airless_luna() {
    let prop = SolarProportions::compressed(16, 0.002);
    let starter = SatchelManifest::parse_server_kit("crust_block:2, iron_ore:3, deadwood_bough:2", "test");
    let world = CelestialSystem::barren_heliosphere(prop, 0.5);
    let chronicle = Box::new(CompactedChronicle::wrapping(ChronicleLog::empty()));
    let survivors = Box::new(MemorySurvivors::with_starter(starter));
    let mut host = SanctuaryHost::bind_cosmos_with_stores("127.0.0.1:0", world, prop, chronicle, survivors)
        .expect("bind host");
    let addr = host.local_addr().expect("host addr");
    let raw_secret = [7u8; 32];

    {
        let mut link = RemoteLink::bind_with_identity(
            JoinTarget::Direct(addr),
            HandshakeIdentity::from_secret_bytes(raw_secret),
        )
        .expect("bind guest");
        for _ in 0..8 {
            let _ = host.tick(0.02);
            if link.step_cosmos_handshake(0.02).is_some() {
                break;
            }
        }
        assert!(link.connected());
        let mut guest = RemotePlanet::from_cosmos_link(link, prop, 0.5);

        // 1. On atmospheric Mars: vacuum titanium (#103) is rejected, while atmospheric iron bloom (#102) succeeds!
        guest.anchor_to_frame(CosmicFrame::Body(BodyId::MARS), Vec3::new(0.25, 8.0, 0.25));
        let mars_spot = guest.constrain_to_surface(Vec3::new(0.25, 8.0, 0.25), 0.0);
        guest.sync_presence(CompanionPresence::new(
            0,
            PresenceKinematics::new(mars_spot, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
            (0.0, 0.0, 0.0, 0.0),
            PresenceAttire::squire(),
        ));
        for _ in 0..4 {
            let _ = guest.pump_portal(0.02);
            let _ = host.tick(0.02);
        }
        guest.note_crafted_recipe(103);
        guest.note_crafted_recipe(102);
        for _ in 0..6 {
            let _ = guest.pump_portal(0.02);
            let _ = host.tick(0.02);
        }

        // 2. On airless Luna: wood-fired bloomery (#102) is rejected, while Space-Forged vacuum titanium (#103) succeeds!
        guest.anchor_to_frame(CosmicFrame::Body(BodyId::LUNA), Vec3::new(0.25, 8.0, 0.25));
        let luna_spot = guest.constrain_to_surface(Vec3::new(0.25, 8.0, 0.25), 0.0);
        guest.sync_presence(CompanionPresence::new(
            0,
            PresenceKinematics::new(luna_spot, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
            (0.0, 0.0, 0.0, 0.0),
            PresenceAttire::squire(),
        ));
        for _ in 0..4 {
            let _ = guest.pump_portal(0.02);
            let _ = host.tick(0.02);
        }
        guest.note_crafted_recipe(102);
        guest.note_crafted_recipe(103);
        for _ in 0..6 {
            let _ = guest.pump_portal(0.02);
            let _ = host.tick(0.02);
        }
    }

    let mut link2 = RemoteLink::bind_with_identity(
        JoinTarget::Direct(addr),
        HandshakeIdentity::from_secret_bytes(raw_secret),
    )
    .expect("rebind guest");
    for _ in 0..8 {
        let _ = host.tick(0.02);
        if link2.step_cosmos_handshake(0.02).is_some() {
            break;
        }
    }
    let guest2 = RemotePlanet::from_cosmos_link(link2, prop, 0.5);
    let manifest = guest2.assigned_survivor().expect("assigned survivor").manifest();
    assert_eq!(manifest.count_of(47), 1, "Only Mars atmospheric iron bloom (#102) may succeed; Luna #102 must be rejected");
    assert_eq!(manifest.count_of(48), 2, "Only Luna vacuum titanium (#103) may succeed; Mars #103 must be rejected");
    assert_eq!(manifest.count_of(12), 1, "Exactly 1 CrustBlock consumed by the valid Luna #103 firing");
}
