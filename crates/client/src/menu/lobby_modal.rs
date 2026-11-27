use winit::keyboard::KeyCode;
use engine::FramePass;
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::surface::PlatformSurface;
use super::{ModalDirective, PortalInvite, ScreenPoint, SessionModal};

/// Polymorphic habitation gate keeping the startup menu open until a world is created, loaded, or joined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LobbyGate {
    Guarding,
    Habited,
}

/// `SessionModal` decorator preventing premature exit into uncreated gameplay before world selection (`2` fields).
pub struct LobbyGateModal<M> {
    origin: M,
    gate: LobbyGate,
}

impl<M> LobbyGateModal<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(origin: M, gate: LobbyGate) -> Self {
        Self { origin, gate }
    }

    pub const fn guarding(origin: M) -> Self {
        Self::new(origin, LobbyGate::Guarding)
    }
}

impl<M: SessionModal> LobbyGateModal<M> {
    fn reconcile(&mut self, directive: ModalDirective, surface: &dyn PlatformSurface) -> ModalDirective {
        match directive {
            ModalDirective::ActivateWorld(spec) => {
                self.gate = LobbyGate::Habited;
                ModalDirective::ActivateWorld(spec)
            }
            ModalDirective::EvictWorld(slug) => {
                self.gate = LobbyGate::Guarding;
                if !self.origin.open() {
                    let _ = self.origin.toggle_escape(surface);
                }
                surface.release_cursor();
                ModalDirective::EvictWorld(slug)
            }
            ModalDirective::OpenPortal if matches!(self.gate, LobbyGate::Guarding) => {
                ModalDirective::ContinueSession
            }
            other => {
                if matches!(self.gate, LobbyGate::Guarding) && !self.origin.open() {
                    let _ = self.origin.toggle_escape(surface);
                    surface.release_cursor();
                }
                other
            }
        }
    }
}

impl<M: SessionModal> SessionModal for LobbyGateModal<M> {
    fn open(&self) -> bool {
        matches!(self.gate, LobbyGate::Guarding) || self.origin.open()
    }

    #[cfg(test)]
    fn dialing_active(&self) -> bool {
        self.origin.dialing_active()
    }

    fn toggle_gameplay_trigger(
        &mut self,
        trigger: InputTrigger,
        surface: &dyn PlatformSurface,
        avatar: &mut dyn Avatar,
        world: &dyn world::PlanetaryWorld,
    ) -> bool {
        !matches!(self.gate, LobbyGate::Guarding)
            && self.origin.toggle_gameplay_trigger(trigger, surface, avatar, world)
    }

    fn sync_world_chart(&mut self, avatar: &dyn Avatar, world: &dyn world::PlanetaryWorld) {
        self.origin.sync_world_chart(avatar, world);
    }

    fn sync_portal_chat(&mut self, world: &mut dyn world::PlanetaryWorld) {
        self.origin.sync_portal_chat(world);
    }

    fn toggle_escape(&mut self, surface: &dyn PlatformSurface) -> ModalDirective {
        let dir = self.origin.toggle_escape(surface);
        self.reconcile(dir, surface)
    }

    fn focus_regained(&mut self) -> ModalDirective {
        self.origin.focus_regained()
    }

    fn move_cursor(&mut self, point: ScreenPoint) {
        self.origin.move_cursor(point);
    }

    fn press_mouse(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> ModalDirective {
        let dir = self.origin.press_mouse(surface, avatar);
        self.reconcile(dir, surface)
    }

    fn release_mouse(&mut self) {
        self.origin.release_mouse();
    }

    fn handle_key(&mut self, code: KeyCode, ctrl_held: bool, avatar: &mut dyn Avatar) -> ModalDirective {
        let dir = self.origin.handle_key(code, ctrl_held, avatar);
        match dir {
            ModalDirective::ActivateWorld(spec) => {
                self.gate = LobbyGate::Habited;
                ModalDirective::ActivateWorld(spec)
            }
            ModalDirective::EvictWorld(slug) => {
                self.gate = LobbyGate::Guarding;
                ModalDirective::EvictWorld(slug)
            }
            ModalDirective::OpenPortal if matches!(self.gate, LobbyGate::Guarding) => {
                ModalDirective::ContinueSession
            }
            other => other,
        }
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
        self.origin.announce_portal(invite);
    }

    fn announce_auto_punch(&mut self) {
        self.origin.announce_auto_punch();
    }

    fn announce_punch_code(&mut self, punch_code: String) {
        self.origin.announce_punch_code(punch_code);
    }

    fn complete_join(&mut self, surface: &dyn PlatformSurface) {
        self.gate = LobbyGate::Habited;
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

    fn draw_layer(&self, pass: &mut dyn FramePass, avatar: &dyn Avatar, aspect: f32) {
        self.origin.draw_layer(pass, avatar, aspect);
    }
}
