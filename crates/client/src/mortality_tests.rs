use glam::Vec3;
use winit::keyboard::KeyCode;
use sanctuary::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};
use world::{BiomedCubeWorld, PlanetaryWorld, WorldEarthworks};
use crate::attire::{HeldRelic, YouthAttire};
use crate::avatar::{Avatar as _, AvatarMotor as _, AvatarPresence as _, PlanetaryAvatar};
use crate::controls::KeyMotion;

#[test]
fn test_multiplayer_lava_death_while_digging_aborts_cracks_attire_and_respawn_reach() {
    let base = BiomedCubeWorld::barren_beveled(16, 1.0);
    let host = SanctuaryHost::bind("127.0.0.1:0", Box::new(base), 16).expect("bind host");
    let addr = host.local_addr().expect("addr");
    let mut host_planet = HostedPlanet::new(host);
    for &(x, z) in &[(0.25_f32, 0.25_f32), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
        for _ in 0..24 {
            let fy = host_planet.constrain_to_surface(Vec3::new(x, 15.0, z), 0.0).y;
            if !host_planet.excavate_surface(Vec3::new(x, fy + 1.0, z), Vec3::NEG_Y) {
                break;
            }
        }
    }
    let core_y = host_planet.constrain_to_surface(Vec3::new(0.50, 15.0, 0.50), 0.0).y;
    let mut guest_planet = RemotePlanet::dial(JoinTarget::Direct(addr), 16, 1.0).expect("dial guest");
    for _ in 0..16 {
        let _ = guest_planet.step_surface(0.03);
        let _ = host_planet.step_surface(0.03);
    }
    assert!((guest_planet.constrain_to_surface(Vec3::new(0.50, 15.0, 0.50), 0.0).y - core_y).abs() < 1e-3);

    // Guest holds ExcavatedEarth + IronSpade above lava, scars the -Z shaft wall, and bare-hand starts right dig
    let ledge = guest_planet.constrain_to_surface(Vec3::new(2.25, 15.0, 0.25), 0.0);
    let mut lava_avatar = PlanetaryAvatar::at(Vec3::new(0.50, core_y + 1.6, 0.50));
    lava_avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::IronSpade));
    let _ = lava_avatar.use_right_hand(&mut guest_planet);
    lava_avatar.sync_world_presence(&mut guest_planet);
    for _ in 0..3 {
        let _ = guest_planet.step_surface(0.03);
        let _ = host_planet.step_surface(0.03);
    }
    let mut digging_peers = Vec::new();
    host_planet.active_companions(&mut digging_peers);
    let digging_garments = digging_peers[0].transfer(|_, _, _, att| att.transfer(|g, _, _| g));
    let _ = lava_avatar.use_left_hand(&mut guest_planet);

    // Gravity drops the digging avatar onto the lethal core floor WITHOUT releasing mouse button
    for _ in 0..25 {
        lava_avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &guest_planet);
    }
    assert!(!lava_avatar.dispatch_wardrobe_key(KeyCode::KeyE), "Perishing avatar must not open satchel");
    assert!(lava_avatar.step_digging(&mut guest_planet), "First tick in lava must clear active dig cracks");
    assert!(!lava_avatar.step_digging(&mut guest_planet), "Second tick in lava must be completely idle");
    lava_avatar.sync_world_presence(&mut guest_planet);
    for _ in 0..3 {
        let _ = guest_planet.step_surface(0.03);
        let _ = host_planet.step_surface(0.03);
    }
    let mut perished_peers = Vec::new();
    host_planet.active_companions(&mut perished_peers);
    let perished_garments = perished_peers[0].transfer(|_, _, _, att| att.transfer(|g, _, _| g));
    assert_ne!(digging_garments, perished_garments, "Dying in lava must immediately restore non-digging hand attire for peers");

    // Advance past 2.6s blackout to respawn on surface; immediate post-respawn dig flushes presence and succeeds on host
    for _ in 0..170 {
        lava_avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &guest_planet);
    }
    assert!(!lava_avatar.step_digging(&mut guest_planet), "Respawned avatar must never ghost-dig");
    lava_avatar.sync_world_presence(&mut guest_planet);
    let before_top = host_planet.constrain_to_surface(Vec3::new(ledge.x, 15.0, ledge.z), 0.0).y;
    assert!(guest_planet.harvest_surface(Vec3::new(ledge.x, before_top + 1.2, ledge.z), Vec3::NEG_Y).is_some());
    for _ in 0..4 {
        let _ = guest_planet.step_surface(0.03);
        let _ = host_planet.step_surface(0.03);
    }
    let after_top = host_planet.constrain_to_surface(Vec3::new(ledge.x, 15.0, ledge.z), 0.0).y;
    assert!(after_top < before_top - 0.4, "Immediate post-respawn dig must flush surface presence and enact on SanctuaryHost");
}
