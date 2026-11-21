use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, CountedMeshTarget, MeshTarget, MountDomain,
    PlanetaryWorld, Quad, WorldEarthworks as _, WorldFact,
};
use crate::attire::{BeltSatchel, HeldRelic, YouthAttire};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarOptics as _, AvatarPresence as _, HandGesture,
    PlanetaryAvatar,
};
use crate::blueprint::{BlueprintArchive, DirectoryBlueprints, MemoryBlueprints, SavedBlueprint};
use crate::controls::{InputTrigger, KeyMotion, KeyboardControls};
use crate::server_satchel::StartingSatchel;
use crate::session::HeadlessSession;

struct NullMeshSink;

impl MeshTarget for NullMeshSink {
    fn push_quad(&mut self, _quad: Quad) {}
}

struct CarPartsKit;

impl StartingSatchel for CarPartsKit {
    fn provision(&self, satchel: BeltSatchel) -> BeltSatchel {
        satchel
            .with_stowed_relic(HeldRelic::SteeringWheel, 1)
            .with_stowed_relic(HeldRelic::CabinSeat, 1)
            .with_stowed_relic(HeldRelic::CarWheel, 1)
    }
}

struct SparseKit;

impl StartingSatchel for SparseKit {
    fn provision(&self, satchel: BeltSatchel) -> BeltSatchel {
        satchel.with_stowed_relic(HeldRelic::CabinSeat, 1)
    }
}

struct SampleCraneCar;

impl SampleCraneCar {
    fn assemble_on(&self, world: &mut dyn PlanetaryWorld) -> f32 {
        let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
        let rig_mold = BlockMold::cube().in_medium(BlockMedium::CantileverRig);
        let eng_mold = BlockMold::cube().turn_clockwise().in_medium(BlockMedium::SteeringWheel);
        let seat_mold = BlockMold::cube().in_medium(BlockMedium::CabinSeat);
        let wheel_mold = BlockMold::cube().in_medium(BlockMedium::CarWheel);
        let av_pos = Vec3::new(0.25, g0, 1.75);
        let aim = Vec3::new(0.0, -0.8, -0.6).normalize();
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), aim, av_pos, rig_mold));
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), aim, av_pos, eng_mold));
        assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 1.75, 1.25), Vec3::NEG_Z, av_pos, seat_mold));
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, av_pos, wheel_mold));
        g0
    }
}

#[test]
fn test_blueprint_block_saves_named_vehicle_and_rebuilds_drivable_craft_on_empty_crane() {
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let g0 = SampleCraneCar.assemble_on(&mut world);

    let temp_dir = std::env::temp_dir().join(format!("lowcube_bp_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut engineer = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.25, 22.0, 2.25), 1.0));
    engineer.install_blueprint_archive(Box::new(DirectoryBlueprints::new(temp_dir.clone())));
    engineer.apply_attire(YouthAttire::squire().with_relics(HeldRelic::VehicleBlueprint, HeldRelic::EmptyPalm));
    engineer.rotate_degrees(0.0, -10.0);

    let preview = engineer.planned_mount(&world);
    assert!(preview.is_some_and(|p| p.domain() == MountDomain::CraftRig));

    let _ = HandGesture::RightTap.perform(&mut engineer, &mut world);
    assert!(engineer.typing_text(), "Clicking crane craft with Blueprint Block must open name input");

    assert!(engineer.dispatch_trigger(InputTrigger::Key(KeyCode::KeyC)));
    assert!(engineer.typing_text(), "Hotkey C must not close naming input while typing");
    assert!(engineer.accept_typed_text("canyon"));
    assert!(engineer.dispatch_trigger(InputTrigger::Key(KeyCode::Backspace)));
    assert!(engineer.accept_typed_text("n #1"));

    let mut sink = NullMeshSink;
    let mut world_figure = CountedMeshTarget::new(&mut sink);
    engineer.emit_appearance(&mut world_figure);
    let mut closed_probe = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.25, 22.0, 2.25), 1.0));
    closed_probe.apply_attire(YouthAttire::squire().with_relics(HeldRelic::VehicleBlueprint, HeldRelic::EmptyPalm));
    closed_probe.rotate_degrees(0.0, -10.0);
    let mut sink_closed = NullMeshSink;
    let mut baseline_figure = CountedMeshTarget::new(&mut sink_closed);
    closed_probe.emit_appearance(&mut baseline_figure);
    assert_eq!(world_figure.count(), baseline_figure.count(), "Open blueprint folio must not leak quads into 3D world mesh");

    let mut sink_plaque = NullMeshSink;
    let mut plaque_quads = CountedMeshTarget::new(&mut sink_plaque);
    engineer.emit_inspection(&mut plaque_quads);
    assert!(plaque_quads.count() > 80);

    assert!(engineer.press_world_key(KeyCode::Enter, &mut world));
    assert!(!engineer.typing_text());
    assert!(
        temp_dir.join("canyon_1.toml").is_file(),
        "Each saved blueprint must live in its own TOML file inside the directory"
    );

    let saved = DirectoryBlueprints::new(temp_dir.clone()).load_saved();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].title_text(), "CANYON #1");
    assert_eq!(saved[0].blueprint().part_count(), 3);

    // Deploy a fresh empty crane inside flat spawn sanctuary at z = -1.75
    let rig_mold = BlockMold::cube().in_medium(BlockMedium::CantileverRig);
    let rig2_aim = Vec3::new(0.0, -0.8, -0.6).normalize();
    let av2_pos = Vec3::new(0.25, g0, -0.75);
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, -1.25), rig2_aim, av2_pos, rig_mold));

    let mut sparse_builder = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.25, 22.0, -1.25), 1.0))
        .with_starting_satchel(&SparseKit);
    sparse_builder.install_blueprint_archive(Box::new(DirectoryBlueprints::new(temp_dir.clone())));
    sparse_builder.apply_attire(YouthAttire::squire().with_relics(HeldRelic::VehicleBlueprint, HeldRelic::EmptyPalm));
    sparse_builder.rotate_degrees(0.0, 35.0);
    let _ = HandGesture::RightTap.perform(&mut sparse_builder, &mut world);
    assert!(!sparse_builder.press_world_key(KeyCode::Enter, &mut world), "Missing parts must prevent crane build");
    assert!(sparse_builder.dispatch_trigger(InputTrigger::Key(KeyCode::Escape)));

    // Provision full parts kit, select saved blueprint on empty crane arm, and build it
    let mut full_builder = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.25, 22.0, -1.25), 1.0))
        .with_starting_satchel(&CarPartsKit);
    full_builder.install_blueprint_archive(Box::new(DirectoryBlueprints::new(temp_dir.clone())));
    full_builder.apply_attire(YouthAttire::squire().with_relics(HeldRelic::VehicleBlueprint, HeldRelic::EmptyPalm));
    full_builder.rotate_degrees(0.0, 35.0);

    let _ = HandGesture::RightTap.perform(&mut full_builder, &mut world);
    assert!(HandGesture::RightTap.perform(&mut full_builder, &mut world), "Clicking ready blueprint must build craft on crane");

    let hoisted_top = world.constrain_to_surface(Vec3::new(1.25, 22.0, -1.75), 0.0).y;
    assert!(hoisted_top > g0 + 1.4, "Built blueprint craft must hang hoisted on crane arm (got {hoisted_top})");

    // Stow reusable Blueprint Block back into satchel with E, then tap mast with empty hand to lower craft
    assert!(full_builder.dispatch_trigger(InputTrigger::Key(KeyCode::KeyE)));
    assert!(HandGesture::RightTap.perform(&mut full_builder, &mut world), "Empty hand lowers built blueprint craft");

    let mut pilot = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.25, 22.0, -0.75), 1.0));
    pilot.rotate_degrees(0.0, 58.0);
    let cam_pos = |a: &PlanetaryAvatar| a.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    let before = cam_pos(&pilot);
    assert!(HandGesture::RightTap.perform(&mut pilot, &mut world));
    assert!(HandGesture::RightTap.perform(&mut pilot, &mut world));
    let mut gas = KeyMotion::idle();
    gas.register_key(KeyCode::KeyW, true);
    for _ in 0..18 {
        pilot.step_motion(1.0 / 60.0, &gas, &world);
    }
    assert!(cam_pos(&pilot).x > before.x + 0.35);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_parked_vehicle_blueprint_capture_world_fact_wire_and_headless_typing() {
    let mut world = BiomedCubeWorld::barren_beveled(32, 1.0);
    let g0 = SampleCraneCar.assemble_on(&mut world);

    // Lower crane, board vehicle via avatar, and dismount to park on terrain
    let mut crane_hand = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.25, 22.0, 1.25), 1.0));
    crane_hand.rotate_degrees(0.0, 35.0);
    assert!(HandGesture::RightTap.perform(&mut crane_hand, &mut world));

    let mut pilot = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.25, 22.0, 1.75), 1.0));
    pilot.rotate_degrees(0.0, 58.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, &mut world));
    assert!(HandGesture::RightTap.perform(&mut pilot, &mut world));
    assert!(HandGesture::RightTap.perform(&mut pilot, &mut world));

    let eye = Vec3::new(1.25, g0 + 1.0, 1.75);
    let dir = (Vec3::new(1.25, g0 + 0.25, 0.75) - eye).normalize();
    let captured = world
        .capture_vehicle_blueprint(eye, dir)
        .expect("capture blueprint from parked vehicle");
    assert_eq!(captured.part_count(), 3);

    // Verify WorldFact::RigBlueprintBuilt binary wire round-trip
    let fact = WorldFact::RigBlueprintBuilt {
        eye: Vec3::new(-4.25, 9.5, 1.25),
        direction: Vec3::new(0.0, -0.8, -0.6).normalize(),
        blueprint: captured.clone(),
    };
    let mut wire = Vec::new();
    fact.inscribe(&mut wire);
    let (decoded, used) = WorldFact::decode(&wire).expect("decode RigBlueprintBuilt fact");
    assert_eq!(used, wire.len());
    assert_eq!(decoded, fact);

    // Verify MemoryBlueprints + HeadlessSession::type_text integration
    let empty = MemoryBlueprints::empty();
    assert!(empty.load_saved().is_empty());
    let mem = MemoryBlueprints::new(vec![SavedBlueprint::new("ROVER".to_string(), captured.clone())]);
    assert_eq!(mem.load_saved().len(), 1);
    let mut session = HeadlessSession::solitary(16, HeadlessSession::closed_modal());
    session.type_text("ROVER");
    session.advance_ticks(1);
    assert!(!session.modal_open());

    // Verify DirectoryBlueprints stores one file per rocket, disambiguates colliding slugs, and deletes isolated files
    let multi_dir = std::env::temp_dir().join(format!("lowcube_bp_multi_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&multi_dir);
    let mut dir_store = DirectoryBlueprints::new(multi_dir.clone());
    dir_store.store(&SavedBlueprint::new("MK/1".to_string(), captured.clone()));
    dir_store.store(&SavedBlueprint::new("MK:1".to_string(), captured.clone()));
    dir_store.store(&SavedBlueprint::new("APOLLO 11".to_string(), captured));
    assert!(multi_dir.join("mk_1.toml").is_file());
    assert!(multi_dir.join("mk_1_2.toml").is_file());
    assert!(multi_dir.join("apollo_11.toml").is_file());
    assert_eq!(dir_store.load_saved().len(), 3);

    dir_store.expunge("MK/1");
    assert!(!multi_dir.join("mk_1.toml").exists());
    assert!(multi_dir.join("mk_1_2.toml").is_file());
    assert!(multi_dir.join("apollo_11.toml").is_file());
    assert_eq!(dir_store.load_saved().len(), 2);
    let _ = std::fs::remove_dir_all(&multi_dir);
}
