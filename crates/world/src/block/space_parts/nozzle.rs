use std::f32::consts::PI;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::cell::GridCell;
use super::hollow_bell::HollowBell;
use super::NozzleAxis;

/// Sculpted hollow De Laval main rocket bell nozzle with top mounting bulkhead and translucent exhaust plume.
pub struct RocketNozzleMesh {
    axis: NozzleAxis,
    plume_glow: f32,
}

impl RocketNozzleMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(axis: NozzleAxis, plume_glow: f32) -> Self {
        Self { axis, plume_glow }
    }

    fn axis_frame(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame) -> (Vec3, Vec3, Vec3, Vec3) {
        let (uc, vc) = cell.center();
        let center = frame.point_at(uc, vc, floor_elev + 0.25);
        let exhaust = self.axis.exhaust_vector(frame).normalize_or_zero();
        let ref_vec = if exhaust.dot(frame.normal()).abs() > 0.9 {
            frame.tangent()
        } else {
            frame.normal()
        };
        let side_a = exhaust.cross(ref_vec).normalize_or_zero();
        let side_b = exhaust.cross(side_a).normalize_or_zero();
        (center, side_a, side_b, exhaust)
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_collar(
        center: Vec3,
        sa: Vec3,
        sb: Vec3,
        ex: Vec3,
        z0: f32,
        r0: f32,
        z1: f32,
        r1: f32,
        col: Vec3,
        caps: (bool, bool),
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
            target.push_quad(Quad::new(a0, a1, b1, b0, norm, col));
            if caps.0 {
                target.push_quad(Quad::new(mid0, a1, a0, a0, -ex, col));
            }
            if caps.1 {
                target.push_quad(Quad::new(mid1, b0, b1, b1, ex, col));
            }
        }
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (center, sa, sb, ex) = self.axis_frame(cell, floor_elev, frame);
        let bulkhead_steel = Vec3::new(0.28, 0.31, 0.38);
        let chamber_slate = Vec3::new(0.20, 0.22, 0.28);
        let bell_graphite = Vec3::new(0.14, 0.15, 0.19);
        let brass_ring = Vec3::new(0.88, 0.68, 0.22);
        let titanium_rim = Vec3::new(0.46, 0.50, 0.58);

        Self::emit_collar(center, sa, sb, ex, -0.25, 0.18, -0.19, 0.18, bulkhead_steel, (true, true), target);
        Self::emit_collar(center, sa, sb, ex, -0.255, 0.09, -0.245, 0.09, brass_ring, (true, false), target);
        Self::emit_collar(center, sa, sb, ex, -0.19, 0.125, -0.06, 0.105, chamber_slate, (false, false), target);
        Self::emit_collar(center, sa, sb, ex, -0.08, 0.122, -0.03, 0.098, brass_ring, (true, true), target);
        Self::emit_collar(center, sa, sb, ex, 0.08, 0.186, 0.105, 0.196, brass_ring, (true, true), target);
        Self::emit_collar(center, sa, sb, ex, 0.175, 0.218, 0.198, 0.228, brass_ring, (true, true), target);

        let bell = HollowBell::new(center - ex * 0.03, (ex, sa), (0.275, 0.238), 8);
        bell.emit_hollow_shell(brass_ring, bell_graphite, titanium_rim, self.plume_glow, target);

        let flame_palette = (
            Vec3::new(1.00, 0.96, 0.76),
            Vec3::new(1.00, 0.52, 0.14),
            Vec3::new(0.96, 0.24, 0.06),
        );
        bell.emit_translucent_jet(self.plume_glow, flame_palette, 0.88, target);
    }
}
