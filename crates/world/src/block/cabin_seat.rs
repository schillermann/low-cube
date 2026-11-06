use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::cell::GridCell;
use super::mold::MoldArchetype;
use super::timber_grain::BlockMedium;
use super::wedge::WedgeHeading;

/// Distinct 3D seat forms selectable by cycling block form archetypes for vehicles and world-cube furniture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeatProfile {
    Bucket,
    ParkBench,
    WorkshopStool,
    RegalThrone,
}

impl SeatProfile {
    pub const fn from_archetype(archetype: MoldArchetype) -> Self {
        match archetype.code() % 4 {
            0 => Self::Bucket,
            1 => Self::ParkBench,
            2 => Self::WorkshopStool,
            _ => Self::RegalThrone,
        }
    }
}

/// Oriented 3D geometry emitter for multi-form cabin seats, park benches, stools, and thrones.
pub struct CabinSeatMesh {
    heading: WedgeHeading,
    archetype: MoldArchetype,
}

impl CabinSeatMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(heading: WedgeHeading, archetype: MoldArchetype) -> Self {
        Self { heading, archetype }
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        match SeatProfile::from_archetype(self.archetype) {
            SeatProfile::Bucket => self.emit_bucket(cell, floor_elev, frame, target),
            SeatProfile::ParkBench => self.emit_park_bench(cell, floor_elev, frame, target),
            SeatProfile::WorkshopStool => self.emit_workshop_stool(cell, floor_elev, frame, target),
            SeatProfile::RegalThrone => self.emit_regal_throne(cell, floor_elev, frame, target),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_part(
        &self,
        cell: GridCell,
        floor_elev: f32,
        depth: (f32, f32),
        side: (f32, f32),
        height: (f32, f32),
        frame: &SurfaceFrame,
        wall_col: Vec3,
        top_col: Vec3,
        bottom_cap: bool,
        target: &mut dyn MeshTarget,
    ) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let ((d0, d1), (s0, s1), (h0, h1)) = (depth, side, height);
        let (bu0, bu1, bv0, bv1) = match self.heading {
            WedgeHeading::East => (u0 + d0, u0 + d1, v0 + s0, v0 + s1),
            WedgeHeading::West => (u1 - d1, u1 - d0, v0 + s0, v0 + s1),
            WedgeHeading::North => (u0 + s0, u0 + s1, v0 + d0, v0 + d1),
            WedgeHeading::South => (u0 + s0, u0 + s1, v1 - d1, v1 - d0),
        };
        BlockMedium::emit_box(bu0, bu1, bv0, bv1, floor_elev + h0, floor_elev + h1, frame, wall_col, top_col, bottom_cap, target);
    }

    fn emit_bucket(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (iron, leather, cushion, trim) = (
            Vec3::new(0.24, 0.27, 0.32),
            Vec3::new(0.62, 0.20, 0.14),
            Vec3::new(0.76, 0.27, 0.18),
            Vec3::new(0.46, 0.15, 0.11),
        );
        self.emit_part(cell, floor_elev, (0.02, 0.48), (0.02, 0.48), (0.00, 0.08), frame, iron, iron, true, target);
        self.emit_part(cell, floor_elev, (0.04, 0.46), (0.04, 0.46), (0.08, 0.20), frame, leather, cushion, false, target);
        self.emit_part(cell, floor_elev, (0.04, 0.46), (0.02, 0.11), (0.18, 0.32), frame, trim, leather, false, target);
        self.emit_part(cell, floor_elev, (0.04, 0.46), (0.39, 0.48), (0.18, 0.32), frame, trim, leather, false, target);
        self.emit_part(cell, floor_elev, (0.03, 0.14), (0.06, 0.44), (0.20, 0.48), frame, leather, cushion, false, target);
    }

    fn emit_park_bench(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (cast_iron, oak_wall, oak_top) = (
            Vec3::new(0.18, 0.20, 0.23),
            Vec3::new(0.68, 0.48, 0.26),
            Vec3::new(0.82, 0.62, 0.36),
        );
        self.emit_part(cell, floor_elev, (0.06, 0.44), (0.03, 0.09), (0.00, 0.17), frame, cast_iron, cast_iron, true, target);
        self.emit_part(cell, floor_elev, (0.06, 0.44), (0.41, 0.47), (0.00, 0.17), frame, cast_iron, cast_iron, true, target);
        for slat in 0..3 {
            let d0 = 0.12 + (slat as f32) * 0.12;
            self.emit_part(cell, floor_elev, (d0, d0 + 0.09), (0.01, 0.49), (0.17, 0.22), frame, oak_wall, oak_top, true, target);
        }
        self.emit_part(cell, floor_elev, (0.04, 0.10), (0.03, 0.09), (0.17, 0.46), frame, cast_iron, cast_iron, false, target);
        self.emit_part(cell, floor_elev, (0.04, 0.10), (0.41, 0.47), (0.17, 0.46), frame, cast_iron, cast_iron, false, target);
        self.emit_part(cell, floor_elev, (0.06, 0.11), (0.01, 0.49), (0.25, 0.33), frame, oak_wall, oak_top, true, target);
        self.emit_part(cell, floor_elev, (0.05, 0.10), (0.01, 0.49), (0.36, 0.45), frame, oak_wall, oak_top, true, target);
    }

    fn emit_workshop_stool(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (walnut, brass_rung, pad_wall, pad_top) = (
            Vec3::new(0.42, 0.27, 0.15),
            Vec3::new(0.78, 0.62, 0.24),
            Vec3::new(0.22, 0.44, 0.52),
            Vec3::new(0.28, 0.56, 0.66),
        );
        for (d0, s0) in [(0.10, 0.10), (0.10, 0.34), (0.34, 0.10), (0.34, 0.34)] {
            self.emit_part(cell, floor_elev, (d0, d0 + 0.06), (s0, s0 + 0.06), (0.00, 0.21), frame, walnut, walnut, true, target);
        }
        self.emit_part(cell, floor_elev, (0.12, 0.38), (0.11, 0.39), (0.08, 0.11), frame, brass_rung, brass_rung, true, target);
        self.emit_part(cell, floor_elev, (0.08, 0.42), (0.08, 0.42), (0.21, 0.28), frame, pad_wall, pad_top, true, target);
        self.emit_part(cell, floor_elev, (0.07, 0.15), (0.12, 0.38), (0.28, 0.35), frame, pad_wall, pad_top, false, target);
    }

    fn emit_regal_throne(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (obsidian, velvet_wall, velvet_top, gold) = (
            Vec3::new(0.20, 0.18, 0.24),
            Vec3::new(0.46, 0.12, 0.38),
            Vec3::new(0.62, 0.18, 0.52),
            Vec3::new(0.94, 0.76, 0.24),
        );
        self.emit_part(cell, floor_elev, (0.01, 0.49), (0.01, 0.49), (0.00, 0.10), frame, obsidian, obsidian, true, target);
        self.emit_part(cell, floor_elev, (0.05, 0.46), (0.08, 0.42), (0.10, 0.22), frame, velvet_wall, velvet_top, false, target);
        self.emit_part(cell, floor_elev, (0.02, 0.47), (0.01, 0.10), (0.10, 0.35), frame, obsidian, gold, false, target);
        self.emit_part(cell, floor_elev, (0.02, 0.47), (0.40, 0.49), (0.10, 0.35), frame, obsidian, gold, false, target);
        self.emit_part(cell, floor_elev, (0.02, 0.13), (0.04, 0.46), (0.22, 0.47), frame, velvet_wall, velvet_top, false, target);
        self.emit_part(cell, floor_elev, (0.01, 0.14), (0.02, 0.48), (0.47, 0.50), frame, gold, gold, false, target);
    }
}
