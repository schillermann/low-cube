use glam::{Mat4, Vec3, Vec4};
use glow::HasContext;
use engine::Mesh;

/// Abstraction for rendering the celestial sky vault.
pub trait SkyFirmament: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    fn draw_firmament(
        &self,
        gl: &glow::Context,
        inv_view_proj: &Mat4,
        sun_dir: Vec3,
        celestial_pole: Vec3,
        local_up: Vec3,
        sky_color: Vec4,
        night_factor: f32,
        aurora_factor: f32,
        time: f32,
        mesh: &dyn Mesh,
    );
}

/// Null object pattern firmament performing no GPU operations.
#[cfg(test)]
pub struct VoidFirmament;

#[cfg(test)]
impl VoidFirmament {
    /// Primary constructor doing nothing except creating instance.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
impl SkyFirmament for VoidFirmament {
    fn draw_firmament(
        &self,
        _gl: &glow::Context,
        _inv_view_proj: &Mat4,
        _sun_dir: Vec3,
        _celestial_pole: Vec3,
        _local_up: Vec3,
        _sky_color: Vec4,
        _night_factor: f32,
        _aurora_factor: f32,
        _time: f32,
        _mesh: &dyn Mesh,
    ) {}
}

/// Null object pattern mesh performing no draw operations.
#[cfg(test)]
pub struct VoidMesh;

#[cfg(test)]
impl VoidMesh {
    /// Primary constructor doing nothing except creating instance.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
impl Mesh for VoidMesh {
    fn draw(&self, _gl: &glow::Context) {}
}

/// Fullscreen NDC quad mesh for firmament rendering at the far clipping plane.
pub struct GlFirmamentQuad {
    vao: glow::VertexArray,
    _vbo: glow::Buffer,
    _ebo: glow::Buffer,
}

impl GlFirmamentQuad {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        vao: glow::VertexArray,
        vbo: glow::Buffer,
        ebo: glow::Buffer,
    ) -> Self {
        Self {
            vao,
            _vbo: vbo,
            _ebo: ebo,
        }
    }

    /// Secondary constructor uploading fullscreen quad geometry to GPU memory.
    pub unsafe fn uploaded(gl: &glow::Context) -> Box<dyn Mesh> {
        let vertices: [f32; 8] = [
            -1.0, -1.0,
             1.0, -1.0,
             1.0,  1.0,
            -1.0,  1.0,
        ];
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

        Box::new(Self::new(vao, vbo, ebo))
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
}
