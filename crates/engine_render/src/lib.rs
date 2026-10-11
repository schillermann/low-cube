use glam::{Mat4, Vec3};
use glow::HasContext;
use world::SurfaceMeshReceiver;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

/// Buffer collecting vertices and triangles via SurfaceMeshReceiver.
pub struct CpuMeshBuffer {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl CpuMeshBuffer {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}

impl SurfaceMeshReceiver for CpuMeshBuffer {
    fn push_quad(
        &mut self,
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        normal: Vec3,
        color: Vec3,
    ) {
        let base_idx = self.vertices.len() as u32;

        let norm_arr = normal.to_array();
        let col_arr = color.to_array();

        self.vertices.push(Vertex {
            position: v0.to_array(),
            normal: norm_arr,
            color: col_arr,
        });
        self.vertices.push(Vertex {
            position: v1.to_array(),
            normal: norm_arr,
            color: col_arr,
        });
        self.vertices.push(Vertex {
            position: v2.to_array(),
            normal: norm_arr,
            color: col_arr,
        });
        self.vertices.push(Vertex {
            position: v3.to_array(),
            normal: norm_arr,
            color: col_arr,
        });

        // Two triangles per quad
        self.indices.push(base_idx);
        self.indices.push(base_idx + 1);
        self.indices.push(base_idx + 2);

        self.indices.push(base_idx);
        self.indices.push(base_idx + 2);
        self.indices.push(base_idx + 3);
    }
}

/// Represents GPU-allocated mesh buffers (VAO, VBO, EBO).
pub struct GlMesh {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    index_count: i32,
}

impl GlMesh {
    pub unsafe fn from_cpu_buffer(gl: &glow::Context, buffer: &CpuMeshBuffer) -> Self {
        let vao = gl.create_vertex_array().expect("Failed to create VAO");
        let vbo = gl.create_buffer().expect("Failed to create VBO");
        let ebo = gl.create_buffer().expect("Failed to create EBO");

        gl.bind_vertex_array(Some(vao));

        // Upload vertex data
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        let vertex_bytes = core::slice::from_raw_parts(
            buffer.vertices.as_ptr() as *const u8,
            buffer.vertices.len() * core::mem::size_of::<Vertex>(),
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertex_bytes, glow::STATIC_DRAW);

        // Upload index data
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        let index_bytes = core::slice::from_raw_parts(
            buffer.indices.as_ptr() as *const u8,
            buffer.indices.len() * core::mem::size_of::<u32>(),
        );
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, index_bytes, glow::STATIC_DRAW);

        let stride = core::mem::size_of::<Vertex>() as i32;

        // Position: attribute 0
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);

        // Normal: attribute 1
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 12);

        // Color: attribute 2
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, stride, 24);

        gl.bind_vertex_array(None);

        Self {
            vao,
            vbo,
            ebo,
            index_count: buffer.indices.len() as i32,
        }
    }

    pub unsafe fn draw(&self, gl: &glow::Context) {
        gl.bind_vertex_array(Some(self.vao));
        gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
        gl.bind_vertex_array(None);
    }

    pub unsafe fn destroy(&self, gl: &glow::Context) {
        gl.delete_vertex_array(self.vao);
        gl.delete_buffer(self.vbo);
        gl.delete_buffer(self.ebo);
    }
}

const VERTEX_SHADER_SRC: &str = r#"#version 330 core
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_color;

uniform mat4 u_view_proj;

out vec3 v_normal;
out vec3 v_color;

void main() {
    v_normal = in_normal;
    v_color = in_color;
    gl_Position = u_view_proj * vec4(in_position, 1.0);
}
"#;

const FRAGMENT_SHADER_SRC: &str = r#"#version 330 core
in vec3 v_normal;
in vec3 v_color;

uniform vec3 u_light_dir;

out vec4 frag_color;

void main() {
    vec3 norm = normalize(v_normal);
    float diff = max(dot(norm, normalize(u_light_dir)), 0.0);
    // Ambient + diffuse directional sunlight
    vec3 ambient = vec3(0.35);
    vec3 light = ambient + vec3(0.65) * diff;
    frag_color = vec4(v_color * light, 1.0);
}
"#;

pub struct VoxelShader {
    program: glow::Program,
    u_view_proj: glow::UniformLocation,
    u_light_dir: glow::UniformLocation,
}

impl VoxelShader {
    pub unsafe fn compile(gl: &glow::Context) -> Self {
        let vs = gl.create_shader(glow::VERTEX_SHADER).expect("VS create failed");
        gl.shader_source(vs, VERTEX_SHADER_SRC);
        gl.compile_shader(vs);
        if !gl.get_shader_compile_status(vs) {
            panic!("VS compile error: {}", gl.get_shader_info_log(vs));
        }

        let fs = gl.create_shader(glow::FRAGMENT_SHADER).expect("FS create failed");
        gl.shader_source(fs, FRAGMENT_SHADER_SRC);
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

        Self {
            program,
            u_view_proj,
            u_light_dir,
        }
    }

    pub unsafe fn bind(&self, gl: &glow::Context, view_proj: &Mat4, light_dir: Vec3) {
        gl.use_program(Some(self.program));
        gl.uniform_matrix_4_f32_slice(Some(&self.u_view_proj), false, &view_proj.to_cols_array());
        gl.uniform_3_f32(Some(&self.u_light_dir), light_dir.x, light_dir.y, light_dir.z);
    }
}
