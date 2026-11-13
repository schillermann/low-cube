use world::PlanetaryWorld;
use crate::attire::{PhotoShutter, SightMode};
use crate::avatar::Avatar;
use crate::companion::CompanionFigure;
use crate::controls::{ControlAction, InputTrigger, KeyboardControls};
use crate::menu::{ModalDirective, SessionModal};
use crate::portal_config::PortalConfig;
use crate::server_scale::{PlanetaryScale, ServerScale};
use crate::sparrow::SparrowFlock;
use crate::surface::PlatformSurface;
use super::display::SessionDisplay;

/// Encapsulated keyboard control state and pause/studio modal overlay.
pub(super) struct StageOverlay {
    controls: Box<dyn KeyboardControls>,
    modal: Box<dyn SessionModal>,
}

impl StageOverlay {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(controls: Box<dyn KeyboardControls>, modal: Box<dyn SessionModal>) -> Self {
        Self { controls, modal }
    }
}

/// Encapsulated planetary world, player avatar, woodland sparrow flock, and modal/control overlay.
pub(super) struct SessionStage {
    world: Box<dyn PlanetaryWorld>,
    avatar: Box<dyn Avatar>,
    sparrows: SparrowFlock,
    overlay: StageOverlay,
}

impl SessionStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn with_flock(
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        sparrows: SparrowFlock,
        overlay: StageOverlay,
    ) -> Self {
        Self { world, avatar, sparrows, overlay }
    }

    /// Secondary constructor initializing a seeded woodland sparrow flock alongside the stage.
    pub(super) fn new(
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        controls: Box<dyn KeyboardControls>,
        modal: Box<dyn SessionModal>,
    ) -> Self {
        Self::with_flock(world, avatar, SparrowFlock::seeded(), StageOverlay::new(controls, modal))
    }

    pub(super) fn world(&self) -> &dyn PlanetaryWorld { self.world.as_ref() }
    pub(super) fn world_mut(&mut self) -> &mut dyn PlanetaryWorld { self.world.as_mut() }
    pub(super) fn avatar(&self) -> &dyn Avatar { self.avatar.as_ref() }
    pub(super) fn modal(&self) -> &dyn SessionModal { self.overlay.modal.as_ref() }
    pub(super) fn modal_open(&self) -> bool { self.overlay.modal.open() }

    pub(super) fn emit_avatar(&self, target: &mut dyn world::MeshTarget) {
        self.avatar.emit_appearance(target);
        CompanionFigure::emit_all(self.world.as_ref(), target);
        self.sparrows.emit(target);
    }

    pub(super) fn emit_avatar_for_sight(&self, sight: SightMode, target: &mut dyn world::MeshTarget) {
        self.avatar.emit_appearance_for_sight(sight, target);
        CompanionFigure::emit_all(self.world.as_ref(), target);
        self.sparrows.emit(target);
    }

    pub(super) fn take_photo_shutter(&mut self) -> PhotoShutter { self.avatar.take_photo_shutter() }
    pub(super) fn apply_polaroid_mosaic(&mut self, mosaic: Vec<glam::Vec3>) { self.avatar.apply_polaroid_mosaic(mosaic); }
    pub(super) fn move_cursor(&mut self, point: crate::menu::ScreenPoint) { self.overlay.modal.move_cursor(point); }
    pub(super) fn release_modal_mouse(&mut self) { self.overlay.modal.release_mouse(); }
    pub(super) fn rotate_avatar(&mut self, dx: f32, dy: f32) { self.avatar.rotate_view(dx, dy); }
    pub(super) fn use_left(&mut self) -> bool { self.avatar.use_left_hand(self.world.as_mut()) }
    pub(super) fn use_right(&mut self, ) -> bool { self.avatar.use_right_hand(self.world.as_mut()) }
    pub(super) fn release_hand(&mut self) -> bool { self.avatar.release_hand(self.world.as_mut()) }
    pub(super) fn matches_tron(&self, trigger: InputTrigger) -> bool {
        self.overlay.controls.matches_action(ControlAction::TronMode, trigger)
    }

    pub(super) fn dispatch_avatar_or_control(&mut self, trigger: InputTrigger, pressed: bool) -> bool {
        if pressed && self.avatar.adjust_aimed_rig(trigger, self.world.as_mut()) {
            return true;
        }
        if !pressed || !self.avatar.dispatch_trigger(trigger) {
            self.overlay.controls.register_trigger(trigger, pressed);
        } else {
            self.overlay.controls.register_trigger(trigger, false);
        }
        false
    }

    pub(super) fn press_modal_mouse(&mut self, surface: &dyn PlatformSurface, tron_active: bool) -> ModalDirective {
        let directive = self.overlay.modal.press_mouse(surface, self.avatar.as_mut());
        self.overlay.modal.sync_controls(self.overlay.controls.as_mut());
        if matches!(directive, ModalDirective::OpenPortal) {
            let current = std::mem::replace(&mut self.world, Box::new(world::BiomedCubeWorld::of_size(4)));
            let blocks = ServerScale::assembled().blocks();
            let (hosted, ticket) = PortalConfig::open_live_portal(current, blocks, tron_active);
            self.world = hosted;
            self.overlay.modal.announce_portal(ticket);
            return ModalDirective::ContinueSession;
        }
        directive
    }

    pub(super) fn handle_modal_trigger(&mut self, trigger: InputTrigger) {
        if self.overlay.modal.handle_trigger(trigger, self.avatar.as_mut()) {
            self.overlay.modal.sync_controls(self.overlay.controls.as_mut());
        }
    }

    pub(super) fn handle_modal_key(&mut self, code: winit::keyboard::KeyCode) {
        if self.overlay.modal.handle_key(code, self.avatar.as_mut()) {
            self.overlay.modal.sync_controls(self.overlay.controls.as_mut());
        }
    }

    pub(super) fn toggle_escape(&mut self, surface: &dyn PlatformSurface) {
        if self.overlay.modal.open() || !self.avatar.dispatch_trigger(InputTrigger::Key(winit::keyboard::KeyCode::Escape)) {
            self.overlay.modal.toggle_escape(surface);
        }
    }

    pub(super) fn step_frame(&mut self, dt: f32, display: &mut SessionDisplay) {
        if self.overlay.modal.open() {
            self.overlay.modal.step(dt);
            self.avatar.sync_world_presence(self.world.as_mut());
            if self.world.pump_portal(dt) {
                display.refresh_modified_face(self.world.as_mut());
                display.refresh_meadow_slice(self.world.as_ref(), self.avatar.as_ref());
            }
        } else {
            let prev_anchor = self.avatar.meadow_anchor();
            let total_dt = dt.clamp(1.0 / 240.0, 0.10);
            let steps = (total_dt * 45.0).ceil().max(1.0) as u32;
            let step = total_dt / (steps as f32);
            let mut changed = false;
            for _ in 0..steps {
                self.avatar.step_motion(step, self.overlay.controls.as_ref(), self.world.as_ref());
                self.avatar.step_fauna(step, &mut self.sparrows, self.world.as_ref());
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
