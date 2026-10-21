use glow::HasContext;
use crate::Mesh;

/// Decorator combining two GPU-renderable meshes into a unified draw sequence.
pub struct CompositeMesh<'a> {
    first: &'a dyn Mesh,
    second: &'a dyn Mesh,
}

impl<'a> CompositeMesh<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(first: &'a dyn Mesh, second: &'a dyn Mesh) -> Self {
        Self { first, second }
    }
}

impl<'a> Mesh for CompositeMesh<'a> {
    fn draw(&self, gl: &glow::Context) {
        self.first.draw(gl);
        self.second.draw(gl);
    }
}

/// Multi-face planetary GPU mesh enabling isolated single-face buffer updates without re-uploading untouched faces.
pub struct PlanetaryMesh {
    faces: Vec<Box<dyn Mesh>>,
}

impl PlanetaryMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(faces: Vec<Box<dyn Mesh>>) -> Self {
        Self { faces }
    }

    pub fn slice_at(&self, index: usize) -> &dyn Mesh {
        self.faces[index].as_ref()
    }

    pub fn store_slice(&mut self, index: usize, mesh: Box<dyn Mesh>) {
        if index < self.faces.len() {
            self.faces[index] = mesh;
        }
    }
}

impl Mesh for PlanetaryMesh {
    fn draw(&self, gl: &glow::Context) {
        for face in &self.faces {
            face.draw(gl);
        }
    }

    fn release(&self, gl: &glow::Context) {
        for face in &self.faces {
            face.release(gl);
        }
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

    fn release(&self, gl: &glow::Context) {
        self.occlusion_mesh.release(gl);
        self.line_mesh.release(gl);
    }
}
