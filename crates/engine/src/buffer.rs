use glow::HasContext;
use world::{MeshTarget, Quad};
use crate::Mesh;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            normal,
            color,
        }
    }
}


/// CPU batch collecting quads via MeshTarget interface.
pub struct CpuMeshBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl CpuMeshBatch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    /// Secondary constructor creating an empty mesh batch.
    pub fn empty() -> Self {
        Self::new(Vec::new(), Vec::new())
    }

    /// # Safety
    /// Requires a valid, current OpenGL context.
    pub unsafe fn upload(&self, gl: &glow::Context) -> Box<dyn Mesh> {
        Box::new(self.upload_gl(gl))
    }

    /// # Safety
    /// Requires a valid, current OpenGL context.
    pub unsafe fn upload_gl(&self, gl: &glow::Context) -> GlMesh {
        let vao = gl.create_vertex_array().expect("Failed to create VAO");
        let vbo = gl.create_buffer().expect("Failed to create VBO");
        let ebo = gl.create_buffer().expect("Failed to create EBO");

        gl.bind_vertex_array(Some(vao));

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        let vertex_bytes = core::slice::from_raw_parts(
            self.vertices.as_ptr() as *const u8,
            self.vertices.len() * core::mem::size_of::<Vertex>(),
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertex_bytes, glow::STATIC_DRAW);

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        let index_bytes = core::slice::from_raw_parts(
            self.indices.as_ptr() as *const u8,
            self.indices.len() * core::mem::size_of::<u32>(),
        );
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, index_bytes, glow::STATIC_DRAW);

        let stride = core::mem::size_of::<Vertex>() as i32;

        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);

        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 12);

        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, stride, 24);

        gl.bind_vertex_array(None);

        GlMesh::new(vao, vbo, ebo, self.indices.len() as i32)
    }
}

impl Mesh for CpuMeshBatch {
    fn draw(&self, gl: &glow::Context) {
        if self.indices.is_empty() {
            return;
        }
        unsafe {
            let mesh = self.upload_gl(gl);
            mesh.draw(gl);
            mesh.dispose(gl);
        }
    }
}

impl MeshTarget for CpuMeshBatch {
    fn push_quad(&mut self, quad: Quad) {
        let base_idx = self.vertices.len() as u32;

        quad.transfer(|v0, v1, v2, v3, normal, color| {
            let norm_arr = normal.to_array();
            let col_arr = color.to_array();

            self.vertices.push(Vertex::new(v0.to_array(), norm_arr, col_arr));
            self.vertices.push(Vertex::new(v1.to_array(), norm_arr, col_arr));
            self.vertices.push(Vertex::new(v2.to_array(), norm_arr, col_arr));
            self.vertices.push(Vertex::new(v3.to_array(), norm_arr, col_arr));
        });

        self.indices.push(base_idx);
        self.indices.push(base_idx + 1);
        self.indices.push(base_idx + 2);

        self.indices.push(base_idx);
        self.indices.push(base_idx + 2);
        self.indices.push(base_idx + 3);
    }
}

/// Encapsulated GPU Mesh implementing Mesh with a clean, code-free primary constructor.
pub struct GlMesh {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    index_count: i32,
}

impl GlMesh {
    pub fn new(
        vao: glow::VertexArray,
        vbo: glow::Buffer,
        ebo: glow::Buffer,
        index_count: i32,
    ) -> Self {
        Self {
            vao,
            vbo,
            ebo,
            index_count,
        }
    }

    /// # Safety
    /// Requires a valid, current OpenGL context and live GPU handles.
    pub unsafe fn dispose(&self, gl: &glow::Context) {
        gl.delete_vertex_array(self.vao);
        gl.delete_buffer(self.vbo);
        gl.delete_buffer(self.ebo);
    }
}

impl Mesh for GlMesh {
    fn draw(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
        }
    }
}
