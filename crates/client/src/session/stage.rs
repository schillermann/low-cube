use fauna::WoodlandFauna;
use world::PlanetaryWorld;
use crate::avatar::Avatar;
use crate::controls::{ControlAction, InputTrigger, KeyboardControls};
use crate::menu::{ModalDirective, SessionModal};
use crate::portal_config::{DialOutcome, PortalConfig, PortalDial};
use crate::server_scale::{PlanetaryScale, ServerScale};
use crate::surface::PlatformSurface;
use super::display::SessionDisplay;

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
}

/// Encapsulated planetary world, player avatar, woodland fauna, and modal/control overlay.
pub(super) struct SessionStage {
    world: Box<dyn PlanetaryWorld>,
    avatar: Box<dyn Avatar>,
    fauna: WoodlandFauna,
    overlay: StageOverlay,
}

impl SessionStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn with_fauna(
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        fauna: WoodlandFauna,
        overlay: StageOverlay,
    ) -> Self {
        Self { world, avatar, fauna, overlay }
    }

    /// Secondary constructor initializing seeded woodland wildlife alongside the stage.
    pub(super) fn new(
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        controls: Box<dyn KeyboardControls>,
        modal: Box<dyn SessionModal>,
    ) -> Self {
        Self::with_fauna(world, avatar, WoodlandFauna::seeded(), StageOverlay::new(controls, modal))
    }

    pub(super) fn modal_open(&self) -> bool { self.overlay.modal.open() }
    pub(super) fn move_cursor(&mut self, point: crate::menu::ScreenPoint) { self.overlay.modal.move_cursor(point); }
    pub(super) fn release_modal_mouse(&mut self) { self.overlay.modal.release_mouse(); }
    pub(super) fn rotate_avatar(&mut self, dx: f32, dy: f32) { self.avatar.rotate_view(dx, dy); }

    pub(super) fn refresh_surface(&mut self, display: &mut SessionDisplay) {
        display.refresh_modified_face(self.world.as_mut());
        display.refresh_meadow_slice(self.world.as_ref(), self.avatar.as_ref());
    }

    pub(super) fn toggle_tron(&self, display: &mut SessionDisplay) {
        display.toggle_tron(self.world.as_ref(), self.avatar.as_ref());
    }

    pub(super) fn render_onto(&mut self, display: &SessionDisplay) {
        display.render_scene(self.world.as_ref(), self.avatar.as_mut(), &self.fauna, self.overlay.modal.as_ref());
    }

    pub(super) fn use_left(&mut self) -> bool {
        let (struck, dropped) = self.avatar.hunt_fauna(-1.0, &mut self.fauna, self.world.as_mut());
        if struck { dropped } else { self.avatar.use_left_hand(self.world.as_mut()) }
    }

    pub(super) fn use_right(&mut self) -> bool {
        let (struck, dropped) = self.avatar.hunt_fauna(1.0, &mut self.fauna, self.world.as_mut());
        if struck { dropped } else { self.avatar.use_right_hand(self.world.as_mut()) }
    }

    pub(super) fn release_hand(&mut self) -> bool { self.avatar.release_hand(self.world.as_mut()) }

    pub(super) fn matches_tron(&self, trigger: InputTrigger) -> bool {
        self.overlay.controls.matches_action(ControlAction::TronMode, trigger)
    }

    pub(super) fn dispatch_avatar_or_control(&mut self, trigger: InputTrigger, pressed: bool) -> bool {
        if pressed
            && (self.avatar.adjust_aimed_rig(trigger, self.world.as_mut())
                || self.avatar.cast_triggered_relic(trigger, self.world.as_mut()))
        {
            self.overlay.controls.register_trigger(trigger, false);
            return true;
        }
        if !pressed || !self.avatar.dispatch_trigger(trigger) {
            self.overlay.controls.register_trigger(trigger, pressed);
        } else {
            self.overlay.controls.register_trigger(trigger, false);
        }
        false
    }

    pub(super) fn update_ctrl(&mut self, held: bool) {
        self.overlay.ctrl_held = held;
    }

    pub(super) fn ctrl_active(&self) -> bool {
        self.overlay.ctrl_held
    }

    fn activate_remote_world(&mut self, remote: Box<dyn PlanetaryWorld>, display: &mut SessionDisplay) {
        self.world = remote;
        self.avatar.land_on_world(self.world.as_ref());
        self.overlay.modal.complete_join(display.surface());
        display.refresh_world_mesh(self.world.as_ref(), self.avatar.as_ref());
    }

    fn resolve_modal_directive(&mut self, directive: ModalDirective, display: &mut SessionDisplay) -> ModalDirective {
        match directive {
            ModalDirective::OpenPortal => {
                let current = std::mem::replace(&mut self.world, Box::new(world::BiomedCubeWorld::of_size(4)));
                let blocks = ServerScale::assembled().blocks();
                let (hosted, invite) = PortalConfig::open_live_portal(current, blocks, display.tron_enabled());
                self.world = hosted;
                self.overlay.modal.announce_portal(invite);
                ModalDirective::ContinueSession
            }
            ModalDirective::JoinPortal(target) => {
                let (dial, outcome) = PortalDial::begin(target);
                self.overlay.dial = dial;
                match outcome {
                    DialOutcome::Connected(remote) => self.activate_remote_world(remote, display),
                    DialOutcome::Punching(reply_code) => self.overlay.modal.announce_punch_code(reply_code),
                    DialOutcome::Failed => self.overlay.modal.report_join_failure(),
                }
                ModalDirective::ContinueSession
            }
            ModalDirective::PunchGuest(peer) => {
                let _ = self.world.punch_nat_peer(peer);
                ModalDirective::ContinueSession
            }
            other => other,
        }
    }

    pub(super) fn press_modal_mouse(&mut self, display: &mut SessionDisplay) -> ModalDirective {
        let directive = self.overlay.modal.press_mouse(display.surface(), self.avatar.as_mut());
        self.overlay.modal.sync_controls(self.overlay.controls.as_mut());
        self.resolve_modal_directive(directive, display)
    }

    pub(super) fn handle_modal_trigger(&mut self, trigger: InputTrigger) {
        if self.overlay.modal.handle_trigger(trigger, self.avatar.as_mut()) {
            self.overlay.modal.sync_controls(self.overlay.controls.as_mut());
        }
    }

    pub(super) fn handle_modal_key(&mut self, code: winit::keyboard::KeyCode, display: &mut SessionDisplay) -> ModalDirective {
        let directive = self.overlay.modal.handle_key(code, self.overlay.ctrl_held, self.avatar.as_mut());
        self.overlay.modal.sync_controls(self.overlay.controls.as_mut());
        self.resolve_modal_directive(directive, display)
    }

    pub(super) fn handle_modal_text(&mut self, text: &str) {
        self.overlay.modal.handle_text(text);
    }

    pub(super) fn toggle_escape(&mut self, surface: &dyn PlatformSurface) {
        if self.overlay.modal.open() || !self.avatar.dispatch_trigger(InputTrigger::Key(winit::keyboard::KeyCode::Escape)) {
            self.overlay.modal.toggle_escape(surface);
            if !self.overlay.modal.dialing_active() {
                self.overlay.dial.cancel();
            }
        }
    }

    pub(super) fn step_frame(&mut self, dt: f32, display: &mut SessionDisplay) {
        if self.overlay.modal.open() {
            self.overlay.modal.step(dt);
            if !self.overlay.modal.dialing_active() {
                self.overlay.dial.cancel();
            } else if let Some(remote) = self.overlay.dial.step(dt) {
                self.activate_remote_world(remote, display);
                return;
            }
            self.avatar.sync_world_presence(self.world.as_mut());
            if self.world.pump_portal(dt) {
                self.refresh_surface(display);
            }
        } else {
            let prev_anchor = self.avatar.meadow_anchor();
            let total_dt = dt.clamp(1.0 / 240.0, 0.10);
            let steps = (total_dt * 45.0).ceil().max(1.0) as u32;
            let step = total_dt / (steps as f32);
            let mut changed = false;
            for _ in 0..steps {
                self.avatar.step_motion(step, self.overlay.controls.as_ref(), self.world.as_ref());
                self.avatar.step_fauna(step, &mut self.fauna, self.world.as_ref());
                if self.avatar.step_digging(self.world.as_mut()) || self.world.step_surface(step) {
                    changed = true;
                }
            }
            self.avatar.sync_world_presence(self.world.as_mut());
            if changed {
                display.refresh_modified_face(self.world.as_mut());
            }
            if changed || self.avatar.meadow_anchor() != prev_anchor {
                display.refresh_meadow_slice(self.world.as_ref(), self.avatar.as_ref());
            }
        }
    }
}
