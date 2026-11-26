mod active;
mod hologram;
mod viewport;

use std::cell::Cell;
use glam::{Mat4, Vec3};
use glow::HasContext;
use winit::keyboard::KeyCode;
use engine::{CpuMeshBatch, Mesh, ShaderProgram};
#[cfg(test)]
use world::{BodyId, CelestialAtlas};
use world::{MeshTarget, PlanetaryWorld};
use crate::avatar::Avatar;
use crate::controls::{ActionBindings, ControlAction, InputTrigger, KeyboardControls, LocomotionTarget};
use crate::drive::CourseMode;
use crate::surface::PlatformSurface;
use super::portal::PortalInvite;
use super::widget::ScreenPoint;
use super::{ModalDirective, SessionModal};
use self::active::ActiveChart;
use self::hologram::ChartHologram;
pub use self::viewport::{ChartSelection, ChartViewport};

#[derive(Clone, Copy, Debug)]
enum ChartModalState {
    Closed(ScreenPoint),
    Open(ActiveChart),
}

struct BindingMirror<'a> {
    target: &'a mut dyn KeyboardControls,
    captured: &'a Cell<ActionBindings>,
}

impl KeyboardControls for BindingMirror<'_> {
    fn apply_bindings(&mut self, bindings: ActionBindings) {
        self.captured.set(bindings);
        self.target.apply_bindings(bindings);
    }

    fn drive_locomotion(&self, forward: Vec3, right: Vec3, target: &mut dyn LocomotionTarget) {
        self.target.drive_locomotion(forward, right, target);
    }
}

/// `SessionModal` decorator opening the interactive 3D Solar System Map on `ControlAction::SystemMap` (`KeyM`).
pub struct SolarChartModal<M> {
    origin: M,
    state: ChartModalState,
    bindings: Cell<ActionBindings>,
}

impl<M> SolarChartModal<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(origin: M, state: ChartModalState, bindings: Cell<ActionBindings>) -> Self {
        Self { origin, state, bindings }
    }

    pub const fn with_bindings(origin: M, bindings: ActionBindings) -> Self {
        Self::new(origin, ChartModalState::Closed(ScreenPoint::center()), Cell::new(bindings))
    }

    #[cfg(test)]
    pub const fn wrapping(origin: M) -> Self {
        Self::with_bindings(origin, ActionBindings::seeded())
    }

    const fn chart_open(&self) -> bool {
        matches!(self.state, ChartModalState::Open(_))
    }

    fn close_chart(&mut self) {
        if let ChartModalState::Open(active) = self.state {
            self.state = ChartModalState::Closed(active.viewport.cursor());
        }
    }

    #[cfg(test)]
    pub fn body_screen_point(&self, body: BodyId, aspect: f32) -> Option<ScreenPoint> {
        let ChartModalState::Open(ref active) = self.state else {
            return None;
        };
        let ux = (1.0 / aspect.max(0.5)).clamp(0.25, 1.5);
        let atlas: CelestialAtlas = active.snapshot.transfer(|a, _, _, _| a);
        active.viewport.body_screen_point(body, atlas, ux)
    }
}

impl<M: SessionModal> SessionModal for SolarChartModal<M> {
    fn open(&self) -> bool {
        self.chart_open() || self.origin.open()
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
        world: &dyn PlanetaryWorld,
    ) -> bool {
        if self.origin.open() || !self.bindings.get().matches(ControlAction::SystemMap, trigger) {
            return self.origin.toggle_gameplay_trigger(trigger, surface, avatar, world);
        }
        let cursor = match self.state {
            ChartModalState::Closed(pt) => pt,
            ChartModalState::Open(_) => {
                self.close_chart();
                surface.capture_cursor();
                return true;
            }
        };
        let (snap, accel, fusion) = avatar.solar_chart_snapshot(world);
        let vp = snap.transfer(|atlas, pos, _, _| ChartViewport::calibrated(atlas, pos, cursor));
        let mode = if fusion { CourseMode::Brachistochrone } else { CourseMode::Ballistic };
        let eff_accel = if accel > 1.0 { accel } else { 58.0 };
        self.state = ChartModalState::Open(ActiveChart::new(snap, vp, ChartSelection::Unselected, (mode, eff_accel)));
        surface.release_cursor();
        true
    }

    fn sync_world_chart(&mut self, avatar: &dyn Avatar, world: &dyn PlanetaryWorld) {
        if let ChartModalState::Open(ref mut active) = self.state {
            let (snap, accel, _) = avatar.solar_chart_snapshot(world);
            active.snapshot = snap;
            if accel > 1.0 {
                active.mode_and_accel.1 = accel;
            }
        }
        self.origin.sync_world_chart(avatar, world);
    }

    fn toggle_escape(&mut self, surface: &dyn PlatformSurface) -> ModalDirective {
        if self.chart_open() {
            self.close_chart();
            surface.capture_cursor();
            return ModalDirective::ContinueSession;
        }
        self.origin.toggle_escape(surface)
    }

    fn focus_regained(&mut self) -> ModalDirective { self.origin.focus_regained() }

    fn move_cursor(&mut self, point: ScreenPoint) {
        match self.state {
            ChartModalState::Closed(ref mut pt) => *pt = point,
            ChartModalState::Open(ref mut active) => active.viewport.move_cursor(point, 9.0 / 16.0),
        }
        self.origin.move_cursor(point);
    }

    fn press_mouse(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> ModalDirective {
        let ChartModalState::Open(ref mut active) = self.state else {
            return self.origin.press_mouse(surface, avatar);
        };
        if active.press_pointer(surface, avatar) {
            self.close_chart();
        }
        ModalDirective::ContinueSession
    }

    fn release_mouse(&mut self) {
        if let ChartModalState::Open(ref mut active) = self.state {
            active.viewport.end_drag();
        }
        self.origin.release_mouse();
    }

    fn handle_key(&mut self, code: KeyCode, ctrl_held: bool, avatar: &mut dyn Avatar) -> ModalDirective {
        if self.chart_open() {
            if self.bindings.get().matches(ControlAction::SystemMap, InputTrigger::Key(code)) {
                self.close_chart();
                return ModalDirective::ContinueSession;
            }
            if code == KeyCode::Tab {
                if let ChartModalState::Open(ref mut active) = self.state {
                    active.mode_and_accel.0 = active.mode_and_accel.0.toggled();
                    active.replot_selected(avatar);
                }
            }
            return ModalDirective::ContinueSession;
        }
        self.origin.handle_key(code, ctrl_held, avatar)
    }

    fn handle_text(&mut self, text: &str) {
        if !self.chart_open() {
            self.origin.handle_text(text);
        }
    }

    fn handle_trigger(&mut self, trigger: InputTrigger, avatar: &mut dyn Avatar) -> bool {
        if let ChartModalState::Open(ref mut active) = self.state {
            let max_s = active.max_span();
            return match trigger {
                InputTrigger::WheelUp => {
                    active.viewport.zoom(0.76, max_s);
                    true
                }
                InputTrigger::WheelDown => {
                    active.viewport.zoom(1.32, max_s);
                    true
                }
                other if self.bindings.get().matches(ControlAction::SystemMap, other) => {
                    self.close_chart();
                    true
                }
                _ => false,
            };
        }
        self.origin.handle_trigger(trigger, avatar)
    }

    fn sync_controls(&self, controls: &mut dyn KeyboardControls) {
        let mut mirror = BindingMirror { target: controls, captured: &self.bindings };
        self.origin.sync_controls(&mut mirror);
    }

    fn announce_portal(&mut self, invite: PortalInvite) { self.origin.announce_portal(invite); }
    fn announce_auto_punch(&mut self) { self.origin.announce_auto_punch(); }
    fn announce_punch_code(&mut self, punch_code: String) { self.origin.announce_punch_code(punch_code); }
    fn complete_join(&mut self, surface: &dyn PlatformSurface) { self.origin.complete_join(surface); }
    fn report_join_failure(&mut self) { self.origin.report_join_failure(); }
    fn step(&mut self, dt: f32) { self.origin.step(dt); }

    #[cfg(test)]
    fn emit_ui(&self, avatar: &dyn Avatar, target: &mut dyn MeshTarget) {
        if let ChartModalState::Open(active) = self.state {
            ChartHologram::new(active.snapshot, active.viewport, active.selection, (active.mode_and_accel.0, 9.0 / 16.0))
                .emit(target);
            return;
        }
        self.origin.emit_ui(avatar, target);
    }

    fn emit_studio(&self, target: &mut dyn MeshTarget) {
        if !self.chart_open() {
            self.origin.emit_studio(target);
        }
    }

    fn draw_layer(&self, gl: &glow::Context, shader: &dyn ShaderProgram, avatar: &dyn Avatar, aspect: f32) {
        if let ChartModalState::Open(active) = self.state {
            let ux = (1.0 / aspect.max(0.5)).clamp(0.25, 1.5);
            let mut ui_batch = CpuMeshBatch::with_quad_capacity(1_024);
            ChartHologram::new(active.snapshot, active.viewport, active.selection, (active.mode_and_accel.0, ux))
                .emit(&mut ui_batch);
            unsafe {
                gl.disable(glow::DEPTH_TEST);
                gl.enable(glow::BLEND);
                gl.blend_func_separate(
                    glow::SRC_ALPHA,
                    glow::ONE_MINUS_SRC_ALPHA,
                    glow::ONE,
                    glow::ONE_MINUS_SRC_ALPHA,
                );
            }
            shader.bind(gl, &Mat4::IDENTITY, Vec3::Z);
            ui_batch.draw(gl);
            unsafe {
                gl.disable(glow::BLEND);
                gl.enable(glow::DEPTH_TEST);
            }
            return;
        }
        self.origin.draw_layer(gl, shader, avatar, aspect);
    }
}
