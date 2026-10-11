use glam::{Mat4, Vec3, Vec4};
use glow::HasContext;
use crate::{GraphicPipeline, Mesh, ShaderProgram};

/// Forward graphic pipeline encapsulating clear color and scene lighting parameters.
pub struct ForwardPipeline {
    clear_color: Vec4,
    light_direction: Vec3,
}

impl ForwardPipeline {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(clear_color: Vec4, light_direction: Vec3) -> Self {
        Self {
            clear_color,
            light_direction,
        }
    }

    /// Secondary constructor normalizing light direction.
    pub fn directed(clear_color: Vec4, light_direction: Vec3) -> Self {
        Self::new(clear_color, light_direction.normalize())
    }

    /// Secondary constructor with natural daylight clear color and sunlight direction.
    pub fn daylight() -> Self {
        Self::directed(
            Vec4::new(0.04, 0.05, 0.08, 1.0),
            Vec3::new(0.6, 0.8, 0.4),
        )
    }
}


impl GraphicPipeline for ForwardPipeline {
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
