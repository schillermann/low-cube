mod wince;

use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BiomedQuadSphere, BlockMedium, BlockMold, MeshTarget, MiningEdge, PlanetaryWorld, Quad,
    SatchelManifest, SurfaceFind, TerrainMesh as _, WorldEarthworks as _,
};
use crate::attire::{EvaAttire, HeldRelic};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarPresence as _, PlanetaryAvatar,
};
use crate::controls::KeyMotion;

struct QuadCounter {
    count: usize,
}

impl QuadCounter {
    const fn empty() -> Self {
        Self { count: 0 }
    }
}

impl MeshTarget for QuadCounter {
    fn push_quad(&mut self, _quad: Quad) {
        self.count += 1;
    }
}

#[test]
fn test_bare_gloves_reject_stone_and_ores_while_digging_earth_and_planks() {
    for hard in [
        BlockMedium::QuarriedStone,
        BlockMedium::CopperOre,
        BlockMedium::IronOre,
        BlockMedium::GoldOre,
        BlockMedium::CrustRock,
    ] {
        assert!(!hard.yields_to(MiningEdge::BarePalm), "{hard:?} must reject bare hands");
    }
    for soft in [BlockMedium::TurfEarth, BlockMedium::Sand, BlockMedium::PlanedPlanks] {
        assert!(soft.yields_to(MiningEdge::BarePalm), "{soft:?} must remain diggable with EVA gloves");
    }

    let mut world = BiomedQuadSphere::barren(16);
    let stand = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let probe = Vec3::new(0.25, 16.0, -0.75);
    let ground_y = world.constrain_to_surface(probe, 0.0).y;
    let aim_eye = Vec3::new(0.25, stand.y + 0.8, 0.25);
    let aim_dir = (Vec3::new(0.25, ground_y + 0.1, -0.75) - aim_eye).normalize();

    for medium in [BlockMedium::QuarriedStone, BlockMedium::GoldOre] {
        assert!(world.place_molded_surface_block(
            aim_eye,
            aim_dir,
            stand,
            BlockMold::cube().in_medium(medium),
        ));
        let top_before = world.constrain_to_surface(probe, 0.0).y;
        let mut miner = PlanetaryAvatar::at(stand);
        miner.rotate_degrees(0.0, 70.5);
        miner.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::SuitRadio, HeldRelic::EmptyPalm));
        for _ in 0..6 {
            assert!(!miner.use_right_hand(&mut world), "SuitRadio must not mine {medium:?}");
            assert!(!miner.use_left_hand(&mut world), "Bare palm must not mine {medium:?}");
        }
        for _ in 0..80 {
            miner.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
            assert!(!miner.step_digging(&mut world), "Sustained hold must not scar or harvest {medium:?}");
        }
        let _ = miner.release_hand(&mut world);
        assert!(
            (world.constrain_to_surface(probe, 0.0).y - top_before).abs() < 1e-4,
            "{medium:?} height must remain intact against bare hands and transceiver"
        );
        assert_eq!(world.harvest_surface(aim_eye, aim_dir), Some(medium));
    }
}

#[test]
fn test_rig_wrench_pierces_crust_ores_and_stone_and_syncs_multiplayer_manifest() {
    let mut manifest = SatchelManifest::empty();
    manifest.credit_code(HeldRelic::TitaniumBillet.code(), 1);
    manifest.credit_code(HeldRelic::CopperOre.code(), 3);
    manifest.credit_code(HeldRelic::GoldOre.code(), 1);
    assert!(manifest.apply_recipe(17), "Recipe #17 must assemble RigWrench from TitaniumBillet + CopperOre");
    assert!(manifest.apply_recipe(18), "Recipe #18 must assemble SuitRadio from CopperOre + GoldOre");
    let mut wire = Vec::new();
    manifest.inscribe_sparse(&mut wire);
    let (decoded_manifest, _) = SatchelManifest::decode_sparse(&wire).expect("decode manifest");
    assert_eq!(decoded_manifest.count_of(HeldRelic::RigWrench.code()), 1);
    assert_eq!(decoded_manifest.count_of(HeldRelic::SuitRadio.code()), 1);

    let mut world = BiomedQuadSphere::barren(16);
    let stand = world.constrain_to_surface(Vec3::new(0.25, 8.0, 0.25), 1.0);
    let probe = Vec3::new(0.25, 8.0, -0.75);
    let ground_pt = world.constrain_to_surface(probe, 0.0);
    let aim_eye = stand + stand.normalize() * 0.8;
    let aim_dir = (ground_pt + ground_pt.normalize() * 0.1 - aim_eye).normalize();

    let mut miner = PlanetaryAvatar::at(stand);
    miner.rotate_degrees(0.0, 70.5);
    miner.apply_attire(EvaAttire::eva_equipped());

    for stratum in [
        BlockMedium::QuarriedStone,
        BlockMedium::CopperOre,
        BlockMedium::IronOre,
        BlockMedium::GoldOre,
        BlockMedium::CrustRock,
    ] {
        assert!(world.place_molded_surface_block(
            aim_eye,
            aim_dir,
            stand,
            BlockMold::cube().in_medium(stratum),
        ));
        let elevated = world.constrain_to_surface(probe, 0.0).length();
        assert!(
            !miner.use_right_hand(&mut world) && miner.use_right_hand(&mut world),
            "Space-Age RigWrench must dismantle {stratum:?} in 2 strokes"
        );
        let _ = miner.release_hand(&mut world);
        assert!((elevated - world.constrain_to_surface(probe, 0.0).length() - 0.5).abs() < 1e-4);
    }

    let eva = EvaAttire::eva_equipped();
    let decoded = EvaAttire::from_multiplayer_bytes(eva.multiplayer_bytes());
    assert_eq!(decoded.held_pair(), (HeldRelic::RigWrench, HeldRelic::SuitRadio));

    let mut bare_quads = QuadCounter::empty();
    let mut armed_quads = QuadCounter::empty();
    let mut avatar = PlanetaryAvatar::at(stand);
    avatar.emit_appearance(&mut bare_quads);
    avatar.apply_attire(eva);
    avatar.emit_appearance(&mut armed_quads);
    assert!(armed_quads.count > bare_quads.count + 30, "RigWrench + SuitRadio must render 3D chassis geometry");

    assert!(avatar.press_world_key(KeyCode::KeyE, &mut world));
    assert!(avatar.press_world_key(KeyCode::KeyE, &mut world), "Double-tap E must cast RigWrench onto ground");
    let mut ground_quads_before = QuadCounter::empty();
    world.emit_surface(&mut ground_quads_before);
    assert_eq!(SurfaceFind::from_code(SurfaceFind::RigWrench.code()), SurfaceFind::RigWrench);
    assert_eq!(SurfaceFind::from_code(SurfaceFind::SuitRadio.code()), SurfaceFind::SuitRadio);
    assert!(ground_quads_before.count > 0);
}
