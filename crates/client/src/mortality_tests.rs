use glam::Vec3;
use winit::keyboard::KeyCode;
use sanctuary::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};
use world::{BiomedCubeWorld, CompanionPortal as _, PlanetaryWorld, WorldEarthworks};
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

    // Guest holds ExcavatedEarth + IronPickaxe above lava, scars the -Z shaft wall, and bare-hand starts right dig
    let ledge = guest_planet.constrain_to_surface(Vec3::new(2.25, 15.0, 0.25), 0.0);
    let mut lava_avatar = PlanetaryAvatar::at(Vec3::new(0.50, core_y + 1.6, 0.50));
    lava_avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::IronPickaxe));
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

#[test]
fn test_heliosphere_origin_stone_anchors_spawn_blocks_destruction_and_prevents_space_respawn() {
    use world::{BlockMedium, BlockMold, BodyId, CelestialSystem, CosmicFrame, SolarProportions};
    use crate::avatar::BodySpan;
    use crate::black_box::PresenceProbe;

    let props = SolarProportions::true_scale(128);
    let mut world = CelestialSystem::barren_heliosphere(props, 1.0);
    let half = props.earth_half();

    let mut origins = Vec::new();
    world.collect_medium_sites(BlockMedium::OriginStone, &mut origins);
    assert_eq!(origins.len(), 1, "Earth must seed a dedicated OriginStone arrival plinth");
    let (origin_top, origin_up) = origins[0];
    assert!(
        (origin_top.x + 0.25).abs() < 1e-3
            && (origin_top.z + 0.25).abs() < 1e-3
            && (origin_top.y - half).abs() < 1e-3,
        "OriginStone must sit flush with Earth's temperate surface, got {origin_top:?} vs half={half}"
    );

    // OriginStone must be indestructible and forbid stacking blocks on top of it
    let eye_above = origin_top + origin_up * 1.4;
    assert!(world.loosen_surface(eye_above, -origin_up, 1).is_none(), "OriginStone must reject loosening");
    assert!(world.harvest_surface(eye_above, -origin_up).is_none(), "OriginStone must reject harvesting");
    assert!(
        !world.place_molded_surface_block(
            eye_above,
            -origin_up,
            origin_top + Vec3::X * 1.5,
            BlockMold::cube().in_medium(BlockMedium::QuarriedStone),
        ),
        "OriginStone column must forbid stacking blocks that would entomb the spawn plinth"
    );

    // Initial landing places the player directly on OriginStone
    let initial = BodySpan::landed_on(world.as_ref()).current();
    assert!((initial - (origin_top + origin_up)).length() < 1e-3, "Initial spawn must stand on OriginStone");

    // Excavate nearby column (0.25, 0.25) down to lethal core floor and die on Earth
    for _ in 0..48 {
        let fy = world.constrain_to_surface(Vec3::new(0.25, half + 6.0, 0.25), 0.0).y;
        if !world.excavate_surface(Vec3::new(0.25, fy + 1.0, 0.25), Vec3::NEG_Y) {
            break;
        }
    }
    let core_y = world.constrain_to_surface(Vec3::new(0.25, half + 6.0, 0.25), 0.0).y;
    assert!(world.lethal_at(Vec3::new(0.25, core_y + 1.0, 0.25)));

    let mut explorer = PlanetaryAvatar::at(Vec3::new(0.25, core_y + 1.0, 0.25));
    for _ in 0..180 {
        explorer.step_motion(1.0 / 60.0, &KeyMotion::idle(), world.as_ref());
    }
    let _ = explorer.reconcile_cosmic_frame(world.as_mut());
    let mut probe = PresenceProbe::at_origin();
    explorer.report_presence(&mut probe);
    let earth_respawn_pos = probe.position();
    assert!(
        (earth_respawn_pos - (origin_top + origin_up)).length() < 0.05,
        "Dying on Earth in HeliosphereWorld must respawn on OriginStone (not in outer space), got pos={earth_respawn_pos:?} vs expected={:?}",
        origin_top + origin_up
    );

    // Dying offworld (e.g. in SolarStar while anchored on Luna) must re-anchor to CosmicFrame::Earth on OriginStone
    world.anchor_to_frame(CosmicFrame::Body(BodyId::LUNA), Vec3::new(0.25, 8.0, 0.25));
    let sun_center = world.body_center(BodyId::SOL).expect("Sun center");
    let mut offworld_doomed = PlanetaryAvatar::at(sun_center);
    assert!(world.lethal_at(sun_center), "Solar interior must be lethal");
    for _ in 0..180 {
        offworld_doomed.step_motion(1.0 / 60.0, &KeyMotion::idle(), world.as_ref());
    }
    assert!(offworld_doomed.reconcile_cosmic_frame(world.as_mut()), "Offworld respawn must re-anchor CosmicFrame to Earth");
    assert!(world.cosmic_frame().earth_centered(), "World frame must return to Earth after offworld death");
    offworld_doomed.report_presence(&mut probe);
    assert!(
        (probe.position() - (origin_top + origin_up)).length() < 0.05,
        "Offworld death must respawn cleanly on Earth's OriginStone, got {:?}",
        probe.position()
    );
}

#[test]
fn test_avatar_stands_on_flush_origin_stone_sees_it_underfoot_and_lands_on_remote_spawn_stone() {
    use sanctuary::RemoteLink;
    use world::{BlockMedium, BlockMold, TerrainMesh as _};
    use crate::avatar::{AvatarOptics as _, BodySpan};
    use crate::black_box::PresenceProbe;
    use crate::controls::{InputTrigger, KeyboardControls as _};
    use crate::server_satchel::ServerSatchel;

    struct RadiantCrest(f32);
    impl world::MeshTarget for RadiantCrest {
        fn push_quad(&mut self, quad: world::Quad) {
            if quad.radiate(|glow| glow) > 0.2 {
                let _ = quad.transfer(|a, b, c, d, _, _| {
                    self.0 = self.0.max(a.y).max(b.y).max(c.y).max(d.y);
                    quad
                });
            }
        }
    }

    let base = BiomedCubeWorld::barren_beveled(16, 1.0);
    let mut origins = Vec::new();
    base.collect_medium_sites(BlockMedium::OriginStone, &mut origins);
    let (origin_top, origin_up) = origins[0];
    assert!(
        (origin_top - Vec3::new(-0.25, 8.0, -0.25)).length() < 1e-3,
        "OriginStone top must sit flush at y=8.0, got {origin_top:?}"
    );
    let mut crest = RadiantCrest(f32::NEG_INFINITY);
    for slot in 0..base.face_patch_count(0) {
        base.emit_face_patch(0, slot, &mut crest);
    }
    assert!(
        (crest.0 - 8.0).abs() < 0.005,
        "OriginStone mesh must sit flush with surrounding y=8.0 blocks without protruding, got crest={}",
        crest.0
    );

    let spawn = BodySpan::landed_on(&base).current();
    assert!((spawn - (origin_top + origin_up)).length() < 1e-3, "Avatar must land directly on OriginStone");
    let mut burdened = PlanetaryAvatar::at(spawn).with_starting_satchel(&ServerSatchel::assembled());
    burdened.rotate_view(0.0, 10_000.0);
    let clip = burdened.view_projection_matrix(16.0 / 9.0).project_point3(origin_top);
    assert!(
        clip.x.abs() < 0.05 && clip.y.abs() < 0.25,
        "Looking straight down in first person must center OriginStone underfoot, got NDC {clip:?}"
    );

    let mut keys = KeyMotion::idle();
    for _ in 0..30 {
        burdened.step_motion(1.0 / 60.0, &keys, &base);
    }
    let mut probe = PresenceProbe::at_origin();
    burdened.report_presence(&mut probe);
    assert!(
        (probe.position() - (origin_top + origin_up)).length() < 0.02,
        "Idle burdened avatar must remain standing on OriginStone, got {:?}",
        probe.position()
    );

    // Step off forward (-Z) onto flush turf, then walk backward (+Z) back onto OriginStone
    keys.register_trigger(InputTrigger::Key(KeyCode::KeyW), true);
    for _ in 0..18 {
        burdened.step_motion(1.0 / 60.0, &keys, &base);
    }
    keys.register_trigger(InputTrigger::Key(KeyCode::KeyW), false);
    keys.register_trigger(InputTrigger::Key(KeyCode::KeyS), true);
    for _ in 0..18 {
        burdened.step_motion(1.0 / 60.0, &keys, &base);
    }
    burdened.report_presence(&mut probe);
    assert!(
        (probe.position().y - (origin_top.y + 1.0)).abs() < 0.03
            && (probe.position().x - origin_top.x).abs() < 0.24
            && (probe.position().z - origin_top.z).abs() < 0.24,
        "Avatar must walk seamlessly across flush OriginStone boundary, got {:?}",
        probe.position()
    );

    // Remote guest joining an unstreamed admin SpawnStone must stand on top of the 0.5m SpawnStone from tick 0
    let host = SanctuaryHost::bind("127.0.0.1:0", Box::new(base), 16).expect("bind host");
    let addr = host.local_addr().expect("addr");
    let mut host_planet = HostedPlanet::new(host);
    let spawn_mold = BlockMold::cube().in_medium(BlockMedium::SpawnStone);
    assert!(host_planet.place_molded_surface_block(
        Vec3::new(3.25, 11.0, 0.25),
        Vec3::NEG_Y,
        Vec3::new(1.25, 9.0, 0.25),
        spawn_mold,
    ));
    let mut link = RemoteLink::bind(JoinTarget::Direct(addr)).expect("guest link");
    for _ in 0..20 {
        let _ = host_planet.step_surface(0.02);
        let _ = link.step_handshake(0.02);
        if link.connected() {
            break;
        }
    }
    assert!(link.connected());
    let mut guest_planet = RemotePlanet::from_link(link, 16, 1.0);
    let guest_spawn = BodySpan::landed_on(&guest_planet).current();
    assert!(
        (guest_spawn - Vec3::new(3.25, 9.5, 0.25)).length() < 1e-3,
        "Joining guest must land on top of admin SpawnStone even before chronicle stream arrives, got {guest_spawn:?}"
    );
    let mut guest_avatar = PlanetaryAvatar::at(guest_spawn);
    for _ in 0..10 {
        guest_avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &guest_planet);
    }
    for _ in 0..8 {
        let _ = guest_planet.step_surface(0.02);
        let _ = host_planet.step_surface(0.02);
        guest_avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &guest_planet);
    }
    guest_avatar.report_presence(&mut probe);
    assert!(
        (probe.position() - Vec3::new(3.25, 9.5, 0.25)).length() < 0.03,
        "Guest must remain standing on admin SpawnStone before and after chronicle sync, got {:?}",
        probe.position()
    );
}
