use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::timber_grain::BlockMedium;
use super::wedge::WedgeHeading;

/// Oriented 3D geometry trait for cockpit floor pan and footwell assemblies.
pub trait FootwellPlinth {
    fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget);
}

/// Full-cell walnut floor pan with brass perimeter welt and stitched rubber footwell mat.
pub struct WalnutFloorPan {
    heading: WedgeHeading,
    plinth_span: f32,
}

impl WalnutFloorPan {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(heading: WedgeHeading, plinth_span: f32) -> Self {
        Self { heading, plinth_span }
    }
}

impl FootwellPlinth for WalnutFloorPan {
    fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((cu0, cu1), (cv0, cv1)) = (cell.u_bounds(), cell.v_bounds());
        let plinth_h = floor_elev + self.plinth_span;
        let (walnut, stitch, mat_col, brass) = (
            Vec3::new(0.32, 0.18, 0.11),
            Vec3::new(0.38, 0.10, 0.08),
            Vec3::new(0.18, 0.16, 0.19),
            Vec3::new(0.92, 0.74, 0.24),
        );
        BlockMedium::emit_box(cu0, cu1, cv0, cv1, floor_elev, plinth_h, frame, walnut, walnut, true, target);
        BlockMedium::emit_box(cu0, cu1, cv0, cv1, plinth_h - 0.006, plinth_h + 0.006, frame, brass, brass, false, target);
        BlockMedium::emit_box(cu0, cu1, cv0, cv1, plinth_h, plinth_h + 0.012, frame, stitch, mat_col, false, target);
        let (su0, su1, sv0, sv1) = match self.heading {
            WedgeHeading::East => (cu1 - 0.025, cu1, cv0, cv1),
            WedgeHeading::West => (cu0, cu0 + 0.025, cv0, cv1),
            WedgeHeading::North => (cu0, cu1, cv1 - 0.025, cv1),
            WedgeHeading::South => (cu0, cu1, cv0, cv0 + 0.025),
        };
        BlockMedium::emit_box(su0, su1, sv0, sv1, plinth_h, plinth_h + 0.018, frame, brass, brass, false, target);
    }
}

/// Decorator over a `FootwellPlinth` mounting an angled firewall toe-board and gas/brake pedals.
pub struct PedalClusterMesh<P: FootwellPlinth> {
    inner: P,
    heading: WedgeHeading,
}

impl<P: FootwellPlinth> PedalClusterMesh<P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: P, heading: WedgeHeading) -> Self {
        Self { inner, heading }
    }
}

impl PedalClusterMesh<WalnutFloorPan> {
    /// Secondary constructor decorating a standard walnut floor pan with gas and brake pedals.
    pub const fn mounted(heading: WedgeHeading) -> Self {
        Self::new(WalnutFloorPan::new(heading, 0.055), heading)
    }
}

impl<P: FootwellPlinth> FootwellPlinth for PedalClusterMesh<P> {
    fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        self.inner.emit(cell, floor_elev, frame, target);
        let (uc, vc) = cell.center();
        let (t, b, up) = (frame.tangent(), frame.bitangent(), frame.normal());
        let (fwd, right) = match self.heading {
            WedgeHeading::East => (t, b),
            WedgeHeading::West => (-t, -b),
            WedgeHeading::North => (b, -t),
            WedgeHeading::South => (-b, t),
        };
        let origin = frame.point_at(uc, vc, floor_elev);
        let pt = |x: f32, y: f32, z: f32| origin + right * x + up * y + fwd * z;
        let (walnut, iron_pan, chrome, rubber, brass, brake_red, gas_green) = (
            Vec3::new(0.32, 0.18, 0.11),
            Vec3::new(0.20, 0.22, 0.26),
            Vec3::new(0.66, 0.70, 0.76),
            Vec3::new(0.14, 0.15, 0.18),
            Vec3::new(0.92, 0.74, 0.24),
            Vec3::new(0.88, 0.22, 0.18),
            Vec3::new(0.24, 0.84, 0.36),
        );
        let emit_prism = |a0: Vec3, a1: Vec3, a2: Vec3, a3: Vec3, b0: Vec3, b1: Vec3, b2: Vec3, b3: Vec3, w_col: Vec3, t_col: Vec3, out: &mut dyn MeshTarget| {
            let n_top = (b1 - b0).cross(b3 - b0).normalize_or_zero();
            out.push_quad(Quad::new(b0, b1, b2, b3, n_top, t_col));
            out.push_quad(Quad::new(a0, a3, a2, a1, -n_top, w_col));
            for [p0, p1, p2, p3] in [[a0, a1, b1, b0], [a1, a2, b2, b1], [a2, a3, b3, b2], [a3, a0, b0, b3]] {
                out.push_quad(Quad::new(p0, p1, p2, p3, (p1 - p0).cross(p3 - p0).normalize_or_zero(), w_col));
            }
        };

        // Angled firewall toe-board, full-height rear bulkhead reaching the block above, & transmission tunnel rib
        emit_prism(
            pt(-0.25, 0.055, 0.04), pt(0.25, 0.055, 0.04), pt(0.25, 0.055, 0.25), pt(-0.25, 0.055, 0.25),
            pt(-0.25, 0.22, 0.15), pt(0.25, 0.22, 0.15), pt(0.25, 0.22, 0.25), pt(-0.25, 0.22, 0.25),
            walnut, iron_pan, target,
        );
        emit_prism(
            pt(-0.25, 0.22, 0.15), pt(0.25, 0.22, 0.15), pt(0.25, 0.22, 0.25), pt(-0.25, 0.22, 0.25),
            pt(-0.25, 0.50, 0.15), pt(0.25, 0.50, 0.15), pt(0.25, 0.50, 0.25), pt(-0.25, 0.50, 0.25),
            walnut, iron_pan, target,
        );
        emit_prism(
            pt(-0.035, 0.055, -0.25), pt(0.035, 0.055, -0.25), pt(0.035, 0.055, 0.15), pt(-0.035, 0.055, 0.15),
            pt(-0.025, 0.085, -0.25), pt(0.025, 0.085, -0.25), pt(0.025, 0.095, 0.15), pt(-0.025, 0.095, 0.15),
            brass, iron_pan, target,
        );

        // Left Brake Pedal & Right Gas Pedal
        emit_prism(
            pt(-0.12, 0.17, 0.12), pt(-0.08, 0.17, 0.12), pt(-0.08, 0.19, 0.16), pt(-0.12, 0.19, 0.16),
            pt(-0.12, 0.11, -0.04), pt(-0.08, 0.11, -0.04), pt(-0.08, 0.13, 0.00), pt(-0.12, 0.13, 0.00),
            chrome, chrome, target,
        );
        emit_prism(
            pt(-0.16, 0.09, -0.08), pt(-0.05, 0.09, -0.08), pt(-0.05, 0.14, 0.01), pt(-0.16, 0.14, 0.01),
            pt(-0.16, 0.11, -0.09), pt(-0.05, 0.11, -0.09), pt(-0.05, 0.16, 0.00), pt(-0.16, 0.16, 0.00),
            brass, rubber, target,
        );
        let brake_n = (-fwd * 0.45 + up * 0.89).normalize_or_zero();
        target.push_quad(Quad::glowing(
            pt(-0.145, 0.116, -0.075), pt(-0.065, 0.116, -0.075), pt(-0.065, 0.154, -0.008), pt(-0.145, 0.154, -0.008),
            brake_n, brake_red, 0.82,
        ));
        emit_prism(
            pt(0.09, 0.17, 0.12), pt(0.12, 0.17, 0.12), pt(0.12, 0.19, 0.16), pt(0.09, 0.19, 0.16),
            pt(0.09, 0.09, -0.05), pt(0.12, 0.09, -0.05), pt(0.12, 0.11, -0.01), pt(0.09, 0.11, -0.01),
            chrome, chrome, target,
        );
        emit_prism(
            pt(0.07, 0.065, -0.11), pt(0.14, 0.065, -0.11), pt(0.14, 0.155, 0.03), pt(0.07, 0.155, 0.03),
            pt(0.07, 0.085, -0.12), pt(0.14, 0.085, -0.12), pt(0.14, 0.175, 0.02), pt(0.07, 0.175, 0.02),
            brass, rubber, target,
        );
        target.push_quad(Quad::glowing(
            pt(0.082, 0.092, -0.102), pt(0.128, 0.092, -0.102), pt(0.128, 0.168, 0.010), pt(0.082, 0.168, 0.010),
            brake_n, gas_green, 0.82,
        ));
    }
}
