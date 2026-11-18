use std::cell::RefCell;
use std::net::{SocketAddr, UdpSocket};
use std::rc::Rc;
use glam::Vec3;
use sanctuary::{HostedPlanet, JoinTarget, PortalCode, RemoteLink, SanctuaryHost};
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::attire::YouthAttire;
use crate::avatar::PlanetaryAvatar;
use crate::black_box::{BlackBoxAvatar, LiveTape};
use crate::controls::ActionBindings;
use crate::menu::{
    AutoPunchModal, EscapeMenu, ModalDirective, PausePanel, PortalBounds, PortalInvite,
    ScreenPoint, SessionModal, TextClipboard, WaitingBounds,
};
use crate::server_satchel::ServerSatchel;
use crate::session::HeadlessSession;
use super::{MockSurface, QuadCapture};

#[derive(Clone)]
struct SharedClipboard {
    buffer: Rc<RefCell<String>>,
}

impl SharedClipboard {
    fn empty() -> Self {
        Self { buffer: Rc::new(RefCell::new(String::new())) }
    }
}

impl TextClipboard for SharedClipboard {
    fn store_text(&self, text: &str) -> bool {
        *self.buffer.borrow_mut() = text.to_string();
        true
    }

    fn read_text(&self) -> String {
        self.buffer.borrow().clone()
    }
}

#[test]
fn test_closing_menu_keeps_punch_beacon_hud_alive_and_explicit_cancel_aborts_dial() {
    let surface = MockSurface::active();
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 9.0, 0.0));
    let shared_clip = SharedClipboard::empty();
    let mut menu = EscapeMenu::with_clipboard(PortalInvite::empty(), Box::new(shared_clip.clone()));

    let mut idle_hud = QuadCapture::empty();
    menu.emit_ui(&avatar, &mut idle_hud);

    menu.announce_punch_code("Z08G-QYHW-AJ86".to_string());
    assert!(menu.open() && menu.dialing_active());
    assert_eq!(shared_clip.read_text(), "CO-OP REPLY CODE: Z08G-QYHW-AJ86");

    let _ = menu.toggle_escape(&surface);
    assert!(!menu.open() && surface.cursor_captured());
    assert!(menu.dialing_active(), "Closing the menu must keep the NAT punch dial alive");

    menu.step(1.2);
    let mut beacon_hud = QuadCapture::empty();
    menu.emit_ui(&avatar, &mut beacon_hud);
    assert!(beacon_hud.count() > idle_hud.count() + 40, "BeaconHud decorator must render live HUD banner");

    let _ = menu.toggle_escape(&surface);
    let mp_btn = PausePanel::new().multiplayer_button();
    menu.move_cursor(ScreenPoint::new(mp_btn.center_x(), mp_btn.center_y()));
    let _ = menu.press_mouse(&surface, &mut avatar);
    let wait_btn = PortalBounds::expanded().join_portal_button();
    menu.move_cursor(ScreenPoint::new(wait_btn.center_x(), wait_btn.center_y()));
    let _ = menu.press_mouse(&surface, &mut avatar);
    let cancel_btn = WaitingBounds::new().cancel_dial_button();
    menu.move_cursor(ScreenPoint::new(cancel_btn.center_x(), cancel_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::CancelJoin);
    assert!(!menu.dialing_active());
}

#[test]
fn test_one_click_hub_paste_and_auto_punch_decorator_deduplicates_clipboard() {
    let surface = MockSurface::active();
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 9.0, 0.0));
    let shared_clip = SharedClipboard::empty();
    let host_code = PortalCode::new(std::net::Ipv4Addr::new(84, 152, 20, 44), 27815, 91);
    let guest_ep: SocketAddr = "91.45.77.19:54120".parse().expect("guest addr");
    let reply = host_code.mint_punch(guest_ep).encode();
    let invite = PortalInvite::hosted(host_code.encode(), "LC1-TEST".to_string());

    let mut decorated = AutoPunchModal::wrapping(
        EscapeMenu::with_clipboard(invite.clone(), Box::new(shared_clip.clone())),
        invite,
        Box::new(shared_clip.clone()),
    );
    let _ = shared_clip.store_text(&format!("Discord: CO-OP REPLY CODE: {reply}"));
    assert_eq!(decorated.focus_regained(), ModalDirective::PunchGuest(guest_ep));
    assert_eq!(decorated.focus_regained(), ModalDirective::ContinueSession);

    // Manual 1-click PASTE GUEST REPLY button on MultiplayerHub always punches on demand!
    let _ = decorated.toggle_escape(&surface);
    let mp_btn = PausePanel::new().multiplayer_button();
    decorated.move_cursor(ScreenPoint::new(mp_btn.center_x(), mp_btn.center_y()));
    let _ = decorated.press_mouse(&surface, &mut avatar);
    let hub_paste = PortalBounds::expanded().hub_paste_button();
    decorated.move_cursor(ScreenPoint::new(hub_paste.center_x(), hub_paste.center_y()));
    assert_eq!(decorated.press_mouse(&surface, &mut avatar), ModalDirective::PunchGuest(guest_ep));
}

#[test]
fn test_headless_guest_closing_menu_connects_in_background_when_host_auto_punch_decorator_fires() {
    let shared_clip = SharedClipboard::empty();
    let host_world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::barren_beveled(8, 1.0));
    let raw_host = SanctuaryHost::bind("127.0.0.1:0", host_world, 8, false).expect("bind host");
    let live_code = raw_host.code();
    let invite = PortalInvite::hosted(live_code.encode(), raw_host.ticket().encode());
    let hosted_planet: Box<dyn PlanetaryWorld> = Box::new(HostedPlanet::new(raw_host));

    let spawn = hosted_planet.constrain_to_surface(Vec3::new(0.25, 6.0, 0.25), 1.0);
    let host_avatar = Box::new(BlackBoxAvatar::wrapping(
        Box::new(
            PlanetaryAvatar::with_attire_and_bindings(
                spawn,
                Box::new(crate::flight::SurfaceFlight::calibrated(0.522, 10.0, 1.0)),
                YouthAttire::squire(),
                ActionBindings::seeded(),
            )
            .with_starting_satchel(&ServerSatchel::assembled()),
        ),
        Box::new(LiveTape::fresh()),
    ));
    let host_modal = Box::new(AutoPunchModal::wrapping(
        EscapeMenu::with_clipboard(invite.clone(), Box::new(shared_clip.clone())),
        invite,
        Box::new(shared_clip.clone()),
    ));
    let mut host_session = HeadlessSession::with_world_and_modal(hosted_planet, host_avatar, host_modal);

    let guest_modal = Box::new(EscapeMenu::with_clipboard(PortalInvite::empty(), Box::new(shared_clip.clone())));
    let mut guest_session = HeadlessSession::solitary(8, guest_modal);
    let guest_sock = UdpSocket::bind("127.0.0.1:0").expect("bind guest");
    let guest_ep = guest_sock.local_addr().expect("guest addr");
    let link = RemoteLink::bind_on_socket(guest_sock, guest_ep, JoinTarget::Wormhole(live_code));
    let reply_code = guest_session.begin_bound_dial(link);
    assert!(guest_session.modal_open() && guest_session.dialing_active());

    let _ = shared_clip.store_text(&format!("[Discord] Friend: CO-OP REPLY CODE: {reply_code}"));
    guest_session.press_escape();
    assert!(!guest_session.modal_open() && guest_session.dialing_active());

    host_session.focus_window();

    for _ in 0..24 {
        host_session.advance_ticks(4);
        guest_session.advance_ticks(4);
        if !guest_session.dialing_active() && guest_session.companion_count() >= 1 {
            break;
        }
    }
    assert!(!guest_session.dialing_active(), "Background dial must complete while guest plays in world");
    assert_eq!(guest_session.companion_count(), 1, "Guest must join Host's world seamlessly after menu close");
}
