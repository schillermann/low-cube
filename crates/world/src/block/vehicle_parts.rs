use std::f32::consts::PI;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::mold::MoldArchetype;
use super::timber_grain::BlockMedium;
use super::wedge::WedgeHeading;

/// Distinct wheel diameter and tread width profiles selectable by cycling block form archetypes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WheelProfile {
    Touring,
    Compact,
    Offroad,
    Monster,
}

impl WheelProfile {
    pub const fn from_archetype(archetype: MoldArchetype) -> Self {
        match archetype.code() % 4 {
            0 => Self::Touring,
            1 => Self::Compact,
            2 => Self::Offroad,
            _ => Self::Monster,
        }
    }

    pub const fn radius(self) -> f32 {
        match self {
            Self::Compact => 0.16,
            Self::Touring => 0.21,
            Self::Offroad => 0.26,
            Self::Monster => 0.32,
        }
    }

    pub const fn half_width(self) -> f32 {
        match self {
            Self::Compact => 0.11,
            Self::Touring => 0.15,
            Self::Offroad => 0.19,
            Self::Monster => 0.23,
        }
    }
}

/// Sculpted 3D geometry emitter for steering wheels, propulsion engines, and rolling wheels.
pub struct VehicleBlockMesh {
    medium: BlockMedium,
    heading: WedgeHeading,
    archetype: MoldArchetype,
    motion: (f32, f32),
}

impl VehicleBlockMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_motion(
        medium: BlockMedium,
        heading: WedgeHeading,
        archetype: MoldArchetype,
        motion: (f32, f32),
    ) -> Self {
        Self { medium, heading, archetype, motion }
    }

    pub const fn with_archetype(medium: BlockMedium, heading: WedgeHeading, archetype: MoldArchetype) -> Self {
        Self::with_motion(medium, heading, archetype, (0.0, 0.0))
    }

    pub const fn cross_grain(&self) -> bool {
        matches!(self.heading, WedgeHeading::East | WedgeHeading::West)
    }

    pub fn emit(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        match self.medium {
            BlockMedium::SteeringWheel => self.emit_steering_wheel(cell, floor_elevation, frame, target),
            BlockMedium::CarWheel => self.emit_wheel(cell, floor_elevation, frame, target),
            _ => self.emit_engine(cell, floor_elevation, frame, target),
        }
    }

    fn emit_steering_wheel(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (uc, vc) = cell.center();
        let (t, b, up) = (frame.tangent(), frame.bitangent(), frame.normal());
        let (fwd, right) = match self.heading {
            WedgeHeading::East => (t, b),
            WedgeHeading::West => (-t, -b),
            WedgeHeading::North => (b, -t),
            WedgeHeading::South => (-b, t),
        };
        let origin = frame.point_at(uc, vc, floor_elev);
        let (walnut, iron_base, column_steel, spoke_chrome, grip_leather, grip_seam, brass, marker_red) = (
            Vec3::new(0.34, 0.19, 0.11),
            Vec3::new(0.20, 0.22, 0.27),
            Vec3::new(0.32, 0.36, 0.42),
            Vec3::new(0.66, 0.70, 0.76),
            Vec3::new(0.14, 0.16, 0.20),
            Vec3::new(0.22, 0.24, 0.30),
            Vec3::new(0.94, 0.76, 0.24),
            Vec3::new(0.86, 0.22, 0.16),
        );
        let emit_prism = |a0: Vec3, a1: Vec3, a2: Vec3, a3: Vec3, b0: Vec3, b1: Vec3, b2: Vec3, b3: Vec3, w_col: Vec3, t_col: Vec3, out: &mut dyn MeshTarget| {
            let n_top = (b1 - b0).cross(b3 - b0).normalize_or_zero();
            out.push_quad(Quad::new(b0, b1, b2, b3, n_top, t_col));
            out.push_quad(Quad::new(a0, a3, a2, a1, -n_top, w_col));
            for [p0, p1, p2, p3] in [[a0, a1, b1, b0], [a1, a2, b2, b1], [a2, a3, b3, b2], [a3, a0, b0, b3]] {
                out.push_quad(Quad::new(p0, p1, p2, p3, (p1 - p0).cross(p3 - p0).normalize_or_zero(), w_col));
            }
        };
        let base_pt = |x: f32, y: f32, z: f32| origin + right * x + up * y + fwd * z;
        // Full-width sculpted dashboard cowl, walnut fascia, and glowing instrument dials
        emit_prism(
            base_pt(-0.24, 0.0, -0.06), base_pt(0.24, 0.0, -0.06), base_pt(0.24, 0.0, 0.24), base_pt(-0.24, 0.0, 0.24),
            base_pt(-0.23, 0.25, -0.02), base_pt(0.23, 0.25, -0.02), base_pt(0.24, 0.29, 0.22), base_pt(-0.24, 0.29, 0.22),
            walnut, grip_leather, target,
        );
        emit_prism(
            base_pt(-0.245, 0.24, -0.04), base_pt(0.245, 0.24, -0.04), base_pt(0.245, 0.25, 0.06), base_pt(-0.245, 0.25, 0.06),
            base_pt(-0.245, 0.27, -0.04), base_pt(0.245, 0.27, -0.04), base_pt(0.245, 0.28, 0.06), base_pt(-0.245, 0.28, 0.06),
            brass, grip_leather, target,
        );
        for (gx, dial_col) in [(-0.14f32, Vec3::new(0.26, 0.90, 0.82)), (0.0f32, brass), (0.14f32, marker_red)] {
            target.push_quad(Quad::glowing(
                base_pt(gx - 0.036, 0.13, -0.052), base_pt(gx + 0.036, 0.13, -0.052),
                base_pt(gx + 0.036, 0.21, -0.032), base_pt(gx - 0.036, 0.21, -0.032),
                -fwd, dial_col, 0.88,
            ));
        }
        target.push_quad(Quad::glowing(
            base_pt(-0.045, 0.285, 0.07), base_pt(0.045, 0.285, 0.07), base_pt(0.0, 0.292, 0.18), base_pt(0.0, 0.292, 0.18),
            up, brass, 0.85,
        ));
        emit_prism(
            base_pt(-0.045, 0.16, -0.02), base_pt(0.045, 0.16, -0.02), base_pt(0.045, 0.18, 0.06), base_pt(-0.045, 0.18, 0.06),
            base_pt(-0.032, 0.30, -0.09), base_pt(0.032, 0.30, -0.09), base_pt(0.032, 0.33, -0.03), base_pt(-0.032, 0.33, -0.03),
            iron_base, column_steel, target,
        );

        let hub = base_pt(0.0, 0.33, -0.08);
        let col_axis = (-fwd * 0.84 + up * 0.54).normalize_or_zero();
        let w_up = col_axis.cross(right).normalize_or_zero();
        let steer_phi = -self.motion.1 * 0.85;
        let wheel_pt = |angle: f32, r: f32, z: f32| {
            let (s, c) = (angle + steer_phi).sin_cos();
            hub + (right * s + w_up * c) * r + col_axis * z
        };
        let (segs, r_in, r_out, r_hub, hz) = (8usize, 0.125f32, 0.165f32, 0.042f32, 0.018f32);
        let step = (2.0 * PI) / (segs as f32);
        for i in 0..segs {
            let (a0, a1) = ((i as f32) * step, ((i + 1) as f32) * step);
            let rim_col = if i == 0 { marker_red } else if i % 2 == 0 { grip_leather } else { grip_seam };
            emit_prism(
                wheel_pt(a0, r_in, -hz), wheel_pt(a1, r_in, -hz), wheel_pt(a1, r_out, -hz), wheel_pt(a0, r_out, -hz),
                wheel_pt(a0, r_in, hz), wheel_pt(a1, r_in, hz), wheel_pt(a1, r_out, hz), wheel_pt(a0, r_out, hz),
                rim_col, rim_col, target,
            );
            let [h0, h1, h2] = [hub + col_axis * (hz + 0.008), wheel_pt(a0, r_hub, hz + 0.008), wheel_pt(a1, r_hub, hz + 0.008)];
            target.push_quad(Quad::glowing(h0, h2, h1, h1, col_axis, brass, 0.72));
        }
        for spoke_ang in [0.52 * PI, PI, 1.48 * PI] {
            emit_prism(
                wheel_pt(spoke_ang - 0.12, r_hub * 0.8, -0.010),
                wheel_pt(spoke_ang + 0.12, r_hub * 0.8, -0.010),
                wheel_pt(spoke_ang + 0.06, r_in + 0.006, -0.010),
                wheel_pt(spoke_ang - 0.06, r_in + 0.006, -0.010),
                wheel_pt(spoke_ang - 0.12, r_hub * 0.8, 0.010),
                wheel_pt(spoke_ang + 0.12, r_hub * 0.8, 0.010),
                wheel_pt(spoke_ang + 0.06, r_in + 0.006, 0.010),
                wheel_pt(spoke_ang - 0.06, r_in + 0.006, 0.010),
                column_steel, spoke_chrome, target,
            );
        }
    }

    fn emit_engine(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top_elev) = (cell.u_bounds(), cell.v_bounds(), floor_elevation + 0.5);
        let (chassis, rib, top_glow, lens_glow) = match self.medium {
            BlockMedium::AircraftEngine => (
                Vec3::new(0.48, 0.56, 0.66),
                Vec3::new(0.74, 0.82, 0.90),
                Vec3::new(0.20, 0.86, 0.98),
                Vec3::new(0.56, 0.96, 1.00),
            ),
            _ => (
                Vec3::new(0.22, 0.20, 0.26),
                Vec3::new(0.84, 0.64, 0.22),
                Vec3::new(0.99, 0.32, 0.12),
                Vec3::new(0.98, 0.18, 0.46),
            ),
        };
        BlockMedium::emit_box(u0, u1, v0, v1, floor_elevation, top_elev, frame, chassis, rib, true, target);
        BlockMedium::emit_box(u0 + 0.10, u1 - 0.10, v0 + 0.10, v1 - 0.10, top_elev, top_elev + 0.016, frame, top_glow, top_glow, false, target);
        let (n, t, b) = (frame.normal(), frame.tangent(), frame.bitangent());
        let [p0, p1, p2, p3] = [
            frame.point_at(u0 + 0.11, v0 + 0.11, top_elev + 0.017),
            frame.point_at(u1 - 0.11, v0 + 0.11, top_elev + 0.017),
            frame.point_at(u1 - 0.11, v1 - 0.11, top_elev + 0.017),
            frame.point_at(u0 + 0.11, v1 - 0.11, top_elev + 0.017),
        ];
        target.push_quad(Quad::glowing(p0, p1, p2, p3, n, top_glow, 0.92));
        let (c0, c1, c2, c3, ln) = match self.heading {
            WedgeHeading::East => (
                frame.point_at(u1 + 0.005, v0 + 0.13, floor_elevation + 0.13),
                frame.point_at(u1 + 0.005, v1 - 0.13, floor_elevation + 0.13),
                frame.point_at(u1 + 0.005, v1 - 0.13, top_elev - 0.13),
                frame.point_at(u1 + 0.005, v0 + 0.13, top_elev - 0.13),
                t,
            ),
            WedgeHeading::West => (
                frame.point_at(u0 - 0.005, v1 - 0.13, floor_elevation + 0.13),
                frame.point_at(u0 - 0.005, v0 + 0.13, floor_elevation + 0.13),
                frame.point_at(u0 - 0.005, v0 + 0.13, top_elev - 0.13),
                frame.point_at(u0 - 0.005, v1 - 0.13, top_elev - 0.13),
                -t,
            ),
            WedgeHeading::North => (
                frame.point_at(u1 - 0.13, v1 + 0.005, floor_elevation + 0.13),
                frame.point_at(u0 + 0.13, v1 + 0.005, floor_elevation + 0.13),
                frame.point_at(u0 + 0.13, v1 + 0.005, top_elev - 0.13),
                frame.point_at(u1 - 0.13, v1 + 0.005, top_elev - 0.13),
                b,
            ),
            WedgeHeading::South => (
                frame.point_at(u0 + 0.13, v0 - 0.005, floor_elevation + 0.13),
                frame.point_at(u1 - 0.13, v0 - 0.005, floor_elevation + 0.13),
                frame.point_at(u1 - 0.13, v0 - 0.005, top_elev - 0.13),
                frame.point_at(u0 + 0.13, v0 - 0.005, top_elev - 0.13),
                -b,
            ),
        };
        target.push_quad(Quad::glowing(c0, c1, c2, c3, ln, lens_glow, 0.95));
    }

    fn emit_wheel(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let profile = WheelProfile::from_archetype(self.archetype);
        let (r, hw, spin) = (profile.radius(), profile.half_width(), self.motion.0);
        let (rim_r, hub_r) = (r * 0.56, r * 0.28);
        let (uc, vc) = cell.center();
        let hc = floor_elev + r;
        let (rubber, tread, rim, hub) = (Vec3::new(0.15, 0.17, 0.20), Vec3::new(0.10, 0.11, 0.13), Vec3::new(0.68, 0.72, 0.78), Vec3::new(0.90, 0.70, 0.22));
        let cross = self.cross_grain();
        let pt = |roll: f32, axle: f32, h: f32| -> Vec3 {
            if cross { frame.point_at(uc + roll, vc + axle, hc + h) } else { frame.point_at(uc + axle, vc + roll, hc + h) }
        };
        let axle_norm = if cross { frame.bitangent() } else { frame.tangent() };
        let (segs, step) = (8usize, (2.0 * PI) / 8.0);
        // Single-sided connecting steel axle rod & brass mounting collar anchoring the wheel to the adjacent block
        match self.heading {
            WedgeHeading::East => {
                BlockMedium::emit_box(uc - 0.030, uc + 0.030, vc, vc + 0.25, hc - 0.030, hc + 0.030, frame, rim, rim, true, target);
                BlockMedium::emit_box(uc - 0.052, uc + 0.052, vc + 0.21, vc + 0.25, hc - 0.052, hc + 0.052, frame, hub, hub, true, target);
            }
            WedgeHeading::West => {
                BlockMedium::emit_box(uc - 0.030, uc + 0.030, vc - 0.25, vc, hc - 0.030, hc + 0.030, frame, rim, rim, true, target);
                BlockMedium::emit_box(uc - 0.052, uc + 0.052, vc - 0.25, vc - 0.21, hc - 0.052, hc + 0.052, frame, hub, hub, true, target);
            }
            WedgeHeading::North => {
                BlockMedium::emit_box(uc - 0.25, uc, vc - 0.030, vc + 0.030, hc - 0.030, hc + 0.030, frame, rim, rim, true, target);
                BlockMedium::emit_box(uc - 0.25, uc - 0.21, vc - 0.052, vc + 0.052, hc - 0.052, hc + 0.052, frame, hub, hub, true, target);
            }
            WedgeHeading::South => {
                BlockMedium::emit_box(uc, uc + 0.25, vc - 0.030, vc + 0.030, hc - 0.030, hc + 0.030, frame, rim, rim, true, target);
                BlockMedium::emit_box(uc + 0.21, uc + 0.25, vc - 0.052, vc + 0.052, hc - 0.052, hc + 0.052, frame, hub, hub, true, target);
            }
        }
        for i in 0..segs {
            let (a0, a1) = ((i as f32) * step + spin, ((i + 1) as f32) * step + spin);
            let ((s0, c0), (s1, c1)) = (a0.sin_cos(), a1.sin_cos());
            let (lug, col) = if i % 2 == 0 { (r * 1.07, tread) } else { (r, rubber) };
            let [t0, t1, t2, t3] = [pt(s0 * lug, -hw, c0 * lug), pt(s1 * lug, -hw, c1 * lug), pt(s1 * lug, hw, c1 * lug), pt(s0 * lug, hw, c0 * lug)];
            target.push_quad(Quad::new(t0, t1, t2, t3, (t1 - t0).cross(t3 - t0).normalize_or_zero(), col));
            for (side_a, norm_s) in [(-hw, -axle_norm), (hw, axle_norm)] {
                let [w0, w1, w2, w3] = [pt(s0 * rim_r, side_a, c0 * rim_r), pt(s1 * rim_r, side_a, c1 * rim_r), pt(s1 * lug, side_a, c1 * lug), pt(s0 * lug, side_a, c0 * lug)];
                let spoke_col = if i % 2 == 0 { hub } else { rim };
                let [m0, m1, m2, m3] = [pt(s0 * hub_r, side_a, c0 * hub_r), pt(s1 * hub_r, side_a, c1 * hub_r), pt(s1 * rim_r, side_a, c1 * rim_r), pt(s0 * rim_r, side_a, c0 * rim_r)];
                let [h0, h1, h2] = [pt(0.0, side_a, 0.0), pt(s0 * hub_r, side_a, c0 * hub_r), pt(s1 * hub_r, side_a, c1 * hub_r)];
                if side_a > 0.0 {
                    target.push_quad(Quad::new(w0, w1, w2, w3, norm_s, rubber));
                    target.push_quad(Quad::new(m0, m1, m2, m3, norm_s, spoke_col));
                    target.push_quad(Quad::new(h0, h1, h2, h2, norm_s, hub));
                } else {
                    target.push_quad(Quad::new(w3, w2, w1, w0, norm_s, rubber));
                    target.push_quad(Quad::new(m3, m2, m1, m0, norm_s, spoke_col));
                    target.push_quad(Quad::new(h0, h2, h1, h1, norm_s, hub));
                }
            }
        }
    }
}

