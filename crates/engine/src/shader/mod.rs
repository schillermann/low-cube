mod glsl;

use glam::{Mat4, Vec3};
use glow::HasContext;
use crate::{ShaderProgram, ShaderSource};
use self::glsl::{FRAGMENT_SHADER_SRC, VERTEX_SHADER_SRC};

/// Encapsulated GPU program with clean, code-free primary constructor.
pub struct VoxelProgram {
    program: glow::Program,
    u_view_proj: glow::UniformLocation,
    u_light_dir: glow::UniformLocation,
    u_model: glow::UniformLocation,
}

impl VoxelProgram {
    pub fn new(
        program: glow::Program,
        u_view_proj: glow::UniformLocation,
        u_light_dir: glow::UniformLocation,
        u_model: glow::UniformLocation,
    ) -> Self {
        Self {
            program,
            u_view_proj,
            u_light_dir,
            u_model,
        }
    }
}

impl ShaderProgram for VoxelProgram {
    fn bind(&self, gl: &glow::Context, view_proj: &Mat4, light_dir: Vec3) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(Some(&self.u_view_proj), false, &view_proj.to_cols_array());
            gl.uniform_matrix_4_f32_slice(Some(&self.u_model), false, &Mat4::IDENTITY.to_cols_array());
            gl.uniform_3_f32(Some(&self.u_light_dir), light_dir.x, light_dir.y, light_dir.z);
        }
    }

    fn apply_model(&self, gl: &glow::Context, model: &Mat4) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(&self.u_model), false, &model.to_cols_array());
        }
    }
}

/// Encapsulated GLSL source code for voxel rendering.
pub struct VoxelShaderSource {
    vertex: &'static str,
    fragment: &'static str,
}

impl VoxelShaderSource {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(vertex: &'static str, fragment: &'static str) -> Self {
        Self { vertex, fragment }
    }

    /// Embedded voxel shader sources compiled into the binary.
    pub fn embedded() -> Self {
        Self::new(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC)
    }
}

impl ShaderSource for VoxelShaderSource {
    unsafe fn compile(&self, gl: &glow::Context) -> Box<dyn ShaderProgram> {
        let vs = gl.create_shader(glow::VERTEX_SHADER).expect("VS create failed");
        gl.shader_source(vs, self.vertex);
        gl.compile_shader(vs);
        if !gl.get_shader_compile_status(vs) {
            panic!("VS compile error: {}", gl.get_shader_info_log(vs));
        }

        let fs = gl.create_shader(glow::FRAGMENT_SHADER).expect("FS create failed");
        gl.shader_source(fs, self.fragment);
        gl.compile_shader(fs);
        if !gl.get_shader_compile_status(fs) {
            panic!("FS compile error: {}", gl.get_shader_info_log(fs));
        }

        let program = gl.create_program().expect("Program create failed");
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("Program link error: {}", gl.get_program_info_log(program));
        }

        gl.delete_shader(vs);
        gl.delete_shader(fs);

        let u_view_proj = gl.get_uniform_location(program, "u_view_proj").expect("u_view_proj missing");
        let u_light_dir = gl.get_uniform_location(program, "u_light_dir").expect("u_light_dir missing");
        let u_model = gl.get_uniform_location(program, "u_model").expect("u_model missing");

        Box::new(VoxelProgram::new(program, u_view_proj, u_light_dir, u_model))
    }
}
