use glam::Vec3;
use world::{MeshTarget, Quad};
use super::burden::SatchelBurden;
use super::compartment::SlotPlaque;
use super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};

/// `MeshTarget` decorator adding subtle ambient radiance so the 3D burlap sack stays legible in dark shafts.
struct RadiantMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
}

impl<'a> RadiantMeshTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(target: &'a mut dyn MeshTarget) -> Self {
        Self { target }
    }
}

impl MeshTarget for RadiantMeshTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let glow = quad.radiate(|g| g.max(0.24));
        let lit = quad.transfer(|v0, v1, v2, v3, n, col| Quad::glowing(v0, v1, v2, v3, n, col, glow));
        self.target.push_quad(lit);
    }
}

/// Unframed 3D burlap sack hung by a hemp cord from the top-right slot corner and leaning left against the slots.
#[derive(Clone, Copy, Debug)]
pub struct HangingSack {
    burden: SatchelBurden,
}

impl HangingSack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(burden: SatchelBurden) -> Self {
        Self { burden }
    }
}

impl SlotPlaque for HangingSack {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        const OCT_COS: f32 = 0.923_879_5;
        const SLOT_WALL_X: f32 = 0.1508;
        let mut lit = RadiantMeshTarget::new(target);
        let corner_hitch = Vec3::new(0.150, 0.241, 0.015);
        let brass = Vec3::new(0.88, 0.70, 0.26);
        let cord = Vec3::new(0.84, 0.70, 0.46);
        let seam_rope = Vec3::new(0.46, 0.33, 0.19);
        let jute_bot = Vec3::new(0.45, 0.33, 0.19);
        let jute_low = Vec3::new(0.56, 0.42, 0.25);
        let jute_mid = Vec3::new(0.64, 0.49, 0.30);
        let jute_top = Vec3::new(0.70, 0.55, 0.35);
        let seal_col = self.burden.pigment() * 0.72;

        let belly_rx = self.burden.swell(0.020, 0.080);
        let belly_rz = self.burden.swell(0.018, 0.056);
        let sack_len = self.burden.swell(0.072, 0.308);
        let cinch_y = self.burden.swell(0.168, -0.024);
        let z_sack = 0.015;

        let low_rx = belly_rx;
        let low_rz = belly_rz;
        let low_x = SLOT_WALL_X + OCT_COS * low_rx;
        let mid_rx = belly_rx * 0.90;
        let mid_rz = belly_rz * 0.90;
        let mid_x = SLOT_WALL_X + OCT_COS * mid_rx;
        let neck_rx = belly_rx * 0.58;
        let neck_rz = belly_rz * 0.58;
        let neck_x = SLOT_WALL_X + OCT_COS * neck_rx;
        let cap_rx = belly_rx * 0.46;
        let cap_rz = belly_rz * 0.46;
        let cap_x = SLOT_WALL_X + OCT_COS * cap_rx + belly_rx * 0.14;
        let cinch_x = SLOT_WALL_X + 0.004 + OCT_COS * belly_rx * 0.42;

        let cap_y = cinch_y - sack_len;
        let round_y = cap_y + sack_len * 0.05;
        let low_y = cap_y + sack_len * 0.22;
        let mid_y = cap_y + sack_len * 0.56;
        let neck_y = cap_y + sack_len * 0.82;
        let tuft_y = cinch_y + self.burden.swell(0.014, 0.016);
        let cinch_r = self.burden.swell(0.0075, 0.012);
        let tuft_r = self.burden.swell(0.011, 0.015);
        let cord_r = self.burden.swell(0.0032, 0.0008);
        let cinch_knot = Vec3::new(cinch_x - cinch_r * 0.50, cinch_y + 0.003, z_sack);

        // Top-right slot corner hitch + taut visible hemp cord down to the sack neck knot
        AxialLimb::new(Vec3::new(0.148, 0.239, 0.019), Vec3::new(0.152, 0.243, 0.014), 0.0034, 0.0028, brass)
            .emit(frame, true, &mut lit);
        AxialLimb::new(Vec3::new(0.147, 0.243, 0.017), Vec3::new(0.152, 0.239, 0.014), cord_r * 1.15, cord_r * 1.15, cord)
            .emit(frame, true, &mut lit);
        AxialLimb::new(corner_hitch, cinch_knot, cord_r, cord_r * 1.10, cord)
            .emit(frame, true, &mut lit);
        AxialLimb::new(
            cinch_knot + Vec3::new(-0.002, 0.002, -0.002),
            cinch_knot + Vec3::new(0.002, -0.002, 0.002),
            cord_r * 1.45,
            cord_r * 1.35,
            cord,
        )
        .emit(frame, true, &mut lit);

        let c_tip = Vec3::new(cap_x + belly_rx * 0.10, cap_y, z_sack);
        let c_cap = Vec3::new(cap_x, round_y, z_sack);
        let c_low = Vec3::new(low_x, low_y, z_sack);
        let c_mid = Vec3::new(mid_x, mid_y, z_sack);
        let c_neck = Vec3::new(neck_x, neck_y, z_sack);
        let c_cinch = Vec3::new(cinch_x, cinch_y, z_sack);
        let c_tuft = Vec3::new(cinch_x + self.burden.swell(0.007, 0.011), tuft_y, z_sack);

        TaperedPrism::new(8, c_tip, c_cap, 0.002, 0.002, cap_rx, cap_rz, jute_bot)
            .emit(frame, false, &mut lit);
        TaperedPrism::new(8, c_cap, c_low, cap_rx, cap_rz, low_rx, low_rz, jute_bot)
            .emit(frame, false, &mut lit);
        TaperedPrism::new(8, c_low, c_mid, low_rx, low_rz, mid_rx, mid_rz, jute_low)
            .emit(frame, false, &mut lit);
        TaperedPrism::new(8, c_mid, c_neck, mid_rx, mid_rz, neck_rx, neck_rz, jute_mid)
            .emit(frame, false, &mut lit);
        TaperedPrism::new(8, c_neck, c_cinch, neck_rx, neck_rz, cinch_r, cinch_r, jute_top)
            .emit(frame, false, &mut lit);
        TaperedPrism::hexagonal(
            Vec3::new(cinch_x, cinch_y - 0.005, z_sack),
            Vec3::new(cinch_x, cinch_y + 0.002, z_sack),
            cinch_r * 1.24,
            cinch_r * 1.24,
            seal_col,
        )
        .emit(frame, true, &mut lit);
        TaperedPrism::hexagonal(
            Vec3::new(cinch_x, cinch_y + 0.001, z_sack),
            Vec3::new(cinch_x, cinch_y + 0.006, z_sack),
            cinch_r * 1.30,
            cinch_r * 1.26,
            cord,
        )
        .emit(frame, true, &mut lit);
        TaperedPrism::hexagonal(Vec3::new(cinch_x, cinch_y + 0.004, z_sack), c_tuft, cinch_r * 0.92, tuft_r, jute_top)
            .emit(frame, true, &mut lit);

        for (y, rx, rz) in [(low_y, low_rx * 1.03, low_rz * 1.03), (mid_y, mid_rx * 1.03, mid_rz * 1.03)] {
            let bx = SLOT_WALL_X + OCT_COS * rx;
            TaperedPrism::new(8, Vec3::new(bx, y - 0.0035, z_sack), Vec3::new(bx, y + 0.0035, z_sack), rx, rz, rx, rz, seam_rope)
                .emit(frame, false, &mut lit);
        }
    }
}
