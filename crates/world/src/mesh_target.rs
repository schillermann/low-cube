use glam::Vec3;
use crate::{MeshTarget, Quad};

/// Decorator counting the number of quads pushed to an underlying target.
pub struct CountedMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
    count: usize,
}

impl<'a> CountedMeshTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn MeshTarget) -> Self {
        Self { target, count: 0 }
    }

    /// Number of quads successfully pushed through this decorator.
    pub fn count(&self) -> usize {
        self.count
    }
}

impl<'a> MeshTarget for CountedMeshTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        self.count += 1;
        self.target.push_quad(quad);
    }
}

/// Decorator overriding quad coloration with an encapsulated material color.
pub struct ColoredMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
    color: Vec3,
}

impl<'a> ColoredMeshTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn MeshTarget, color: Vec3) -> Self {
        Self { target, color }
    }
}

impl<'a> MeshTarget for ColoredMeshTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let recolored = quad.transfer(|v0, v1, v2, v3, normal, _| {
            Quad::new(v0, v1, v2, v3, normal, self.color)
        });
        self.target.push_quad(recolored);
    }
}

/// Decorator scaling quad vertex colors by an encapsulated tint factor.
pub struct TintedMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
    tint: Vec3,
}

impl<'a> TintedMeshTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn MeshTarget, tint: Vec3) -> Self {
        Self { target, tint }
    }
}

impl<'a> MeshTarget for TintedMeshTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let tinted = quad.transfer(|v0, v1, v2, v3, normal, color| {
            Quad::new(v0, v1, v2, v3, normal, color * self.tint)
        });
        self.target.push_quad(tinted);
    }
}

/// Decorator translating quad vertex positions by an encapsulated spatial offset.
pub struct OffsetMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
    offset: Vec3,
}

impl<'a> OffsetMeshTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn MeshTarget, offset: Vec3) -> Self {
        Self { target, offset }
    }
}

impl<'a> MeshTarget for OffsetMeshTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let shifted = quad.transfer(|v0, v1, v2, v3, normal, color| {
            Quad::new(
                v0 + self.offset,
                v1 + self.offset,
                v2 + self.offset,
                v3 + self.offset,
                normal,
                color,
            )
        });
        self.target.push_quad(shifted);
    }
}

/// Decorator subdividing pushed quads into recessed 3D digging craters for stages 1..=3.
pub struct CrateredMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
    stage: u8,
}

impl<'a> CrateredMeshTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn MeshTarget, stage: u8) -> Self {
        Self { target, stage }
    }
}

impl MeshTarget for CrateredMeshTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        quad.emit_crater(self.stage, self.target);
    }
}

/// Decorator coloring upward canopy faces and vertical walls while sealing unemitted undersides.
pub struct TurfMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
    up: Vec3,
    base_corners: [Vec3; 4],
    top_color: Vec3,
    wall_color: Vec3,
    bottom_emitted: bool,
}

impl<'a> TurfMeshTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        target: &'a mut dyn MeshTarget,
        up: Vec3,
        base_corners: [Vec3; 4],
        top_color: Vec3,
        wall_color: Vec3,
        bottom_emitted: bool,
    ) -> Self {
        Self { target, up, base_corners, top_color, wall_color, bottom_emitted }
    }

    /// Emits a sealing underside quad if the decorated shape did not emit its own bottom face.
    pub fn seal_underside(&mut self) {
        if !self.bottom_emitted {
            let [b0, b1, b2, b3] = self.base_corners;
            self.target.push_quad(Quad::new(b0, b3, b2, b1, -self.up, self.wall_color));
            self.bottom_emitted = true;
        }
    }
}

impl MeshTarget for TurfMeshTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let recolored = quad.transfer(|v0, v1, v2, v3, normal, _| {
            if normal.dot(self.up) < -0.9 {
                self.bottom_emitted = true;
            }
            let col = if normal.dot(self.up) > 0.25 { self.top_color } else { self.wall_color };
            Quad::new(v0, v1, v2, v3, normal, col)
        });
        self.target.push_quad(recolored);
    }
}

