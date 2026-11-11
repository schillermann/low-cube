mod crown;

use glam::Vec3;
use crate::{MeshTarget, Quad};
pub use self::crown::MeadowCrown;

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
        let crown = MeadowCrown::new(self.coat.up, 7);
        let allow_relief = quad.radiate(|g| g >= -1e-4 && g < 0.1);
        let recolored = quad.transfer(|v0, v1, v2, v3, normal, _| {
            if normal.dot(self.coat.up) < -0.9 {
                self.bottom_emitted = true;
            }
            let top_surface = normal.dot(self.coat.up) > 0.25;
            let col = if top_surface { self.coat.top_color } else { self.coat.wall_color };
            let glow = if top_surface && crown.verdant(col) { 0.02 } else { 0.0 };
            Quad::glowing(v0, v1, v2, v3, normal, col, glow)
        });
        self.target.push_quad(recolored);
        if allow_relief {
            recolored.transfer(|v0, v1, v2, v3, normal, col| {
                if crown.upward_verdant(normal, col) {
                    crown.emit_contour_terraces([v0, v1, v2, v3], normal, col, true, self.target);
                    crown.emit_hybrid([v0, v1, v2, v3], normal, col, self.target);
                }
            });
        }
    }
}

/// Decorator over MeshTarget that sprouts Tiefen-Variante 1 3D contour terraces, original grass tufts, and overhanging turf rims.
pub struct NaturalMeadowTarget<'a> {
    target: &'a mut dyn MeshTarget,
    crown: MeadowCrown,
    top_col: Vec3,
    lod: u8,
}

impl<'a> NaturalMeadowTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn MeshTarget, crown: MeadowCrown, lod: u8) -> Self {
        Self { target, crown, top_col: Vec3::ZERO, lod }
    }

    /// Secondary constructor decorating a target for a specific grid cell and surface normal.
    pub fn for_cell(target: &'a mut dyn MeshTarget, cell: crate::block::GridCell, up: Vec3) -> Self {
        Self::new(target, MeadowCrown::for_cell(cell, up), 2)
    }

    /// Secondary constructor decorating a target with explicit detail level (`lod`: 2 = full 3D terraces + tufts, 1 = coarse terraces + tufts, 0 = tufts only).
    pub fn with_lod(target: &'a mut dyn MeshTarget, cell: crate::block::GridCell, up: Vec3, lod: u8) -> Self {
        Self::new(target, MeadowCrown::for_cell(cell, up), lod)
    }
}

impl MeshTarget for NaturalMeadowTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        if quad.radiate(|g| g >= -1e-4 && g < 0.1) {
            let crown = self.crown;
            let lod = self.lod;
            quad.transfer(|v0, v1, v2, v3, normal, col| {
                if crown.upward_verdant(normal, col) {
                    self.top_col = col;
                    if lod >= 1 {
                        crown.emit_contour_terraces([v0, v1, v2, v3], normal, col, lod >= 2, self.target);
                    }
                    crown.emit_natural_patch([v0, v1, v2, v3], lod >= 1, col, self.target);
                } else if lod >= 1 && crown.vertical_flank(normal, self.top_col) {
                    crown.emit_wall_turf([v0, v1, v2, v3], normal, self.top_col, lod >= 2, self.target);
                }
            });
        }
    }
}
