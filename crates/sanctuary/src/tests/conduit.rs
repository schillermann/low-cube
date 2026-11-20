use std::time::{Duration, Instant};
use glam::{Quat, Vec3};
use world::{
    BiomedCubeWorld, CompanionPresence, PlanetaryWorld, PresenceAttire, PresenceKinematics,
};
use crate::{ConduitPlanet, HostedPlanet, JoinTarget, PortalMailbox, RemotePlanet, SanctuaryHost};

#[test]
fn test_portal_mailbox_atomic_latch_and_halt() {
    let mailbox = PortalMailbox::active();
    assert!(mailbox.running());
    assert!(!mailbox.take_surface_change());
    mailbox.note_surface_change();
    assert!(mailbox.take_surface_change());
    assert!(!mailbox.take_surface_change(), "Surface latch must clear atomically upon consumption");
    mailbox.halt();
    assert!(!mailbox.running());
}

#[test]
fn test_conduit_planet_relays_guest_motion_while_host_render_loop_stalls() {
    let host = SanctuaryHost::bind(
        "127.0.0.1:0",
        Box::new(BiomedCubeWorld::barren_beveled(16, 0.5)),
        16,
    )
    .expect("bind host");
    let addr = host.local_addr().expect("host addr");
    let conduit_host = ConduitPlanet::spawning(Box::new(HostedPlanet::new(host)));
    assert!(conduit_host.active());

    let mut guest_a = RemotePlanet::connect(JoinTarget::Direct(addr), 16, 0.5)
        .expect("guest A connects purely via autonomous SocketLoop while host render thread is idle");
    let mut guest_b = RemotePlanet::connect(JoinTarget::Direct(addr), 16, 0.5)
        .expect("guest B connects purely via autonomous SocketLoop while host render thread is idle");

    let spawn = guest_a.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 0.0);
    let moved_pos = Vec3::new(3.25, spawn.y, 0.25);
    guest_a.sync_presence(CompanionPresence::new(
        0,
        PresenceKinematics::new(moved_pos, Quat::IDENTITY, Quat::IDENTITY, (0.8, 0.0)),
        (1.0, 0.5, 0.0, 0.0),
        PresenceAttire::with_garments([1; 9], [0; 4]),
    ));

    // Host render thread calls ZERO step_surface / RedrawRequested frames during this entire window:
    let started = Instant::now();
    let mut reached_b = false;
    while started.elapsed() < Duration::from_millis(220) {
        let _ = guest_a.step_surface(1.0 / 120.0);
        let _ = guest_b.step_surface(1.0 / 120.0);
        let mut peers = Vec::new();
        guest_b.active_companions(&mut peers);
        if peers.iter().any(|p| p.transfer(|_, k, _, _| k.transfer(|pos, _, _, _, _| pos.x > 2.5))) {
            reached_b = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(4));
    }
    assert!(
        reached_b,
        "Autonomous SocketLoop must relay Guest A motion to Guest B even while Host render loop is completely stalled"
    );
    drop(conduit_host);
}
