mod steering;

use std::f32::consts::PI;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::mold::MoldArchetype;
use super::timber_grain::BlockMedium;
use super::wedge::WedgeHeading;
use self::steering::SteeringColumn;

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
            BlockMedium::SteeringWheel => SteeringColumn::new(self.heading, self.motion.1).emit(cell, floor_elevation, frame, target),
            BlockMedium::CarWheel => self.emit_wheel(cell, floor_elevation, frame, target),
            BlockMedium::CrossAxle => self.emit_cross_axle(cell, floor_elevation, frame, target),
            _ => self.emit_engine(cell, floor_elevation, frame, target),
        }
    }

    fn emit_engine(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top_elev) = (cell.u_bounds(), cell.v_bounds(), floor_elevation + 0.5);
        let (chassis, rib, top_glow, lens_glow) = match self.medium {
            BlockMedium::AircraftEngine => (
                Vec3::new(0.48, 0.56, 0.66), Vec3::new(0.74, 0.82, 0.90),
                Vec3::new(0.20, 0.86, 0.98), Vec3::new(0.56, 0.96, 1.00),
            ),
            _ => (
                Vec3::new(0.22, 0.20, 0.26), Vec3::new(0.84, 0.64, 0.22),
                Vec3::new(0.99, 0.32, 0.12), Vec3::new(0.98, 0.18, 0.46),
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
                frame.point_at(u1 + 0.005, v0 + 0.13, floor_elevation + 0.13), frame.point_at(u1 + 0.005, v1 - 0.13, floor_elevation + 0.13),
                frame.point_at(u1 + 0.005, v1 - 0.13, top_elev - 0.13), frame.point_at(u1 + 0.005, v0 + 0.13, top_elev - 0.13), t,
            ),
            WedgeHeading::West => (
                frame.point_at(u0 - 0.005, v1 - 0.13, floor_elevation + 0.13), frame.point_at(u0 - 0.005, v0 + 0.13, floor_elevation + 0.13),
                frame.point_at(u0 - 0.005, v0 + 0.13, top_elev - 0.13), frame.point_at(u0 - 0.005, v1 - 0.13, top_elev - 0.13), -t,
            ),
            WedgeHeading::North => (
                frame.point_at(u1 - 0.13, v1 + 0.005, floor_elevation + 0.13), frame.point_at(u0 + 0.13, v1 + 0.005, floor_elevation + 0.13),
                frame.point_at(u0 + 0.13, v1 + 0.005, top_elev - 0.13), frame.point_at(u1 - 0.13, v1 + 0.005, top_elev - 0.13), b,
            ),
            WedgeHeading::South => (
                frame.point_at(u0 + 0.13, v0 - 0.005, floor_elevation + 0.13), frame.point_at(u1 - 0.13, v0 - 0.005, floor_elevation + 0.13),
                frame.point_at(u1 - 0.13, v0 - 0.005, top_elev - 0.13), frame.point_at(u0 + 0.13, v0 - 0.005, top_elev - 0.13), -b,
            ),
        };
        target.push_quad(Quad::glowing(c0, c1, c2, c3, ln, lens_glow, 0.95));
    }

    fn emit_cross_axle(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), (uc, vc)) = (cell.u_bounds(), cell.v_bounds(), cell.center());
        let hc = floor_elev + 0.11;
        let (walnut, iron_dark, axle_steel, brass) = (
            Vec3::new(0.32, 0.18, 0.11),
            Vec3::new(0.20, 0.22, 0.27),
            Vec3::new(0.66, 0.70, 0.76),
            Vec3::new(0.90, 0.70, 0.22),
        );
        BlockMedium::emit_box(u0, u1, vc - 0.042, vc + 0.042, hc - 0.042, hc + 0.042, frame, axle_steel, axle_steel, true, target);
        BlockMedium::emit_box(uc - 0.042, uc + 0.042, v0, v1, hc - 0.042, hc + 0.042, frame, axle_steel, axle_steel, true, target);
        for (a0, a1) in [(u0, u0 + 0.036), (u1 - 0.036, u1)] {
            BlockMedium::emit_box(a0, a1, vc - 0.064, vc + 0.064, hc - 0.064, hc + 0.064, frame, brass, brass, true, target);
        }
        for (b0, b1) in [(v0, v0 + 0.036), (v1 - 0.036, v1)] {
            BlockMedium::emit_box(uc - 0.064, uc + 0.064, b0, b1, hc - 0.064, hc + 0.064, frame, brass, brass, true, target);
        }
        BlockMedium::emit_box(uc - 0.11, uc + 0.11, vc - 0.11, vc + 0.11, floor_elev + 0.02, floor_elev + 0.24, frame, iron_dark, brass, true, target);
        BlockMedium::emit_box(uc - 0.16, uc + 0.16, vc - 0.06, vc + 0.06, floor_elev + 0.22, floor_elev + 0.43, frame, iron_dark, iron_dark, false, target);
        BlockMedium::emit_box(uc - 0.06, uc + 0.06, vc - 0.16, vc + 0.16, floor_elev + 0.22, floor_elev + 0.43, frame, iron_dark, iron_dark, false, target);
        BlockMedium::emit_box(u0, u1, v0, v1, floor_elev + 0.43, floor_elev + 0.50, frame, walnut, iron_dark, true, target);
    }

    fn emit_wheel(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let profile = WheelProfile::from_archetype(self.archetype);
        let (r, hw, spin) = (profile.radius(), profile.half_width(), self.motion.0);
        let (rim_r, hub_r) = (r * 0.56, r * 0.28);
        let (uc, vc) = cell.center();
        let hc = floor_elev + 0.11;
        let (rubber, tread, rim, hub) = (Vec3::new(0.15, 0.17, 0.20), Vec3::new(0.10, 0.11, 0.13), Vec3::new(0.68, 0.72, 0.78), Vec3::new(0.90, 0.70, 0.22));
        let cross = self.cross_grain();
        let axle_shift = match self.heading {
            WedgeHeading::East | WedgeHeading::North => 0.25 - hw,
            WedgeHeading::West | WedgeHeading::South => -0.25 + hw,
        };
        let pt = |roll: f32, axle: f32, h: f32| -> Vec3 {
            let a = axle_shift + axle;
            if cross { frame.point_at(uc + a, vc + roll, hc + h) } else { frame.point_at(uc + roll, vc + a, hc + h) }
        };
        let axle_norm = if cross { frame.tangent() } else { frame.bitangent() };
        let (segs, step) = (8usize, (2.0 * PI) / 8.0);
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
