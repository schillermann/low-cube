use std::f32::consts::PI;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::cell::GridCell;
use super::super::wedge::WedgeHeading;

/// Side-mounted spacecraft reaction-control Steuerdüse pod with outward nozzle cone facing the player in palm.
pub struct AttitudeNozzleMesh {
    heading: WedgeHeading,
    motion: (f32, f32),
}

impl AttitudeNozzleMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(heading: WedgeHeading, motion: (f32, f32)) -> Self {
        Self { heading, motion }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_cone(
        origin: Vec3,
        axis: Vec3,
        ref_up: Vec3,
        z0: f32,
        r0: f32,
        z1: f32,
        r1: f32,
        col: Vec3,
        glow: f32,
        end_cap: bool,
        target: &mut dyn MeshTarget,
    ) {
        let sa = axis.cross(ref_up).normalize_or_zero();
        let sb = axis.cross(sa).normalize_or_zero();
        let (segs, step) = (6usize, (2.0 * PI) / 6.0);
        let end_mid = origin + axis * z1;
        for i in 0..segs {
            let ((s0, c0), (s1, c1)) = (((i as f32) * step).sin_cos(), (((i + 1) as f32) * step).sin_cos());
            let (d0, d1) = (sa * c0 + sb * s0, sa * c1 + sb * s1);
            let [a0, a1] = [origin + axis * z0 + d0 * r0, origin + axis * z0 + d1 * r0];
            let [b0, b1] = [origin + axis * z1 + d0 * r1, origin + axis * z1 + d1 * r1];
            let norm = (a1 - a0).cross(b0 - a0).normalize_or_zero();
            target.push_quad(Quad::glowing(a0, a1, b1, b0, norm, col, glow));
            if end_cap {
                target.push_quad(Quad::glowing(end_mid, b0, b1, b1, axis, col, glow));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_oriented_box(
        center: Vec3,
        side: Vec3,
        up: Vec3,
        out: Vec3,
        half_su: (f32, f32),
        z_span: (f32, f32),
        wall_col: Vec3,
        face_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let (hs, hu) = half_su;
        let (z0, z1) = z_span;
        let pt = |sx: f32, uy: f32, oz: f32| center + side * sx + up * uy + out * oz;
        let [a0, a1, a2, a3] = [pt(-hs, -hu, z0), pt(hs, -hu, z0), pt(hs, hu, z0), pt(-hs, hu, z0)];
        let [b0, b1, b2, b3] = [pt(-hs, -hu, z1), pt(hs, -hu, z1), pt(hs, hu, z1), pt(-hs, hu, z1)];
        target.push_quad(Quad::new(b0, b1, b2, b3, out, face_col));
        target.push_quad(Quad::new(a0, a3, a2, a1, -out, wall_col));
        for ([p0, p1, p2, p3], n) in [
            ([a0, a1, b1, b0], -up),
            ([a1, a2, b2, b1], side),
            ([a2, a3, b3, b2], up),
            ([a3, a0, b0, b3], -side),
        ] {
            target.push_quad(Quad::new(p0, p1, p2, p3, n, wall_col));
        }
    }

    fn emit_rcs_horn(
        origin: Vec3,
        dir: Vec3,
        ref_up: Vec3,
        z_start: f32,
        length: f32,
        mouth_r: f32,
        plume: f32,
        target: &mut dyn MeshTarget,
    ) {
        let brass = Vec3::new(0.90, 0.72, 0.24);
        let graphite = Vec3::new(0.18, 0.20, 0.26);
        let throat_r = mouth_r * 0.44;
        let throat_end = z_start + length * 0.26;
        let bell_end = z_start + length;
        Self::emit_cone(origin, dir, ref_up, z_start, throat_r * 1.25, throat_end, throat_r, brass, 0.0, false, target);
        Self::emit_cone(origin, dir, ref_up, throat_end, throat_r, bell_end, mouth_r, graphite, 0.0, false, target);
        let (ion_col, ion_glow) = if plume > 0.04 {
            (Vec3::new(0.78, 0.98, 1.00), 0.98)
        } else {
            (Vec3::new(0.20, 0.78, 0.96), 0.68)
        };
        Self::emit_cone(origin, dir, ref_up, bell_end - 0.012, mouth_r * 0.82, bell_end - 0.002, mouth_r * 0.82, ion_col, ion_glow, true, target);
        if plume > 0.04 {
            let reach = (0.26 + 0.22 * plume) * (mouth_r / 0.095);
            let core_col = Vec3::new(0.92, 0.99, 1.00);
            let halo_col = Vec3::new(0.24, 0.86, 1.00);
            Self::emit_cone(origin, dir, ref_up, bell_end, mouth_r * 0.68, bell_end + reach * 0.6, mouth_r * 0.18, core_col, 0.99, true, target);
            Self::emit_cone(origin, dir, ref_up, bell_end + 0.005, mouth_r * 0.88, bell_end + reach, mouth_r * 0.06, halo_col, 0.92, true, target);
        }
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (uc, vc) = cell.center();
        let center = frame.point_at(uc, vc, floor_elev + 0.25);
        let up = frame.normal();
        let mount_dir = self.heading.forward_vector(frame).normalize_or_zero();
        let out_dir = -mount_dir;
        let side_dir = out_dir.cross(up).normalize_or_zero();

        let slate_flange = Vec3::new(0.24, 0.27, 0.34);
        let titanium = Vec3::new(0.62, 0.68, 0.76);
        let brass = Vec3::new(0.90, 0.72, 0.24);

        Self::emit_oriented_box(center, side_dir, up, out_dir, (0.185, 0.185), (-0.25, -0.175), slate_flange, brass, target);
        Self::emit_oriented_box(center, side_dir, up, out_dir, (0.145, 0.055), (-0.175, -0.045), slate_flange, slate_flange, target);
        Self::emit_oriented_box(center, side_dir, up, out_dir, (0.055, 0.145), (-0.175, -0.045), slate_flange, slate_flange, target);
        Self::emit_oriented_box(center, side_dir, up, out_dir, (0.105, 0.105), (-0.175, 0.025), titanium, titanium, target);
        Self::emit_oriented_box(center, side_dir, up, out_dir, (0.118, 0.118), (-0.055, -0.010), brass, brass, target);

        let hub = center - out_dir * 0.06;
        let planar_plume = if self.motion.1.abs() > 0.04 { self.motion.1.abs().clamp(0.0, 1.2) } else { 0.0 };
        let vertical_plume = if self.motion.0.abs() > 0.04 { self.motion.0.abs().clamp(0.0, 1.2) } else { 0.0 };
        let active_plume = planar_plume.max(vertical_plume);

        Self::emit_rcs_horn(hub, out_dir, up, 0.080, 0.220, 0.125, active_plume, target);
    }
}
