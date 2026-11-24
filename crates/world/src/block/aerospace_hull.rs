use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::shape::VoxelShape;
use super::timber_grain::BlockMedium;

/// Sculpted 3D Expanse-class titanium keel plating and carbon-silicate ceramic composite hull armor.
#[derive(Clone, Copy, Debug)]
pub struct AerospacePlating {
    medium: BlockMedium,
    cross_grain: bool,
}

impl AerospacePlating {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(medium: BlockMedium, cross_grain: bool) -> Self {
        Self { medium, cross_grain }
    }

    pub const fn sculpted_tones(medium: BlockMedium) -> (Vec3, Vec3) {
        match medium {
            BlockMedium::CarbonSilicate => (Vec3::new(0.18, 0.26, 0.36), Vec3::new(0.13, 0.18, 0.26)),
            _ => (Vec3::new(0.58, 0.63, 0.70), Vec3::new(0.42, 0.47, 0.54)),
        }
    }

    pub fn emit_cube(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        if self.medium == BlockMedium::CarbonSilicate {
            self.emit_carbon_silicate_cube(cell, floor_elev, frame, target);
        } else {
            self.emit_titanium_cube(cell, floor_elev, frame, target);
        }
    }

    pub fn emit_shaped(
        &self,
        inner: &dyn VoxelShape,
        cell: GridCell,
        floor_elev: f32,
        frame: &SurfaceFrame,
        target: &mut dyn MeshTarget,
    ) {
        let (cap, wall) = Self::sculpted_tones(self.medium);
        let glow = if self.medium == BlockMedium::CarbonSilicate { 0.18 } else { 0.03 };
        let corners = cell.corner_points(frame, floor_elev);
        let mut coated = AerospaceFacetTarget {
            inner: target,
            up: frame.normal(),
            tones: (cap, wall, glow),
        };
        inner.emit_shape(cell, floor_elev, frame, wall, &mut coated);
        let [b0, b1, b2, b3] = corners;
        target.push_quad(Quad::glowing(b0, b3, b2, b1, -frame.normal(), wall * 0.88, glow * 0.5));
    }

    fn emit_titanium_cube(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top) = (cell.u_bounds(), cell.v_bounds(), floor_elev + 0.5);
        let seam = Vec3::new(0.19, 0.22, 0.27);
        let plate_a = Vec3::new(0.58, 0.63, 0.70);
        let plate_b = Vec3::new(0.49, 0.54, 0.61);
        let rivet = Vec3::new(0.77, 0.82, 0.89);

        BlockMedium::emit_box(
            u0 + 0.014, u1 - 0.014, v0 + 0.014, v1 - 0.014,
            floor_elev + 0.008, top - 0.004, frame, seam, seam, true, target,
        );
        let mid_h = floor_elev + 0.25;
        BlockMedium::emit_box(u0, u1, v0, v1, floor_elev + 0.012, mid_h - 0.010, frame, plate_b, plate_b, true, target);
        BlockMedium::emit_box(u0, u1, v0, v1, mid_h + 0.010, top - 0.008, frame, plate_a, plate_a, false, target);

        let (um, vm, pad) = ((u0 + u1) * 0.5, (v0 + v1) * 0.5, 0.012);
        if self.cross_grain {
            BlockMedium::emit_box(u0 + pad, u1 - pad, v0 + pad, vm - 0.009, top - 0.006, top + 0.008, frame, plate_a, plate_a, false, target);
            BlockMedium::emit_box(u0 + pad, u1 - pad, vm + 0.009, v1 - pad, top - 0.006, top + 0.008, frame, plate_b, plate_b, false, target);
        } else {
            BlockMedium::emit_box(u0 + pad, um - 0.009, v0 + pad, v1 - pad, top - 0.006, top + 0.008, frame, plate_a, plate_a, false, target);
            BlockMedium::emit_box(um + 0.009, u1 - pad, v0 + pad, v1 - pad, top - 0.006, top + 0.008, frame, plate_b, plate_b, false, target);
        }
        for (ru, rv) in [(u0 + 0.055, v0 + 0.055), (u1 - 0.055, v0 + 0.055), (u1 - 0.055, v1 - 0.055), (u0 + 0.055, v1 - 0.055)] {
            BlockMedium::emit_box(ru - 0.016, ru + 0.016, rv - 0.016, rv + 0.016, top + 0.006, top + 0.014, frame, rivet, rivet, false, target);
        }
    }

    fn emit_carbon_silicate_cube(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top) = (cell.u_bounds(), cell.v_bounds(), floor_elev + 0.5);
        let lace = Vec3::new(0.24, 0.78, 0.98);
        let cermet_dark = Vec3::new(0.13, 0.17, 0.24);
        let cermet_slate = Vec3::new(0.18, 0.24, 0.33);

        Self::emit_glowing_box(
            u0 + 0.012, u1 - 0.012, v0 + 0.012, v1 - 0.012,
            floor_elev + 0.006, top - 0.004, frame, lace, 0.42, true, target,
        );
        let mid_h = floor_elev + 0.25;
        BlockMedium::emit_box(u0, u1, v0, v1, floor_elev + 0.012, mid_h - 0.011, frame, cermet_dark, cermet_dark, true, target);
        BlockMedium::emit_box(u0, u1, v0, v1, mid_h + 0.011, top - 0.008, frame, cermet_slate, cermet_slate, false, target);

        let (um, vm, pad, gap) = ((u0 + u1) * 0.5, (v0 + v1) * 0.5, 0.012, 0.010);
        let tiles = if self.cross_grain {
            [
                (u0 + pad, um + 0.04, v0 + pad, vm - gap, cermet_slate),
                (um + 0.04 + gap * 2.0, u1 - pad, v0 + pad, vm - gap, cermet_dark),
                (u0 + pad, um - 0.04 - gap * 2.0, vm + gap, v1 - pad, cermet_dark),
                (um - 0.04, u1 - pad, vm + gap, v1 - pad, cermet_slate),
            ]
        } else {
            [
                (u0 + pad, um - gap, v0 + pad, vm + 0.04, cermet_slate),
                (u0 + pad, um - gap, vm + 0.04 + gap * 2.0, v1 - pad, cermet_dark),
                (um + gap, u1 - pad, v0 + pad, vm - 0.04 - gap * 2.0, cermet_dark),
                (um + gap, u1 - pad, vm - 0.04, v1 - pad, cermet_slate),
            ]
        };
        for (tu0, tu1, tv0, tv1, col) in tiles {
            Self::emit_glowing_box(tu0, tu1, tv0, tv1, top - 0.005, top + 0.009, frame, col, 0.10, false, target);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_glowing_box(
        u0: f32, u1: f32, v0: f32, v1: f32, h0: f32, h1: f32,
        frame: &SurfaceFrame, col: Vec3, glow: f32, bottom_cap: bool, target: &mut dyn MeshTarget,
    ) {
        let (n, t, b) = (frame.normal(), frame.tangent(), frame.bitangent());
        let [b0, b1, b2, b3] = [
            frame.point_at(u0, v0, h0), frame.point_at(u1, v0, h0),
            frame.point_at(u1, v1, h0), frame.point_at(u0, v1, h0),
        ];
        let [p0, p1, p2, p3] = [
            frame.point_at(u0, v0, h1), frame.point_at(u1, v0, h1),
            frame.point_at(u1, v1, h1), frame.point_at(u0, v1, h1),
        ];
        let wall = col * 0.86;
        target.push_quad(Quad::glowing(p0, p1, p2, p3, n, col, glow));
        if bottom_cap {
            target.push_quad(Quad::glowing(b0, b3, b2, b1, -n, wall, glow));
        }
        target.push_quad(Quad::glowing(b0, b1, p1, p0, -b, wall, glow));
        target.push_quad(Quad::glowing(p3, p2, b2, b3, b, wall, glow));
        target.push_quad(Quad::glowing(b3, b0, p0, p3, -t, wall, glow));
        target.push_quad(Quad::glowing(b1, b2, p2, p1, t, wall, glow));
    }
}

struct AerospaceFacetTarget<'a> {
    inner: &'a mut dyn MeshTarget,
    up: Vec3,
    tones: (Vec3, Vec3, f32),
}

impl MeshTarget for AerospaceFacetTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let (cap, wall, glow) = self.tones;
        let up = self.up;
        self.inner.push_quad(quad.transfer(|v0, v1, v2, v3, norm, _| {
            let tone = if norm.dot(up) > 0.25 { cap } else { wall };
            Quad::glowing(v0, v1, v2, v3, norm, tone, glow)
        }));
    }
}
