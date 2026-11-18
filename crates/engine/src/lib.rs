use glam::{Mat4, Quat, Vec3};

mod buffer;
mod composite_mesh;
mod line_buffer;
mod pipeline;
mod sector_mesh;
mod shader;
mod streaming_mesh;
mod tron_pipeline;
mod tron_shader;

#[cfg(test)]
mod tron_tests;

pub use buffer::{CpuMeshBatch, GlMesh};
pub use composite_mesh::{CompositeMesh, PlanetaryMesh, TronMesh};
pub use line_buffer::{CpuLineBatch, CpuTronBatch, GlLineMesh};
pub use pipeline::ForwardPipeline;
pub use sector_mesh::SectorGridMesh;
pub use shader::{VoxelProgram, VoxelShaderSource};
pub use streaming_mesh::StreamingMesh;
pub use tron_pipeline::TronPipeline;
pub use tron_shader::{TronProgram, TronShaderSource};

/// Interface for GPU-renderable mesh objects.
pub trait Mesh {
    fn draw(&self, gl: &glow::Context);
    fn release(&self, _gl: &glow::Context) {}
    fn patch_at(&self, _slot: usize) -> Option<&dyn Mesh> { None }
    fn store_patch(&mut self, _slot: usize, _patch: Box<dyn Mesh>) {}
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
