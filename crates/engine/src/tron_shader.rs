use glam::{Mat4, Vec3};
use glow::HasContext;
use crate::{ShaderProgram, ShaderSource};

const TRON_VERTEX_SHADER_SRC: &str = r#"#version 330 core
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_color;

uniform mat4 u_view_proj;
uniform vec3 u_light_dir;

out vec3 v_color;

void main() {
    v_color = in_color;
    gl_Position = u_view_proj * vec4(in_position, 1.0);
}
"#;

const TRON_FRAGMENT_SHADER_SRC: &str = r#"#version 330 core
in vec3 v_color;
out vec4 frag_color;

void main() {
    frag_color = vec4(v_color, 1.0);
}
"#;

/// GPU shader program for unlit Tron wireframe line rendering.
pub struct TronProgram {
    program: glow::Program,
    u_view_proj: glow::UniformLocation,
}

impl TronProgram {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(program: glow::Program, u_view_proj: glow::UniformLocation) -> Self {
        Self {
            program,
            u_view_proj,
        }
    }
}

impl ShaderProgram for TronProgram {
    fn bind(&self, gl: &glow::Context, view_proj: &Mat4, _light_dir: Vec3) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(
                Some(&self.u_view_proj),
                false,
                &view_proj.to_cols_array(),
            );
        }
    }
}

/// Encapsulated GLSL source code for unlit Tron wireframe rendering.
pub struct TronShaderSource {
    vertex: &'static str,
    fragment: &'static str,
}

impl TronShaderSource {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(vertex: &'static str, fragment: &'static str) -> Self {
        Self { vertex, fragment }
    }

    /// Embedded Tron shader sources compiled into the binary.
    pub fn embedded() -> Self {
        Self::new(TRON_VERTEX_SHADER_SRC, TRON_FRAGMENT_SHADER_SRC)
    }
}

impl ShaderSource for TronShaderSource {
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
            panic!("Link error: {}", gl.get_program_info_log(program));
        }

        gl.delete_shader(vs);
        gl.delete_shader(fs);

        let u_view_proj = gl
            .get_uniform_location(program, "u_view_proj")
            .expect("Uniform u_view_proj not found");

        Box::new(TronProgram::new(program, u_view_proj))
    }
}
