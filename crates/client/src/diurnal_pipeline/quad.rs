use glow::HasContext;
use engine::Mesh;

/// Focused contract uploading fullscreen firmament quad geometry to GPU memory.
pub trait QuadUpload {
    /// # Safety
    /// Requires a valid, current OpenGL context.
    unsafe fn upload(&self, gl: &glow::Context) -> Box<dyn Mesh>;
}

/// Fullscreen NDC quad mesh for firmament rendering at the far clipping plane (`<= 4` fields).
pub struct GlFirmamentQuad {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
}

impl GlFirmamentQuad {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(vao: glow::VertexArray, vbo: glow::Buffer, ebo: glow::Buffer) -> Self {
        Self { vao, vbo, ebo }
    }
}

impl Mesh for GlFirmamentQuad {
    fn draw(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
        }
    }

    fn release(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_buffer(self.ebo);
        }
    }
}

/// Declarative fullscreen NDC quad geometry source (`[-1, +1]^2`).
#[derive(Clone, Copy, Debug)]
pub struct FirmamentQuadSource;

impl FirmamentQuadSource {
    /// Primary constructor doing nothing except creating the source instance.
    pub const fn new() -> Self {
        Self
    }
}

impl QuadUpload for FirmamentQuadSource {
    unsafe fn upload(&self, gl: &glow::Context) -> Box<dyn Mesh> {
        let vertices: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0];
        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let vao = gl.create_vertex_array().expect("VAO creation failed");
        let vbo = gl.create_buffer().expect("VBO creation failed");
        let ebo = gl.create_buffer().expect("EBO creation failed");

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        let vertex_bytes = core::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * core::mem::size_of::<f32>(),
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertex_bytes, glow::STATIC_DRAW);

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        let index_bytes = core::slice::from_raw_parts(
            indices.as_ptr() as *const u8,
            indices.len() * core::mem::size_of::<u32>(),
        );
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, index_bytes, glow::STATIC_DRAW);

        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
        gl.bind_vertex_array(None);

        Box::new(GlFirmamentQuad::new(vao, vbo, ebo))
    }
}
