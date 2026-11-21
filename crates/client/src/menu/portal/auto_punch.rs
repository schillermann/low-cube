use std::net::SocketAddr;
use winit::keyboard::KeyCode;
use engine::ShaderProgram;
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::menu::clipboard::TextClipboard;
use crate::menu::widget::ScreenPoint;
use crate::menu::{ModalDirective, SessionModal};
use crate::surface::PlatformSurface;
use super::invite::PortalInvite;

/// Decorator over any `SessionModal` auto-punching NAT when the host copies a guest's signed reply code.
pub struct AutoPunchModal<M> {
    origin: M,
    invite: PortalInvite,
    clipboard: Box<dyn TextClipboard>,
    last_sensed: String,
}

impl<M> AutoPunchModal<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: M,
        invite: PortalInvite,
        clipboard: Box<dyn TextClipboard>,
        last_sensed: String,
    ) -> Self {
        Self { origin, invite, clipboard, last_sensed }
    }

    pub fn wrapping(origin: M, invite: PortalInvite, clipboard: Box<dyn TextClipboard>) -> Self {
        Self::new(origin, invite, clipboard, String::new())
    }
}

impl<M: SessionModal> AutoPunchModal<M> {
    fn sense_clipboard_punch(&mut self) -> Option<SocketAddr> {
        if !self.invite.active() {
            return None;
        }
        let raw = self.clipboard.read_text();
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == self.last_sensed {
            return None;
        }
        self.last_sensed = trimmed.to_string();
        let peer = self.invite.verify_guest_punch(trimmed)?;
        self.origin.announce_auto_punch();
        println!("=== AUTO-DETECTED GUEST REPLY CODE - PUNCHING NAT PEER {peer} ===");
        Some(peer)
    }
}

impl<M: SessionModal> SessionModal for AutoPunchModal<M> {
    fn open(&self) -> bool {
        self.origin.open()
    }

    #[cfg(test)]
    fn dialing_active(&self) -> bool {
        self.origin.dialing_active()
    }

    fn toggle_escape(&mut self, surface: &dyn PlatformSurface) -> ModalDirective {
        let directive = self.origin.toggle_escape(surface);
        if matches!(directive, ModalDirective::ContinueSession) {
            if let Some(peer) = self.sense_clipboard_punch() {
                return ModalDirective::PunchGuest(peer);
            }
        }
        directive
    }

    fn focus_regained(&mut self) -> ModalDirective {
        if let Some(peer) = self.sense_clipboard_punch() {
            return ModalDirective::PunchGuest(peer);
        }
        self.origin.focus_regained()
    }

    fn move_cursor(&mut self, point: ScreenPoint) {
        self.origin.move_cursor(point);
    }

    fn press_mouse(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> ModalDirective {
        self.origin.press_mouse(surface, avatar)
    }

    fn release_mouse(&mut self) {
        self.origin.release_mouse();
    }

    fn handle_key(&mut self, code: KeyCode, ctrl_held: bool, avatar: &mut dyn Avatar) -> ModalDirective {
        self.origin.handle_key(code, ctrl_held, avatar)
    }

    fn handle_text(&mut self, text: &str) {
        self.origin.handle_text(text);
    }

    fn handle_trigger(&mut self, trigger: InputTrigger, avatar: &mut dyn Avatar) -> bool {
        self.origin.handle_trigger(trigger, avatar)
    }

    fn sync_controls(&self, controls: &mut dyn KeyboardControls) {
        self.origin.sync_controls(controls);
    }

    fn announce_portal(&mut self, invite: PortalInvite) {
        self.invite = invite.clone();
        self.last_sensed.clear();
        self.origin.announce_portal(invite);
    }

    fn announce_auto_punch(&mut self) {
        self.origin.announce_auto_punch();
    }

    fn announce_punch_code(&mut self, punch_code: String) {
        self.origin.announce_punch_code(punch_code);
    }

    fn complete_join(&mut self, surface: &dyn PlatformSurface) {
        self.origin.complete_join(surface);
    }

    fn report_join_failure(&mut self) {
        self.origin.report_join_failure();
    }

    fn step(&mut self, dt: f32) {
        self.origin.step(dt);
    }

    #[cfg(test)]
    fn emit_ui(&self, avatar: &dyn Avatar, target: &mut dyn MeshTarget) {
        self.origin.emit_ui(avatar, target);
    }

    fn emit_studio(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_studio(target);
    }

    fn draw_layer(&self, gl: &glow::Context, shader: &dyn ShaderProgram, avatar: &dyn Avatar, aspect: f32) {
        self.origin.draw_layer(gl, shader, avatar, aspect);
    }
}
