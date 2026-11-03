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

struct TurfCoat {
    up: Vec3,
    top_color: Vec3,
    wall_color: Vec3,
}

/// Decorator coloring upward canopy faces and vertical walls while sealing unemitted undersides.
pub struct TurfMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
    coat: TurfCoat,
    base_corners: [Vec3; 4],
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
        Self {
            target,
            coat: TurfCoat { up, top_color, wall_color },
            base_corners,
            bottom_emitted,
        }
    }

    /// Emits a sealing underside quad if the decorated shape did not emit its own bottom face.
    pub fn seal_underside(&mut self) {
        if !self.bottom_emitted {
            let [b0, b1, b2, b3] = self.base_corners;
            self.target.push_quad(Quad::new(b0, b3, b2, b1, -self.coat.up, self.coat.wall_color));
            self.bottom_emitted = true;
        }
    }
}

impl MeshTarget for TurfMeshTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let recolored = quad.transfer(|v0, v1, v2, v3, normal, _| {
            if normal.dot(self.coat.up) < -0.9 {
                self.bottom_emitted = true;
            }
            let col = if normal.dot(self.coat.up) > 0.25 { self.coat.top_color } else { self.coat.wall_color };
            Quad::new(v0, v1, v2, v3, normal, col)
        });
        self.target.push_quad(recolored);
    }
}
