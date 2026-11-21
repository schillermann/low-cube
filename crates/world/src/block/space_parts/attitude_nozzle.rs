use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::cell::GridCell;
use super::super::wedge::WedgeHeading;
use super::hollow_bell::HollowBell;
use super::rcs_manifold::RcsManifold;

/// Side-mounted 5-nozzle spacecraft RCS manifold pod with hollow thruster bells and per-horn translucent plumes.
pub struct AttitudeNozzleMesh {
    heading: WedgeHeading,
    throttles: [f32; 5],
}

impl AttitudeNozzleMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_throttles(heading: WedgeHeading, throttles: [f32; 5]) -> Self {
        Self { heading, throttles }
    }

    /// Secondary constructor mapping legacy `(vertical_burn, planar_burn)` channels onto the 5-horn manifold.
    pub fn new(heading: WedgeHeading, motion: (f32, f32)) -> Self {
        let dorsal = (-motion.0).clamp(0.0, 1.2);
        let ventral = motion.0.clamp(0.0, 1.2);
        let port = (-motion.1).clamp(0.0, 1.2);
        let starboard = motion.1.clamp(0.0, 1.2);
        let outward = if motion.1.abs() > 0.04 { motion.1.abs().clamp(0.0, 1.2) } else { 0.0 };
        Self::with_throttles(heading, [outward, dorsal, ventral, port, starboard])
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

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (uc, vc) = cell.center();
        let center = frame.point_at(uc, vc, floor_elev + 0.25);
        let up = frame.normal().normalize_or_zero();
        let out_dir = (-self.heading.forward_vector(frame)).normalize_or_zero();
        let side_dir = out_dir.cross(up).normalize_or_zero();

        let slate_flange = Vec3::new(0.24, 0.27, 0.34);
        let titanium = Vec3::new(0.60, 0.66, 0.74);
        let brass = Vec3::new(0.90, 0.72, 0.24);
        let graphite = Vec3::new(0.16, 0.18, 0.24);

        Self::emit_oriented_box(center, side_dir, up, out_dir, (0.185, 0.185), (-0.25, -0.175), slate_flange, brass, target);
        Self::emit_oriented_box(center, side_dir, up, out_dir, (0.148, 0.056), (-0.175, -0.020), slate_flange, titanium, target);
        Self::emit_oriented_box(center, side_dir, up, out_dir, (0.056, 0.148), (-0.175, -0.020), slate_flange, titanium, target);
        Self::emit_oriented_box(center, side_dir, up, out_dir, (0.096, 0.096), (-0.175, 0.022), titanium, brass, target);

        let jet_palette = (
            Vec3::new(0.98, 0.94, 0.80),
            Vec3::new(0.98, 0.62, 0.22),
            Vec3::new(0.42, 0.80, 1.00),
        );
        let horns = RcsManifold::new(self.heading).horns(cell, floor_elev, frame);
        for (horn, &throttle) in horns.iter().zip(self.throttles.iter()) {
            let bell = HollowBell::new(
                horn.origin(),
                (horn.exhaust_dir(), horn.ref_up()),
                horn.envelope(),
                6,
            );
            bell.emit_hollow_shell(brass, graphite, titanium, throttle, target);
            bell.emit_translucent_jet(throttle, jet_palette, 0.38, target);
        }
    }
}
