use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{BlockMedium, GridCell};
use crate::frame::SurfaceFrame;

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Polyhedral 3D geometry representation of an unfolded cantilever crane (`Galgen-Kranarm`).
pub struct RigGeometry {
    ground_top: f32,
    hoist_elev: f32,
    reach_steps: i32,
    turn: u8,
}

impl RigGeometry {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ground_top: f32, hoist_elev: f32, reach_steps: i32, turn: u8) -> Self {
        Self {
            ground_top,
            hoist_elev,
            reach_steps,
            turn,
        }
    }

    const fn step_vectors(&self) -> (i32, i32) {
        match self.turn & 3 {
            0 => (1, 0),
            1 => (0, 1),
            2 => (-1, 0),
            _ => (0, -1),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_sloped_beam(
        p0: Vec3,
        p1: Vec3,
        side: Vec3,
        half_w: f32,
        half_h: f32,
        top_col: Vec3,
        flank_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let axis = (p1 - p0).normalize_or_zero();
        let ortho = side.cross(axis).normalize_or_zero();
        let w = side * half_w;
        let h = ortho * half_h;

        let [b0, b1, b2, b3] = [p0 - w - h, p0 + w - h, p0 + w + h, p0 - w + h];
        let [t0, t1, t2, t3] = [p1 - w - h, p1 + w - h, p1 + w + h, p1 - w + h];

        target.push_quad(Quad::new(b3, b2, t2, t3, ortho, top_col));
        target.push_quad(Quad::new(b0, t0, t1, b1, -ortho, flank_col));
        target.push_quad(Quad::new(b1, t1, t2, b2, side, flank_col));
        target.push_quad(Quad::new(b3, t3, t0, b0, -side, flank_col));
    }

    /// Emits the sculpted non-block polyhedral 3D crane geometry (`Galgen-Kranarm`).
    pub fn emit(&self, cell: GridCell, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (cx, cy) = cell.center();
        let (du, dv) = self.step_vectors();
        let (dir_u, dir_v) = (du as f32, dv as f32);
        let (perp_u, perp_v) = (-dir_v, dir_u);
        let base_z = self.ground_top;
        let hoist_z = self.hoist_elev;
        let mast_top = hoist_z + 0.65;

        let (stone, iron_dark, iron_steel, timber, timber_dark, brass, amber_glow, cyan_glow) = (
            Vec3::new(0.52, 0.54, 0.58),
            Vec3::new(0.24, 0.27, 0.32),
            Vec3::new(0.42, 0.46, 0.52),
            Vec3::new(0.76, 0.56, 0.32),
            Vec3::new(0.54, 0.38, 0.20),
            Vec3::new(0.88, 0.72, 0.28),
            Vec3::new(0.98, 0.68, 0.16),
            Vec3::new(0.22, 0.88, 0.98),
        );

        let mast_u = cx - dir_u * 0.12;
        let mast_v = cy - dir_v * 0.12;

        // 1. Beveled stone & forged-iron pedestal on the ground
        BlockMedium::emit_box(cx - 0.21, cx + 0.21, cy - 0.21, cy + 0.21, base_z, base_z + 0.08, frame, stone, stone, true, target);
        BlockMedium::emit_box(mast_u - 0.13, mast_u + 0.13, mast_v - 0.13, mast_v + 0.13, base_z + 0.08, base_z + 0.18, frame, iron_dark, iron_steel, false, target);

        // 2. Octagonal/chamfered timber mast with iron rack-and-pinion track
        BlockMedium::emit_box(mast_u - 0.075, mast_u + 0.075, mast_v - 0.075, mast_v + 0.075, base_z + 0.18, mast_top, frame, timber, timber_dark, false, target);
        BlockMedium::emit_box(mast_u - 0.045, mast_u + 0.045, mast_v - 0.092, mast_v + 0.092, base_z + 0.18, mast_top, frame, timber_dark, timber, false, target);
        BlockMedium::emit_box(mast_u - 0.092, mast_u + 0.092, mast_v - 0.045, mast_v + 0.045, base_z + 0.18, mast_top, frame, timber_dark, timber, false, target);
        BlockMedium::emit_box(mast_u - 0.10, mast_u + 0.10, mast_v - 0.10, mast_v + 0.10, mast_top, mast_top + 0.06, frame, iron_dark, iron_steel, true, target);

        let mut tooth_z = base_z + 0.28;
        while tooth_z < mast_top - 0.08 {
            let tu0 = mast_u + dir_u * 0.07 - 0.028;
            let tu1 = mast_u + dir_u * 0.07 + 0.028;
            let tv0 = mast_v + dir_v * 0.07 - 0.028;
            let tv1 = mast_v + dir_v * 0.07 + 0.028;
            BlockMedium::emit_box(tu0, tu1, tv0, tv1, tooth_z, tooth_z + 0.035, frame, iron_dark, iron_steel, true, target);
            tooth_z += 0.14;
        }

        // 3. Sliding hoist carriage collar & brass hand-crank wheel at hoist_z
        let col_z0 = hoist_z + 0.12;
        let col_z1 = hoist_z + 0.38;
        BlockMedium::emit_box(mast_u - 0.115, mast_u + 0.115, mast_v - 0.115, mast_v + 0.115, col_z0, col_z1, frame, iron_dark, iron_steel, true, target);
        let crank_u = mast_u + perp_u * 0.14;
        let crank_v = mast_v + perp_v * 0.14;
        BlockMedium::emit_box(crank_u - 0.04, crank_u + 0.04, crank_v - 0.04, crank_v + 0.04, hoist_z + 0.19, hoist_z + 0.31, frame, brass, brass, true, target);

        // 4. Sleek telescoping horizontal cantilever crossbar (`Querbalken`) & lateral clamping jaws
        let arm_len = (self.reach_steps as f32) * BLOCK_STEP - 0.25 + 0.12;
        let end_u = mast_u + dir_u * arm_len;
        let end_v = mast_v + dir_v * arm_len;
        let (au0, au1) = (mast_u.min(end_u) - perp_u.abs() * 0.055, mast_u.max(end_u) + perp_u.abs() * 0.055);
        let (av0, av1) = (mast_v.min(end_v) - perp_v.abs() * 0.055, mast_v.max(end_v) + perp_v.abs() * 0.055);
        BlockMedium::emit_box(au0, au1, av0, av1, hoist_z + 0.19, hoist_z + 0.31, frame, timber, timber_dark, true, target);

        // 5. Continuous sloped diagonal knee brace (`Kopfband`) anchored in mast & crossbar iron shoes
        let brace_z0 = (hoist_z - 0.20).max(base_z + 0.16);
        let brace_reach = (arm_len * 0.56).clamp(0.24, 0.48);
        let join_u = mast_u + dir_u * brace_reach;
        let join_v = mast_v + dir_v * brace_reach;
        BlockMedium::emit_box(mast_u - 0.105, mast_u + 0.105, mast_v - 0.105, mast_v + 0.105, brace_z0 - 0.04, brace_z0 + 0.07, frame, iron_dark, iron_steel, true, target);
        let gw_u = dir_u.abs() * 0.055 + perp_u.abs() * 0.068;
        let gw_v = dir_v.abs() * 0.055 + perp_v.abs() * 0.068;
        BlockMedium::emit_box(join_u - gw_u, join_u + gw_u, join_v - gw_v, join_v + gw_v, hoist_z + 0.175, hoist_z + 0.325, frame, iron_dark, iron_steel, true, target);

        let side_vec = (frame.tangent() * perp_u + frame.bitangent() * perp_v).normalize_or_zero();
        let strut_start = frame.point_at(mast_u + dir_u * 0.02, mast_v + dir_v * 0.02, brace_z0 + 0.01);
        let strut_end = frame.point_at(join_u, join_v, hoist_z + 0.23);
        Self::emit_sloped_beam(strut_start, strut_end, side_vec, 0.038, 0.042, timber, timber_dark, target);
        Self::emit_sloped_beam(strut_start, strut_end, side_vec, 0.018, 0.048, iron_steel, iron_dark, target);

        let jaw_u = end_u - dir_u * 0.03;
        let jaw_v = end_v - dir_v * 0.03;
        let jw_u = dir_u.abs() * 0.035 + perp_u.abs() * 0.18;
        let jw_v = dir_v.abs() * 0.035 + perp_v.abs() * 0.18;
        BlockMedium::emit_box(jaw_u - jw_u, jaw_u + jw_u, jaw_v - jw_v, jaw_v + jw_v, hoist_z + 0.08, hoist_z + 0.42, frame, iron_dark, iron_steel, true, target);

        let n = frame.normal();
        let p0 = frame.point_at(jaw_u - 0.05, jaw_v - 0.05, hoist_z + 0.425);
        let p1 = frame.point_at(jaw_u + 0.05, jaw_v - 0.05, hoist_z + 0.425);
        let p2 = frame.point_at(jaw_u + 0.05, jaw_v + 0.05, hoist_z + 0.425);
        let p3 = frame.point_at(jaw_u - 0.05, jaw_v + 0.05, hoist_z + 0.425);
        target.push_quad(Quad::glowing(p0, p1, p2, p3, n, amber_glow, 0.90));
        let c0 = frame.point_at(mast_u - 0.04, mast_v - 0.04, mast_top + 0.062);
        let c1 = frame.point_at(mast_u + 0.04, mast_v - 0.04, mast_top + 0.062);
        let c2 = frame.point_at(mast_u + 0.04, mast_v + 0.04, mast_top + 0.062);
        let c3 = frame.point_at(mast_u - 0.04, mast_v + 0.04, mast_top + 0.062);
        target.push_quad(Quad::glowing(c0, c1, c2, c3, n, cyan_glow, 0.88));
    }
}
