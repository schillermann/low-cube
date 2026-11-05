use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::timber_grain::BlockMedium;
use super::wedge::WedgeHeading;

/// Sculpted 3D geometry emitter for vehicle engines, cabin seats, and rolling wheels.
pub struct VehicleBlockMesh {
    medium: BlockMedium,
    heading: WedgeHeading,
}

impl VehicleBlockMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(medium: BlockMedium, heading: WedgeHeading) -> Self {
        Self { medium, heading }
    }

    const fn cross_grain(&self) -> bool {
        matches!(self.heading, WedgeHeading::East | WedgeHeading::West)
    }

    pub fn emit(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        match self.medium {
            BlockMedium::CarEngine => self.emit_car_steering(cell, floor_elevation, frame, target),
            BlockMedium::CabinSeat => self.emit_seat(cell, floor_elevation, frame, target),
            BlockMedium::CarWheel => self.emit_wheel(cell, floor_elevation, frame, target),
            _ => self.emit_engine(cell, floor_elevation, frame, target),
        }
    }

    fn emit_car_steering(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
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
                BlockMedium::emit_box(u0 + 0.05, u0 + 0.12, v0 + 0.10, v1 - 0.10, floor_elev + 0.28, floor_elev + 0.48, frame, wheel_rim, brass, true, target);
            }
            WedgeHeading::West => {
                BlockMedium::emit_box(u0 + 0.02, u1 - 0.22, v0 + 0.04, v1 - 0.04, floor_elev + 0.10, floor_elev + 0.30, frame, chassis, hood, false, target);
                BlockMedium::emit_box(u0 - 0.01, u0 + 0.04, v0 + 0.10, v1 - 0.10, floor_elev + 0.13, floor_elev + 0.26, frame, beacon, beacon, false, target);
                BlockMedium::emit_box(u1 - 0.24, u1 - 0.11, v0 + 0.19, v1 - 0.19, floor_elev + 0.16, floor_elev + 0.38, frame, column, column, false, target);
                BlockMedium::emit_box(u1 - 0.12, u1 - 0.05, v0 + 0.10, v1 - 0.10, floor_elev + 0.28, floor_elev + 0.48, frame, wheel_rim, brass, true, target);
            }
            WedgeHeading::North => {
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.22, v1 - 0.02, floor_elev + 0.10, floor_elev + 0.30, frame, chassis, hood, false, target);
                BlockMedium::emit_box(u0 + 0.10, u1 - 0.10, v1 - 0.04, v1 + 0.01, floor_elev + 0.13, floor_elev + 0.26, frame, beacon, beacon, false, target);
                BlockMedium::emit_box(u0 + 0.19, u1 - 0.19, v0 + 0.11, v0 + 0.24, floor_elev + 0.16, floor_elev + 0.38, frame, column, column, false, target);
                BlockMedium::emit_box(u0 + 0.10, u1 - 0.10, v0 + 0.05, v0 + 0.12, floor_elev + 0.28, floor_elev + 0.48, frame, wheel_rim, brass, true, target);
            }
            WedgeHeading::South => {
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.02, v1 - 0.22, floor_elev + 0.10, floor_elev + 0.30, frame, chassis, hood, false, target);
                BlockMedium::emit_box(u0 + 0.10, u1 - 0.10, v0 - 0.01, v0 + 0.04, floor_elev + 0.13, floor_elev + 0.26, frame, beacon, beacon, false, target);
                BlockMedium::emit_box(u0 + 0.19, u1 - 0.19, v1 - 0.24, v1 - 0.11, floor_elev + 0.16, floor_elev + 0.38, frame, column, column, false, target);
                BlockMedium::emit_box(u0 + 0.10, u1 - 0.10, v1 - 0.12, v1 - 0.05, floor_elev + 0.28, floor_elev + 0.48, frame, wheel_rim, brass, true, target);
            }
        }
    }

    fn emit_engine(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top_elev) = (cell.u_bounds(), cell.v_bounds(), floor_elevation + 0.5);
        let (chassis, rib, top_glow, lens_glow) = match self.medium {
            BlockMedium::CarEngine => (
                Vec3::new(0.34, 0.36, 0.40),
                Vec3::new(0.72, 0.56, 0.26),
                Vec3::new(0.98, 0.68, 0.16),
                Vec3::new(0.99, 0.82, 0.28),
            ),
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

    fn emit_seat(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (iron, leather, cushion, trim) = (
            Vec3::new(0.24, 0.27, 0.32),
            Vec3::new(0.62, 0.20, 0.14),
            Vec3::new(0.76, 0.27, 0.18),
            Vec3::new(0.46, 0.15, 0.11),
        );
        BlockMedium::emit_box(u0 + 0.02, u1 - 0.02, v0 + 0.02, v1 - 0.02, floor_elev, floor_elev + 0.08, frame, iron, iron, true, target);
        BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.04, v1 - 0.04, floor_elev + 0.08, floor_elev + 0.20, frame, leather, cushion, false, target);
        match self.heading {
            WedgeHeading::East => {
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.02, v0 + 0.11, floor_elev + 0.18, floor_elev + 0.32, frame, trim, leather, false, target);
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v1 - 0.11, v1 - 0.02, floor_elev + 0.18, floor_elev + 0.32, frame, trim, leather, false, target);
                BlockMedium::emit_box(u0 + 0.03, u0 + 0.14, v0 + 0.06, v1 - 0.06, floor_elev + 0.20, floor_elev + 0.48, frame, leather, cushion, false, target);
            }
            WedgeHeading::West => {
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.02, v0 + 0.11, floor_elev + 0.18, floor_elev + 0.32, frame, trim, leather, false, target);
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v1 - 0.11, v1 - 0.02, floor_elev + 0.18, floor_elev + 0.32, frame, trim, leather, false, target);
                BlockMedium::emit_box(u1 - 0.14, u1 - 0.03, v0 + 0.06, v1 - 0.06, floor_elev + 0.20, floor_elev + 0.48, frame, leather, cushion, false, target);
            }
            WedgeHeading::North => {
                BlockMedium::emit_box(u0 + 0.02, u0 + 0.11, v0 + 0.04, v1 - 0.04, floor_elev + 0.18, floor_elev + 0.32, frame, trim, leather, false, target);
                BlockMedium::emit_box(u1 - 0.11, u1 - 0.02, v0 + 0.04, v1 - 0.04, floor_elev + 0.18, floor_elev + 0.32, frame, trim, leather, false, target);
                BlockMedium::emit_box(u0 + 0.06, u1 - 0.06, v0 + 0.03, v0 + 0.14, floor_elev + 0.20, floor_elev + 0.48, frame, leather, cushion, false, target);
            }
            WedgeHeading::South => {
                BlockMedium::emit_box(u0 + 0.02, u0 + 0.11, v0 + 0.04, v1 - 0.04, floor_elev + 0.18, floor_elev + 0.32, frame, trim, leather, false, target);
                BlockMedium::emit_box(u1 - 0.11, u1 - 0.02, v0 + 0.04, v1 - 0.04, floor_elev + 0.18, floor_elev + 0.32, frame, trim, leather, false, target);
                BlockMedium::emit_box(u0 + 0.06, u1 - 0.06, v1 - 0.14, v1 - 0.03, floor_elev + 0.20, floor_elev + 0.48, frame, leather, cushion, false, target);
            }
        }
    }

    fn emit_wheel(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (rubber, tread, rim, hub) = (
            Vec3::new(0.15, 0.17, 0.20),
            Vec3::new(0.10, 0.11, 0.13),
            Vec3::new(0.68, 0.72, 0.78),
            Vec3::new(0.90, 0.70, 0.22),
        );
        if self.cross_grain() {
            BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.10, v1 - 0.10, floor_elev + 0.05, floor_elev + 0.45, frame, rubber, tread, true, target);
            BlockMedium::emit_box(u0 + 0.11, u1 - 0.11, v0 + 0.08, v1 - 0.08, floor_elev, floor_elev + 0.50, frame, tread, tread, true, target);
            BlockMedium::emit_box(u0, u1, v0 + 0.09, v1 - 0.09, floor_elev + 0.11, floor_elev + 0.39, frame, tread, rubber, true, target);
            BlockMedium::emit_box(u0 + 0.13, u1 - 0.13, v0 + 0.06, v1 - 0.06, floor_elev + 0.13, floor_elev + 0.37, frame, rim, rim, true, target);
            BlockMedium::emit_box(u0 + 0.19, u1 - 0.19, v0 + 0.03, v1 - 0.03, floor_elev + 0.19, floor_elev + 0.31, frame, hub, hub, true, target);
        } else {
            BlockMedium::emit_box(u0 + 0.10, u1 - 0.10, v0 + 0.04, v1 - 0.04, floor_elev + 0.05, floor_elev + 0.45, frame, rubber, tread, true, target);
            BlockMedium::emit_box(u0 + 0.08, u1 - 0.08, v0 + 0.11, v1 - 0.11, floor_elev, floor_elev + 0.50, frame, tread, tread, true, target);
            BlockMedium::emit_box(u0 + 0.09, u1 - 0.09, v0, v1, floor_elev + 0.11, floor_elev + 0.39, frame, tread, rubber, true, target);
            BlockMedium::emit_box(u0 + 0.06, u1 - 0.06, v0 + 0.13, v1 - 0.13, floor_elev + 0.13, floor_elev + 0.37, frame, rim, rim, true, target);
            BlockMedium::emit_box(u0 + 0.03, u1 - 0.03, v0 + 0.19, v1 - 0.19, floor_elev + 0.19, floor_elev + 0.31, frame, hub, hub, true, target);
        }
    }
}
