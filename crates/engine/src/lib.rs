use glam::{Mat4, Quat, Vec3};

mod buffer;
mod composite_mesh;
mod pipeline;
mod sector_mesh;
mod shader;
mod streaming_mesh;

pub use buffer::{CpuMeshBatch, GlMesh};
pub use composite_mesh::{CoarseMesh, CompositeMesh, PlanetaryMesh, StarryPipeline};
pub use pipeline::ForwardPipeline;
pub use sector_mesh::SectorGridMesh;
pub use shader::{VoxelProgram, VoxelShaderSource};
pub use streaming_mesh::StreamingMesh;

/// Interface for GPU-renderable mesh objects.
pub trait Mesh {
    fn draw(&self, gl: &glow::Context);
    fn draw_posed(&self, gl: &glow::Context, _shader: &dyn ShaderProgram) {
        self.draw(gl);
    }
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
    fn apply_model(&self, _gl: &glow::Context, _model: &Mat4) {}
}

/// Interface for scene rendering.
pub trait GraphicPipeline {
    fn sync_ephemeris(&self, _planet_cubes: &[[f32; 4]; 7], _eclipse_occluder: [f32; 4]) {}

    fn render(
        &self,
        gl: &glow::Context,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        view_proj: &Mat4,
        surface_rot: Quat,
    );
}
