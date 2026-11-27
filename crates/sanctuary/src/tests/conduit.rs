use std::time::{Duration, Instant};
use glam::{Quat, Vec3};
use world::{
    BiomedQuadSphere, CompanionPortal, CompanionPresence, PilotCraft, PlanetaryWorld,
    PresenceAttire, PresenceKinematics, PresenceRhythm,
};
use crate::{
    CompanionGlide, HostedPlanet, JoinTarget, PortalMailbox, RemotePlanet, SanctuaryHost,
};

#[test]
fn test_portal_mailbox_atomic_standby_and_craft_count() {
    let now = Instant::now();
    let mailbox = PortalMailbox::active();
    assert!(mailbox.running());
    mailbox.record_main_beat(now);
    assert!(
        !mailbox.standby_needed(now + Duration::from_millis(8)),
        "SocketLoop must stay lock-free dormant while main simulation loop steps every <= 16.7ms"
    );
    assert!(
        mailbox.standby_needed(now + Duration::from_millis(25)),
        "SocketLoop standby must activate once main simulation loop stalls >= 20ms"
    );
    assert_eq!(mailbox.active_crafts(), 0);
    mailbox.note_active_crafts(2);
    assert_eq!(mailbox.active_crafts(), 2);
    mailbox.halt();
    assert!(!mailbox.running());
}

#[test]
fn test_hosted_planet_standby_wire_relays_guest_motion_without_planet_mutex() {
    let host = SanctuaryHost::bind(
        "127.0.0.1:0",
        Box::new(BiomedQuadSphere::barren(16)),
        16,
    )
    .expect("bind host");
    let addr = host.local_addr().expect("host addr");
    let standby_host = HostedPlanet::spawning(host);
    assert!(standby_host.active());

    let mut guest_a = RemotePlanet::connect(JoinTarget::Direct(addr), 16, 0.5)
        .expect("guest A connects purely via HostWire standby while host render thread is idle")
        .spawning();
    let mut guest_b = RemotePlanet::connect(JoinTarget::Direct(addr), 16, 0.5)
        .expect("guest B connects purely via HostWire standby while host render thread is idle")
        .spawning();

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
    while started.elapsed() < Duration::from_millis(260) {
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
        "Standby HostWire must relay Guest A motion to Guest B without locking PlanetaryWorld"
    );
    drop(standby_host);
}

#[test]
fn test_presence_rhythm_wraps_stride_phase_and_cleave_forward_without_backwards_whip() {
    let from = PresenceRhythm::new(5.95, 0.45, 0.90, 0.0);
    let target = PresenceRhythm::new(0.30, 0.45, 0.0, 0.0);
    let mid = from.interpolated(target, 0.5).wire_tuple();
    let dist_from_pi = (mid.0 - std::f32::consts::PI).abs();
    assert!(
        dist_from_pi > 2.5,
        "Stride phase wrap across TAU must step forward near 0/TAU instead of whipping backwards through PI (got {})",
        mid.0
    );
    assert!(
        mid.2 > 0.92,
        "Ending cleave swing must complete forward toward 1.0 instead of rewinding through 0.45 (got {})",
        mid.2
    );
}

#[test]
fn test_immutable_companion_glide_maintains_dead_stop_free_stride_under_arrival_jitter() {
    let make_pres = |x: f32| {
        CompanionPresence::new(
            2,
            PresenceKinematics::new(Vec3::new(x, 8.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
            ((x * 2.4).rem_euclid(std::f32::consts::TAU), 0.45, 0.0, 0.0),
            PresenceAttire::astronaut(),
        )
    };
    let mut glide = CompanionGlide::anchored(0.0, make_pres(0.0), PilotCraft::Pedestrian);
    let dt = 1.0 / 120.0_f64;
    let speed = 6.0_f32;
    let mut sim_time = 0.0_f64;
    let mut next_arrival = 0.012_f64;
    let mut short_leg = false;
    let mut prev_x = 0.0_f32;
    let mut frozen_ticks = 0usize;

    for tick in 0..72 {
        sim_time += dt;
        if sim_time >= next_arrival {
            glide = glide.absorbed(sim_time, make_pres(speed * (sim_time as f32)), PilotCraft::Pedestrian);
            let jittered = if short_leg { 0.012 } else { 0.026 };
            short_leg = !short_leg;
            next_arrival = sim_time + jittered;
        }
        let cur_x = glide.presence_at(sim_time).transfer(|_, k, _, _| k.transfer(|p, _, _, _, _| p.x));
        if tick >= 6 {
            assert!(cur_x >= prev_x - 1e-5, "Immutable trajectory must never step backwards ({cur_x} < {prev_x})");
            if (cur_x - prev_x).abs() < 1e-4 {
                frozen_ticks += 1;
            }
        }
        prev_x = cur_x;
    }
    assert_eq!(frozen_ticks, 0, "Walking companion must never hit dead-stop clamping during jittered 60 Hz pulses");
}

#[test]
fn test_standby_host_ignores_uncorrelated_sawtooth_subtick_without_time_reversal() {
    let host = SanctuaryHost::bind(
        "127.0.0.1:0",
        Box::new(BiomedQuadSphere::barren(16)),
        16,
    )
    .expect("bind host");
    let addr = host.local_addr().expect("host addr");
    let mut standby_host = HostedPlanet::spawning(host);
    let mut walker = RemotePlanet::connect(JoinTarget::Direct(addr), 16, 0.5).expect("connect walker");

    let mut x = 0.25f32;
    let mut prev_seen_x = -1.0f32;
    for step_idx in 0..24 {
        x += 0.06;
        walker.sync_presence(CompanionPresence::new(
            0,
            PresenceKinematics::new(Vec3::new(x, 8.5, 0.25), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
            ((x * 2.4).rem_euclid(std::f32::consts::TAU), 0.45, 0.0, 0.0),
            PresenceAttire::astronaut(),
        ));
        let _ = walker.step_surface(1.0 / 120.0);
        let uncorrelated_sawtooth = if step_idx % 2 == 0 { 0.0082 } else { 0.0001 };
        standby_host.blend_companions(uncorrelated_sawtooth);
        let mut peers = Vec::new();
        standby_host.active_companions(&mut peers);
        if let Some(peer) = peers.first() {
            let seen_x = peer.transfer(|_, k, _, _| k.transfer(|p, _, _, _, _| p.x));
            if prev_seen_x > 0.30 {
                assert!(
                    seen_x >= prev_seen_x - 1e-5,
                    "Standby HostedPlanet must never reverse companion stride ({seen_x} < {prev_seen_x})"
                );
            }
            prev_seen_x = seen_x;
        }
        std::thread::sleep(Duration::from_millis(3));
    }
}
