use std::cell::RefCell;
use glow::HasContext;
use world::{MeshTarget, Quad};
use crate::{CpuMeshBatch, Mesh};

/// Persistent OpenGL VAO/VBO/EBO handles reused across frames via `STREAM_DRAW` buffer orphaning.
struct GlStreamSlot {
    vao: glow::VertexArray,
    buffers: (glow::Buffer, glow::Buffer),
    capacities: (usize, usize),
}

impl GlStreamSlot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(
        vao: glow::VertexArray,
        buffers: (glow::Buffer, glow::Buffer),
        capacities: (usize, usize),
    ) -> Self {
        Self { vao, buffers, capacities }
    }

    unsafe fn allocate(gl: &glow::Context) -> Self {
        let vao = gl.create_vertex_array().expect("Failed to create streaming VAO");
        let vbo = gl.create_buffer().expect("Failed to create streaming VBO");
        let ebo = gl.create_buffer().expect("Failed to create streaming EBO");
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        let stride = CpuMeshBatch::vertex_stride();
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 12);
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, stride, 24);
        gl.enable_vertex_attrib_array(3);
        gl.vertex_attrib_pointer_f32(3, 1, glow::FLOAT, false, stride, 36);
        gl.bind_vertex_array(None);
        Self::new(vao, (vbo, ebo), (0, 0))
    }

    unsafe fn stream_and_draw(&mut self, gl: &glow::Context, v_bytes: &[u8], i_bytes: &[u8], index_count: i32) {
        gl.bind_vertex_array(Some(self.vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.buffers.0));
        if v_bytes.len() > self.capacities.0 {
            let next_cap = v_bytes.len().next_power_of_two().max(4096);
            gl.buffer_data_size(glow::ARRAY_BUFFER, next_cap as i32, glow::STREAM_DRAW);
            self.capacities.0 = next_cap;
        } else {
            gl.buffer_data_size(glow::ARRAY_BUFFER, self.capacities.0 as i32, glow::STREAM_DRAW);
        }
        gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, v_bytes);

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.buffers.1));
        if i_bytes.len() > self.capacities.1 {
            let next_cap = i_bytes.len().next_power_of_two().max(4096);
            gl.buffer_data_size(glow::ELEMENT_ARRAY_BUFFER, next_cap as i32, glow::STREAM_DRAW);
            self.capacities.1 = next_cap;
        } else {
            gl.buffer_data_size(glow::ELEMENT_ARRAY_BUFFER, self.capacities.1 as i32, glow::STREAM_DRAW);
        }
        gl.buffer_sub_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, 0, i_bytes);

        gl.draw_elements(glow::TRIANGLES, index_count, glow::UNSIGNED_INT, 0);
        gl.bind_vertex_array(None);
    }

    unsafe fn dispose(&self, gl: &glow::Context) {
        gl.delete_vertex_array(self.vao);
        gl.delete_buffer(self.buffers.0);
        gl.delete_buffer(self.buffers.1);
    }
}

/// Polymorphic lifecycle state of `StreamingMesh` avoiding `Option<GlStreamSlot>`.
enum StreamState {
    Cold,
    Warm(GlStreamSlot),
}

/// Dynamic actor mesh reusing a single persistent GPU VAO/VBO/EBO across frames without driver handle churn.
pub struct StreamingMesh {
    batch: CpuMeshBatch,
    state: RefCell<StreamState>,
}

impl StreamingMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(batch: CpuMeshBatch, state: RefCell<StreamState>) -> Self {
        Self { batch, state }
    }

    pub fn empty() -> Self {
        Self::new(CpuMeshBatch::empty(), RefCell::new(StreamState::Cold))
    }

    pub fn with_quad_capacity(quads: usize) -> Self {
        Self::new(CpuMeshBatch::with_quad_capacity(quads), RefCell::new(StreamState::Cold))
    }

    pub fn clear(&mut self) {
        self.batch.clear();
    }
}

impl MeshTarget for StreamingMesh {
    fn push_quad(&mut self, quad: Quad) {
        self.batch.push_quad(quad);
    }
}

impl Mesh for StreamingMesh {
    fn draw(&self, gl: &glow::Context) {
        let (v_bytes, i_bytes, index_count) = self.batch.raw_slices();
        if index_count == 0 {
            return;
        }
        let mut state = self.state.borrow_mut();
        unsafe {
            if matches!(*state, StreamState::Cold) {
                *state = StreamState::Warm(GlStreamSlot::allocate(gl));
            }
            if let StreamState::Warm(ref mut slot) = *state {
                slot.stream_and_draw(gl, v_bytes, i_bytes, index_count);
            }
        }
    }

    fn release(&self, gl: &glow::Context) {
        let prev = std::mem::replace(&mut *self.state.borrow_mut(), StreamState::Cold);
        if let StreamState::Warm(slot) = prev {
            unsafe {
                slot.dispose(gl);
            }
        }
    }
}
