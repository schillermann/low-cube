use std::f32::consts::PI;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cabin_seat::CabinSeatMesh;
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

/// Sculpted 3D geometry emitter for vehicle engines, cabin seats, and rolling wheels.
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
            BlockMedium::CarEngine => self.emit_car_steering(cell, floor_elevation, frame, target),
            BlockMedium::CabinSeat => CabinSeatMesh::new(self.heading, self.archetype).emit(cell, floor_elevation, frame, target),
            BlockMedium::CarWheel => self.emit_wheel(cell, floor_elevation, frame, target),
            _ => self.emit_engine(cell, floor_elevation, frame, target),
        }
    }

    fn emit_car_steering(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let steer_shift = (self.motion.1 * 0.035).clamp(-0.04, 0.04);
        let (chassis, hood, column, wheel_rim, brass, beacon) = (
            Vec3::new(0.26, 0.29, 0.34),
            Vec3::new(0.36, 0.40, 0.46),
            Vec3::new(0.22, 0.25, 0.30),
            Vec3::new(0.14, 0.16, 0.20),
            Vec3::new(0.92, 0.74, 0.26),
            Vec3::new(0.99, 0.78, 0.20),
        );
        BlockMedium::emit_box(u0 + 0.02, u1 - 0.02, v0 + 0.02, v1 - 0.02, floor_elev, floor_elev + 0.10, frame, chassis, hood, true, target);
        match self.heading {
            WedgeHeading::East => {
                BlockMedium::emit_box(u0 + 0.22, u1 - 0.02, v0 + 0.04, v1 - 0.04, floor_elev + 0.10, floor_elev + 0.30, frame, chassis, hood, false, target);
                BlockMedium::emit_box(u1 - 0.04, u1 + 0.01, v0 + 0.10, v1 - 0.10, floor_elev + 0.13, floor_elev + 0.26, frame, beacon, beacon, false, target);
                BlockMedium::emit_box(u0 + 0.11, u0 + 0.24, v0 + 0.19, v1 - 0.19, floor_elev + 0.16, floor_elev + 0.38, frame, column, column, false, target);
                BlockMedium::emit_box(u0 + 0.05, u0 + 0.12, v0 + 0.10 - steer_shift, v1 - 0.10 - steer_shift, floor_elev + 0.28, floor_elev + 0.48, frame, wheel_rim, brass, true, target);
            }
            WedgeHeading::West => {
                BlockMedium::emit_box(u0 + 0.02, u1 - 0.22, v0 + 0.04, v1 - 0.04, floor_elev + 0.10, floor_elev + 0.30, frame, chassis, hood, false, target);
                BlockMedium::emit_box(u0 - 0.01, u0 + 0.04, v0 + 0.10, v1 - 0.10, floor_elev + 0.13, floor_elev + 0.26, frame, beacon, beacon, false, target);
                BlockMedium::emit_box(u1 - 0.24, u1 - 0.11, v0 + 0.19, v1 - 0.19, floor_elev + 0.16, floor_elev + 0.38, frame, column, column, false, target);
                BlockMedium::emit_box(u1 - 0.12, u1 - 0.05, v0 + 0.10 + steer_shift, v1 - 0.10 + steer_shift, floor_elev + 0.28, floor_elev + 0.48, frame, wheel_rim, brass, true, target);
            }
            WedgeHeading::North => {
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.22, v1 - 0.02, floor_elev + 0.10, floor_elev + 0.30, frame, chassis, hood, false, target);
                BlockMedium::emit_box(u0 + 0.10, u1 - 0.10, v1 - 0.04, v1 + 0.01, floor_elev + 0.13, floor_elev + 0.26, frame, beacon, beacon, false, target);
                BlockMedium::emit_box(u0 + 0.19, u1 - 0.19, v0 + 0.11, v0 + 0.24, floor_elev + 0.16, floor_elev + 0.38, frame, column, column, false, target);
                BlockMedium::emit_box(u0 + 0.10 + steer_shift, u1 - 0.10 + steer_shift, v0 + 0.05, v0 + 0.12, floor_elev + 0.28, floor_elev + 0.48, frame, wheel_rim, brass, true, target);
            }
            WedgeHeading::South => {
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.02, v1 - 0.22, floor_elev + 0.10, floor_elev + 0.30, frame, chassis, hood, false, target);
                BlockMedium::emit_box(u0 + 0.10, u1 - 0.10, v0 - 0.01, v0 + 0.04, floor_elev + 0.13, floor_elev + 0.26, frame, beacon, beacon, false, target);
                BlockMedium::emit_box(u0 + 0.19, u1 - 0.19, v1 - 0.24, v1 - 0.11, floor_elev + 0.16, floor_elev + 0.38, frame, column, column, false, target);
                BlockMedium::emit_box(u0 + 0.10 - steer_shift, u1 - 0.10 - steer_shift, v1 - 0.12, v1 - 0.05, floor_elev + 0.28, floor_elev + 0.48, frame, wheel_rim, brass, true, target);
            }
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
        let (rubber, tread, rim, hub) = (
            Vec3::new(0.15, 0.17, 0.20),
            Vec3::new(0.10, 0.11, 0.13),
            Vec3::new(0.68, 0.72, 0.78),
            Vec3::new(0.90, 0.70, 0.22),
        );
        let cross = self.cross_grain();
        let pt = |roll: f32, axle: f32, h: f32| -> Vec3 {
            if cross {
                frame.point_at(uc + roll, vc + axle, hc + h)
            } else {
                frame.point_at(uc + axle, vc + roll, hc + h)
            }
        };
        let axle_norm = if cross { frame.bitangent() } else { frame.tangent() };
        let segs = 8usize;
        let step = (2.0 * PI) / (segs as f32);
        for i in 0..segs {
            let a0 = (i as f32) * step + spin;
            let a1 = ((i + 1) as f32) * step + spin;
            let (s0, c0) = a0.sin_cos();
            let (s1, c1) = a1.sin_cos();
            let lug = if i % 2 == 0 { r * 1.07 } else { r };
            let col = if i % 2 == 0 { tread } else { rubber };
            let [t0, t1, t2, t3] = [
                pt(s0 * lug, -hw, c0 * lug),
                pt(s1 * lug, -hw, c1 * lug),
                pt(s1 * lug, hw, c1 * lug),
                pt(s0 * lug, hw, c0 * lug),
            ];
            let n_out = (t1 - t0).cross(t3 - t0).normalize_or_zero();
            target.push_quad(Quad::new(t0, t1, t2, t3, n_out, col));
            for (side_a, norm_s) in [(-hw, -axle_norm), (hw, axle_norm)] {
                let [w0, w1, w2, w3] = [
                    pt(s0 * rim_r, side_a, c0 * rim_r),
                    pt(s1 * rim_r, side_a, c1 * rim_r),
                    pt(s1 * lug, side_a, c1 * lug),
                    pt(s0 * lug, side_a, c0 * lug),
                ];
                let spoke_col = if i % 2 == 0 { hub } else { rim };
                let [m0, m1, m2, m3] = [
                    pt(s0 * hub_r, side_a, c0 * hub_r),
                    pt(s1 * hub_r, side_a, c1 * hub_r),
                    pt(s1 * rim_r, side_a, c1 * rim_r),
                    pt(s0 * rim_r, side_a, c0 * rim_r),
                ];
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
