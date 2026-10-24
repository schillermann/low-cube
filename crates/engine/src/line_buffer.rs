use glam::Vec3;
use glow::HasContext;
use world::{MeshTarget, Quad};
use crate::buffer::{CpuMeshBatch, Vertex};
use crate::composite_mesh::TronMesh;
use crate::Mesh;

/// Encapsulated GPU line mesh rendering GL_LINES elements.
pub struct GlLineMesh {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    index_count: i32,
}

impl GlLineMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(vao: glow::VertexArray, vbo: glow::Buffer, ebo: glow::Buffer, index_count: i32) -> Self {
        Self { vao, vbo, ebo, index_count }
    }

    /// # Safety
    /// Requires a valid, current OpenGL context and live GPU handles.
    pub unsafe fn dispose(&self, gl: &glow::Context) {
        gl.delete_vertex_array(self.vao);
        gl.delete_buffer(self.vbo);
        gl.delete_buffer(self.ebo);
    }
}

impl Mesh for GlLineMesh {
    fn draw(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_elements(glow::LINES, self.index_count, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
        }
    }

    fn release(&self, gl: &glow::Context) {
        unsafe {
            self.dispose(gl);
        }
    }
}

/// CPU line batch collecting quads as wireframe boundary lines via MeshTarget.
pub struct CpuLineBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl CpuLineBatch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    /// Secondary constructor creating an empty line batch.
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
    pub unsafe fn upload_gl(&self, gl: &glow::Context) -> GlLineMesh {
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

        GlLineMesh::new(vao, vbo, ebo, self.indices.len() as i32)
    }
}

impl Mesh for CpuLineBatch {
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

impl MeshTarget for CpuLineBatch {
    fn push_quad(&mut self, quad: Quad) {
        let base_idx = self.vertices.len() as u32;
        quad.transfer(|v0, v1, v2, v3, normal, color| {
            let norm_arr = normal.to_array();
            let col_arr = color.to_array();
            self.vertices.push(Vertex::new(v0.to_array(), norm_arr, col_arr, 0.0));
            self.vertices.push(Vertex::new(v1.to_array(), norm_arr, col_arr, 0.0));
            self.vertices.push(Vertex::new(v2.to_array(), norm_arr, col_arr, 0.0));
            self.vertices.push(Vertex::new(v3.to_array(), norm_arr, col_arr, 0.0));
            let pairs = [(0, 1), (1, 2), (2, 3), (3, 0)];
            let pts = [v0, v1, v2, v3];
            for (a, b) in pairs {
                if (pts[a] - pts[b]).length_squared() > 1e-6 {
                    self.indices.push(base_idx + a as u32);
                    self.indices.push(base_idx + b as u32);
                }
            }
        });
    }
}

/// Combined CPU batch feeding both dark occluding triangles and vibrant wireframe lines.
pub struct CpuTronBatch {
    solid_batch: CpuMeshBatch,
    line_batch: CpuLineBatch,
}

impl CpuTronBatch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(solid_batch: CpuMeshBatch, line_batch: CpuLineBatch) -> Self {
        Self { solid_batch, line_batch }
    }

    /// Secondary constructor initializing empty solid and wireframe batches.
    pub fn empty() -> Self {
        Self::new(CpuMeshBatch::empty(), CpuLineBatch::empty())
    }

    /// # Safety
    /// Requires a valid, current OpenGL context.
    pub unsafe fn upload(&self, gl: &glow::Context) -> Box<dyn Mesh> {
        let solid_mesh = self.solid_batch.upload(gl);
        let line_mesh = self.line_batch.upload(gl);
        Box::new(TronMesh::new(solid_mesh, line_mesh))
    }
}

impl Mesh for CpuTronBatch {
    fn draw(&self, gl: &glow::Context) {
        unsafe {
            gl.enable(glow::POLYGON_OFFSET_FILL);
            gl.polygon_offset(1.0, 1.0);
        }
        self.solid_batch.draw(gl);
        unsafe {
            gl.disable(glow::POLYGON_OFFSET_FILL);
        }
        self.line_batch.draw(gl);
    }
}

impl MeshTarget for CpuTronBatch {
    fn push_quad(&mut self, quad: Quad) {
        self.line_batch.push_quad(quad);
        let dark_quad = quad.transfer(|v0, v1, v2, v3, normal, _| {
            Quad::new(v0, v1, v2, v3, normal, Vec3::new(0.01, 0.01, 0.02))
        });
        self.solid_batch.push_quad(dark_quad);
    }
}
