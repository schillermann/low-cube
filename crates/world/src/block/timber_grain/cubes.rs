use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::super::cell::GridCell;
use super::BlockMedium;

/// Sculpted full-cube mesh emitter for barked timber, crust rock, and grooved planks (`<= 4` fields).
pub(super) struct GrainCubeMesh {
    cell: GridCell,
    floor_elev: f32,
}

impl GrainCubeMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(cell: GridCell, floor_elev: f32) -> Self {
        Self { cell, floor_elev }
    }

    /// Emits Variant 1A Timber Block: Bark-ridged timber block with bright sapwood and concentric annual rings.
    pub(super) fn emit_raw_timber(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top_elev) = (self.cell.u_bounds(), self.cell.v_bounds(), self.floor_elev + 0.5);
        let (bark_dark, bark_ridge, sapwood, ring_dark, heartwood) = (
            Vec3::new(0.32, 0.20, 0.11),
            Vec3::new(0.44, 0.28, 0.16),
            Vec3::new(0.85, 0.70, 0.46),
            Vec3::new(0.64, 0.47, 0.27),
            Vec3::new(0.48, 0.32, 0.17),
        );
        BlockMedium::emit_box(u0, u1, v0, v1, self.floor_elev, top_elev, frame, bark_dark, sapwood, true, target);
        let (r_h0, r_h1, d, w) = (self.floor_elev + 0.01, top_elev - 0.01, 0.014, 0.040);
        for step in 0..3 {
            let center_u = u0 + (0.20 + (step as f32) * 0.30) * (u1 - u0);
            let center_v = v0 + (0.20 + (step as f32) * 0.30) * (v1 - v0);
            BlockMedium::emit_box(center_u - w, center_u + w, v1, v1 + d, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
            BlockMedium::emit_box(center_u - w, center_u + w, v0 - d, v0, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
            BlockMedium::emit_box(u1, u1 + d, center_v - w, center_v + w, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
            BlockMedium::emit_box(u0 - d, u0, center_v - w, center_v + w, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
        }
        for (inset, h_off, col) in [
            (0.035, 0.000, sapwood),
            (0.105, 0.006, ring_dark),
            (0.165, 0.012, sapwood),
            (0.212, 0.018, heartwood),
        ] {
            BlockMedium::emit_box(u0 + inset, u1 - inset, v0 + inset, v1 - inset, top_elev + h_off, top_elev + h_off + 0.006, frame, col, col, false, target);
        }
    }

    /// Emits a full-cube planetary crust rock block with dark craggy relief and glowing mineral seams.
    pub(super) fn emit_crust_rock(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let vault = crate::CrustVault::beneath(64);
        let geom = vault.geometry(frame);
        let ((u0, u1), (v0, v1), y0, y1) = (self.cell.u_bounds(), self.cell.v_bounds(), self.floor_elev, self.floor_elev + 0.5);
        let layer = (self.floor_elev * 2.0).round() as i32;
        geom.emit_crust_floor(self.cell, y1, false, target);
        geom.emit_crust_sole(self.cell, y0, target);
        for (edge, norm) in [
            (((u0, v1), (u1, v1)), frame.bitangent()),
            (((u1, v0), (u0, v0)), -frame.bitangent()),
            (((u1, v0), (u1, v1)), frame.tangent()),
            (((u0, v1), (u0, v0)), -frame.tangent()),
        ] {
            geom.emit_crust_flank(edge, layer, layer - 2, norm, target);
        }
    }

    pub(super) fn emit_planed_planks(&self, cross_grain: bool, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top_elev) = (self.cell.u_bounds(), self.cell.v_bounds(), self.floor_elev + 0.5);
        let groove_core = Vec3::new(0.22, 0.14, 0.08);
        let board_tones = [
            Vec3::new(0.86, 0.69, 0.43),
            Vec3::new(0.76, 0.58, 0.34),
            Vec3::new(0.82, 0.65, 0.39),
            Vec3::new(0.73, 0.55, 0.32),
        ];
        BlockMedium::emit_box(u0 + 0.016, u1 - 0.016, v0 + 0.016, v1 - 0.016, self.floor_elev + 0.008, top_elev - 0.002, frame, groove_core, groove_core, true, target);
        let (bh, v_pad, bw, h_pad) = (0.125, 0.009, 0.125, 0.009);
        for (i, &tone) in board_tones.iter().enumerate() {
            let (y0, y1) = (self.floor_elev + (i as f32) * bh + v_pad, self.floor_elev + ((i + 1) as f32) * bh - v_pad);
            BlockMedium::emit_box(u0, u1, v0, v1, y0, y1, frame, tone, tone, true, target);
        }
        for i in 0..4 {
            let tone = board_tones[(i + 1) % 4];
            if cross_grain {
                let (pv0, pv1) = (v0 + (i as f32) * bw + h_pad, v0 + ((i + 1) as f32) * bw - h_pad);
                BlockMedium::emit_box(u0, u1, pv0, pv1, top_elev - 0.002, top_elev + 0.010, frame, tone, tone, false, target);
            } else {
                let (pu0, pu1) = (u0 + (i as f32) * bw + h_pad, u0 + ((i + 1) as f32) * bw - h_pad);
                BlockMedium::emit_box(pu0, pu1, v0, v1, top_elev - 0.002, top_elev + 0.010, frame, tone, tone, false, target);
            }
        }
    }
}
