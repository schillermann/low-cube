use std::path::PathBuf;
use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BlockMedium, BlockMold, PlanetaryWorld as _, SolarProportions, WorldSeed};
use crate::attire::{HeldRelic, YouthAttire};
use crate::avatar::{Avatar as _, PlanetaryAvatar};
use crate::menu::{
    EscapeMenu, LobbyGateModal, ModalDirective, PausePanel, SavedWorldStudio, ScreenPoint,
    SessionModal, WorldLayout, WorldStudio,
};
use crate::session::HeadlessSession;
use crate::surface::PlatformSurface as _;
use crate::world_vault::{
    DiskWorldVault, RealmPort, ScaleTier, SeedPhrase, WallEpoch, WorldArchive, WorldSpec,
    WorldTitle,
};
use super::{MockSurface, QuadCapture};

fn temp_worlds_dir(tag: &str) -> PathBuf {
    let nanos = WallEpoch::new().nanos();
    std::env::temp_dir().join(format!("lowcube_world_vault_{tag}_{}_{nanos}", std::process::id()))
}

fn sample_spec(name: &str, factor: u32, seed_raw: &str) -> WorldSpec {
    let title = WorldTitle::sanitized(name);
    let slug = title.derive_slug();
    WorldSpec::new(slug, title, ScaleTier::clamped(factor), SeedPhrase::from_input(seed_raw))
}

#[test]
fn test_lobby_gate_decorator_guards_startup_and_create_world_configures_name_size_and_seed() {
    let surface = MockSurface::active();
    surface.release_cursor();
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 9.0, 0.0));
    let mut menu = LobbyGateModal::guarding(EscapeMenu::opened_on_worlds(Box::new(WorldStudio::empty())));

    assert!(menu.open(), "LobbyGateModal must keep startup menu open before any world is entered");
    let mut ui = QuadCapture::empty();
    menu.emit_ui(&avatar, &mut ui);
    assert!(ui.count() > 80, "Create World screen must render name, size, seed, and create controls");

    // Pressing Escape or clicking RESUME before a world exists stays guarded by LobbyGateModal
    assert_eq!(menu.toggle_escape(&surface), ModalDirective::ContinueSession);
    assert!(menu.open() && !surface.cursor_captured());
    let resume_btn = PausePanel::new().resume_button();
    menu.move_cursor(ScreenPoint::new(resume_btn.center_x(), resume_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    assert!(menu.open() && !surface.cursor_captured(), "LobbyGateModal must prevent entering uncreated gameplay");

    // Click WORLDS button on Main Menu to open Create World
    let worlds_btn = PausePanel::new().worlds_button();
    menu.move_cursor(ScreenPoint::new(worlds_btn.center_x(), worlds_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    let layout = WorldLayout::new();
    let clear_btn = layout.clear_name_button();
    menu.move_cursor(ScreenPoint::new(clear_btn.center_x(), clear_btn.center_y()));
    let _ = menu.press_mouse(&surface, &mut avatar);
    menu.handle_text("Heimat!");
    let _ = menu.handle_key(KeyCode::Backspace, false, &mut avatar);
    menu.handle_text("t");

    // Advance world size twice: 1X (128m) -> 2X (256m) -> 4X (512m)
    let next_size = layout.size_next_button();
    menu.move_cursor(ScreenPoint::new(next_size.center_x(), next_size.center_y()));
    let _ = menu.press_mouse(&surface, &mut avatar);
    let _ = menu.press_mouse(&surface, &mut avatar);

    // Focus seed field, clear via Backspace, and type custom seed "ALPINE-7"
    let seed_box = layout.seed_field();
    menu.move_cursor(ScreenPoint::new(seed_box.center_x(), seed_box.center_y()));
    let _ = menu.press_mouse(&surface, &mut avatar);
    for _ in 0..16 {
        let _ = menu.handle_key(KeyCode::Backspace, false, &mut avatar);
    }
    menu.handle_text("alpine-7");

    // Click CREATE WORLD -> LobbyGateModal opens gate and returns ActivateWorld(spec) directly
    let create_btn = layout.create_button();
    menu.move_cursor(ScreenPoint::new(create_btn.center_x(), create_btn.center_y()));
    let ModalDirective::ActivateWorld(created) = menu.press_mouse(&surface, &mut avatar) else {
        panic!("Expected ModalDirective::ActivateWorld");
    };
    assert!(!menu.open() && surface.cursor_captured(), "Creating world must open gate and capture cursor");
    assert!(created.matches_spec("HEIMAT", 512, "ALPINE-7", WorldSeed::from_phrase("ALPINE-7")));
}

#[test]
fn test_saved_world_studio_decorator_persists_roster_and_relocks_lobby_gate_on_active_delete() {
    let dir = temp_worlds_dir("studio_decorator");
    let mut vault = DiskWorldVault::at_path(&dir, world::SatchelManifest::starter_kit());
    let alpha = vault.persist_world(&sample_spec("ALPHA", 1, "111"));
    let beta = vault.persist_world(&sample_spec("BETA", 2, "222"));

    let surface = MockSurface::active();
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 9.0, 0.0));
    let studio = SavedWorldStudio::live(vault.clone());
    let mut menu = LobbyGateModal::guarding(EscapeMenu::opened_on_worlds(Box::new(studio)));

    assert!(menu.open(), "Startup with existing worlds must open MY WORLDS instead of entering gameplay");
    let layout = WorldLayout::new();

    // Select ALPHA and click PLAY WORLD
    let row0 = layout.slot_row(0);
    menu.move_cursor(ScreenPoint::new(row0.center_x(), row0.center_y()));
    let _ = menu.press_mouse(&surface, &mut avatar);
    let play_btn = layout.play_button();
    menu.move_cursor(ScreenPoint::new(play_btn.center_x(), play_btn.center_y()));
    let ModalDirective::ActivateWorld(chosen) = menu.press_mouse(&surface, &mut avatar) else {
        panic!("Expected ActivateWorld");
    };
    assert!(chosen.same_slug(alpha.slug()) || chosen.same_slug(beta.slug()));
    assert!(!menu.open());

    // Open Escape Menu -> WORLDS -> delete active row 0 -> LobbyGateModal re-locks gate!
    let _ = menu.toggle_escape(&surface);
    let worlds_btn = PausePanel::new().worlds_button();
    menu.move_cursor(ScreenPoint::new(worlds_btn.center_x(), worlds_btn.center_y()));
    let _ = menu.press_mouse(&surface, &mut avatar);
    menu.move_cursor(ScreenPoint::new(row0.center_x(), row0.center_y()));
    let _ = menu.press_mouse(&surface, &mut avatar);
    let del_btn = layout.delete_button();
    menu.move_cursor(ScreenPoint::new(del_btn.center_x(), del_btn.center_y()));
    let ModalDirective::EvictWorld(evicted_slug) = menu.press_mouse(&surface, &mut avatar) else {
        panic!("Expected EvictWorld when deleting active world");
    };
    vault.purge_evicted(&evicted_slug);
    assert_eq!(vault.load_roster().len(), 1);
    assert!(menu.open(), "Deleting active world must re-engage LobbyGateModal");
    let _ = menu.toggle_escape(&surface);
    let _ = menu.toggle_escape(&surface);
    assert!(menu.open(), "Re-engaged LobbyGateModal must block Escape from entering evicted world");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_disk_world_vault_persists_blocks_and_inventory_and_seeded_decorators_vary_terrain_and_flora() {
    let seed_a = WorldSeed::from_phrase("GLACIER");
    let seed_b = WorldSeed::from_phrase("CANYON");
    assert_ne!(seed_a, seed_b);
    let prop = SolarProportions::true_scale(256).with_seed(seed_a);
    let mut wire = Vec::new();
    prop.inscribe_tail(&mut wire);
    assert_eq!(SolarProportions::decode_tail(256, &wire).seed(), seed_a);

    // Verify SeededCosmos<EpsteinReachCosmos> caps distance_scale identically to EpsteinReachCosmos
    let continental = sample_spec("CONTINENT", 32, "GLACIER").proportions(1.0);
    assert!(continental.astronomical_unit() <= SolarProportions::EPSTEIN_HOUR_AU_CAP + 1.0);
    assert_eq!(continental.seed(), seed_a);

    // Verify SeededFrequency + SeededGround produce distinct terrain & forest layouts
    let world_a = world::BiomedCubeWorld::seeded_beveled(128, 1.0, seed_a);
    let world_b = world::BiomedCubeWorld::seeded_beveled(128, 1.0, seed_b);
    let differ = (6..28).any(|i| {
        let x = (i as f32) * 1.5;
        let a = world_a.constrain_to_surface(Vec3::new(x, 80.0, 14.0), 1.0);
        let b = world_b.constrain_to_surface(Vec3::new(x, 80.0, 14.0), 1.0);
        (a - b).length() > 0.25
    });
    assert!(differ, "SeededFrequency and SeededGround decorators must vary planetary relief and groves");

    let dir = temp_worlds_dir("persist_vault");
    let mut vault = DiskWorldVault::at_path(&dir, world::SatchelManifest::starter_kit());
    let spec = vault.persist_world(&sample_spec("SOLO REALM", 1, "GLACIER"));
    assert_eq!(vault.load_roster().len(), 1);

    let eye = Vec3::new(4.25, 68.0, 4.25);
    let baseline_ground = {
        let mut world = vault.mount_solitary(&spec);
        let ground = world.constrain_to_surface(eye, 1.0);
        let mold = BlockMold::cube().in_medium(BlockMedium::QuarriedStone);
        assert!(world.place_molded_surface_block(ground + Vec3::Y * 1.6, Vec3::NEG_Y, ground + Vec3::X * 2.0, mold));
        let mut custom_kit = world::SatchelManifest::starter_kit();
        custom_kit.credit_medium(BlockMedium::GoldOre);
        world.commit_local_survivor(world::CosmicFrame::earth(), ground + Vec3::X * 6.0, custom_kit);
        ground
    };

    {
        let reloaded = vault.mount_solitary(&spec);
        let after_ground = reloaded.constrain_to_surface(eye, 1.0);
        assert!(after_ground.y > baseline_ground.y + 0.3, "Placed block must persist across world reloads");
        let restored = reloaded
            .assigned_survivor()
            .and_then(|w| w.restored_record())
            .expect("Survivor checkpoint must reload from disk");
        let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 70.0, 0.0));
        avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm));
        avatar.land_on_world(reloaded.as_ref());
        assert!(
            restored.manifest().count_of(world::RelicCatalog::from_medium(BlockMedium::GoldOre)) >= 1,
            "Saved player inventory must persist across world reloads"
        );
    }

    assert!(vault.erase_world(spec.slug()));
    assert!(vault.load_roster().is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_headless_session_mounts_world_through_lobby_gate_without_shared_refcell() {
    let modal = Box::new(LobbyGateModal::guarding(EscapeMenu::opened_on_worlds(Box::new(WorldStudio::empty()))));
    let world: Box<dyn world::PlanetaryWorld> = Box::new(world::BiomedCubeWorld::barren_beveled(16, 1.0));
    let avatar = Box::new(PlanetaryAvatar::at(Vec3::new(0.0, 10.0, 0.0)));
    let mut session = HeadlessSession::with_world_and_modal(world, avatar, modal);

    assert!(session.modal_open(), "Session must start guarded on Create World screen");
    let clear_btn = WorldLayout::new().clear_name_button();
    session.move_cursor(ScreenPoint::new(clear_btn.center_x(), clear_btn.center_y()));
    session.click_modal();
    session.type_text("MY CUBE");
    session.input_key(KeyCode::Enter, true);
    session.input_key(KeyCode::Enter, false);

    assert!(!session.modal_open(), "Pressing Enter on Create World must mount the world and open LobbyGateModal");
    session.advance_ticks(4);
}
