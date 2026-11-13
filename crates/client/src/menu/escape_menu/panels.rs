use glam::{Mat4, Vec3};
use glow::HasContext;
use engine::{CpuMeshBatch, Mesh, ShaderProgram};
use crate::menu::portal::PortalStudio;
use crate::menu::settings::ControlStudio;
use crate::menu::studio::WardrobeStudio;

/// Polymorphic modal state machine for gameplay, Escape pause menu, Character studio, Settings, and Co-op join.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ModalPhase {
    Gameplay,
    EscapePause,
    CharacterWardrobe,
    ControlSettings,
    PortalJoin,
}

/// Collaborator panels owned by `EscapeMenu` (wardrobe, controls, and Co-op portal).
pub(super) struct StudioPanels {
    pub(super) studio: WardrobeStudio,
    pub(super) settings: ControlStudio,
    pub(super) portal: PortalStudio,
}

impl StudioPanels {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(studio: WardrobeStudio, settings: ControlStudio, portal: PortalStudio) -> Self {
        Self { studio, settings, portal }
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
            let vp = self.studio.stage_projection(aspect);
            let light = Vec3::new(0.35, 0.55, 0.75).normalize();
            unsafe {
                gl.clear(glow::DEPTH_BUFFER_BIT);
            }
            shader.bind(gl, &vp, light);
            stage_batch.draw(gl);
        }
        unsafe {
            gl.disable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
        shader.bind(gl, &Mat4::IDENTITY, Vec3::Z);
        ui_batch.draw(gl);
        unsafe {
            gl.disable(glow::BLEND);
            gl.enable(glow::DEPTH_TEST);
        }
    }
}
