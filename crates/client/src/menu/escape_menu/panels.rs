use glam::{Mat4, Vec3};
use glow::HasContext;
use winit::keyboard::KeyCode;
use engine::{CpuMeshBatch, Mesh, ShaderProgram};
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::menu::clipboard::TextClipboard;
use crate::menu::pause_panel::{PausePanel, PauseSelection};
use crate::menu::portal::{BeaconHud, PortalInvite, PortalStudio, ReticleHud};
use crate::menu::view::{MenuView, VersionedView};
use crate::menu::widget::ScreenPoint;
use crate::menu::worlds::{SavedWorldStudio, WorldDirectiveSink, WorldPanel};
use crate::menu::ModalDirective;
use crate::surface::PlatformSurface;
use crate::world_vault::{DiskWorldVault, WorldSlug, WorldSpec};
use super::personal::PersonalPanels;
use super::phase::ModalPhase;

/// Tell-Don't-Ask collector converting `WorldPanel` events into a `ModalDirective` (`1` field).
struct DirectiveCollector {
    directive: ModalDirective,
}

impl DirectiveCollector {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(directive: ModalDirective) -> Self {
        Self { directive }
    }

    const fn idle() -> Self {
        Self::new(ModalDirective::ContinueSession)
    }
}

impl WorldDirectiveSink for DirectiveCollector {
    fn create_and_enter(&mut self, draft: WorldSpec) -> WorldSpec {
        self.directive = ModalDirective::ActivateWorld(draft.clone());
        draft
    }

    fn enter_existing(&mut self, spec: WorldSpec) {
        self.directive = ModalDirective::ActivateWorld(spec);
    }

    fn discard_world(&mut self, slug: WorldSlug, active_evicted: bool) {
        if active_evicted {
            self.directive = ModalDirective::EvictWorld(slug);
        }
    }
}

/// Strictly encapsulated collaborator panels owned by `EscapeMenu` (`3` fields).
pub(super) struct StudioPanels {
    worlds: Box<dyn WorldPanel>,
    portal: PortalStudio,
    personal: PersonalPanels,
}

impl StudioPanels {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        worlds: Box<dyn WorldPanel>,
        portal: PortalStudio,
        personal: PersonalPanels,
    ) -> Self {
        Self { worlds, portal, personal }
    }

    pub(super) fn assembled(
        vault: DiskWorldVault,
        invite: PortalInvite,
        portal_clipboard: Box<dyn TextClipboard>,
        report_clipboard: Box<dyn TextClipboard>,
    ) -> Self {
        Self::new(
            Box::new(SavedWorldStudio::live(vault)),
            PortalStudio::with_invite(invite, portal_clipboard),
            PersonalPanels::live(report_clipboard),
        )
    }

    #[cfg(test)]
    pub(super) fn seeded(
        worlds: Box<dyn WorldPanel>,
        portal: PortalStudio,
        black_box: crate::menu::black_box_panel::BlackBoxPanel,
    ) -> Self {
        Self::new(
            worlds,
            portal,
            PersonalPanels::new(
                Box::new(crate::menu::studio::WardrobeStudio::seeded()),
                Box::new(crate::menu::settings::ControlStudio::seeded()),
                black_box,
            ),
        )
    }

    pub(super) fn empty_worlds(&self) -> bool { self.worlds.empty_roster() }
    pub(super) fn dialing_active(&self) -> bool { self.portal.awaiting_host_punch() }
    pub(super) fn open_pause(&mut self) { self.portal.refresh_startup_copy(); }
    pub(super) fn clear_active_world(&mut self) { self.worlds.clear_active_world(); }
    pub(super) fn cancel_join_dial(&mut self) { self.portal.clear_dial_state(); }
    pub(super) fn cancel_rebind_focus(&mut self) { self.personal.cancel_rebind(); }
    pub(super) fn move_pointer(&mut self, phase: ModalPhase, point: ScreenPoint) { self.personal.move_pointer(phase, point); }
    pub(super) fn release_grip(&mut self) { self.personal.release_grip(); }

    fn open_worlds(&mut self) -> ModalPhase {
        self.worlds.sync_roster();
        if self.worlds.empty_roster() {
            self.worlds.prepare_new_draft();
            ModalPhase::WorldCreation
        } else {
            ModalPhase::WorldRoster
        }
    }

    pub(super) fn press_pause(&mut self, pointer: ScreenPoint, surface: &dyn PlatformSurface) -> (ModalPhase, ModalDirective) {
        match PausePanel::new().select(pointer) {
            PauseSelection::Resume => {
                surface.capture_cursor();
                (ModalPhase::Gameplay, ModalDirective::ContinueSession)
            }
            PauseSelection::OpenWorlds => (self.open_worlds(), ModalDirective::ContinueSession),
            PauseSelection::OpenWardrobe => {
                self.personal.focus_wardrobe(pointer);
                (ModalPhase::CharacterWardrobe, ModalDirective::ContinueSession)
            }
            PauseSelection::OpenSettings => (ModalPhase::SettingsHub, ModalDirective::ContinueSession),
            PauseSelection::OpenMultiplayer => (ModalPhase::MultiplayerHub, ModalDirective::ContinueSession),
            PauseSelection::Quit => (ModalPhase::EscapePause, ModalDirective::ExitGame),
            PauseSelection::Unmatched => (ModalPhase::EscapePause, ModalDirective::ContinueSession),
        }
    }

    fn resolve_world_step(
        &mut self,
        phase: ModalPhase,
        dir: crate::menu::worlds::WorldPanelDirective,
        outcome: ModalDirective,
    ) -> (ModalPhase, ModalDirective) {
        if !matches!(outcome, ModalDirective::ContinueSession) {
            self.portal.announce_hosted(PortalInvite::empty());
        }
        phase.apply_worlds(dir, outcome)
    }

    pub(super) fn press_subpanel(
        &mut self,
        phase: ModalPhase,
        pointer: ScreenPoint,
        surface: &dyn PlatformSurface,
        avatar: &mut dyn Avatar,
    ) -> (ModalPhase, ModalDirective) {
        match phase {
            ModalPhase::WorldRoster | ModalPhase::WorldCreation => {
                let mut collector = DirectiveCollector::idle();
                let dir = if matches!(phase, ModalPhase::WorldRoster) {
                    self.worlds.press_roster(pointer, &mut collector)
                } else {
                    self.worlds.press_creation(pointer, &mut collector)
                };
                let (next, out) = self.resolve_world_step(phase, dir, collector.directive);
                if matches!(next, ModalPhase::Gameplay) {
                    surface.capture_cursor();
                }
                (next, out)
            }
            ModalPhase::MultiplayerHub => phase.apply_portal(self.portal.press_hub_pointer(pointer)),
            ModalPhase::PortalJoin => {
                let (next, directive) = phase.apply_portal(self.portal.press_pointer(pointer));
                if matches!(next, ModalPhase::Gameplay) {
                    surface.capture_cursor();
                }
                (next, directive)
            }
            other => (self.personal.press_panel(other, pointer, surface, avatar), ModalDirective::ContinueSession),
        }
    }

    pub(super) fn handle_key(
        &mut self,
        phase: ModalPhase,
        code: KeyCode,
        ctrl_held: bool,
        avatar: &mut dyn Avatar,
    ) -> (ModalPhase, ModalDirective) {
        match phase {
            ModalPhase::CharacterWardrobe | ModalPhase::ControlSettings => {
                self.personal.handle_key(phase, code, avatar);
                (phase, ModalDirective::ContinueSession)
            }
            ModalPhase::WorldCreation => {
                let mut collector = DirectiveCollector::idle();
                let dir = self.worlds.handle_creation_key(code, ctrl_held, self.portal.clipboard(), &mut collector);
                self.resolve_world_step(phase, dir, collector.directive)
            }
            ModalPhase::PortalJoin => phase.apply_portal(self.portal.handle_key(code, ctrl_held)),
            ModalPhase::EscapePause | ModalPhase::MultiplayerHub if ctrl_held && code == KeyCode::KeyC => {
                self.portal.copy_hosted_ticket();
                (phase, ModalDirective::ContinueSession)
            }
            ModalPhase::EscapePause | ModalPhase::MultiplayerHub if ctrl_held && code == KeyCode::KeyV => {
                phase.apply_portal(self.portal.paste_guest_reply())
            }
            _ => (phase, ModalDirective::ContinueSession),
        }
    }

    pub(super) fn append_text(&mut self, phase: ModalPhase, text: &str) {
        match phase {
            ModalPhase::PortalJoin => self.portal.append_text(text),
            ModalPhase::WorldCreation => self.worlds.append_creation_text(text),
            _ => {}
        }
    }

    pub(super) fn handle_trigger(&mut self, trigger: InputTrigger, avatar: &mut dyn Avatar) -> bool {
        self.personal.handle_trigger(trigger, avatar)
    }

    pub(super) fn sync_controls(&self, controls: &mut dyn KeyboardControls) {
        self.personal.sync_controls(controls);
    }

    pub(super) fn announce_portal(&mut self, invite: PortalInvite) { self.portal.announce_hosted(invite); }
    pub(super) fn announce_auto_punch(&mut self) { self.portal.announce_auto_punch(); }
    pub(super) fn announce_punch_code(&mut self, punch_code: String) { self.portal.announce_punch_reply(punch_code); }
    pub(super) fn report_join_failure(&mut self) { self.portal.report_connection_failure(); }

    pub(super) fn step_overlays(&mut self, phase: ModalPhase, dt: f32) {
        self.portal.step(dt);
        if matches!(phase, ModalPhase::CharacterWardrobe) {
            self.personal.step_wardrobe(dt);
        }
    }

    pub(super) fn emit_studio(&self, target: &mut dyn MeshTarget) {
        self.personal.emit_stage(target);
    }

    pub(super) fn emit_ui(&self, phase: ModalPhase, pointer: ScreenPoint, avatar: &dyn Avatar, aspect: f32, target: &mut dyn MeshTarget) {
        match phase {
            ModalPhase::Gameplay => BeaconHud::new(self.portal.beacon(), ReticleHud::new(avatar, aspect)).emit(target),
            ModalPhase::EscapePause => {
                let pause = PausePanel::new().view(pointer, self.portal.hosted_active());
                VersionedView::current(BeaconHud::new(self.portal.beacon(), pause)).emit(target);
            }
            ModalPhase::WorldRoster => self.worlds.emit_roster(pointer, target),
            ModalPhase::WorldCreation => self.worlds.emit_creation(pointer, target),
            ModalPhase::MultiplayerHub => VersionedView::current(self.portal.hub_view(pointer)).emit(target),
            ModalPhase::PortalJoin => self.portal.emit_join_screen(pointer, target),
            other => self.personal.emit_ui(other, pointer, avatar, target),
        }
    }

    pub(super) fn draw_overlay(
        &self,
        phase: ModalPhase,
        gl: &glow::Context,
        shader: &dyn ShaderProgram,
        stage_batch: CpuMeshBatch,
        ui_batch: CpuMeshBatch,
        aspect: f32,
    ) {
        if matches!(phase, ModalPhase::CharacterWardrobe) {
            let vp = self.personal.stage_projection(aspect);
            let light = Vec3::new(0.35, 0.55, 0.75).normalize();
            unsafe { gl.clear(glow::DEPTH_BUFFER_BIT); }
            shader.bind(gl, &vp, light);
            stage_batch.draw(gl);
        }
        unsafe {
            gl.disable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func_separate(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA, glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
        }
        shader.bind(gl, &Mat4::IDENTITY, Vec3::Z);
        ui_batch.draw(gl);
        unsafe {
            gl.disable(glow::BLEND);
            gl.enable(glow::DEPTH_TEST);
        }
    }
}
