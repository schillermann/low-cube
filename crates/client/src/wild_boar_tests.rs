use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, MeshTarget, PlanetaryWorld, Quad, SurfaceFind};
use acoustics::{AcousticWave, HeartyBite};
use fauna::{
    BoarActivity, BoarSounder, RoeHerd, SparrowFlock, TestWildBoar as WildBoar, WoodlandFauna,
    WoodlandPlanet,
};
use crate::attire::{HeldRelic, YouthAttire};
use crate::avatar::{Avatar as _, AvatarMotor as _, AvatarPresence as _, PlanetaryAvatar};
use crate::controls::{KeyMotion, KeyboardControls};
use crate::flight::SurfaceFlight;
use crate::posture::AvatarPosture;

struct FacetBatch {
    quads: Vec<Quad>,
}

impl FacetBatch {
    fn empty() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for FacetBatch {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

#[test]
fn test_hunting_wild_boar_with_sword_drops_boar_meat_that_heals_and_invigorates_when_eaten() {
    let posture = AvatarPosture::standing().gore(0.40);
    assert!(posture.bruised() && posture.shock_active() && !posture.invigorated());
    let (beat_peak, _) = posture.step_vital_recovery(0.08);
    assert!(beat_peak.cardiac_fov() < 45.0_f32.to_radians(), "First heartbeat peak must constrict FOV into tunnel vision");
    assert!(beat_peak.vertigo_sway().abs() > 0.04, "Horizon must sway with dizziness during first ~0.8s");
    let mut hit_hud = FacetBatch::empty();
    beat_peak.emit_reticle(&mut hit_hud);
    let max_red = hit_hud.quads.iter().fold(0.0_f32, |acc, q| {
        let mut r = 0.0_f32;
        q.transfer(|_, _, _, _, _, col| {
            if col.y == 0.0 && col.z == 0.0 {
                r = col.x;
            }
        });
        acc.max(r)
    });
    assert!(
        (0.16..0.44).contains(&max_red),
        "Tusk-hit border vignette must be deep dark crimson rather than bright neon scarlet, got {max_red}"
    );
    let (settled, second_beat) = beat_peak.step_vital_recovery(0.88);
    assert!(second_beat && settled.vertigo_sway().abs() < 1e-5, "Second heartbeat fires and vertigo settles after ~0.88s");
    let healed = settled.nourish();
    assert!(!healed.bruised() && healed.invigorated() && healed.pace_boost() > 1.10);

    let bite = HeartyBite::savory();
    assert_eq!(bite.sample_rate(), 22050);
    let bite_peak = (0..16).map(|k| bite.amplitude_at(0.03 + (k as f32) * 0.002).abs()).fold(0.0_f32, f32::max);
    assert!(bite_peak > 0.15 && &bite.riff_wave()[0..4] == b"RIFF");

    let world = BiomedCubeWorld::beveled(32, 1.0);
    let hunter_pos = world.constrain_to_surface(Vec3::new(0.0, 20.0, 3.0), 1.0);
    let armed = YouthAttire::squire().with_relics(HeldRelic::KnightlySword, HeldRelic::EmptyPalm);
    let mut hunter = PlanetaryAvatar::with_attire(hunter_pos, Box::new(SurfaceFlight::terrestrial(1.0)), armed);

    // Place a wild boar 3.6m ahead: out of sword melee reach (realistic melee forbids sniping from afar)
    let distant_spot = world.constrain_to_surface(Vec3::new(hunter_pos.x, hunter_pos.y + 4.0, hunter_pos.z - 3.6), 0.0);
    hunter.rotate_degrees(0.0, 18.0);
    let distant_boar = WildBoar::new(
        BoarActivity::Rooting { pos: distant_spot, heading: Vec3::Z, timer: 5.0, phase: 0.0 },
        Vec3::Y,
        202,
        (8, 0),
    );
    let far_spot = world.constrain_to_surface(Vec3::new(-9.0, 20.0, -9.0), 0.0);
    let far_boar = WildBoar::new(
        BoarActivity::Rooting { pos: far_spot, heading: Vec3::X, timer: 5.0, phase: 0.0 },
        Vec3::Y,
        404,
        (8, 0),
    );
    let far_fauna = WoodlandFauna::new(
        SparrowFlock::seeded(),
        RoeHerd::seeded(),
        BoarSounder::active([distant_boar, far_boar, far_boar]),
    );
    let mut far_world = WoodlandPlanet::new(Box::new(BiomedCubeWorld::beveled(32, 1.0)), far_fauna);
    assert_eq!(
        hunter.hunt_fauna(1.0, &mut far_world),
        (false, false),
        "Melee strike from 3.6m horizontal distance must not hit a wild boar from afar"
    );

    // Step into realistic sword lunge distance (1.75m ahead) and aim downward at its shoulders
    let boar_spot = world.constrain_to_surface(Vec3::new(hunter_pos.x, hunter_pos.y + 4.0, hunter_pos.z - 1.75), 0.0);
    let mate_spot = world.constrain_to_surface(Vec3::new(hunter_pos.x + 3.2, hunter_pos.y + 4.0, hunter_pos.z - 2.4), 0.0);
    hunter.rotate_degrees(0.0, 16.0);
    let target_boar = WildBoar::new(
        BoarActivity::Rooting { pos: boar_spot, heading: Vec3::Z, timer: 5.0, phase: 0.0 },
        Vec3::Y,
        303,
        (8, 0),
    );
    let mate_boar = WildBoar::new(
        BoarActivity::Rooting { pos: mate_spot, heading: Vec3::Z, timer: 5.0, phase: 0.0 },
        Vec3::Y,
        305,
        (8, 0),
    );
    let fauna = WoodlandFauna::new(
        SparrowFlock::seeded(),
        RoeHerd::seeded(),
        BoarSounder::active([target_boar, mate_boar, far_boar]),
    );
    let mut world = WoodlandPlanet::new(Box::new(world), fauna);

    // First KnightlySword strike (power = 4) wounds boar to 4 HP, provokes TuskCharge, and alerts nearby sounder mate
    let (hit_first, dropped_first) = hunter.hunt_fauna(1.0, &mut world);
    assert!(hit_first && !dropped_first);
    assert!(world.fauna().boars().boars()[0].vitality_at(4));
    assert!(world.fauna().boars().boars()[0].charging());
    assert!(world.fauna().boars().boars()[1].bristled());

    // Second KnightlySword strike slays boar (0 HP) and drops SurfaceFind::BoarMeat on the ground
    let (hit_second, dropped_second) = hunter.hunt_fauna(1.0, &mut world);
    assert!(hit_second && dropped_second && !world.fauna().boars().boars()[0].alive());

    // Aim downward onto the fallen boar's ground spot to forage BoarMeat into empty left palm
    hunter.rotate_degrees(0.0, 8.0);
    assert!(hunter.use_left_hand(&mut world), "Hunter must forage dropped BoarMeat from the ground");

    // Look back up to the horizon and click with held BoarMeat to eat the provision
    hunter.rotate_degrees(0.0, -42.0);
    let mut before_eat = FacetBatch::empty();
    hunter.emit_appearance(&mut before_eat);

    let mut walk = KeyMotion::idle();
    walk.register_key(KeyCode::KeyW, true);
    assert!(hunter.use_left_hand(&mut world), "Clicking with held BoarMeat must eat the provision");

    let mut after_eat = FacetBatch::empty();
    hunter.emit_appearance(&mut after_eat);
    assert!(before_eat.quads.len() > after_eat.quads.len(), "Consuming BoarMeat must clear the held haunch from the palm");

    assert_eq!(HeldRelic::from_surface_find(SurfaceFind::BoarMeat), HeldRelic::BoarMeat);
    assert_eq!(HeldRelic::BoarMeat.surface_find(), SurfaceFind::BoarMeat);
    hunter.step_motion(1.0 / 60.0, &walk, &world);
}

#[test]
fn test_joining_multiplayer_discards_solitary_boars_without_phantom_gore() {
    use std::net::UdpSocket;
    use sanctuary::{RemoteLink, SanctuaryHost};
    use crate::black_box::{BlackBoxAvatar, LiveTape};
    use crate::session::HeadlessSession;

    // Solitary world has an aggressive charging boar 3.6m away (reaches player in ~0.35s if not discarded on join)
    let solitary_terrain = BiomedCubeWorld::barren_beveled(32, 1.0);
    let spawn = solitary_terrain.constrain_to_surface(Vec3::new(0.25, 24.0, 0.25), 1.0);
    let boar_pos = solitary_terrain.constrain_to_surface(Vec3::new(spawn.x + 3.6, 24.0, spawn.z), 0.0);
    let solitary_charger = WildBoar::new(
        BoarActivity::TuskCharge { pos: boar_pos, target: spawn, phase: 0.0, timer: 3.0 },
        Vec3::Y,
        777,
        (8, 0),
    );
    let solitary_fauna = WoodlandFauna::new(
        SparrowFlock::seeded(),
        RoeHerd::seeded(),
        BoarSounder::active([solitary_charger, solitary_charger, solitary_charger]),
    );
    let solitary_world: Box<dyn PlanetaryWorld> =
        Box::new(WoodlandPlanet::new(Box::new(solitary_terrain), solitary_fauna));

    let avatar = Box::new(BlackBoxAvatar::wrapping(
        Box::new(PlanetaryAvatar::at(spawn)),
        Box::new(LiveTape::fresh()),
    ));
    let mut session = HeadlessSession::with_world_and_modal(
        solitary_world,
        avatar,
        HeadlessSession::closed_modal(),
    );

    // Join a remote SanctuaryHost via bound P2P dial (simulating Esc -> MULTIPLAYER -> JOIN)
    let remote_terrain: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::barren_beveled(32, 1.0));
    let mut host = SanctuaryHost::bind("127.0.0.1:0", remote_terrain, 32).expect("bind host");
    let host_code = host.code();
    let guest_sock = UdpSocket::bind("127.0.0.1:0").expect("bind guest");
    let guest_addr = guest_sock.local_addr().expect("guest addr");
    let link = RemoteLink::bind_on_socket(guest_sock, guest_addr, sanctuary::JoinTarget::Wormhole(host_code));
    let _ = session.begin_bound_dial(link);
    host.punch_nat_peer(guest_addr);

    for _ in 0..12 {
        let _ = host.tick(0.03);
        session.advance_ticks(4);
    }
    assert!(!session.dialing_active(), "Guest session must complete join and activate RemotePlanet");

    // Step 60 ticks on the joined multiplayer server: the old solitary charging boar must NOT exist or gore!
    for _ in 0..15 {
        let _ = host.tick(0.03);
        session.advance_ticks(4);
    }
    assert_eq!(
        session.ui_quad_count(),
        10,
        "HUD must show only clean 10-quad crosshair with zero crimson tusk-hit vignette after joining server"
    );
}
