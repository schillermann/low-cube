use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, MeshTarget, PlanetaryWorld, Quad, SurfaceFind};
use acoustics::{AcousticWave, HeartyBite};
use fauna::{BoarActivity, BoarSounder, RoeHerd, SparrowFlock, TestWildBoar as WildBoar, WoodlandFauna};
use crate::attire::{HeldRelic, YouthAttire};
use crate::avatar::{Avatar, PlanetaryAvatar};
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

    let mut world = BiomedCubeWorld::beveled(32, 1.0);
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
    let mut far_fauna = WoodlandFauna::new(
        SparrowFlock::seeded(),
        RoeHerd::seeded(),
        BoarSounder::new([distant_boar, far_boar, far_boar], 0.0, true, 0),
    );
    assert_eq!(
        hunter.hunt_fauna(1.0, &mut far_fauna, &mut world),
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
    let mut fauna = WoodlandFauna::new(
        SparrowFlock::seeded(),
        RoeHerd::seeded(),
        BoarSounder::new([target_boar, mate_boar, far_boar], 0.0, true, 0),
    );

    // First KnightlySword strike (power = 4) wounds boar to 4 HP, provokes TuskCharge, and alerts nearby sounder mate
    let (hit_first, dropped_first) = hunter.hunt_fauna(1.0, &mut fauna, &mut world);
    assert!(hit_first && !dropped_first);
    assert!(fauna.boars().boars()[0].vitality_at(4));
    assert!(fauna.boars().boars()[0].charging());
    assert!(fauna.boars().boars()[1].bristled());

    // Second KnightlySword strike slays boar (0 HP) and drops SurfaceFind::BoarMeat on the ground
    let (hit_second, dropped_second) = hunter.hunt_fauna(1.0, &mut fauna, &mut world);
    assert!(hit_second && dropped_second && !fauna.boars().boars()[0].alive());

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
