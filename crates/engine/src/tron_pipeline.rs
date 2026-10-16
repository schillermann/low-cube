use glam::{Mat4, Quat, Vec3, Vec4};
use glow::HasContext;
use crate::{GraphicPipeline, Mesh, ShaderProgram};

/// Forward graphic pipeline for Tron wireframe rendering on pitch-black void.
pub struct TronPipeline {
    clear_color: Vec4,
}

impl TronPipeline {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(clear_color: Vec4) -> Self {
        Self { clear_color }
    }

    /// Secondary constructor with pitch-black void clear color.
    pub fn void() -> Self {
        Self::new(Vec4::new(0.01, 0.01, 0.02, 1.0))
    }
}

impl GraphicPipeline for TronPipeline {
    fn render(
        &self,
        gl: &glow::Context,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        view_proj: &Mat4,
        _surface_rot: Quat,
    ) {
        unsafe {
            gl.clear_color(
                self.clear_color.x,
                self.clear_color.y,
                self.clear_color.z,
                self.clear_color.w,
            );
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
        shader.bind(gl, view_proj, Vec3::Y);
        mesh.draw(gl);
    }
}
