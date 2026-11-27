use world::PlanetaryWorld;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::cosmos_scale::{AssembledCosmos, CosmosScale};
use crate::menu::{ModalDirective, SessionModal};
use crate::portal_config::LivePortal;
use crate::surface::PlatformSurface;
use crate::world_vault::RealmPort;
use super::display::SessionDisplay;
use super::mounted_realm::MountedRealm;
use super::overlay::{OverlayTransition, StageOverlay};
use super::pulse::PulseBeat;

/// Encapsulated planetary world, player avatar, modal/control overlay, and mounted realm (`4` fields).
pub struct SessionStage {
    world: Box<dyn PlanetaryWorld>,
    avatar: Box<dyn Avatar>,
    overlay: StageOverlay,
    realm: MountedRealm,
}

impl SessionStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn from_parts(
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        overlay: StageOverlay,
        realm: MountedRealm,
    ) -> Self {
        Self { world, avatar, overlay, realm }
    }

    /// Secondary constructor assembling the stage with an owned `RealmPort`.
    pub fn with_port(
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        controls: Box<dyn KeyboardControls>,
        modal: Box<dyn SessionModal>,
        port: Box<dyn RealmPort>,
    ) -> Self {
        Self::from_parts(world, avatar, StageOverlay::new(controls, modal), MountedRealm::unbound(port))
    }

    pub(super) fn modal_open(&self) -> bool { self.overlay.modal_open() }
    #[cfg(test)]
    pub(super) fn dialing_active(&self) -> bool { self.overlay.dialing_active() }
    pub(super) fn move_cursor(&mut self, point: crate::menu::ScreenPoint) { self.overlay.move_cursor(point); }
    pub(super) fn release_modal_mouse(&mut self) { self.overlay.release_modal_mouse(); }
    pub(super) fn rotate_avatar(&mut self, dx: f32, dy: f32) { self.avatar.rotate_view(dx, dy); }

    #[cfg(test)]
    pub(super) fn begin_bound_dial(&mut self, link: sanctuary::RemoteLink) -> String {
        self.overlay.begin_bound_dial(link)
    }

    #[cfg(test)]
    pub(super) fn companion_count(&self) -> usize {
        let mut peers = Vec::new();
        self.world.active_companions(&mut peers);
        peers.len()
    }

    pub(super) fn refresh_surface(&mut self, display: &mut SessionDisplay) {
        display.refresh_modified_face(self.world.as_mut());
        display.refresh_meadow_slice(self.world.as_ref(), self.avatar.as_ref());
    }

    pub(super) fn render_onto(&mut self, display: &mut SessionDisplay, sight_scale: f32) {
        display.render_scene(self.world.as_ref(), self.avatar.as_mut(), self.overlay.modal(), sight_scale);
    }

    pub(super) fn use_left(&mut self) -> bool {
        self.avatar.use_left_hand(self.world.as_mut())
    }

    pub(super) fn use_right(&mut self) -> bool {
        self.avatar.use_right_hand(self.world.as_mut())
    }

    pub(super) fn release_hand(&mut self) -> bool { self.avatar.release_hand(self.world.as_mut()) }

    pub(super) fn dispatch_avatar_or_control(
        &mut self,
        trigger: InputTrigger,
        pressed: bool,
        surface: &dyn PlatformSurface,
    ) -> bool {
        if pressed
            && !self.typing_avatar_text()
            && self.overlay.toggle_gameplay_modal(trigger, surface, self.avatar.as_mut(), self.world.as_ref())
        {
            self.overlay.register_trigger(trigger, false);
            return false;
        }
        if pressed
            && (self.avatar.adjust_aimed_rig(trigger, self.world.as_mut())
                || self.avatar.cast_triggered_relic(trigger, self.world.as_mut()))
        {
            self.overlay.register_trigger(trigger, false);
            return true;
        }
        if !pressed || !self.avatar.dispatch_trigger(trigger) {
            self.overlay.register_trigger(trigger, pressed);
        } else {
            self.avatar.flush_world_notes(self.world.as_mut());
            self.overlay.register_trigger(trigger, false);
        }
        false
    }

    pub(super) fn update_ctrl(&mut self, held: bool) { self.overlay.update_ctrl(held); }
    pub(super) fn ctrl_active(&self) -> bool { self.overlay.ctrl_active() }

    fn activate_remote_world(&mut self, remote: Box<dyn PlanetaryWorld>, display: &mut SessionDisplay) {
        self.realm.unbind();
        self.world = remote;
        self.avatar.land_on_world(self.world.as_ref());
        self.overlay.complete_join(display.surface());
        display.refresh_world_mesh(self.world.as_ref(), self.avatar.as_ref());
    }

    fn apply_transition(&mut self, transition: OverlayTransition, display: &mut SessionDisplay) -> ModalDirective {
        match transition {
            OverlayTransition::Unchanged => ModalDirective::ContinueSession,
            OverlayTransition::ActivateWorld(spec) => {
                self.avatar.sync_world_presence(self.world.as_mut());
                self.world = Box::new(world::BiomedQuadSphere::barren(4));
                self.world = self.realm.mount_solitary(spec);
                self.avatar.land_on_world(self.world.as_ref());
                display.capture_cursor();
                display.refresh_world_mesh(self.world.as_ref(), self.avatar.as_ref());
                ModalDirective::ContinueSession
            }
            OverlayTransition::EvictWorld(slug) => {
                self.world = Box::new(world::BiomedQuadSphere::barren(32));
                self.realm.evict(&slug);
                display.surface().release_cursor();
                display.refresh_world_mesh(self.world.as_ref(), self.avatar.as_ref());
                ModalDirective::ContinueSession
            }
            OverlayTransition::OpenLivePortal => {
                self.avatar.sync_world_presence(self.world.as_mut());
                let current = std::mem::replace(&mut self.world, Box::new(world::BiomedQuadSphere::of_size(4)));
                if let Some((hosted, invite)) = self.realm.mount_hosted_if_local() {
                    drop(current);
                    self.world = hosted;
                    self.overlay.announce_portal(invite);
                } else {
                    let proportions = AssembledCosmos::assembled().proportions();
                    let (hosted, invite) = LivePortal::new(proportions).open(current);
                    self.world = hosted;
                    self.overlay.announce_portal(invite);
                }
                ModalDirective::ContinueSession
            }
            OverlayTransition::Connected(remote) => {
                self.activate_remote_world(remote, display);
                ModalDirective::ContinueSession
            }
            OverlayTransition::PunchPeer(peer) => {
                let _ = self.world.punch_nat_peer(peer);
                ModalDirective::ContinueSession
            }
            OverlayTransition::ExitSession => ModalDirective::ExitGame,
        }
    }

    pub(super) fn focus_regained(&mut self) {
        if let Some(peer) = self.overlay.focus_regained() {
            let _ = self.world.punch_nat_peer(peer);
        }
    }

    pub(super) fn press_modal_mouse(&mut self, display: &mut SessionDisplay) -> ModalDirective {
        let transition = self.overlay.press_mouse(display, self.avatar.as_mut());
        self.apply_transition(transition, display)
    }

    pub(super) fn handle_modal_trigger(&mut self, trigger: InputTrigger) {
        self.overlay.handle_trigger(trigger, self.avatar.as_mut());
    }

    pub(super) fn handle_modal_key(&mut self, code: winit::keyboard::KeyCode, display: &mut SessionDisplay) -> ModalDirective {
        let transition = self.overlay.handle_key(code, display, self.avatar.as_mut());
        self.apply_transition(transition, display)
    }

    pub(super) fn handle_modal_text(&mut self, text: &str) { self.overlay.handle_text(text); }
    pub(super) fn typing_avatar_text(&self) -> bool { self.avatar.typing_text() }
    pub(super) fn handle_avatar_text(&mut self, text: &str) -> bool { self.avatar.accept_typed_text(text) }
    pub(super) fn release_control_key(&mut self, code: winit::keyboard::KeyCode) {
        self.overlay.register_trigger(InputTrigger::Key(code), false);
    }

    pub(super) fn toggle_escape(&mut self, surface: &dyn PlatformSurface) {
        if self.overlay.modal_open() || !self.avatar.dispatch_trigger(InputTrigger::Key(winit::keyboard::KeyCode::Escape)) {
            if let Some(peer) = self.overlay.toggle_escape(surface) {
                let _ = self.world.punch_nat_peer(peer);
            }
            if self.overlay.modal_open() {
                let _ = self.avatar.release_hand(self.world.as_mut());
            }
        }
    }

    pub(super) fn step_frame(&mut self, beat: PulseBeat, display: &mut SessionDisplay) {
        if self.overlay.modal_open() {
            let dt = beat.modal_seconds();
            if dt > 0.0 {
                self.overlay.step_modal(dt);
                if let Some(remote) = self.overlay.advance_dial(dt, display.surface()) {
                    self.activate_remote_world(remote, display);
                    return;
                }
                self.avatar.sync_world_presence(self.world.as_mut());
                if self.world.pump_portal(dt) {
                    self.refresh_surface(display);
                }
                self.overlay.sync_portal_chat(self.world.as_mut());
                self.overlay.sync_world_chart(self.avatar.as_ref(), self.world.as_ref());
            }
            return;
        }
        let ticks = beat.due_ticks();
        let mut reopen_meadow = false;
        if ticks > 0 {
            let prev_sector = self.avatar.sector_stamp();
            let step = beat.tick_seconds();
            let total_dt = step * (ticks as f32);
            self.overlay.step_modal(total_dt);
            if let Some(remote) = self.overlay.advance_dial(total_dt, display.surface()) {
                self.activate_remote_world(remote, display);
                return;
            }
            let mut changed = false;
            for _ in 0..ticks {
                self.avatar.step_motion(step, self.overlay.controls(), self.world.as_ref());
                let dug = self.avatar.step_digging(self.world.as_mut());
                let stepped = self.world.step_surface(step);
                let handed_off = self.avatar.reconcile_cosmic_frame(self.world.as_mut());
                if dug || stepped || handed_off {
                    changed = true;
                }
            }
            let _ = self.avatar.visible_slices(self.world.as_ref());
            let dirty_slices = self.world.pending_slices();
            if changed || dirty_slices {
                display.refresh_modified_face(self.world.as_mut());
            }
            reopen_meadow = changed || dirty_slices || self.avatar.sector_stamp() != prev_sector;
        }
        self.avatar.blend_subtick(beat.subtick_blend());
        self.avatar.sync_world_presence(self.world.as_mut());
        self.overlay.sync_portal_chat(self.world.as_mut());
        display.stream_meadow_slice(self.world.as_ref(), self.avatar.as_ref(), reopen_meadow);
        self.world.blend_companions(beat.subtick_blend() * beat.tick_seconds());
    }
}

impl Drop for SessionStage {
    fn drop(&mut self) {
        self.avatar.sync_world_presence(self.world.as_mut());
    }
}
