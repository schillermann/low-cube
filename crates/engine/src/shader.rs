use glam::{Mat4, Vec3};
use glow::HasContext;
use crate::{ShaderProgram, ShaderSource};

const VERTEX_SHADER_SRC: &str = r#"#version 330 core
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_color;
layout(location = 3) in float in_glow;

uniform mat4 u_view_proj;

out vec3 v_normal;
out vec3 v_color;
out float v_glow;

void main() {
    v_normal = in_normal;
    v_color = in_color;
    v_glow = in_glow;
    gl_Position = u_view_proj * vec4(in_position, 1.0);
}
"#;

const FRAGMENT_SHADER_SRC: &str = r#"#version 330 core
in vec3 v_normal;
in vec3 v_color;
in float v_glow;

uniform vec3 u_light_dir;

out vec4 frag_color;

void main() {
    vec3 norm = normalize(v_normal);
    float diff = max(dot(norm, normalize(u_light_dir)), 0.0);
    vec3 ambient = vec3(0.35);
    vec3 light = ambient + vec3(0.65) * diff;
    vec3 radiant = min(v_color * 1.15, vec3(1.0));
    frag_color = vec4(mix(v_color * light, radiant, v_glow), 1.0);
}
"#;

/// Encapsulated GPU program with clean, code-free primary constructor.
pub struct VoxelProgram {
    program: glow::Program,
    u_view_proj: glow::UniformLocation,
    u_light_dir: glow::UniformLocation,
}

impl VoxelProgram {
    pub fn new(
        program: glow::Program,
        u_view_proj: glow::UniformLocation,
        u_light_dir: glow::UniformLocation,
    ) -> Self {
        Self {
            program,
            u_view_proj,
            u_light_dir,
        }
    }
}

impl ShaderProgram for VoxelProgram {
    fn bind(&self, gl: &glow::Context, view_proj: &Mat4, light_dir: Vec3) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(Some(&self.u_view_proj), false, &view_proj.to_cols_array());
            gl.uniform_3_f32(Some(&self.u_light_dir), light_dir.x, light_dir.y, light_dir.z);
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

        Box::new(VoxelProgram::new(program, u_view_proj, u_light_dir))
    }
}
