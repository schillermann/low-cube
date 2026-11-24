use crate::{Mesh, ShaderProgram};

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

    fn draw_posed(&self, gl: &glow::Context, shader: &dyn ShaderProgram) {
        self.first.draw_posed(gl, shader);
        self.second.draw_posed(gl, shader);
    }
}

/// Multi-face planetary GPU mesh enabling isolated single-face buffer updates and horizon slice culling.
pub struct PlanetaryMesh {
    faces: Vec<Box<dyn Mesh>>,
    visible_mask: u32,
    uploaded_mask: u32,
}

impl PlanetaryMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_horizon(faces: Vec<Box<dyn Mesh>>, visible_mask: u32, uploaded_mask: u32) -> Self {
        Self { faces, visible_mask, uploaded_mask }
    }

    pub fn with_mask(faces: Vec<Box<dyn Mesh>>, visible_mask: u32) -> Self {
        Self::with_horizon(faces, visible_mask, u32::MAX)
    }

    /// Secondary constructor enabling all slices by default.
    pub fn new(faces: Vec<Box<dyn Mesh>>) -> Self {
        Self::with_mask(faces, u32::MAX)
    }

    pub fn select_visible(&mut self, visible_mask: u32) {
        self.visible_mask = visible_mask;
    }

    pub fn reset_uploaded(&mut self) {
        self.uploaded_mask = 0;
    }

    pub fn mark_uploaded(&mut self, index: usize) {
        if index < 32 {
            self.uploaded_mask |= 1u32 << index;
        }
    }

    pub fn missing_slices(&self, visible_mask: u32, slice_limit: usize) -> u32 {
        let cap_mask = if slice_limit >= 32 { u32::MAX } else { (1u32 << slice_limit) - 1 };
        (visible_mask & cap_mask) & !self.uploaded_mask
    }

    pub fn ensure_slice(&mut self, index: usize, make_empty: impl Fn() -> Box<dyn Mesh>) {
        while self.faces.len() <= index {
            self.faces.push(make_empty());
        }
    }

    pub fn slice_at(&self, index: usize) -> &dyn Mesh {
        self.faces[index].as_ref()
    }

    pub fn store_slice(&mut self, index: usize, mesh: Box<dyn Mesh>) {
        if index < self.faces.len() {
            self.faces[index] = mesh;
        }
    }

    pub fn patch_at(&self, slice: usize, slot: usize) -> Option<&dyn Mesh> {
        self.faces.get(slice).and_then(|m| m.patch_at(slot))
    }

    pub fn store_patch(&mut self, slice: usize, slot: usize, patch: Box<dyn Mesh>) {
        if let Some(face) = self.faces.get_mut(slice) {
            face.store_patch(slot, patch);
        }
    }
}

impl Mesh for PlanetaryMesh {
    fn draw(&self, gl: &glow::Context) {
        self.draw_excluding(gl, usize::MAX);
    }

    fn release(&self, gl: &glow::Context) {
        for face in &self.faces {
            face.release(gl);
        }
    }
}

impl PlanetaryMesh {
    fn draw_excluding(&self, gl: &glow::Context, excluded: usize) {
        for (idx, face) in self.faces.iter().enumerate() {
            if idx != excluded && (self.visible_mask >> idx.min(31)) & 1 != 0 {
                face.draw(gl);
            }
        }
    }
}

/// `Mesh` decorator omitting a single fine-detail slice (such as `MeadowRing`) during coarse preview passes.
pub struct CoarseMesh<'a> {
    planet: &'a PlanetaryMesh,
    excluded_slice: usize,
}

impl<'a> CoarseMesh<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(planet: &'a PlanetaryMesh, excluded_slice: usize) -> Self {
        Self { planet, excluded_slice }
    }
}

impl Mesh for CoarseMesh<'_> {
    fn draw(&self, gl: &glow::Context) {
        self.planet.draw_excluding(gl, self.excluded_slice);
    }
}
