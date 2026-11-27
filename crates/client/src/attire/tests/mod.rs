mod burden;
mod camera;
mod encumbrance;

use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{
    ClimateZone, CubePlanet, FacetedReliefFace, MeshTarget, NullFace, PlanetaryWorld,
    Quad, WorldEarthworks as _,
};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarOptics as _, AvatarPresence as _, PlanetaryAvatar,
};
use crate::controls::{InputTrigger, KeyboardControls, LocomotionTarget};
use super::{
    AttireProfile, BodyPose, CrewPhysique, EvaAttire, FileAttire, HeldRelic, SightMode,
    StrideRhythm, Wardrobe,
};

pub(super) struct QuadLedger {
    pub(super) quads: Vec<Quad>,
}

impl QuadLedger {
    pub(super) fn empty() -> Self {
        Self { quads: Vec::new() }
    }

    pub(super) fn populated(&self) -> bool {
        !self.quads.is_empty()
    }

    pub(super) fn exceeds(&self, other: &Self) -> bool {
        self.quads.len() > other.quads.len()
    }

    pub(super) fn visible_in_frustum(&self, vp: glam::Mat4) -> usize {
        self.quads
            .iter()
            .filter(|q| {
                q.transfer(|v0, _, _, _, _, _| {
                    let clip = vp * v0.extend(1.0);
                    clip.w > 0.03
                        && (clip.x / clip.w).abs() < 1.0
                        && (clip.y / clip.w).abs() < 1.0
                        && (clip.z / clip.w).abs() <= 1.0
                })
            })
            .count()
    }
}

impl MeshTarget for QuadLedger {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

fn world_with_cube_at(u: i32, v: i32) -> CubePlanet {
    let face = Box::new(
        FacetedReliefFace::in_zone(ClimateZone::Temperate, 8.0, 16)
            .with_molded_block(u, v, 0.0, world::RawAtom::turf_cube()),
    );
    CubePlanet::new(vec![face], Box::new(NullFace))
}

struct WalkEastControls;

impl KeyboardControls for WalkEastControls {
    fn register_key(&mut self, _code: KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, _forward: Vec3, right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(right);
    }
}

#[test]
fn test_body_envelope_prevents_camera_from_clipping_into_block_wall() {
    let world = world_with_cube_at(1, 0);
    let start = world.constrain_to_surface(Vec3::new(0.0, 5.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(start);
    let controls = WalkEastControls;
    for _ in 0..60 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
    let cam_pos = view_proj.inverse().project_point3(Vec3::ZERO);
    let wall_clearance = 0.5 - cam_pos.x;
    assert!(wall_clearance >= 0.15, "Avatar camera must maintain safe body clearance, got {}", wall_clearance);
}

#[test]
fn test_first_person_omits_head_while_multiplayer_and_mirror_include_eva_helm_and_lamp() {
    let pose = BodyPose::new(
        Vec3::new(0.0, 9.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, 0.8,
        StrideRhythm::new(0.0, 1.0), CrewPhysique::OrbitalCadet,
    );
    let attire = EvaAttire::astronaut();
    assert!(attire.lamp_illuminated(), "EVA suit helmet lamp must be illuminated from second 1");
    let mut fp_ledger = QuadLedger::empty();
    attire.emit(&pose, SightMode::FirstPersonSight, &mut fp_ledger);
    let mut mp_ledger = QuadLedger::empty();
    attire.emit(&pose, SightMode::LookingGlassSight, &mut mp_ledger);
    assert!(fp_ledger.populated(), "First-person view must emit suit shell, legs, mag-boots, and gauntlets");
    assert!(mp_ledger.exceeds(&fp_ledger), "Multiplayer and Looking-Glass view must emit EVA helmet, visor, radio mast, and lamp");

    // Verify glowing helmet lamp quad (`glow > 1.5`) turns dark when toggled off and round-trips over 9 wire bytes:
    let max_glow = |l: &QuadLedger| l.quads.iter().map(|q| q.radiate(|g| g)).fold(0.0_f32, f32::max);
    assert!(max_glow(&mp_ledger) > 1.5, "Illuminated HelmetLampHead decorator must emit radiant LED lens quad");

    let dark_attire = attire.with_toggled_lamp();
    assert!(!dark_attire.lamp_illuminated());
    let mut dark_ledger = QuadLedger::empty();
    dark_attire.emit(&pose, SightMode::LookingGlassSight, &mut dark_ledger);
    assert!(max_glow(&dark_ledger) < 0.5, "Extinguished HelmetLampHead decorator must extinguish LED glow");
    assert_eq!(EvaAttire::from_multiplayer_bytes(dark_attire.multiplayer_bytes()), dark_attire);
}

#[test]
fn test_hands_raised_only_when_holding_item_and_body_visible_when_looking_down() {
    let start = Vec3::new(0.0, 9.0, 0.0);
    let mut avatar = PlanetaryAvatar::at(start);
    let mut empty_ledger = QuadLedger::empty();
    avatar.emit_appearance(&mut empty_ledger);
    let vp_level = avatar.view_projection_matrix(16.0 / 9.0);
    let visible_empty_level = empty_ledger.visible_in_frustum(vp_level);
    assert_eq!(visible_empty_level, 0, "Empty hands must hang at the sides and not obstruct level forward view");

    avatar.apply_attire(EvaAttire::eva_equipped());
    let mut held_ledger = QuadLedger::empty();
    avatar.emit_appearance(&mut held_ledger);
    let visible_held_level = held_ledger.visible_in_frustum(vp_level);
    assert!(visible_held_level > 0, "Holding RigWrench + SuitRadio must raise EVA gauntlets into first-person view");
    assert!(avatar.dispatch_trigger(InputTrigger::Key(KeyCode::KeyL)), "KeyL must toggle integrated EVA helmet lamp");

    avatar.rotate_view(0.0, 350.0);
    let vp_down = avatar.view_projection_matrix(16.0 / 9.0);
    let mut down_ledger = QuadLedger::empty();
    avatar.emit_appearance(&mut down_ledger);
    let visible_down = down_ledger.visible_in_frustum(vp_down);
    assert!(visible_down > visible_held_level, "Looking down must bring pressure suit, PLSS belt, legs, and mag-boots into frustum");
}

#[test]
fn test_wardrobe_customization_keys_and_file_persistence() {
    let temp_path = std::env::temp_dir().join("test_eva_crew_profile.toml");
    let store = FileAttire::new(temp_path.to_str().unwrap().to_string());
    let cadet = EvaAttire::astronaut();
    let initial_bytes = cadet.multiplayer_bytes();
    let customized = cadet
        .with_next_shell().with_next_gauntlet().with_next_harness().with_next_boots()
        .with_next_helm().with_next_relic().with_next_pigment().with_next_complexion();
    store.save(&customized);
    let loaded = store.load_or_astronaut();
    std::fs::remove_file(&temp_path).expect("Failed to clean up temporary profile");
    assert_eq!(customized, loaded);
    assert_ne!(initial_bytes, loaded.multiplayer_bytes());
}

#[test]
fn test_rig_wrench_fells_timber_and_salvages_placed_planks() {
    let mut world = world::BiomedCubeWorld::beveled(32, 1.0);
    let (satchel, attire) = super::BeltSatchel::empty()
        .collect_felled_timber(1.0, EvaAttire::astronaut().with_relics(HeldRelic::RigWrench, HeldRelic::EmptyPalm));
    assert_eq!(attire.held_pair(), (HeldRelic::RigWrench, HeldRelic::TimberBlock));
    assert_eq!(satchel.resource_tally(HeldRelic::TimberBlock, attire), 3, "Felling a tree must yield 3 WOOD BLOCKs into hand + satchel");
    let (satchel_planks, attire_planks) = satchel.consume_and_equip(
        [(HeldRelic::TimberBlock, 1), (HeldRelic::EmptyPalm, 0)],
        HeldRelic::PlankBlock,
        4,
        -1.0,
        attire,
    );
    assert_eq!(attire_planks.held_pair(), (HeldRelic::RigWrench, HeldRelic::PlankBlock));
    assert_eq!(satchel_planks.resource_tally(HeldRelic::TimberBlock, attire_planks), 2);
    assert_eq!(satchel_planks.resource_tally(HeldRelic::PlankBlock, attire_planks), 4);
    let (aim_eye, aim_dir) = (Vec3::new(0.25, 16.92, 0.25), Vec3::new(0.0, -0.75, -0.75).normalize());
    assert!(world.place_molded_surface_block(aim_eye, aim_dir, Vec3::new(0.25, 16.0, 0.25), world::BlockMold::cube().in_medium(world::BlockMedium::PlanedPlanks)));
    assert_eq!(world.harvest_surface(aim_eye, aim_dir).map(HeldRelic::from_medium), Some(HeldRelic::PlankBlock));
}

#[test]
fn test_rig_tool_and_dirt_block_replaces_held_block_when_excavating_different_medium() {
    let dual_tools = EvaAttire::eva_equipped();
    let (satchel, next_attire) = super::BeltSatchel::empty().collect_excavated_block(1.0, HeldRelic::ExcavatedEarth, dual_tools);
    assert_eq!(next_attire.held_pair(), (HeldRelic::RigWrench, HeldRelic::SuitRadio));
    assert_eq!(satchel.resource_tally(HeldRelic::ExcavatedEarth, next_attire), 1);

    let wrench_and_dirt = EvaAttire::astronaut().with_relics(HeldRelic::RigWrench, HeldRelic::ExcavatedEarth);
    let satchel = super::BeltSatchel::empty().reconcile_attire(wrench_and_dirt);
    let crust_harvested = HeldRelic::from_medium(world::BlockMedium::CrustRock);
    assert_eq!(crust_harvested, HeldRelic::CrustBlock);
    let (satchel_after_crust, attire_after_crust) = satchel.collect_excavated_block(-1.0, crust_harvested, wrench_and_dirt);
    assert_eq!(attire_after_crust.held_pair(), (HeldRelic::RigWrench, HeldRelic::CrustBlock));
    assert_eq!(satchel_after_crust.resource_tally(HeldRelic::ExcavatedEarth, attire_after_crust), 1);
    assert_eq!(satchel_after_crust.resource_tally(HeldRelic::CrustBlock, attire_after_crust), 1);
    let (satchel_same, attire_same) = satchel_after_crust.collect_excavated_block(-1.0, HeldRelic::CrustBlock, attire_after_crust);
    assert_eq!(attire_same.held_pair(), (HeldRelic::RigWrench, HeldRelic::CrustBlock));
    assert_eq!(satchel_same.resource_tally(HeldRelic::CrustBlock, attire_same), 2);

    let wheel_in_right = EvaAttire::astronaut().with_relics(HeldRelic::CarWheel, HeldRelic::EmptyPalm);
    let wheel_satchel = super::BeltSatchel::empty().reconcile_attire(wheel_in_right);
    let (satchel_after_wheel, attire_after_wheel) = wheel_satchel.collect_excavated_block(1.0, HeldRelic::CarWheel, wheel_in_right);
    assert_eq!(attire_after_wheel.held_pair(), (HeldRelic::CarWheel, HeldRelic::EmptyPalm));
    assert_eq!(satchel_after_wheel.resource_tally(HeldRelic::CarWheel, attire_after_wheel), 2);
    let pose = BodyPose::new(
        Vec3::new(0.0, 9.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, 0.8,
        StrideRhythm::new(0.0, 1.0), CrewPhysique::OrbitalCadet,
    );
    let (mut tray_one, mut tray_eight) = (QuadLedger::empty(), QuadLedger::empty());
    satchel_after_wheel.with_open(true).emit_for_attire(&pose, attire_after_wheel, &mut tray_one);
    satchel_after_wheel.with_stowed_relic(HeldRelic::CarWheel, 7).with_open(true).emit_for_attire(&pose, attire_after_wheel, &mut tray_eight);
    assert_ne!(tray_one.quads.len(), tray_eight.quads.len());
}

#[test]
fn test_sculpted_eva_helm_command_crest_and_three_quarter_intercom_portrait() {
    let mut commander = EvaAttire::astronaut();
    for _ in 0..7 {
        commander = commander.with_next_helm();
    }
    let roundtrip = EvaAttire::from_multiplayer_bytes(commander.multiplayer_bytes());
    assert_eq!(roundtrip.multiplayer_bytes(), commander.multiplayer_bytes());

    let mut clean_bust = QuadLedger::empty();
    let mut bearded_bust = QuadLedger::empty();
    let vp = super::IntercomBust::new(commander, (0.0, 0.0)).emit(1.62, &mut bearded_bust);
    let _ = super::IntercomBust::new(EvaAttire::astronaut(), (0.0, 0.0)).emit(1.62, &mut clean_bust);

    assert!(
        bearded_bust.quads.len() > clean_bust.quads.len() + 20,
        "CommandCrest must layer sculpted beard and visor over neck-ring EVA helm"
    );
    let cam_eye = vp.inverse().project_point3(Vec3::ZERO);
    assert!(
        cam_eye.x.abs() > 0.08,
        "IntercomBust portrait camera must frame the pilot from a sculpted 3/4 offset angle (eye={cam_eye:?})"
    );

    for fem in [false, true] {
        let mut cur = EvaAttire::astronaut().with_feminine(fem);
        for _ in 0..8 {
            let mut quads = QuadLedger::empty();
            let _ = super::IntercomBust::new(cur, (0.1, -0.05)).emit(1.62, &mut quads);
            assert!(quads.quads.len() > 120, "Every male and female HelmStyle decorator chain must emit sculpted geometry");
            cur = cur.with_next_helm();
        }
    }
}
