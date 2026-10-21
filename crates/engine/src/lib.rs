use glam::{Mat4, Quat, Vec3};

mod buffer;
mod composite_mesh;
mod line_buffer;
mod pipeline;
mod shader;
mod tron_pipeline;
mod tron_shader;

#[cfg(test)]
mod tron_tests;

pub use buffer::{CpuMeshBatch, GlMesh};
pub use composite_mesh::{CompositeMesh, PlanetaryMesh, TronMesh};
pub use line_buffer::{CpuLineBatch, CpuTronBatch, GlLineMesh};
pub use pipeline::ForwardPipeline;
pub use shader::{VoxelProgram, VoxelShaderSource};
pub use tron_pipeline::TronPipeline;
pub use tron_shader::{TronProgram, TronShaderSource};

/// Interface for GPU-renderable mesh objects.
pub trait Mesh {
    fn draw(&self, gl: &glow::Context);
    fn release(&self, _gl: &glow::Context) {}
}

/// Interface for compiling shader sources into GPU shader programs.
pub trait ShaderSource {
    /// # Safety
    /// Requires a valid, current OpenGL context.
    unsafe fn compile(&self, gl: &glow::Context) -> Box<dyn ShaderProgram>;
}

/// Interface for shader pipelines.
pub trait ShaderProgram {
    fn bind(&self, gl: &glow::Context, view_proj: &Mat4, light_dir: Vec3);
}

/// Interface for scene rendering.
pub trait GraphicPipeline {
    fn render(
        &self,
        gl: &glow::Context,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        view_proj: &Mat4,
        surface_rot: Quat,
    );
}
