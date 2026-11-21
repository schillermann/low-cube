use winit::keyboard::KeyCode;
use glow::HasContext;
use engine::{CpuMeshBatch, Mesh, ShaderProgram};
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::surface::PlatformSurface;
use super::portal::PortalInvite;
use super::widget::ScreenPoint;
use super::{ModalDirective, SessionModal};

/// Decorator over any `SessionModal` rendering open 3D satchel, crafting tome, and blueprint stages over a cleared depth buffer.
pub struct InspectionModal<M> {
    origin: M,
}

impl<M> InspectionModal<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: M) -> Self {
        Self { origin }
    }
}

impl<M: SessionModal> SessionModal for InspectionModal<M> {
    fn open(&self) -> bool {
        self.origin.open()
    }

    #[cfg(test)]
    fn dialing_active(&self) -> bool {
        self.origin.dialing_active()
    }

    fn toggle_escape(&mut self, surface: &dyn PlatformSurface) -> ModalDirective {
        self.origin.toggle_escape(surface)
    }

    fn focus_regained(&mut self) -> ModalDirective {
        self.origin.focus_regained()
    }

    fn move_cursor(&mut self, point: ScreenPoint) {
        self.origin.move_cursor(point)
    }

    fn press_mouse(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> ModalDirective {
        self.origin.press_mouse(surface, avatar)
    }

    fn release_mouse(&mut self) {
        self.origin.release_mouse()
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
        if !self.origin.open() {
            let _ = avatar.emit_inspection_stage(16.0 / 9.0, target);
        }
        self.origin.emit_ui(avatar, target);
    }

    fn emit_studio(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_studio(target);
    }

    fn draw_layer(&self, gl: &glow::Context, shader: &dyn ShaderProgram, avatar: &dyn Avatar, aspect: f32) {
        if !self.origin.open() {
            let mut stage_batch = CpuMeshBatch::empty();
            if let Some((vp, light)) = avatar.emit_inspection_stage(aspect, &mut stage_batch) {
                unsafe {
                    gl.clear(glow::DEPTH_BUFFER_BIT);
                }
                shader.bind(gl, &vp, light);
                stage_batch.draw(gl);
            }
        }
        self.origin.draw_layer(gl, shader, avatar, aspect);
    }
}
