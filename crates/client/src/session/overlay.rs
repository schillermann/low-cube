use world::PlanetaryWorld;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::menu::{ModalDirective, ScreenPoint, SessionModal};
use crate::portal_config::{DialOutcome, DialStep, PortalDial};
use crate::surface::PlatformSurface;
use crate::world_vault::{WorldSlug, WorldSpec};
use super::display::SessionDisplay;

/// Outcome of advancing or interacting with `StageOverlay` when world topology changes.
pub(super) enum OverlayTransition {
    Unchanged,
    ActivateWorld(WorldSpec),
    EvictWorld(WorldSlug),
    OpenLivePortal,
    Connected(Box<dyn PlanetaryWorld>),
    PunchPeer(std::net::SocketAddr),
    ExitSession,
}

/// Encapsulated keyboard control state, pause/studio modal overlay, shortcut state, and P2P NAT dial.
pub(super) struct StageOverlay {
    controls: Box<dyn KeyboardControls>,
    modal: Box<dyn SessionModal>,
    ctrl_held: bool,
    dial: PortalDial,
}

impl StageOverlay {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn with_modifiers(
        controls: Box<dyn KeyboardControls>,
        modal: Box<dyn SessionModal>,
        ctrl_held: bool,
        dial: PortalDial,
    ) -> Self {
        Self { controls, modal, ctrl_held, dial }
    }

    pub(super) fn new(controls: Box<dyn KeyboardControls>, modal: Box<dyn SessionModal>) -> Self {
        Self::with_modifiers(controls, modal, false, PortalDial::Idle)
    }

    pub(super) fn modal_open(&self) -> bool { self.modal.open() }
    #[cfg(test)]
    pub(super) fn dialing_active(&self) -> bool { self.modal.dialing_active() }
    pub(super) fn controls(&self) -> &dyn KeyboardControls { self.controls.as_ref() }
    pub(super) fn modal(&self) -> &dyn SessionModal { self.modal.as_ref() }
    pub(super) fn ctrl_active(&self) -> bool { self.ctrl_held }
    pub(super) fn update_ctrl(&mut self, held: bool) { self.ctrl_held = held; }
    pub(super) fn move_cursor(&mut self, point: ScreenPoint) { self.modal.move_cursor(point); }
    pub(super) fn release_modal_mouse(&mut self) { self.modal.release_mouse(); }
    pub(super) fn handle_text(&mut self, text: &str) { self.modal.handle_text(text); }
    pub(super) fn step_modal(&mut self, dt: f32) { self.modal.step(dt); }

    pub(super) fn register_trigger(&mut self, trigger: InputTrigger, pressed: bool) {
        self.controls.register_trigger(trigger, pressed);
    }

    pub(super) fn toggle_gameplay_modal(
        &mut self,
        trigger: InputTrigger,
        surface: &dyn PlatformSurface,
        avatar: &mut dyn Avatar,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        let opened = self.modal.toggle_gameplay_trigger(trigger, surface, avatar, world);
        if opened && self.modal.open() {
            self.controls.clear_triggers();
        }
        opened
    }

    pub(super) fn sync_world_chart(&mut self, avatar: &dyn Avatar, world: &dyn PlanetaryWorld) {
        self.modal.sync_world_chart(avatar, world);
    }

    pub(super) fn sync_portal_chat(&mut self, world: &mut dyn PlanetaryWorld) {
        self.modal.sync_portal_chat(world);
    }

    #[cfg(test)]
    pub(super) fn begin_bound_dial(&mut self, link: sanctuary::RemoteLink) -> String {
        let (dial, reply_code) = PortalDial::from_link(link);
        self.dial = dial;
        self.modal.announce_punch_code(reply_code.clone());
        reply_code
    }

    pub(super) fn announce_portal(&mut self, invite: crate::menu::PortalInvite) {
        self.modal.announce_portal(invite);
    }

    pub(super) fn complete_join(&mut self, surface: &dyn PlatformSurface) {
        self.modal.complete_join(surface);
    }

    pub(super) fn handle_trigger(&mut self, trigger: InputTrigger, avatar: &mut dyn Avatar) {
        if self.modal.handle_trigger(trigger, avatar) {
            self.modal.sync_controls(self.controls.as_mut());
        }
    }

    fn translate_directive(&mut self, directive: ModalDirective) -> OverlayTransition {
        match directive {
            ModalDirective::ContinueSession => OverlayTransition::Unchanged,
            ModalDirective::ActivateWorld(spec) => {
                self.dial.cancel();
                OverlayTransition::ActivateWorld(spec)
            }
            ModalDirective::EvictWorld(slug) => {
                self.dial.cancel();
                OverlayTransition::EvictWorld(slug)
            }
            ModalDirective::OpenPortal => {
                self.dial.cancel();
                OverlayTransition::OpenLivePortal
            }
            ModalDirective::JoinPortal(target) => {
                let (dial, outcome) = PortalDial::begin(target);
                self.dial = dial;
                match outcome {
                    DialOutcome::Connected(remote) => OverlayTransition::Connected(remote),
                    DialOutcome::Punching(reply_code) => {
                        self.modal.announce_punch_code(reply_code);
                        OverlayTransition::Unchanged
                    }
                    DialOutcome::Failed => {
                        self.modal.report_join_failure();
                        OverlayTransition::Unchanged
                    }
                }
            }
            ModalDirective::CancelJoin => {
                self.dial.cancel();
                OverlayTransition::Unchanged
            }
            ModalDirective::PunchGuest(peer) => OverlayTransition::PunchPeer(peer),
            ModalDirective::ExitGame => OverlayTransition::ExitSession,
        }
    }

    pub(super) fn press_mouse(&mut self, display: &SessionDisplay, avatar: &mut dyn Avatar) -> OverlayTransition {
        let directive = self.modal.press_mouse(display.surface(), avatar);
        self.modal.sync_controls(self.controls.as_mut());
        self.translate_directive(directive)
    }

    pub(super) fn handle_key(
        &mut self,
        code: winit::keyboard::KeyCode,
        display: &SessionDisplay,
        avatar: &mut dyn Avatar,
    ) -> OverlayTransition {
        let directive = self.modal.handle_key(code, self.ctrl_held, avatar);
        if !self.modal.open() {
            display.capture_cursor();
        }
        self.modal.sync_controls(self.controls.as_mut());
        self.translate_directive(directive)
    }

    pub(super) fn focus_regained(&mut self) -> Option<std::net::SocketAddr> {
        match self.modal.focus_regained() {
            ModalDirective::PunchGuest(peer) => Some(peer),
            _ => None,
        }
    }

    pub(super) fn toggle_escape(&mut self, surface: &dyn PlatformSurface) -> Option<std::net::SocketAddr> {
        let directive = self.modal.toggle_escape(surface);
        if self.modal.open() {
            self.controls.clear_triggers();
        }
        match directive {
            ModalDirective::PunchGuest(peer) => Some(peer),
            _ => None,
        }
    }

    pub(super) fn advance_dial(&mut self, dt: f32, surface: &dyn PlatformSurface) -> Option<Box<dyn PlanetaryWorld>> {
        match self.dial.step(dt) {
            DialStep::Idle | DialStep::Listening => None,
            DialStep::Connected(remote) => Some(remote),
            DialStep::TimedOut => {
                self.modal.report_join_failure();
                surface.release_cursor();
                None
            }
        }
    }
}
