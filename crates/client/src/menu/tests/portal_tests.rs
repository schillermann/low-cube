use std::net::{Ipv4Addr, SocketAddr};
use glam::Vec3;
use sanctuary::{JoinTarget, PortalCode, PortalTicket};
use winit::keyboard::KeyCode;
use crate::avatar::PlanetaryAvatar;
use crate::menu::{
    EscapeMenu, MemoryClipboard, ModalDirective, PausePanel, PortalBounds, PortalInvite,
    PortalNotice, PortalStudio, ScratchClipboard, ScreenPoint, SessionModal,
};
use super::{MockSurface, QuadCapture};

#[test]
fn test_in_game_menu_supports_twelve_char_code_fritzbox_punch_clipboard_paste_and_server_join() {
    let surface = MockSurface::active();
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 9.0, 0.0));
    let addr: SocketAddr = "127.0.0.1:27777".parse().expect("addr");
    let sample_ticket = PortalTicket::new([9u8; 32], addr);
    let encoded_ticket = sample_ticket.encode();
    let host_code = PortalCode::new(Ipv4Addr::new(84, 152, 20, 44), 27815, 91);
    let host_code_str = host_code.encode();

    let mut menu = EscapeMenu::with_custom_portal(PortalStudio::with_clipboard(
        PortalInvite::empty(),
        Box::new(MemoryClipboard::seeded(&encoded_ticket)),
    ));

    let _ = menu.toggle_escape(&surface);
    let pause = PausePanel::new();
    let mp_btn = pause.multiplayer_button();
    let quit_btn = pause.quit_button();
    assert!(
        mp_btn.center_y() > quit_btn.center_y(),
        "MULTIPLAYER button must sit above QUIT on the Main Menu"
    );
    menu.move_cursor(ScreenPoint::new(mp_btn.center_x(), mp_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    let compact_bounds = PortalBounds::compact();
    let host_btn = compact_bounds.portal_button();
    menu.move_cursor(ScreenPoint::new(host_btn.center_x(), host_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::OpenPortal);
    menu.announce_portal(PortalInvite::hosted(host_code_str, encoded_ticket.clone()));
    let bounds = PortalBounds::new(true);
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    let join_btn = bounds.join_portal_button();
    menu.move_cursor(ScreenPoint::new(join_btn.center_x(), join_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    let mut join_ui = QuadCapture::empty();
    menu.emit_ui(&avatar, &mut join_ui);
    assert!(join_ui.count() > 50, "Portal Join panel must render input field and action buttons");

    let guest_ep: SocketAddr = "91.45.77.19:54120".parse().expect("guest addr");
    let guest_punch = host_code.mint_punch(guest_ep).encode();
    menu.handle_text(&guest_punch.to_ascii_lowercase());
    assert_eq!(
        menu.handle_key(KeyCode::Enter, false, &mut avatar),
        ModalDirective::PunchGuest(guest_ep)
    );

    let paste_btn = bounds.paste_button();
    menu.move_cursor(ScreenPoint::new(paste_btn.center_x(), paste_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    let connect_btn = bounds.join_button();
    menu.move_cursor(ScreenPoint::new(connect_btn.center_x(), connect_btn.center_y()));
    assert_eq!(
        menu.press_mouse(&surface, &mut avatar),
        ModalDirective::JoinPortal(JoinTarget::Ticket(sample_ticket))
    );

    let clear_btn = bounds.clear_button();
    menu.move_cursor(ScreenPoint::new(clear_btn.center_x(), clear_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    let remote_code = PortalCode::new(Ipv4Addr::new(79, 210, 14, 88), 27833, 205);
    menu.handle_text(&remote_code.encode().replace('-', "").to_ascii_lowercase());
    assert_eq!(
        menu.handle_key(KeyCode::Enter, false, &mut avatar),
        ModalDirective::JoinPortal(JoinTarget::Wormhole(remote_code))
    );

    let back_to_mp = bounds.back_button();
    menu.move_cursor(ScreenPoint::new(back_to_mp.center_x(), back_to_mp.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    let back_to_main = bounds.hub_back_button();
    menu.move_cursor(ScreenPoint::new(back_to_main.center_x(), back_to_main.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    menu.announce_punch_code("ABCD-EFGH-JKMN".to_string());
    assert!(menu.dialing_active());
    menu.complete_join(&surface);
    assert!(!menu.open() && !menu.dialing_active() && surface.cursor_captured());
}

#[test]
fn test_portal_studio_distinguishes_os_clipboard_copy_from_local_scratch_fallback() {
    let scratch_path = std::env::temp_dir().join("test_low_cube_scratch_clipboard.txt");
    let _ = std::fs::remove_file(&scratch_path);
    let mut local_only = PortalStudio::with_clipboard(
        PortalInvite::empty(),
        Box::new(ScratchClipboard::new(scratch_path.clone())),
    );
    local_only.announce_hosted(PortalInvite::hosted(
        "K7M9-P2X4-R8W3".to_string(),
        "LC1-FALLBACK-TEST".to_string(),
    ));
    assert_eq!(local_only.current_notice(), PortalNotice::ClipboardUnavailable);
    let _ = local_only.paste_from_clipboard();
    let _ = std::fs::remove_file(&scratch_path);

    let mut startup = PortalStudio::with_invite(
        PortalInvite::hosted("K7M9-P2X4-R8W3".to_string(), "LC1-STARTUP-TICKET".to_string()),
        Box::new(MemoryClipboard::seeded("")),
    );
    assert_eq!(startup.current_notice(), PortalNotice::StartupTicketCopied);
    startup.refresh_startup_copy();
    assert_eq!(startup.current_notice(), PortalNotice::TicketCopied);
}

#[test]
fn test_every_crockford_portal_symbol_renders_unique_5x7_glyph_geometry() {
    use crate::menu::PixelStamp;

    let stamp = PixelStamp::title(Vec3::ONE);
    let symbols: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
    let mut rendered: Vec<(char, Vec<Vec3>)> = Vec::with_capacity(symbols.len());

    for &ch in &symbols {
        let mut capture = QuadCapture::empty();
        stamp.emit_centered(&ch.to_string(), 0.0, 0.0, &mut capture);
        assert!(capture.count() >= 5, "Glyph '{ch}' must emit visible pixel quads");
        let verts = capture.positions();
        for (prev_ch, prev_verts) in &rendered {
            assert_ne!(
                &verts, prev_verts,
                "Glyph '{ch}' must never share identical pixel geometry with '{prev_ch}' (e.g. '5' vs 'S')"
            );
        }
        rendered.push((ch, verts));
    }
}

#[test]
fn test_portal_draft_auto_groups_three_four_char_blocks_without_manual_hyphens() {
    let mut studio = PortalStudio::with_clipboard(
        PortalInvite::empty(),
        Box::new(MemoryClipboard::seeded("")),
    );
    studio.append_text("sj01");
    assert_eq!(studio.formatted_draft(), "SJ01-");
    studio.append_text("sqgk");
    assert_eq!(studio.formatted_draft(), "SJ01-SQGK-");
    studio.append_text("w85b");
    assert_eq!(studio.formatted_draft(), "SJ01-SQGK-W85B");

    let _ = studio.handle_key(KeyCode::Backspace, false);
    assert_eq!(studio.formatted_draft(), "SJ01-SQGK-W85");

    let mut host_studio = PortalStudio::with_clipboard(
        PortalInvite::empty(),
        Box::new(MemoryClipboard::seeded("")),
    );
    host_studio.append_text("localhost");
    assert_eq!(host_studio.formatted_draft(), "localhost");
}

#[test]
fn test_multiplayer_hub_renders_open_join_and_back_button_labels() {
    use crate::menu::{MenuView, PixelStamp};

    let studio = PortalStudio::with_clipboard(
        PortalInvite::empty(),
        Box::new(MemoryClipboard::seeded("")),
    );
    let pointer = ScreenPoint::new(-0.95, -0.95);
    let mut hub_capture = QuadCapture::empty();
    studio.hub_view(pointer).emit(&mut hub_capture);
    let hub_positions = hub_capture.positions();

    let bounds = PortalBounds::compact();
    let pause = PausePanel::new();
    assert!(
        (bounds.portal_button().center_y() - bounds.join_portal_button().center_y()
            - (bounds.join_portal_button().center_y() - bounds.hub_back_button().center_y()))
        .abs()
            < 1e-5,
        "Compact Multiplayer hub must keep identical vertical spacing between OPEN, JOIN, and BACK"
    );
    assert!(
        (bounds.portal_button().center_y() - pause.resume_button().center_y()).abs() < 1e-5
            && (bounds.join_portal_button().center_y() - pause.character_button().center_y()).abs() < 1e-5
            && (bounds.hub_back_button().center_y() - pause.settings_button().center_y()).abs() < 1e-5,
        "Compact Multiplayer hub rows must align directly with Main Menu button rows"
    );
    assert!(
        hub_positions.iter().all(|v| v.x.abs() <= 0.371),
        "Compact Multiplayer hub card width must match Main Menu (-0.36..0.36 plus border)"
    );
    let expected = [
        (bounds.portal_button(), "OPEN"),
        (bounds.join_portal_button(), "JOIN"),
        (bounds.hub_back_button(), "BACK"),
    ];
    for (btn, label) in expected {
        let mut label_capture = QuadCapture::empty();
        PixelStamp::caption(Vec3::ONE).emit_centered(label, btn.center_x(), btn.center_y(), &mut label_capture);
        let label_positions = label_capture.positions();
        assert!(!label_positions.is_empty());
        assert!(
            hub_positions.windows(label_positions.len()).any(|win| win == label_positions.as_slice()),
            "Multiplayer hub button must emit exact glyph geometry for '{label}'"
        );
    }

    let mut join_capture = QuadCapture::empty();
    studio.emit_join_screen(pointer, &mut join_capture);
    let join_positions = join_capture.positions();
    for (btn, label) in [
        (bounds.paste_button(), "PASTE"),
        (bounds.join_button(), "JOIN HOST"),
        (bounds.back_button(), "BACK"),
    ] {
        let mut label_capture = QuadCapture::empty();
        PixelStamp::caption(Vec3::ONE).emit_centered(label, btn.center_x(), btn.center_y(), &mut label_capture);
        let label_positions = label_capture.positions();
        assert!(
            join_positions.windows(label_positions.len()).any(|win| win == label_positions.as_slice()),
            "Join screen button must emit exact glyph geometry for '{label}'"
        );
    }
}
