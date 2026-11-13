use std::net::{Ipv4Addr, SocketAddr};
use glam::Vec3;
use sanctuary::{JoinTarget, PortalCode, PortalTicket};
use winit::keyboard::KeyCode;
use crate::avatar::PlanetaryAvatar;
use crate::menu::{
    EscapeMenu, MemoryClipboard, ModalDirective, PausePanel, PortalInvite, PortalNotice,
    PortalStudio, ScratchClipboard, ScreenPoint, SessionModal,
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

    menu.toggle_escape(&surface);
    let pause = PausePanel::new();
    let host_btn = pause.portal_button();
    menu.move_cursor(ScreenPoint::new(host_btn.center_x(), host_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::OpenPortal);
    menu.announce_portal(PortalInvite::hosted(host_code_str.clone(), encoded_ticket.clone()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    let join_btn = pause.join_portal_button();
    menu.move_cursor(ScreenPoint::new(join_btn.center_x(), join_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    let mut join_ui = QuadCapture::empty();
    menu.emit_ui(&avatar, &mut join_ui);
    assert!(join_ui.count() > 50, "Portal Join panel must render input field and action buttons");

    // 1) Host enters Guest's 12-char FRITZ!Box counter-punch code -> emits PunchGuest(guest_ep)!
    let guest_ep: SocketAddr = "91.45.77.19:54120".parse().expect("guest addr");
    let guest_punch = host_code.mint_punch(guest_ep).encode();
    menu.handle_text(&guest_punch.to_ascii_lowercase());
    assert_eq!(
        menu.handle_key(KeyCode::Enter, false, &mut avatar),
        ModalDirective::PunchGuest(guest_ep)
    );

    // 2) Paste LC1-... ticket and join!
    let paste_btn = PortalStudio::paste_button();
    menu.move_cursor(ScreenPoint::new(paste_btn.center_x(), paste_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    let connect_btn = PortalStudio::join_button();
    menu.move_cursor(ScreenPoint::new(connect_btn.center_x(), connect_btn.center_y()));
    assert_eq!(
        menu.press_mouse(&surface, &mut avatar),
        ModalDirective::JoinPortal(JoinTarget::Ticket(sample_ticket))
    );

    // 3) Type 12-char PortalCode without hyphens -> resolves to JoinTarget::Wormhole!
    let clear_btn = PortalStudio::clear_button();
    menu.move_cursor(ScreenPoint::new(clear_btn.center_x(), clear_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    let remote_code = PortalCode::new(Ipv4Addr::new(79, 210, 14, 88), 27833, 205);
    menu.handle_text(&remote_code.encode().replace('-', "").to_ascii_lowercase());
    assert_eq!(
        menu.handle_key(KeyCode::Enter, false, &mut avatar),
        ModalDirective::JoinPortal(JoinTarget::Wormhole(remote_code))
    );

    // 4) Guest receives NAT-locked punch code banner, then completes join!
    menu.announce_punch_code("ABCD-EFGH-JKMN".to_string());
    assert!(menu.dialing_active());
    menu.complete_join(&surface);
    assert!(!menu.open() && surface.cursor_captured());
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
    local_only.paste_from_clipboard();
    let _ = std::fs::remove_file(&scratch_path);

    let mut startup = PortalStudio::with_invite(
        PortalInvite::hosted("K7M9-P2X4-R8W3".to_string(), "LC1-STARTUP-TICKET".to_string()),
        Box::new(MemoryClipboard::seeded("")),
    );
    assert_eq!(startup.current_notice(), PortalNotice::StartupTicketCopied);
    startup.refresh_startup_copy();
    assert_eq!(startup.current_notice(), PortalNotice::TicketCopied);
}
