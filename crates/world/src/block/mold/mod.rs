mod archetype;

use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::corner::CornerHeading;
use super::cuboid::CuboidShape;
use super::diagonal_slope::DiagonalSlope;
use super::gentle_ridge::GentleRidge;
use super::gentle_valley::GentleValley;
use super::gentle_wedge::GentleWedge;
use super::pyramid::PyramidPeak;
use super::ridge::RidgeCorner;
use super::shape::VoxelShape;
use super::stair::StairFlight;
use super::stair_corner::StairCorner;
use super::timber_grain::{BlockMedium, MediumShape};
use super::valley::ValleyCorner;
use super::wedge::{RampWedge, WedgeHeading};

pub use self::archetype::{CardinalTurn, MoldArchetype};

/// Immutable palm-molded block specification pairing form archetype, 4-way directional turn, and material medium.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockMold {
    archetype: MoldArchetype,
    turn: CardinalTurn,
    medium: BlockMedium,
}

impl BlockMold {
    /// Primary constructor assigning archetype, turn, and medium.
    pub const fn with_medium(archetype: MoldArchetype, turn: CardinalTurn, medium: BlockMedium) -> Self {
        Self { archetype, turn, medium }
    }

    /// Secondary constructor initializing a turf-earth mold with archetype and turn.
    pub const fn new(archetype: MoldArchetype, turn: CardinalTurn) -> Self {
        Self::with_medium(archetype, turn, BlockMedium::TurfEarth)
    }

    /// Secondary constructor initializing a forward-facing full cube mold.
    pub const fn cube() -> Self { Self::new(MoldArchetype::FullCube, CardinalTurn::Forward) }

    pub const fn in_medium(self, medium: BlockMedium) -> Self {
        let archetype = if matches!(medium, BlockMedium::CabinSeat) && !self.archetype.flat_square_top() {
            MoldArchetype::FullCube
        } else if matches!(medium, BlockMedium::FieldCamera) {
            MoldArchetype::from_code(self.archetype.code() % 8)
        } else {
            self.archetype
        };
        Self::with_medium(archetype, self.turn, medium)
    }

    pub const fn next_form(self) -> Self {
        let next = if matches!(self.medium, BlockMedium::CabinSeat) {
            self.archetype.next_flat_square_top()
        } else if matches!(self.medium, BlockMedium::FieldCamera) {
            self.archetype.next_camera_turn()
        } else {
            self.archetype.next()
        };
        Self::with_medium(next, self.turn, self.medium)
    }

    pub const fn prev_form(self) -> Self {
        let prev = if matches!(self.medium, BlockMedium::CabinSeat) {
            self.archetype.next_flat_square_top()
        } else if matches!(self.medium, BlockMedium::FieldCamera) {
            self.archetype.prev_camera_turn()
        } else {
            self.archetype.prev()
        };
        Self::with_medium(prev, self.turn, self.medium)
    }

    pub const fn turn_clockwise(self) -> Self { Self::with_medium(self.archetype, self.turn.turn_clockwise(), self.medium) }
    pub const fn turn_counter_clockwise(self) -> Self { Self::with_medium(self.archetype, self.turn.turn_counter_clockwise(), self.medium) }
    pub const fn full_height_flat(self) -> bool { self.archetype.full_height_flat() }
    pub const fn placed_camera(self) -> bool { matches!(self.medium, BlockMedium::FieldCamera) }
    pub const fn camera_turn(self) -> u8 { self.archetype.code() % 8 }

    pub fn palm_camera_yaw(self, side: f32) -> f32 {
        side * match self.camera_turn() {
            0 => 0.15,
            1 | 7 => 0.80,
            2 | 6 => 1.45,
            3 | 5 => 2.10,
            _ => 2.75,
        }
    }

    pub const fn quarter_turns(self) -> u8 {
        let turn_step = match self.turn {
            CardinalTurn::Forward => 0,
            CardinalTurn::Right => 1,
            CardinalTurn::Backward => 2,
            CardinalTurn::Left => 3,
        };
        self.archetype.code().wrapping_add(turn_step) % 4
    }

    fn resolve_orientation(self, direction: Vec3, frame: &SurfaceFrame) -> (MoldArchetype, WedgeHeading, CornerHeading) {
        let effective = if matches!(self.medium, BlockMedium::CabinSeat) && !self.archetype.flat_square_top() {
            MoldArchetype::FullCube
        } else {
            self.archetype
        };
        let (du, dv) = frame.decompose(self.turn.orient_vector(direction, frame.normal()));
        let wedge = if dv.abs() >= du.abs() {
            if dv >= 0.0 { WedgeHeading::North } else { WedgeHeading::South }
        } else if du >= 0.0 { WedgeHeading::East } else { WedgeHeading::West };
        let corner = match wedge {
            WedgeHeading::North => CornerHeading::NorthEast,
            WedgeHeading::East => CornerHeading::SouthEast,
            WedgeHeading::South => CornerHeading::SouthWest,
            WedgeHeading::West => CornerHeading::NorthWest,
        };
        (effective, wedge, corner)
    }

    /// Resolves the concrete VoxelShape oriented along the placement direction and current turn.
    pub fn shape_for(self, direction: Vec3, frame: &SurfaceFrame) -> Box<dyn VoxelShape> {
        let (arch, wedge, corner) = self.resolve_orientation(direction, frame);
        let wrap = |s: Box<dyn VoxelShape>| -> Box<dyn VoxelShape> {
            Box::new(MediumShape::with_archetype(s, self.medium, arch, wedge))
        };
        match arch {
            MoldArchetype::FullCube => Box::new(MediumShape::with_archetype(CuboidShape::full_cube(), self.medium, arch, wedge)),
            MoldArchetype::HalfSlab => Box::new(MediumShape::with_archetype(CuboidShape::half_slab(), self.medium, arch, wedge)),
            MoldArchetype::RampWedge => Box::new(MediumShape::with_archetype(RampWedge::new(wedge), self.medium, arch, wedge)),
            MoldArchetype::SteppedStair => Box::new(MediumShape::with_archetype(StairFlight::new(wedge), self.medium, arch, wedge)),
            MoldArchetype::PyramidApex => Box::new(MediumShape::with_archetype(PyramidPeak::slope_aligned(), self.medium, arch, wedge)),
            MoldArchetype::ConvexRidge => Box::new(MediumShape::with_archetype(RidgeCorner::new(corner), self.medium, arch, wedge)),
            MoldArchetype::ConcaveValley => Box::new(MediumShape::with_archetype(ValleyCorner::new(corner), self.medium, arch, wedge)),
            MoldArchetype::DiagonalRamp => Box::new(MediumShape::with_archetype(DiagonalSlope::new(corner), self.medium, arch, wedge)),
            MoldArchetype::GentleLowerWedge => wrap(Box::new(GentleWedge::lower(wedge))),
            MoldArchetype::GentleUpperWedge => wrap(Box::new(GentleWedge::upper(wedge))),
            MoldArchetype::GentleLowerRidge => wrap(Box::new(GentleRidge::lower(corner))),
            MoldArchetype::GentleUpperRidge => wrap(Box::new(GentleRidge::upper(corner))),
            MoldArchetype::GentleLowerValley => wrap(Box::new(GentleValley::lower(corner))),
            MoldArchetype::GentleUpperValley => wrap(Box::new(GentleValley::upper(corner))),
            MoldArchetype::ConvexStairCorner => wrap(Box::new(StairCorner::convex(corner))),
            MoldArchetype::ConcaveStairCorner => wrap(Box::new(StairCorner::concave(corner))),
        }
    }

    /// Emits a miniature 3D preview of this molded block oriented inside a hand basis without heap allocations.
    #[allow(clippy::too_many_arguments)]
    pub fn emit_miniature(
        self, origin: Vec3, right: Vec3, up: Vec3, forward: Vec3,
        scale: f32, side_color: Vec3, _top_color: Vec3, target: &mut dyn MeshTarget,
    ) {
        struct MiniatureBasis { origin: Vec3, right: Vec3, up: Vec3, back: Vec3 }
        struct MiniatureTarget<'a> { basis: MiniatureBasis, scale: f32, out: &'a mut dyn MeshTarget }
        impl MeshTarget for MiniatureTarget<'_> {
            fn push_quad(&mut self, quad: Quad) {
                let (b, s) = (&self.basis, self.scale);
                let map_pt = |p: Vec3| b.origin + b.right * ((p.x - 0.25) * s) + b.up * (p.y * s) + b.back * ((p.z - 0.25) * s);
                let map_n = |n: Vec3| (b.right * n.x + b.up * n.y + b.back * n.z).normalize_or_zero();
                let glow = quad.radiate(|g| g);
                let mapped = quad.transfer(|v0, v1, v2, v3, n, col| {
                    Quad::glowing(map_pt(v0), map_pt(v1), map_pt(v2), map_pt(v3), map_n(n), col, glow)
                });
                self.out.push_quad(mapped);
            }
        }
        let unit_frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let (arch, wedge, corner) = self.resolve_orientation(Vec3::NEG_Z, &unit_frame);
        let mut mini = MiniatureTarget { basis: MiniatureBasis { origin, right, up, back: -forward }, scale, out: target };
        let mut emit = |raw: &dyn VoxelShape| {
            struct RefShape<'a>(&'a dyn VoxelShape);
            impl VoxelShape for RefShape<'_> {
                fn elevation_at(&self, u: f32, v: f32) -> f32 { self.0.elevation_at(u, v) }
                fn shape_color(&self) -> Vec3 { self.0.shape_color() }
                fn emit_shape(&self, c: GridCell, b: f32, f: &SurfaceFrame, col: Vec3, t: &mut dyn MeshTarget) {
                    self.0.emit_shape(c, b, f, col, t);
                }
            }
            MediumShape::with_archetype(RefShape(raw), self.medium, arch, wedge)
                .emit_shape(GridCell::new(0, 0), 0.0, &unit_frame, side_color, &mut mini);
        };
        match arch {
            MoldArchetype::FullCube => emit(&CuboidShape::full_cube()),
            MoldArchetype::HalfSlab => emit(&CuboidShape::half_slab()),
            MoldArchetype::RampWedge => emit(&RampWedge::new(wedge)),
            MoldArchetype::SteppedStair => emit(&StairFlight::new(wedge)),
            MoldArchetype::PyramidApex => emit(&PyramidPeak::slope_aligned()),
            MoldArchetype::ConvexRidge => emit(&RidgeCorner::new(corner)),
            MoldArchetype::ConcaveValley => emit(&ValleyCorner::new(corner)),
            MoldArchetype::DiagonalRamp => emit(&DiagonalSlope::new(corner)),
            MoldArchetype::GentleLowerWedge => emit(&GentleWedge::lower(wedge)),
            MoldArchetype::GentleUpperWedge => emit(&GentleWedge::upper(wedge)),
            MoldArchetype::GentleLowerRidge => emit(&GentleRidge::lower(corner)),
            MoldArchetype::GentleUpperRidge => emit(&GentleRidge::upper(corner)),
            MoldArchetype::GentleLowerValley => emit(&GentleValley::lower(corner)),
            MoldArchetype::GentleUpperValley => emit(&GentleValley::upper(corner)),
            MoldArchetype::ConvexStairCorner => emit(&StairCorner::convex(corner)),
            MoldArchetype::ConcaveStairCorner => emit(&StairCorner::concave(corner)),
        }
    }
}
