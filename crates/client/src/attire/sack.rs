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

/// Unframed 3D burlap sack hanging directly from the top-right slot corner tip and bulging rightward with weight.
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
        let mut lit = RadiantMeshTarget::new(target);
        let slot_tip = Vec3::new(0.148, 0.240, 0.015);
        let brass = Vec3::new(0.88, 0.70, 0.26);
        let rope = Vec3::new(0.36, 0.25, 0.14);
        let jute_bot = Vec3::new(0.45, 0.33, 0.19);
        let jute_low = Vec3::new(0.56, 0.42, 0.25);
        let jute_mid = Vec3::new(0.64, 0.49, 0.30);
        let jute_top = Vec3::new(0.70, 0.55, 0.35);
        let seal_col = self.burden.pigment() * 0.72;

        let belly_rx = self.burden.swell(0.018, 0.082);
        let belly_rz = self.burden.swell(0.016, 0.066);
        let sack_len = self.burden.swell(0.066, 0.340);
        let cinch_x = self.burden.swell(0.156, 0.010);
        let cinch_y = self.burden.swell(0.226, -0.022);
        let z_sack = 0.008;

        let wall_x = 0.175;
        let low_rx = belly_rx;
        let low_rz = belly_rz;
        let low_x = wall_x + low_rx;
        let mid_rx = belly_rx * 0.88;
        let mid_rz = belly_rz * 0.88;
        let mid_x = wall_x + mid_rx;
        let neck_rx = belly_rx * 0.50;
        let neck_rz = belly_rz * 0.50;
        let neck_x = cinch_x + (low_x - cinch_x) * 0.46;
        let cap_rx = belly_rx * 0.42;
        let cap_rz = belly_rz * 0.42;
        let cap_x = low_x - belly_rx * 0.08;

        let cap_y = cinch_y - sack_len;
        let low_y = cap_y + sack_len * 0.25;
        let mid_y = cap_y + sack_len * 0.58;
        let neck_y = cap_y + sack_len * 0.86;
        let tuft_y = cinch_y + self.burden.swell(0.015, 0.020);
        let cinch_r = self.burden.swell(0.008, 0.014);
        let tuft_r = self.burden.swell(0.014, 0.022);
        let rope_r = self.burden.swell(0.0028, 0.0024);

        // Small brass corner eyelet right on the top-right vertex of the top-right slot + short hemp loop
        AxialLimb::new(Vec3::new(0.145, 0.238, 0.017), Vec3::new(0.151, 0.242, 0.011), 0.0038, 0.0032, brass)
            .emit(frame, true, &mut lit);
        AxialLimb::new(slot_tip, Vec3::new(cinch_x, cinch_y + 0.003, z_sack), rope_r, rope_r * 1.15, rope)
            .emit(frame, true, &mut lit);

        let c_cap = Vec3::new(cap_x, cap_y, z_sack);
        let c_low = Vec3::new(low_x, low_y, z_sack);
        let c_mid = Vec3::new(mid_x, mid_y, z_sack);
        let c_neck = Vec3::new(neck_x, neck_y, z_sack);
        let c_cinch = Vec3::new(cinch_x, cinch_y, z_sack);
        let c_tuft = Vec3::new(cinch_x - 0.003, tuft_y, z_sack);

        TaperedPrism::new(8, c_cap, c_low, cap_rx, cap_rz, low_rx, low_rz, jute_bot)
            .emit(frame, true, &mut lit);
        TaperedPrism::new(8, c_low, c_mid, low_rx, low_rz, mid_rx, mid_rz, jute_low)
            .emit(frame, false, &mut lit);
        TaperedPrism::new(8, c_mid, c_neck, mid_rx, mid_rz, neck_rx, neck_rz, jute_mid)
            .emit(frame, false, &mut lit);
        TaperedPrism::new(8, c_neck, c_cinch, neck_rx, neck_rz, cinch_r, cinch_r, jute_top)
            .emit(frame, false, &mut lit);
        TaperedPrism::hexagonal(
            Vec3::new(cinch_x, cinch_y - 0.005, z_sack),
            Vec3::new(cinch_x, cinch_y + 0.006, z_sack),
            cinch_r * 1.28,
            cinch_r * 1.28,
            seal_col,
        )
        .emit(frame, true, &mut lit);
        TaperedPrism::hexagonal(Vec3::new(cinch_x, cinch_y + 0.004, z_sack), c_tuft, cinch_r * 0.94, tuft_r, jute_top)
            .emit(frame, true, &mut lit);

        for (center, rx, rz) in [(c_low, low_rx, low_rz), (c_mid, mid_rx, mid_rz)] {
            TaperedPrism::new(
                8,
                Vec3::new(center.x, center.y - 0.004, center.z),
                Vec3::new(center.x, center.y + 0.004, center.z),
                rx * 1.03,
                rz * 1.03,
                rx * 1.03,
                rz * 1.03,
                rope,
            )
            .emit(frame, false, &mut lit);
        }
    }
}
