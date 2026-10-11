use glam::{Mat4, Vec3};

mod buffer;
mod renderer;
mod shader;

pub use buffer::{CpuMeshBuffer, GlMesh, MeshUploader};
pub use renderer::ForwardRenderer;
pub use shader::{ShaderCompiler, VoxelShader};

/// Interface for GPU-renderable mesh objects.
pub trait Mesh {
    fn draw(&self, gl: &glow::Context);
}

/// Interface for shader pipelines.
pub trait ShaderProgram {
    fn bind(&self, gl: &glow::Context, view_proj: &Mat4, light_dir: Vec3);
}

/// Interface for scene rendering.
pub trait SceneRenderer {
    fn render(
        &self,
        gl: &glow::Context,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        view_proj: &Mat4,
    );
}
