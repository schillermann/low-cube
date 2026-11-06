use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::shape::VoxelShape;
use super::timber_grain::BlockMedium;
use super::wedge::WedgeHeading;

/// Oriented 3D geometry emitter for richly upholstered flat-top seat blocks (`FullCube` & `HalfSlab`).
pub struct CabinSeatMesh {
    heading: WedgeHeading,
}

impl CabinSeatMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(heading: WedgeHeading) -> Self {
        Self { heading }
    }

    pub fn emit(
        &self,
        shape: &dyn VoxelShape,
        cell: GridCell,
        floor_elev: f32,
        frame: &SurfaceFrame,
        target: &mut dyn MeshTarget,
    ) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let span_h = shape.elevation_at(0.5, 0.5).max(0.25);
        let top_h = floor_elev + span_h;
        let plinth_h = floor_elev + (span_h * 0.16).clamp(0.04, 0.065);
        let (walnut, stitch, leather_base, leather_mid, leather_crown, brass, glow_col) = (
            Vec3::new(0.32, 0.18, 0.11),
            Vec3::new(0.38, 0.10, 0.08),
            Vec3::new(0.64, 0.16, 0.13),
            Vec3::new(0.76, 0.22, 0.17),
            Vec3::new(0.85, 0.29, 0.22),
            Vec3::new(0.92, 0.74, 0.24),
            Vec3::new(0.99, 0.82, 0.28),
        );

        // 1. Dark walnut base plinth + brass trim welt + deep stitch-channel core
        BlockMedium::emit_box(u0, u1, v0, v1, floor_elev, plinth_h, frame, walnut, walnut, true, target);
        BlockMedium::emit_box(u0 - 0.004, u1 + 0.004, v0 - 0.004, v1 + 0.004, plinth_h - 0.006, plinth_h + 0.006, frame, brass, brass, false, target);
        BlockMedium::emit_box(u0 + 0.008, u1 - 0.008, v0 + 0.008, v1 - 0.008, plinth_h, top_h - 0.006, frame, stitch, stitch, false, target);

        // 2. Plump upholstered leather body & side cushion bolsters
        self.emit_side_upholstery(cell, plinth_h, top_h, frame, leather_base, leather_mid, brass, target);

        // 3. Soft perimeter cushion rim + 4 plump two-tier beveled top pillows
        BlockMedium::emit_box(u0 + 0.012, u1 - 0.012, v0 + 0.012, v1 - 0.012, top_h - 0.016, top_h + 0.008, frame, leather_base, stitch, false, target);
        self.emit_quilted_pads(cell, top_h, frame, leather_mid, leather_crown, stitch, target);

        // 4. Sunken brass tufting buttons + upholstered front thigh-bolster roll & direction crest
        self.emit_tuft_buttons(cell, top_h, frame, brass, glow_col, target);
        self.emit_front_bolster(cell, top_h, frame, leather_crown, brass, glow_col, target);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_side_upholstery(
        &self,
        cell: GridCell,
        plinth_h: f32,
        top_h: f32,
        frame: &SurfaceFrame,
        leather_base: Vec3,
        leather_mid: Vec3,
        brass: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let body_span = top_h - plinth_h;
        let bands: &[(f32, f32)] = if body_span > 0.30 {
            &[(0.05, 0.47), (0.53, 0.95)]
        } else {
            &[(0.08, 0.92)]
        };
        for &(f0, f1) in bands {
            let (z0, z1) = (plinth_h + body_span * f0, plinth_h + body_span * f1);
            let (zm0, zm1) = (z0 + (z1 - z0) * 0.16, z1 - (z1 - z0) * 0.16);
            // Tier 1 wrap around all 4 sides
            BlockMedium::emit_box(u0 - 0.006, u1 + 0.006, v0 + 0.032, v1 - 0.032, z0, z1, frame, leather_base, leather_mid, false, target);
            BlockMedium::emit_box(u0 + 0.032, u1 - 0.032, v0 - 0.006, v1 + 0.006, z0, z1, frame, leather_base, leather_mid, false, target);
            // Tier 2 plump center side pillows on all 4 sides
            BlockMedium::emit_box(u0 - 0.015, u1 + 0.015, v0 + 0.075, v1 - 0.075, zm0, zm1, frame, leather_mid, leather_mid, false, target);
            BlockMedium::emit_box(u0 + 0.075, u1 - 0.075, v0 - 0.015, v1 + 0.015, zm0, zm1, frame, leather_mid, leather_mid, false, target);
        }
        // Corner leather columns & brass upholstery tacks
        for (cu0, cu1) in [(u0, u0 + 0.042), (u1 - 0.042, u1)] {
            for (cv0, cv1) in [(v0, v0 + 0.042), (v1 - 0.042, v1)] {
                BlockMedium::emit_box(cu0, cu1, cv0, cv1, plinth_h + 0.006, top_h - 0.004, frame, leather_base, leather_base, false, target);
                BlockMedium::emit_box(cu0 - 0.003, cu1 + 0.003, cv0 - 0.003, cv1 + 0.003, top_h - 0.036, top_h - 0.016, frame, brass, brass, false, target);
            }
        }
    }

    fn emit_quilted_pads(
        &self,
        cell: GridCell,
        top_h: f32,
        frame: &SurfaceFrame,
        pad_mid: Vec3,
        pad_crown: Vec3,
        seam_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (du, dv) = (u1 - u0, v1 - v0);
        for (fu0, fu1) in [(0.07f32, 0.47f32), (0.53f32, 0.93f32)] {
            for (fv0, fv1) in [(0.07f32, 0.47f32), (0.53f32, 0.93f32)] {
                let (pu0, pu1, pv0, pv1) = (u0 + fu0 * du, u0 + fu1 * du, v0 + fv0 * dv, v0 + fv1 * dv);
                // Lower plump cushion tier
                BlockMedium::emit_box(pu0, pu1, pv0, pv1, top_h + 0.004, top_h + 0.024, frame, seam_col, pad_mid, false, target);
                // Upper beveled pillow crown
                BlockMedium::emit_box(pu0 + 0.022, pu1 - 0.022, pv0 + 0.022, pv1 - 0.022, top_h + 0.022, top_h + 0.038, frame, pad_mid, pad_crown, false, target);
            }
        }
    }

    fn emit_tuft_buttons(
        &self,
        cell: GridCell,
        top_h: f32,
        frame: &SurfaceFrame,
        brass: Vec3,
        glow_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (du, dv, r) = (u1 - u0, v1 - v0, 0.019);
        let n = frame.normal();
        for (fu, fv, z_top) in [
            (0.27f32, 0.27f32, top_h + 0.044),
            (0.73f32, 0.27f32, top_h + 0.044),
            (0.27f32, 0.73f32, top_h + 0.044),
            (0.73f32, 0.73f32, top_h + 0.044),
            (0.50f32, 0.50f32, top_h + 0.026),
        ] {
            let (cu, cv) = (u0 + fu * du, v0 + fv * dv);
            BlockMedium::emit_box(cu - r, cu + r, cv - r, cv + r, top_h + 0.006, z_top, frame, brass, glow_col, false, target);
            let [p0, p1, p2, p3] = [
                frame.point_at(cu - r, cv - r, z_top + 0.001),
                frame.point_at(cu + r, cv - r, z_top + 0.001),
                frame.point_at(cu + r, cv + r, z_top + 0.001),
                frame.point_at(cu - r, cv + r, z_top + 0.001),
            ];
            target.push_quad(Quad::glowing(p0, p1, p2, p3, n, glow_col, 0.78));
        }
    }

    fn emit_front_bolster(
        &self,
        cell: GridCell,
        top_h: f32,
        frame: &SurfaceFrame,
        leather_crown: Vec3,
        brass: Vec3,
        glow_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let n = frame.normal();
        let (ru0, ru1, rv0, rv1, bu0, bu1, bv0, bv1) = match self.heading {
            WedgeHeading::East => (u1 - 0.068, u1 + 0.010, v0 + 0.05, v1 - 0.05, u1 - 0.052, u1 + 0.014, v0 + 0.14, v1 - 0.14),
            WedgeHeading::West => (u0 - 0.010, u0 + 0.068, v0 + 0.05, v1 - 0.05, u0 - 0.014, u0 + 0.052, v0 + 0.14, v1 - 0.14),
            WedgeHeading::North => (u0 + 0.05, u1 - 0.05, v1 - 0.068, v1 + 0.010, u0 + 0.14, u1 - 0.14, v1 - 0.052, v1 + 0.014),
            WedgeHeading::South => (u0 + 0.05, u1 - 0.05, v0 - 0.010, v0 + 0.068, u0 + 0.14, u1 - 0.14, v0 - 0.014, v0 + 0.052),
        };
        // Plump leather front thigh bolster roll + glowing brass directional crest
        BlockMedium::emit_box(ru0, ru1, rv0, rv1, top_h - 0.022, top_h + 0.042, frame, leather_crown, leather_crown, false, target);
        BlockMedium::emit_box(bu0, bu1, bv0, bv1, top_h - 0.032, top_h + 0.048, frame, brass, glow_col, true, target);
        let [p0, p1, p2, p3] = [
            frame.point_at(bu0, bv0, top_h + 0.049),
            frame.point_at(bu1, bv0, top_h + 0.049),
            frame.point_at(bu1, bv1, top_h + 0.049),
            frame.point_at(bu0, bv1, top_h + 0.049),
        ];
        target.push_quad(Quad::glowing(p0, p1, p2, p3, n, glow_col, 0.88));
    }
}

