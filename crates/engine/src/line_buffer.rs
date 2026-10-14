use glam::Vec3;
use glow::HasContext;
use world::{MeshTarget, Quad};
use crate::buffer::{CpuMeshBatch, Vertex};
use crate::Mesh;

/// Encapsulated GPU line mesh rendering GL_LINES elements.
pub struct GlLineMesh {
    vao: glow::VertexArray,
    _vbo: glow::Buffer,
    _ebo: glow::Buffer,
    index_count: i32,
}

impl GlLineMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        vao: glow::VertexArray,
        vbo: glow::Buffer,
        ebo: glow::Buffer,
        index_count: i32,
    ) -> Self {
        Self {
            vao,
            _vbo: vbo,
            _ebo: ebo,
            index_count,
        }
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

    /// Uploads vertex and line index buffers to the GPU.
    pub unsafe fn upload(&self, gl: &glow::Context) -> Box<dyn Mesh> {
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

        Box::new(GlLineMesh::new(vao, vbo, ebo, self.indices.len() as i32))
    }
}

impl MeshTarget for CpuLineBatch {
    fn push_quad(&mut self, quad: Quad) {
        let base_idx = self.vertices.len() as u32;

        quad.transfer(|v0, v1, v2, v3, normal, color| {
            let norm_arr = normal.to_array();
            let col_arr = color.to_array();

            self.vertices.push(Vertex::new(v0.to_array(), norm_arr, col_arr));
            self.vertices.push(Vertex::new(v1.to_array(), norm_arr, col_arr));
            self.vertices.push(Vertex::new(v2.to_array(), norm_arr, col_arr));
            self.vertices.push(Vertex::new(v3.to_array(), norm_arr, col_arr));

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

/// Composite GPU mesh combining occluding dark facets and glowing neon wireframe lines.
pub struct TronMesh {
    occlusion_mesh: Box<dyn Mesh>,
    line_mesh: Box<dyn Mesh>,
}

impl TronMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(occlusion_mesh: Box<dyn Mesh>, line_mesh: Box<dyn Mesh>) -> Self {
        Self {
            occlusion_mesh,
            line_mesh,
        }
    }
}

impl Mesh for TronMesh {
    fn draw(&self, gl: &glow::Context) {
        unsafe {
            gl.enable(glow::POLYGON_OFFSET_FILL);
            gl.polygon_offset(1.0, 1.0);
        }
        self.occlusion_mesh.draw(gl);
        unsafe {
            gl.disable(glow::POLYGON_OFFSET_FILL);
        }
        self.line_mesh.draw(gl);
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
        Self {
            solid_batch,
            line_batch,
        }
    }

    /// Secondary constructor initializing empty solid and wireframe batches.
    pub fn empty() -> Self {
        Self::new(CpuMeshBatch::empty(), CpuLineBatch::empty())
    }

    /// Uploads both dark occlusion facets and vibrant shape lines to the GPU.
    pub unsafe fn upload(&self, gl: &glow::Context) -> Box<dyn Mesh> {
        let solid_mesh = self.solid_batch.upload(gl);
        let line_mesh = self.line_batch.upload(gl);
        Box::new(TronMesh::new(solid_mesh, line_mesh))
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
