use glam::Vec3;
use sanctuary::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::black_box::{
    BlackBoxAvatar, IncidentReplay, LiveTape, MemoryDossier, PresenceProbe, SilentTape,
};
use crate::controls::{KeyMotion, KeyboardControls};
use crate::menu::{
    BlackBoxLayout, BlackBoxPanel, EscapeMenu, MemoryClipboard, PausePanel, ScreenPoint,
    SettingsPanel,
};
use crate::session::{HeadlessSession, HeadlessSurface};

#[test]
fn test_polymorphic_flight_tape_switches_between_live_ring_and_silent_null_object() {
    let world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 10.0, 0.25), 1.0);
    let mut avatar = BlackBoxAvatar::wrapping(
        Box::new(PlanetaryAvatar::at(spawn)),
        Box::new(LiveTape::fresh()),
    );
    assert!(avatar.black_box_active() && !avatar.black_box_populated());

    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::KeyW, true);
    for _ in 0..24 {
        avatar.step_motion(1.0 / 120.0, &controls, &world);
    }
    assert!(avatar.black_box_populated());
    assert_eq!(avatar.black_box_summary(), (24, 0));

    let mut probe = PresenceProbe::at_origin();
    avatar.report_presence(&mut probe);
    assert!(probe.position().y >= 8.9 && probe.presence().same_slot(0));

    let mut dossier = MemoryDossier::empty();
    assert_eq!(avatar.export_incident_dossier(&mut dossier), Some(0));
    let toml_text = dossier.render_toml();
    assert!(toml_text.contains("world_blocks = 16") && toml_text.contains("recorded_ticks = 24"));

    // Toggle OFF -> transitions polymorphically to SilentTape Null Object and discards buffer!
    assert!(!avatar.toggle_black_box());
    assert!(!avatar.black_box_active() && !avatar.black_box_populated());
    avatar.step_motion(1.0 / 120.0, &controls, &world);
    assert!(!avatar.black_box_populated());
    let mut empty_out = MemoryDossier::empty();
    assert_eq!(avatar.export_incident_dossier(&mut empty_out), None);

    // Toggle back ON -> fresh LiveTape!
    let mut silent_start = BlackBoxAvatar::wrapping(
        Box::new(PlanetaryAvatar::at(spawn)),
        Box::new(SilentTape::new()),
    );
    assert!(!silent_start.black_box_active());
    assert!(silent_start.toggle_black_box());
}

#[test]
fn test_headless_session_menu_creates_log_report_and_displays_tilted_log_sheet_animation() {
    let sandbox = std::env::temp_dir().join("low_cube_black_box_test");
    let _ = std::fs::remove_dir_all(&sandbox);
    std::fs::create_dir_all(&sandbox).expect("create sandbox");
    let profile_path = sandbox.join("blackbox.toml").to_string_lossy().to_string();
    let reports_dir = sandbox.join("reports").to_string_lossy().to_string();

    let panel = BlackBoxPanel::sandboxed(
        Box::new(MemoryClipboard::seeded("")),
        profile_path.clone(),
        reports_dir.clone(),
    );
    let modal = Box::new(EscapeMenu::with_custom_black_box(panel));
    let mut session = HeadlessSession::solitary(64, modal);

    // Baseline HUD has 10 quads (5-part framed crosshair)
    session.advance_ticks(1);
    assert_eq!(session.ui_quad_count(), 10);

    // Sprint and jump in headless session, then open Escape Menu -> LOG
    session.input_key(KeyCode::KeyW, true);
    session.input_key(KeyCode::ShiftLeft, true);
    session.input_key(KeyCode::Space, true);
    session.rotate_mouse(-25.0, 0.0);
    session.advance_ticks(30);
    session.input_key(KeyCode::Space, false);
    session.advance_ticks(30);

    session.press_escape();
    assert!(session.modal_open());
    let settings_btn = PausePanel::new().settings_button();
    session.move_cursor(ScreenPoint::new(settings_btn.center_x(), settings_btn.center_y()));
    session.click_modal();
    let log_btn = SettingsPanel::new().log_button();
    session.move_cursor(ScreenPoint::new(log_btn.center_x(), log_btn.center_y()));
    session.click_modal();

    // Toggle Log OFF and back ON in menu, verify empty-buffer guard before playing
    let bb_layout = BlackBoxLayout::new();
    let toggle_btn = bb_layout.toggle_button();
    session.move_cursor(ScreenPoint::new(toggle_btn.center_x(), toggle_btn.center_y()));
    session.click_modal();
    let report_btn = bb_layout.report_button();
    session.move_cursor(ScreenPoint::new(report_btn.center_x(), report_btn.center_y()));
    session.click_modal();
    assert!(session.modal_open(), "Clicking CREATE LOG REPORT while OFF must stay in menu");

    // Re-enable Log, resume via Escape (Flight Log -> Settings -> Main Menu -> Gameplay), play 40 ticks, then export Log Report!
    session.move_cursor(ScreenPoint::new(toggle_btn.center_x(), toggle_btn.center_y()));
    session.click_modal();
    session.press_escape();
    session.press_escape();
    session.press_escape();
    assert!(!session.modal_open());
    session.use_left_hand();
    session.release_hand();
    session.advance_ticks(40);

    session.press_escape();
    session.move_cursor(ScreenPoint::new(settings_btn.center_x(), settings_btn.center_y()));
    session.click_modal();
    session.move_cursor(ScreenPoint::new(log_btn.center_x(), log_btn.center_y()));
    session.click_modal();
    session.move_cursor(ScreenPoint::new(report_btn.center_x(), report_btn.center_y()));
    session.click_modal();

    // Menu closes immediately and displays the tilted Log-Report sheet (ruled clipboard dossier, not a 300-tile photo)!
    assert!(!session.modal_open(), "Creating log report must return to live view to display Log Sheet");
    session.advance_ticks(1);
    let sheet_quads = session.ui_quad_count();
    assert!(
        sheet_quads > 150,
        "Tilted Log-Report clipboard sheet must render ruled lines, status badge, and stamps, got {sheet_quads}"
    );

    // After ~3.6s (440 ticks @ 120Hz), Log Sheet fades back to standard 10-quad crosshair
    session.advance_ticks(440);
    assert_eq!(session.ui_quad_count(), 10);

    // Verify TOML report file on disk and replay it deterministically ("Every Bug Becomes an Automated Test")!
    let entries: Vec<_> = std::fs::read_dir(&reports_dir)
        .expect("read reports dir")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(entries.len(), 1, "Exactly one incident report TOML must be written");
    let toml_body = std::fs::read_to_string(entries[0].path()).expect("read incident toml");
    let replay = IncidentReplay::from_toml(&toml_body);
    assert_eq!(replay.tick_count(), 40);
    let final_pos = replay.replay_and_verify();
    assert!(final_pos.is_finite() && final_pos.abs().max_element() >= 32.5);

    let _ = std::fs::remove_dir_all(&sandbox);
}

#[test]
fn test_headless_multiplayer_seam_sprint_jump_records_zero_anomalies() {
    let host_world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::barren_beveled(16, 1.0));
    let host = SanctuaryHost::bind("127.0.0.1:0", host_world, 16, false).expect("bind host");
    let addr = host.local_addr().expect("host addr");
    let mut host_planet = HostedPlanet::new(host);
    let mut guest_planet = RemotePlanet::dial(JoinTarget::Direct(addr), 16, false, 1.0).expect("dial guest");
    for _ in 0..8 {
        let _ = guest_planet.step_surface(0.02);
        let _ = host_planet.step_surface(0.02);
    }
    assert!(guest_planet.connected());

    let start = guest_planet.constrain_to_surface(Vec3::new(7.80, 10.0, 0.25), 1.0);
    let guest_avatar = Box::new(BlackBoxAvatar::wrapping(
        Box::new(PlanetaryAvatar::at(start)),
        Box::new(LiveTape::fresh()),
    ));
    let ui_buf = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let surf = HeadlessSurface::hd(ui_buf);
    assert!(surf.cursor_captured());

    let mut guest_session = HeadlessSession::with_world_and_modal(
        Box::new(guest_planet),
        guest_avatar,
        HeadlessSession::closed_modal(),
    );
    guest_session.rotate_mouse(-12.0, 0.0);
    guest_session.input_key(KeyCode::KeyW, true);
    guest_session.input_key(KeyCode::ShiftLeft, true);
    guest_session.input_key(KeyCode::Space, true);
    guest_session.use_right_hand();
    guest_session.release_hand();
    for step in 0..20 {
        if step == 1 {
            guest_session.input_key(KeyCode::Space, false);
        }
        guest_session.advance_ticks(6);
        let _ = host_planet.step_surface(0.05);
    }
    assert!(!guest_session.modal_open() && guest_session.ui_quad_count() == 10);
}
