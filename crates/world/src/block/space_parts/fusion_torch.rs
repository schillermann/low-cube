use std::f32::consts::PI;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::cell::GridCell;
use super::hollow_bell::HollowBell;

/// Sculpted Epstein-class magnetic-confinement `FusionTorch` cone with superconducting coils and blue-white plasma lance.
pub struct FusionTorchMesh {
    plume_glow: f32,
}

impl FusionTorchMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(plume_glow: f32) -> Self {
        Self { plume_glow }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_coil(
        center: Vec3,
        sa: Vec3,
        sb: Vec3,
        ex: Vec3,
        z0: f32,
        r0: f32,
        z1: f32,
        r1: f32,
        col: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        let (segs, step) = (8usize, (2.0 * PI) / 8.0);
        let (mid0, mid1) = (center + ex * z0, center + ex * z1);
        for i in 0..segs {
            let ((s0, c0), (s1, c1)) = (((i as f32) * step).sin_cos(), (((i + 1) as f32) * step).sin_cos());
            let (d0, d1) = (sa * c0 + sb * s0, sa * c1 + sb * s1);
            let [a0, a1] = [mid0 + d0 * r0, mid0 + d1 * r0];
            let [b0, b1] = [mid1 + d0 * r1, mid1 + d1 * r1];
            let norm = (a1 - a0).cross(b0 - a0).normalize_or_zero();
            target.push_quad(Quad::glowing(a0, a1, b1, b0, norm, col, glow));
        }
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (uc, vc) = cell.center();
        let center = frame.point_at(uc, vc, floor_elev + 0.25);
        let ex = -frame.normal().normalize_or_zero();
        let sa = ex.cross(frame.tangent()).normalize_or_zero();
        let sb = ex.cross(sa).normalize_or_zero();

        let reactor_drum = Vec3::new(0.18, 0.21, 0.28);
        let tung_shield = Vec3::new(0.32, 0.36, 0.44);
        let mag_coil = Vec3::new(0.22, 0.86, 1.00);
        let liner_dark = Vec3::new(0.10, 0.13, 0.20);
        let rim_alloy = Vec3::new(0.58, 0.68, 0.82);

        Self::emit_coil(center, sa, sb, ex, -0.25, 0.22, -0.14, 0.21, reactor_drum, 0.0, target);
        Self::emit_coil(center, sa, sb, ex, -0.13, 0.225, -0.09, 0.225, mag_coil, 0.65, target);
        Self::emit_coil(center, sa, sb, ex, -0.09, 0.16, 0.01, 0.14, tung_shield, 0.0, target);
        Self::emit_coil(center, sa, sb, ex, 0.04, 0.205, 0.075, 0.215, mag_coil, 0.72, target);
        Self::emit_coil(center, sa, sb, ex, 0.15, 0.238, 0.185, 0.246, mag_coil, 0.80, target);

        let bell = HollowBell::new(center - ex * 0.04, (ex, sa), (0.29, 0.248), 8);
        bell.emit_hollow_shell(mag_coil, liner_dark, rim_alloy, self.plume_glow, target);

        let fusion_palette = (
            Vec3::new(0.96, 0.99, 1.00),
            Vec3::new(0.28, 0.80, 1.00),
            Vec3::new(0.12, 0.42, 0.98),
        );
        bell.emit_translucent_jet(self.plume_glow, fusion_palette, 2.85, target);
    }
}
