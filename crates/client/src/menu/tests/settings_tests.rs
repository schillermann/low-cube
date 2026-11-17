use glam::Vec3;
use winit::keyboard::KeyCode;
use crate::attire::{AttireProfile, FileAttire};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{BindingProfile, ControlAction, FileBindings, InputTrigger};
use crate::menu::{
    BlackBoxLayout, ControlLayout, ControlPanel, ControlStudio, EscapeMenu, MenuView,
    ModalDirective, PausePanel, SavedControls, SavedWardrobe, ScreenPoint, SessionModal,
    SettingsPanel, VersionedView, WardrobePanel, WardrobeStudio,
};
use super::{MockSurface, QuadCapture};

#[test]
fn test_settings_submenu_groups_keybindings_and_log_and_resume_closes_menu() {
    let surface = MockSurface::active();
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 9.0, 0.0));
    let mut menu = EscapeMenu::closed();

    menu.toggle_escape(&surface);
    assert!(menu.open() && !surface.cursor_captured());

    let pause = PausePanel::new();
    let settings_btn = pause.settings_button();
    menu.move_cursor(ScreenPoint::new(settings_btn.center_x(), settings_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    let mut settings_hub_ui = QuadCapture::empty();
    menu.emit_ui(&avatar, &mut settings_hub_ui);
    assert!(settings_hub_ui.count() > 40, "Settings submenu must render KEYBINDINGS, LOG, and BACK");

    let settings = SettingsPanel::new();
    let log_btn = settings.log_button();
    menu.move_cursor(ScreenPoint::new(log_btn.center_x(), log_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    let log_back = BlackBoxLayout::new().back_button();
    menu.move_cursor(ScreenPoint::new(log_back.center_x(), log_back.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    let settings_back = settings.back_button();
    menu.move_cursor(ScreenPoint::new(settings_back.center_x(), settings_back.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    let resume_btn = pause.resume_button();
    menu.move_cursor(ScreenPoint::new(resume_btn.center_x(), resume_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    assert!(!menu.open() && surface.cursor_captured());
}

#[test]
fn test_saved_decorators_persist_profiles_and_view_decorators_compose_quads() {
    let mut raw_avatar = PlanetaryAvatar::at(Vec3::new(0.0, 9.0, 0.0));
    let mut avatar: &mut dyn Avatar = &mut raw_avatar;
    let tmp = std::env::temp_dir();
    let ctrl_file = tmp.join("test_saved_controls_decorator.toml");
    let attire_file = tmp.join("test_saved_wardrobe_decorator.toml");
    let _ = std::fs::remove_file(&ctrl_file);
    let _ = std::fs::remove_file(&attire_file);

    let ctrl_path = ctrl_file.to_string_lossy().to_string();
    let mut saved_controls = SavedControls::new(
        Box::new(ControlStudio::seeded()),
        Box::new(FileBindings::new(ctrl_path.clone())),
    );
    let satchel = ControlLayout::new().action_bounds(ControlAction::Satchel);
    saved_controls.move_pointer(ScreenPoint::new(satchel.center_x(), satchel.center_y()));
    saved_controls.press_pointer(&mut avatar);
    assert!(saved_controls.handle_trigger(InputTrigger::Key(KeyCode::KeyM), &mut avatar));
    assert_eq!(
        FileBindings::new(ctrl_path).load_or_seeded().trigger_for(ControlAction::Satchel),
        InputTrigger::Key(KeyCode::KeyM)
    );

    let attire_path = attire_file.to_string_lossy().to_string();
    let mut saved_wardrobe = SavedWardrobe::new(
        Box::new(WardrobeStudio::seeded()),
        Box::new(FileAttire::new(attire_path.clone())),
    );
    saved_wardrobe.move_pointer(ScreenPoint::new(-0.59, -0.70));
    saved_wardrobe.press_pointer(&mut avatar);
    assert!(FileAttire::new(attire_path).load_or_squire().feminine());

    let (mut card_only, mut versioned_card) = (QuadCapture::empty(), QuadCapture::empty());
    SettingsPanel::new().view(ScreenPoint::center(), false).emit(&mut card_only);
    VersionedView::current(SettingsPanel::new().view(ScreenPoint::center(), false)).emit(&mut versioned_card);
    assert!(versioned_card.count() > card_only.count());

    let _ = std::fs::remove_file(&ctrl_file);
    let _ = std::fs::remove_file(&attire_file);
}
