use glam::{Mat4, Vec3, Vec4};
use glow::HasContext;
use crate::{Mesh, SceneRenderer, ShaderProgram};

/// Forward renderer encapsulating clear color and scene lighting parameters.
pub struct ForwardRenderer {
    clear_color: Vec4,
    light_direction: Vec3,
}

impl ForwardRenderer {
    pub fn new(clear_color: Vec4, light_direction: Vec3) -> Self {
        Self {
            clear_color,
            light_direction: light_direction.normalize(),
        }
    }
}

impl SceneRenderer for ForwardRenderer {
    fn render(
        &self,
        gl: &glow::Context,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        view_proj: &Mat4,
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
        shader.bind(gl, view_proj, self.light_direction);
        mesh.draw(gl);
    }
}
