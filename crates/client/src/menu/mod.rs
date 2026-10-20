use glam::{Mat4, Vec3};
use glow::HasContext;
use winit::keyboard::KeyCode;
use engine::{CpuMeshBatch, Mesh, ShaderProgram};
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::surface::PlatformSurface;

mod archetype;
mod glyph;
mod stage;
mod studio;
#[cfg(test)]
mod tests;
mod widget;

#[cfg(test)]
pub use archetype::AvatarArchetype;
pub use glyph::PixelStamp;
#[cfg(test)]
pub use stage::{StudioAnimation, TurntableStage};
pub use studio::{StudioDirective, WardrobeStudio};
pub use widget::{ScreenBounds, ScreenPoint};

/// Directive returned when interacting with the Escape menu or Character studio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalDirective {
    ContinueSession,
    ExitGame,
}

/// Polymorphic modal state machine for gameplay, Escape pause menu, and Character studio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModalPhase {
    Gameplay,
    EscapePause,
    CharacterWardrobe,
}

/// Interface for session modal overlay interaction and rendering.
pub trait SessionModal {
    fn open(&self) -> bool;
    fn toggle_escape(&mut self, surface: &dyn PlatformSurface);
    fn move_cursor(&mut self, point: ScreenPoint);
    fn press_mouse(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> ModalDirective;
    fn release_mouse(&mut self);
    fn handle_key(&mut self, code: KeyCode, avatar: &mut dyn Avatar) -> bool;
    fn step(&mut self, dt: f32);
    fn emit_ui(&self, target: &mut dyn MeshTarget);
    fn emit_studio(&self, target: &mut dyn MeshTarget);
    fn draw_layer(&self, gl: &glow::Context, shader: &dyn ShaderProgram, aspect: f32);
}

/// Encapsulated Escape menu and 3D Character Studio modal overlay.
pub struct EscapeMenu {
    phase: ModalPhase,
    pointer: ScreenPoint,
    studio: WardrobeStudio,
}

impl EscapeMenu {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(phase: ModalPhase, pointer: ScreenPoint, studio: WardrobeStudio) -> Self {
        Self { phase, pointer, studio }
    }

    pub fn closed() -> Self {
        Self::new(ModalPhase::Gameplay, ScreenPoint::center(), WardrobeStudio::from_profile())
    }

    fn resume_button() -> ScreenBounds { ScreenBounds::new(-0.28, 0.12, 0.28, 0.26) }
    fn character_button() -> ScreenBounds { ScreenBounds::new(-0.28, -0.08, 0.28, 0.06) }
    fn quit_button() -> ScreenBounds { ScreenBounds::new(-0.28, -0.28, 0.28, -0.14) }

    fn emit_pause_panel(&self, target: &mut dyn MeshTarget) {
        let (bg, rim, gold) = (Vec3::new(0.08, 0.10, 0.15), Vec3::new(0.32, 0.39, 0.50), Vec3::new(0.96, 0.82, 0.34));
        ScreenBounds::new(-0.36, -0.38, 0.36, 0.44).emit_framed(bg, rim, 0.008, target);
        PixelStamp::title(gold).emit_centered("MAIN MENU", 0.0, 0.35, target);
        Self::resume_button().emit_button("RESUME GAME", self.pointer, Vec3::new(0.18, 0.42, 0.24), false, target);
        Self::character_button().emit_button("CHARACTER", self.pointer, Vec3::new(0.20, 0.32, 0.52), false, target);
        Self::quit_button().emit_button("QUIT GAME", self.pointer, Vec3::new(0.54, 0.18, 0.18), false, target);
    }
}

impl SessionModal for EscapeMenu {
    fn open(&self) -> bool { !matches!(self.phase, ModalPhase::Gameplay) }

    fn toggle_escape(&mut self, surface: &dyn PlatformSurface) {
        match self.phase {
            ModalPhase::Gameplay => { self.phase = ModalPhase::EscapePause; surface.release_cursor(); }
            ModalPhase::EscapePause => { self.phase = ModalPhase::Gameplay; surface.capture_cursor(); }
            ModalPhase::CharacterWardrobe => { self.phase = ModalPhase::EscapePause; surface.release_cursor(); }
        }
    }

    fn move_cursor(&mut self, point: ScreenPoint) {
        self.pointer = point;
        if matches!(self.phase, ModalPhase::CharacterWardrobe) { self.studio.move_pointer(point); }
    }

    fn press_mouse(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> ModalDirective {
        match self.phase {
            ModalPhase::Gameplay => ModalDirective::ContinueSession,
            ModalPhase::EscapePause => {
                if Self::resume_button().contains(self.pointer) {
                    self.phase = ModalPhase::Gameplay;
                    surface.capture_cursor();
                } else if Self::character_button().contains(self.pointer) {
                    self.phase = ModalPhase::CharacterWardrobe;
                    self.studio.move_pointer(self.pointer);
                } else if Self::quit_button().contains(self.pointer) {
                    return ModalDirective::ExitGame;
                }
                ModalDirective::ContinueSession
            }
            ModalPhase::CharacterWardrobe => {
                let directive = self.studio.press_pointer();
                avatar.apply_attire(self.studio.current_attire());
                match directive {
                    StudioDirective::StayInStudio => ModalDirective::ContinueSession,
                    StudioDirective::ReturnToPauseMenu => { self.phase = ModalPhase::EscapePause; ModalDirective::ContinueSession }
                }
            }
        }
    }

    fn release_mouse(&mut self) { self.studio.release_pointer(); }

    fn handle_key(&mut self, code: KeyCode, avatar: &mut dyn Avatar) -> bool {
        if matches!(self.phase, ModalPhase::CharacterWardrobe) && self.studio.handle_key(code) {
            avatar.apply_attire(self.studio.current_attire());
            return true;
        }
        false
    }

    fn step(&mut self, dt: f32) {
        if matches!(self.phase, ModalPhase::CharacterWardrobe) { self.studio.step(dt); }
    }

    fn emit_ui(&self, target: &mut dyn MeshTarget) {
        match self.phase {
            ModalPhase::Gameplay => {}
            ModalPhase::EscapePause => self.emit_pause_panel(target),
            ModalPhase::CharacterWardrobe => self.studio.emit_overlay(target),
        }
    }

    fn emit_studio(&self, target: &mut dyn MeshTarget) {
        if matches!(self.phase, ModalPhase::CharacterWardrobe) { self.studio.emit_stage(target); }
    }

    fn draw_layer(&self, gl: &glow::Context, shader: &dyn ShaderProgram, aspect: f32) {
        if matches!(self.phase, ModalPhase::Gameplay) { return; }
        if matches!(self.phase, ModalPhase::CharacterWardrobe) {
            let mut stage_batch = CpuMeshBatch::empty();
            self.emit_studio(&mut stage_batch);
            let vp = self.studio.stage_projection(aspect);
            let light = Vec3::new(0.35, 0.55, 0.75).normalize();
            unsafe { gl.clear(glow::DEPTH_BUFFER_BIT); }
            shader.bind(gl, &vp, light);
            stage_batch.draw(gl);
        }
        let mut ui_batch = CpuMeshBatch::empty();
        self.emit_ui(&mut ui_batch);
        unsafe { gl.disable(glow::DEPTH_TEST); }
        shader.bind(gl, &Mat4::IDENTITY, Vec3::Z);
        ui_batch.draw(gl);
        unsafe { gl.enable(glow::DEPTH_TEST); }
    }
}
