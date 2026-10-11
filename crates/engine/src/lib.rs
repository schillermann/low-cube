use glam::{Mat4, Vec3};

mod buffer;
mod pipeline;
mod shader;

pub use buffer::{CpuMeshBatch, GlMesh};
pub use pipeline::ForwardPipeline;
pub use shader::{VoxelProgram, VoxelShaderSource};

/// Interface for GPU-renderable mesh objects.
pub trait Mesh {
    fn draw(&self, gl: &glow::Context);
}

/// Interface for compiling shader sources into GPU shader programs.
pub trait ShaderSource {
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
    );
}
