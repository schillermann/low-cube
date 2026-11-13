use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, BlockMedium, MeshTarget, PlanetaryWorld, Quad, SurfaceFind, WorldFact};
use crate::attire::{HeldRelic, YouthAttire};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::KeyMotion;

struct QuadTally {
    count: usize,
}

impl QuadTally {
    const fn empty() -> Self {
        Self { count: 0 }
    }

    const fn quad_count(&self) -> usize {
        self.count
    }
}

impl MeshTarget for QuadTally {
    fn push_quad(&mut self, _quad: Quad) {
        self.count += 1;
    }
}

#[test]
fn test_single_tap_stows_while_double_tap_casts_to_ground_and_retrieves() {
    let mut world = BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 9.0, 0.25), 1.0);
    let controls = KeyMotion::idle();
    let mut avatar = PlanetaryAvatar::at(spawn);
    avatar.rotate_degrees(0.0, 58.0);

    let (mut empty_hands, mut empty_satchel) = (QuadTally::empty(), QuadTally::empty());
    avatar.emit_appearance(&mut empty_hands);
    assert!(avatar.dispatch_wardrobe_key(KeyCode::Tab));
    avatar.emit_appearance(&mut empty_satchel);
    assert!(avatar.dispatch_wardrobe_key(KeyCode::Tab));

    let mut base_flora = QuadTally::empty();
    world.emit_face(6, &mut base_flora);

    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::WoodmanAxe, HeldRelic::GoldOre));

    // Single tap E stows WoodmanAxe; waiting > 0.38s expires double-tap so next E opens satchel instead of dropping.
    assert!(avatar.press_world_key(KeyCode::KeyE, &mut world));
    for _ in 0..30 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
        let _ = avatar.step_digging(&mut world);
    }
    assert!(avatar.press_world_key(KeyCode::KeyE, &mut world));
    let mut after_slow_e = QuadTally::empty();
    world.emit_face(6, &mut after_slow_e);
    assert_eq!(after_slow_e.quad_count(), base_flora.quad_count(), "Expired second tap must open satchel, not drop");
    assert!(avatar.press_world_key(KeyCode::KeyE, &mut world));

    // Double-tap E within 0.15s casts WoodmanAxe onto the ground and removes it from satchel.
    assert!(avatar.press_world_key(KeyCode::KeyE, &mut world));
    for _ in 0..9 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
        let _ = avatar.step_digging(&mut world);
    }
    assert!(avatar.press_world_key(KeyCode::KeyE, &mut world));

    let mut axe_on_ground = QuadTally::empty();
    world.emit_face(6, &mut axe_on_ground);
    assert!(axe_on_ground.quad_count() > base_flora.quad_count(), "Dropped axe must appear in flora mesh on ground");

    // Double-tap Q immediately casts left-hand GoldOre onto the ground beside it; satchel must now be completely empty.
    assert!(avatar.press_world_key(KeyCode::KeyQ, &mut world));
    assert!(avatar.press_world_key(KeyCode::KeyQ, &mut world));
    let mut both_on_ground = QuadTally::empty();
    world.emit_face(6, &mut both_on_ground);
    assert!(both_on_ground.quad_count() > axe_on_ground.quad_count(), "Dropped ore block must also lie on ground");

    assert!(avatar.dispatch_wardrobe_key(KeyCode::Tab));
    let mut check_satchel = QuadTally::empty();
    avatar.emit_appearance(&mut check_satchel);
    assert_eq!(check_satchel.quad_count(), empty_satchel.quad_count(), "Dropped items must leave satchel empty");
    assert!(avatar.dispatch_wardrobe_key(KeyCode::Tab));

    // Clicking on the ground items retrieves both back into the avatar's hands.
    assert!(avatar.use_left_hand(&mut world) && avatar.use_right_hand(&mut world));
    let mut cleared_flora = QuadTally::empty();
    world.emit_face(6, &mut cleared_flora);
    assert_eq!(cleared_flora.quad_count(), base_flora.quad_count(), "Retrieving dropped items must clear ground mesh");
    let mut rearmed = QuadTally::empty();
    avatar.emit_appearance(&mut rearmed);
    assert!(rearmed.quad_count() > empty_hands.quad_count(), "Retrieved items must be back in hands");
}

#[test]
fn test_cast_litter_world_fact_roundtrips_and_syncs_in_multiplayer() {
    use sanctuary::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};

    for find in [
        SurfaceFind::KnightlySword,
        SurfaceFind::TimberTorch,
        SurfaceFind::LooseBlock(BlockMedium::CopperOre),
        SurfaceFind::LooseBlock(BlockMedium::FieldCamera),
    ] {
        let fact = WorldFact::LitterCast {
            eye: Vec3::new(0.5, 9.5, 0.5),
            direction: Vec3::new(0.0, -0.8, -0.6).normalize(),
            find,
        };
        let mut buf = Vec::new();
        fact.inscribe(&mut buf);
        assert_eq!(WorldFact::decode(&buf), Some((fact, buf.len())));
    }

    let base_world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::beveled(16, 0.5));
    let raw_host = SanctuaryHost::bind("127.0.0.1:0", base_world, 16, false).expect("bind");
    let ticket = raw_host.ticket().encode();
    let mut host_planet = HostedPlanet::new(raw_host);
    let mut guest_planet = RemotePlanet::dial(JoinTarget::parse(&ticket).expect("ticket"), 16, false, 0.5).expect("dial");

    let spawn = host_planet.constrain_to_surface(Vec3::new(0.25, 12.0, 0.25), 1.0);
    let mut guest_avatar = PlanetaryAvatar::at(spawn);
    guest_avatar.rotate_degrees(0.0, 58.0);
    guest_avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::KnightlySword, HeldRelic::EmptyPalm));

    for _ in 0..8 {
        guest_avatar.sync_world_presence(&mut guest_planet);
        let _ = guest_planet.step_surface(0.06);
        let _ = host_planet.step_surface(0.06);
    }

    let mut host_before = QuadTally::empty();
    host_planet.emit_face(6, &mut host_before);

    assert!(guest_avatar.press_world_key(KeyCode::KeyE, &mut guest_planet));
    assert!(guest_avatar.press_world_key(KeyCode::KeyE, &mut guest_planet));

    for _ in 0..8 {
        guest_avatar.sync_world_presence(&mut guest_planet);
        let _ = guest_planet.step_surface(0.06);
        let _ = host_planet.step_surface(0.06);
    }

    let mut host_after = QuadTally::empty();
    host_planet.emit_face(6, &mut host_after);
    assert!(host_after.quad_count() > host_before.quad_count(), "Guest dropped sword must sync onto host planet");
}
