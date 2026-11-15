use glam::{Quat, Vec3};
use world::{
    BiomedCubeWorld, CompanionPresence, MeshTarget, PlanetaryWorld, PresenceKinematics, Quad,
};
use crate::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};

struct NullTarget;

impl MeshTarget for NullTarget {
    fn push_quad(&mut self, _quad: Quad) {}
}

fn stand_at(world: &mut dyn PlanetaryWorld, spot: Vec3) {
    let kin = PresenceKinematics::new(spot, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
    world.sync_presence(CompanionPresence::new(0, kin, (0.0, 0.0, 0.0, 0.0), ([0; 9], [0; 4])));
}

fn step_trio(host: &mut HostedPlanet, alice: &mut RemotePlanet, bob: &mut RemotePlanet, rounds: usize) {
    for _ in 0..rounds {
        let _ = alice.step_surface(0.06);
        let _ = bob.step_surface(0.06);
        let _ = host.step_surface(0.06);
    }
}

fn drain_all_slices(world: &mut dyn PlanetaryWorld) -> Vec<usize> {
    let mut drained = Vec::new();
    let mut sink = NullTarget;
    while let Some(idx) = world.drain_modified_slice(&mut sink) {
        drained.push(idx);
    }
    drained
}

#[test]
fn test_host_and_guests_excavate_simultaneously_without_wiping_cracks_or_slices() {
    let inner: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::beveled(16, 0.5));
    let raw_host = SanctuaryHost::bind("127.0.0.1:0", inner, 16, false).expect("bind host");
    let addr = raw_host.local_addr().expect("host addr");
    let mut host = HostedPlanet::new(raw_host);
    let mut alice = RemotePlanet::dial(JoinTarget::Direct(addr), 16, false, 0.5).expect("dial alice");
    let mut bob = RemotePlanet::dial(JoinTarget::Direct(addr), 16, false, 0.5).expect("dial bob");

    step_trio(&mut host, &mut alice, &mut bob, 8);
    assert!(alice.connected() && bob.connected());

    let host_site = host.constrain_to_surface(Vec3::new(-2.25, 16.0, 0.25), 0.0);
    let alice_site = alice.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0);
    let bob_east = bob.constrain_to_surface(Vec3::new(16.0, 0.25, 0.25), 0.0);
    stand_at(&mut host, host_site);
    stand_at(&mut alice, alice_site);
    stand_at(&mut bob, bob_east);
    step_trio(&mut host, &mut alice, &mut bob, 4);
    let _ = (drain_all_slices(&mut host), drain_all_slices(&mut alice), drain_all_slices(&mut bob));

    assert!(host.loosen_surface(host_site + Vec3::Y * 1.5, Vec3::NEG_Y, 2).is_some());
    assert!(alice.loosen_surface(alice_site + Vec3::Y * 1.5, Vec3::NEG_Y, 3).is_some());
    assert!(bob.loosen_surface(bob_east + Vec3::X * 1.5, Vec3::NEG_X, 2).is_some());
    step_trio(&mut host, &mut alice, &mut bob, 3);

    assert!(alice.loosen_surface(alice_site + Vec3::Y * 1.5, Vec3::NEG_Y, 0).is_some());
    step_trio(&mut host, &mut alice, &mut bob, 3);

    assert!(host.harvest_surface(host_site + Vec3::Y * 1.5, Vec3::NEG_Y).is_some());
    assert!(alice.harvest_surface(alice_site + Vec3::Y * 1.5, Vec3::NEG_Y).is_some());
    assert!(bob.harvest_surface(bob_east + Vec3::X * 1.5, Vec3::NEG_X).is_some());
    step_trio(&mut host, &mut alice, &mut bob, 6);

    let alice_drained = drain_all_slices(&mut alice);
    assert!(alice_drained.len() >= 2, "Multi-face simultaneous P2P mining must retain every modified slice");

    for peer in [&host as &dyn PlanetaryWorld, &alice, &bob] {
        let h_after = peer.constrain_to_surface(Vec3::new(-2.25, 16.0, 0.25), 0.0).y;
        let a_after = peer.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0).y;
        let b_after = peer.constrain_to_surface(Vec3::new(16.0, 0.25, 0.25), 0.0).x;
        assert!((host_site.y - h_after - 0.5).abs() < 0.05, "Host shaft must converge across all peers");
        assert!((alice_site.y - a_after - 0.5).abs() < 0.05, "Alice shaft must converge across all peers");
        assert!((bob_east.x - b_after - 0.5).abs() < 0.05, "Bob east-face shaft must converge across all peers");
    }
}
