use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, MeshTarget, PlanetaryWorld, Quad, SatchelManifest,
    SurfaceFind, TerrainMesh as _, WorldEarthworks as _,
};
use crate::attire::{
    BeltSatchel, BodyPose, CrewPhysique, EvaAttire, HeldRelic, MoldedWardrobe, SightMode,
    StrideRhythm, ToolCondition, Wardrobe as _,
};
use crate::avatar::{Avatar as _, PlanetaryAvatar};

struct MeshDigest {
    colors: Vec<Vec3>,
}

impl MeshDigest {
    const fn empty() -> Self {
        Self { colors: Vec::new() }
    }
}

impl MeshTarget for MeshDigest {
    fn push_quad(&mut self, quad: Quad) {
        self.colors.push(quad.transfer(|_, _, _, _, _, col| col));
    }
}

#[test]
fn test_space_age_rig_wrench_and_suit_radio_sculpt_distinct_conditions_and_cast_cleanly() {
    let cadet = EvaAttire::astronaut();
    let mut world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let stand = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let probe = Vec3::new(0.25, 16.0, -0.75);
    let ground_y = world.constrain_to_surface(probe, 0.0).y;
    let aim_eye = Vec3::new(0.25, stand.y + 0.8, 0.25);
    let aim_dir = (Vec3::new(0.25, ground_y + 0.1, -0.75) - aim_eye).normalize();

    assert!(world.place_molded_surface_block(aim_eye, aim_dir, stand, BlockMold::cube().in_medium(BlockMedium::CrustRock)));
    let mut engineer = PlanetaryAvatar::at(stand);
    engineer.rotate_degrees(0.0, 70.5);
    engineer.apply_attire(cadet.with_relics(HeldRelic::RigWrench, HeldRelic::SuitRadio));
    assert!(
        !engineer.use_right_hand(&mut world) && engineer.use_right_hand(&mut world),
        "RigWrench must dismantle CrustRock in 2 strokes without shattering"
    );

    let satchel = BeltSatchel::empty().reconcile_attire(EvaAttire::eva_equipped());
    assert_eq!(satchel.hand_condition(1.0, EvaAttire::eva_equipped()), ToolCondition::Pristine);
    assert_eq!(ToolCondition::from_wear(5, 24), ToolCondition::Pristine);
    assert_eq!(ToolCondition::from_wear(8, 24), ToolCondition::Scuffed);
    assert_eq!(ToolCondition::from_wear(15, 24), ToolCondition::Chipped);
    assert_eq!(ToolCondition::from_wear(21, 24), ToolCondition::Splintered);
    assert_eq!(ToolCondition::Scuffed.legend_prefix(), "SCUFFED ");

    // Distinct metal weathering across all 4 surface conditions over PresenceAttire replication:
    let pose = BodyPose::new(
        Vec3::new(0.0, 9.0, 0.0),
        Quat::IDENTITY,
        Quat::IDENTITY,
        0.8,
        StrideRhythm::idle(),
        CrewPhysique::OrbitalCadet,
    );
    let mut condition_palettes = Vec::new();
    for stage in [
        ToolCondition::Pristine,
        ToolCondition::Scuffed,
        ToolCondition::Chipped,
        ToolCondition::Splintered,
    ] {
        let molds = world::PalmMolds::new(stage.into_mold(), BlockMold::cube());
        let presence = world::PresenceAttire::new(
            cadet.with_relics(HeldRelic::RigWrench, HeldRelic::SuitRadio).multiplayer_bytes(),
            cadet.region_bytes(),
            molds,
        );
        let mut digest = MeshDigest::empty();
        MoldedWardrobe::from_presence(presence).emit(&pose, SightMode::FirstPersonSight, &mut digest);
        assert!(!digest.colors.is_empty());
        condition_palettes.push(digest.colors);
    }
    for pair in condition_palettes.windows(2) {
        assert_ne!(pair[0], pair[1], "Each surface-finish stage must tint RigWrench alloy distinctly");
    }

    let manifest = SatchelManifest::starter_kit();
    assert_eq!(manifest.held_codes(), (HeldRelic::RigWrench.code(), HeldRelic::SuitRadio.code()));
    assert_eq!(manifest.count_of(HeldRelic::RigWrench.code()), 1);
    assert_eq!(manifest.count_of(HeldRelic::SuitRadio.code()), 1);
    assert_eq!(manifest.count_of(HeldRelic::CantileverCrane.code()), 1);

    let spawn = world.constrain_to_surface(Vec3::new(2.25, 16.0, 2.25), 1.0);
    let mut dropper = PlanetaryAvatar::at(spawn);
    dropper.rotate_degrees(0.0, 68.0);
    dropper.apply_attire(EvaAttire::eva_equipped());
    assert!(dropper.press_world_key(KeyCode::KeyE, &mut world));
    assert!(dropper.press_world_key(KeyCode::KeyE, &mut world));
    let mut surface_mesh = MeshDigest::empty();
    world.emit_surface(&mut surface_mesh);
    assert!(!surface_mesh.colors.is_empty());
    for find in [SurfaceFind::RigWrench, SurfaceFind::SuitRadio, SurfaceFind::TitaniumBillet] {
        assert_eq!(SurfaceFind::from_code(find.code()), find);
    }
}
