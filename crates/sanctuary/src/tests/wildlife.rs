use glam::{Quat, Vec3};
use world::{
    BiomedCubeWorld, CompanionPortal, CompanionPresence, MeshTarget, PlanetaryWorld,
    PresenceAttire, PresenceKinematics, Quad, SightCone, SurfaceFind, WoodlandLife,
    WorldEarthworks,
};
use crate::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};

struct QuadCollector {
    centers: Vec<Vec3>,
}

impl QuadCollector {
    fn empty() -> Self {
        Self { centers: Vec::new() }
    }
}

impl MeshTarget for QuadCollector {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, v1, v2, v3, _, _| {
            self.centers.push((v0 + v1 + v2 + v3) * 0.25);
        });
    }
}

fn pump_duo(host: &mut HostedPlanet, guest: &mut RemotePlanet, rounds: usize, dt: f32) {
    for _ in 0..rounds {
        let _ = guest.step_surface(dt);
        let _ = host.step_surface(dt);
    }
}

#[test]
fn test_multiplayer_boar_sounder_syncs_authoritatively_gores_visibly_and_drops_shared_meat() {
    let world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::beveled(32, 1.0));
    let host = SanctuaryHost::bind("127.0.0.1:0", world, 32).expect("bind sanctuary host");
    let addr = host.local_addr().expect("host addr");
    let mut host_planet = HostedPlanet::new(host);
    let mut guest_planet = RemotePlanet::dial(JoinTarget::Direct(addr), 32, 1.0).expect("dial guest");

    // Before any HostPulse arrives from SanctuaryHost, RemotePlanet is Unseen and never phantom-attacks
    let spawn = guest_planet.constrain_to_surface(Vec3::new(0.25, 24.0, 0.25), 1.0);
    assert!(guest_planet.step_wildlife(1.0 / 60.0, spawn, Vec3::NEG_Z, false).is_none());
    let mut unseen_mesh = QuadCollector::empty();
    guest_planet.emit_wildlife(SightCone::omnidirectional(spawn), &mut unseen_mesh);
    assert_eq!(unseen_mesh.centers.len(), 0, "Pre-welcome guest must not emit local phantom boars");

    // Complete handshake & exchange Temperate presence: host settles authoritative sounder and replicates all 3 boars
    pump_duo(&mut host_planet, &mut guest_planet, 6, 0.03);
    assert!(guest_planet.connected());
    let guest_kin = PresenceKinematics::new(spawn, Quat::IDENTITY, Quat::IDENTITY, (0.80, 0.0));
    guest_planet.sync_presence(CompanionPresence::new(0, guest_kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire()));
    pump_duo(&mut host_planet, &mut guest_planet, 6, 0.03);

    let mut host_mesh = QuadCollector::empty();
    let mut guest_mesh = QuadCollector::empty();
    host_planet.emit_wildlife(SightCone::omnidirectional(spawn), &mut host_mesh);
    guest_planet.emit_wildlife(SightCone::omnidirectional(spawn), &mut guest_mesh);
    assert!(host_mesh.centers.len() >= 360, "Host must render 3 authoritative wild boars");
    assert_eq!(
        host_mesh.centers.len(),
        guest_mesh.centers.len(),
        "Guest must render the exact same authoritative wild boar sounder as the host"
    );

    // Move guest 2.6m inward from the first visible boar (staying inside Temperate face) to provoke charge
    let boar_approx = guest_mesh.centers[0];
    let boar_ground = guest_planet.constrain_to_surface(Vec3::new(boar_approx.x, 24.0, boar_approx.z), 0.0);
    let inward = Vec3::new(-boar_ground.x, 0.0, -boar_ground.z).normalize_or_zero();
    let step_dir = if inward.length_squared() > 0.2 { inward } else { Vec3::X };
    let intruder_stand = guest_planet.constrain_to_surface(
        Vec3::new(boar_ground.x + step_dir.x * 2.6, 24.0, boar_ground.z + step_dir.z * 2.6),
        1.0,
    );
    let intruder_kin = PresenceKinematics::new(intruder_stand, Quat::IDENTITY, Quat::IDENTITY, (0.80, 0.0));
    guest_planet.sync_presence(CompanionPresence::new(0, intruder_kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire()));

    let mut received_shove = None;
    for _ in 0..35 {
        pump_duo(&mut host_planet, &mut guest_planet, 1, 0.03);
        if let Some(shove) = guest_planet.step_wildlife(0.03, intruder_stand, Vec3::NEG_X, false) {
            received_shove = Some(shove);
            break;
        }
    }
    assert!(received_shove.is_some(), "Authoritative boar charge must deliver synced tusk knockback to guest");

    // Subsequent duplicate pulses without a second charge must NOT re-trigger phantom knockback
    pump_duo(&mut host_planet, &mut guest_planet, 3, 0.03);
    assert!(
        guest_planet.step_wildlife(0.03, intruder_stand, Vec3::NEG_X, false).is_none(),
        "GoreStamp sequence guard must prevent duplicate knockback from the same hit"
    );

    // Locate calm boar #1 (quad index 200), step 1.3m inward within sword reach, sync presence, then aim at live boar
    let mut initial_hunt = QuadCollector::empty();
    guest_planet.emit_wildlife(SightCone::omnidirectional(intruder_stand), &mut initial_hunt);
    let calm_pt = initial_hunt.centers[200];
    let target_ground = guest_planet.constrain_to_surface(Vec3::new(calm_pt.x, 24.0, calm_pt.z), 0.0);
    let hunt_inward = Vec3::new(-target_ground.x, 0.0, -target_ground.z).normalize_or_zero();
    let hunt_dir = if hunt_inward.length_squared() > 0.2 { hunt_inward } else { Vec3::Z };
    let hunter_stand = guest_planet.constrain_to_surface(
        Vec3::new(target_ground.x + hunt_dir.x * 1.25, 24.0, target_ground.z + hunt_dir.z * 1.25),
        1.0,
    );
    let hunter_kin = PresenceKinematics::new(hunter_stand, Quat::IDENTITY, Quat::IDENTITY, (0.80, 0.0));
    guest_planet.sync_presence(CompanionPresence::new(0, hunter_kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire()));
    pump_duo(&mut host_planet, &mut guest_planet, 3, 0.02);

    let mut hunt_mesh = QuadCollector::empty();
    guest_planet.emit_wildlife(SightCone::omnidirectional(hunter_stand), &mut hunt_mesh);
    let live_pt = hunt_mesh.centers[200];
    let eye = Vec3::new(live_pt.x, live_pt.y + 0.9, live_pt.z + 1.2);
    let hunter_kin = PresenceKinematics::new(eye - Vec3::Y * 0.8, Quat::IDENTITY, Quat::IDENTITY, (0.80, 0.0));
    guest_planet.sync_presence(CompanionPresence::new(0, hunter_kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire()));
    pump_duo(&mut host_planet, &mut guest_planet, 2, 0.02);

    let mut fresh_mesh = QuadCollector::empty();
    guest_planet.emit_wildlife(SightCone::omnidirectional(eye), &mut fresh_mesh);
    let live_pt = fresh_mesh.centers[200];
    let aim = (live_pt - eye).normalize();

    // Far telekinesis strike (> 5.2m away from body) must be rejected by WardedPlanet / HostState guard
    let fake_far_eye = hunter_stand + Vec3::X * 25.0;
    assert_eq!(guest_planet.strike_boar(fake_far_eye, Vec3::NEG_Z, 4, 2.35), (false, false));

    // Two legitimate sword strikes (power 4 each = 8 HP) slay the boar and chronicle BoarMeat onto ground
    assert!(guest_planet.strike_boar(eye, aim, 4, 2.45).0, "First guest sword strike must hit synced boar");
    pump_duo(&mut host_planet, &mut guest_planet, 2, 0.02);

    let mut mid_mesh = QuadCollector::empty();
    guest_planet.emit_wildlife(SightCone::omnidirectional(hunter_stand), &mut mid_mesh);
    let wounded_pt = mid_mesh.centers[200];
    let aim_second = (wounded_pt - eye).normalize();
    assert!(guest_planet.strike_boar(eye, aim_second, 4, 2.45).0, "Second guest sword strike must slay boar");
    pump_duo(&mut host_planet, &mut guest_planet, 6, 0.03);

    let mut after_slay_guest = QuadCollector::empty();
    let mut after_slay_host = QuadCollector::empty();
    guest_planet.emit_wildlife(SightCone::omnidirectional(hunter_stand), &mut after_slay_guest);
    host_planet.emit_wildlife(SightCone::omnidirectional(hunter_stand), &mut after_slay_host);
    assert!(
        after_slay_guest.centers.len() < guest_mesh.centers.len()
            && after_slay_guest.centers.len() == after_slay_host.centers.len(),
        "Slain boar must transition to Fallen on both host and guest"
    );

    // Dropped BoarMeat is chronicled to guest_planet and can be foraged from the ground
    let meat_probe_eye = Vec3::new(wounded_pt.x, wounded_pt.y + 1.4, wounded_pt.z);
    let foraged = guest_planet.forage_surface(meat_probe_eye, Vec3::NEG_Y);
    assert_eq!(foraged, SurfaceFind::BoarMeat, "Slain multiplayer boar must drop chronicled BoarMeat for guests");
}
