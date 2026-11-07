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

/// Four-way 90-degree horizontal rotation relative to the placement sight direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardinalTurn { Forward, Right, Backward, Left }

impl CardinalTurn {
    pub const fn turn_clockwise(self) -> Self {
        match self { Self::Forward => Self::Right, Self::Right => Self::Backward, Self::Backward => Self::Left, Self::Left => Self::Forward }
    }

    pub const fn turn_counter_clockwise(self) -> Self {
        match self { Self::Forward => Self::Left, Self::Right => Self::Forward, Self::Backward => Self::Right, Self::Left => Self::Backward }
    }

    /// Rotates a 3D look direction around the surface normal in 90-degree increments.
    pub fn orient_vector(self, direction: Vec3, surface_up: Vec3) -> Vec3 {
        let flat = direction - surface_up * direction.dot(surface_up);
        let fwd = if flat.length_squared() > 1e-6 { flat.normalize() } else { Vec3::NEG_Z };
        let right = fwd.cross(surface_up).normalize_or_zero();
        match self {
            Self::Forward => fwd,
            Self::Right => right,
            Self::Backward => -fwd,
            Self::Left => -right,
        }
    }
}

/// Complete catalog of all 16 sculptable voxel form archetypes available for palm molding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoldArchetype {
    FullCube, HalfSlab, RampWedge, SteppedStair, PyramidApex, ConvexRidge, ConcaveValley, DiagonalRamp,
    GentleLowerWedge, GentleUpperWedge, GentleLowerRidge, GentleUpperRidge, GentleLowerValley, GentleUpperValley,
    ConvexStairCorner, ConcaveStairCorner,
}

impl MoldArchetype {
    pub const fn from_code(code: u8) -> Self {
        match code % 16 {
            0 => Self::FullCube, 1 => Self::HalfSlab, 2 => Self::RampWedge, 3 => Self::SteppedStair,
            4 => Self::PyramidApex, 5 => Self::ConvexRidge, 6 => Self::ConcaveValley, 7 => Self::DiagonalRamp,
            8 => Self::GentleLowerWedge, 9 => Self::GentleUpperWedge, 10 => Self::GentleLowerRidge, 11 => Self::GentleUpperRidge,
            12 => Self::GentleLowerValley, 13 => Self::GentleUpperValley, 14 => Self::ConvexStairCorner, _ => Self::ConcaveStairCorner,
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Self::FullCube => 0, Self::HalfSlab => 1, Self::RampWedge => 2, Self::SteppedStair => 3,
            Self::PyramidApex => 4, Self::ConvexRidge => 5, Self::ConcaveValley => 6, Self::DiagonalRamp => 7,
            Self::GentleLowerWedge => 8, Self::GentleUpperWedge => 9, Self::GentleLowerRidge => 10, Self::GentleUpperRidge => 11,
            Self::GentleLowerValley => 12, Self::GentleUpperValley => 13, Self::ConvexStairCorner => 14, Self::ConcaveStairCorner => 15,
        }
    }

    pub const fn next(self) -> Self { Self::from_code(self.code() + 1) }
    pub const fn prev(self) -> Self { Self::from_code(self.code() + 15) }
    pub const fn full_height_flat(self) -> bool { matches!(self, Self::FullCube) }
    pub const fn flat_square_top(self) -> bool { matches!(self, Self::FullCube | Self::HalfSlab) }

    pub const fn next_flat_square_top(self) -> Self {
        match self {
            Self::FullCube => Self::HalfSlab,
            _ => Self::FullCube,
        }
    }

    pub const fn next_camera_turn(self) -> Self {
        Self::from_code((self.code() + 1) % 6)
    }

    pub const fn prev_camera_turn(self) -> Self {
        Self::from_code((self.code() + 5) % 6)
    }
}

/// Immutable palm-molded block specification pairing form archetype, 4-way directional turn, and material medium.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
            MoldArchetype::from_code(self.archetype.code() % 6)
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
    pub const fn camera_turn(self) -> u8 { self.archetype.code() % 6 }

    pub const fn quarter_turns(self) -> u8 {
        let turn_step = match self.turn {
            CardinalTurn::Forward => 0,
            CardinalTurn::Right => 1,
            CardinalTurn::Backward => 2,
            CardinalTurn::Left => 3,
        };
        self.archetype.code().wrapping_add(turn_step) % 4
    }

    /// Resolves the concrete VoxelShape oriented along the placement direction and current turn.
    pub fn shape_for(self, direction: Vec3, frame: &SurfaceFrame) -> Box<dyn VoxelShape> {
        let effective_archetype = if matches!(self.medium, BlockMedium::CabinSeat) && !self.archetype.flat_square_top() {
            MoldArchetype::FullCube
        } else {
            self.archetype
        };
        let oriented = self.turn.orient_vector(direction, frame.normal());
        let (du, dv) = frame.decompose(oriented);
        let wedge = if dv.abs() >= du.abs() {
            if dv >= 0.0 { WedgeHeading::North } else { WedgeHeading::South }
        } else if du >= 0.0 { WedgeHeading::East } else { WedgeHeading::West };
        let corner = match wedge {
            WedgeHeading::North => CornerHeading::NorthEast,
            WedgeHeading::East => CornerHeading::SouthEast,
            WedgeHeading::South => CornerHeading::SouthWest,
            WedgeHeading::West => CornerHeading::NorthWest,
        };
        let raw: Box<dyn VoxelShape> = match effective_archetype {
            MoldArchetype::FullCube => Box::new(CuboidShape::full_cube()),
            MoldArchetype::HalfSlab => Box::new(CuboidShape::half_slab()),
            MoldArchetype::RampWedge => Box::new(RampWedge::new(wedge)),
            MoldArchetype::SteppedStair => Box::new(StairFlight::new(wedge)),
            MoldArchetype::PyramidApex => Box::new(PyramidPeak::slope_aligned()),
            MoldArchetype::ConvexRidge => Box::new(RidgeCorner::new(corner)),
            MoldArchetype::ConcaveValley => Box::new(ValleyCorner::new(corner)),
            MoldArchetype::DiagonalRamp => Box::new(DiagonalSlope::new(corner)),
            MoldArchetype::GentleLowerWedge => Box::new(GentleWedge::lower(wedge)),
            MoldArchetype::GentleUpperWedge => Box::new(GentleWedge::upper(wedge)),
            MoldArchetype::GentleLowerRidge => Box::new(GentleRidge::lower(corner)),
            MoldArchetype::GentleUpperRidge => Box::new(GentleRidge::upper(corner)),
            MoldArchetype::GentleLowerValley => Box::new(GentleValley::lower(corner)),
            MoldArchetype::GentleUpperValley => Box::new(GentleValley::upper(corner)),
            MoldArchetype::ConvexStairCorner => Box::new(StairCorner::convex(corner)),
            MoldArchetype::ConcaveStairCorner => Box::new(StairCorner::concave(corner)),
        };
        Box::new(MediumShape::with_archetype(raw, self.medium, effective_archetype, wedge))
    }

    /// Emits a miniature 3D preview of this molded block oriented inside a hand basis.
    #[allow(clippy::too_many_arguments)]
    pub fn emit_miniature(
        self, origin: Vec3, right: Vec3, up: Vec3, forward: Vec3,
        scale: f32, side_color: Vec3, top_color: Vec3, target: &mut dyn MeshTarget,
    ) {
        struct MiniatureBasis {
            origin: Vec3,
            right: Vec3,
            up: Vec3,
            back: Vec3,
        }
        struct MiniatureCoat {
            scale: f32,
            side_col: Vec3,
            top_col: Vec3,
            self_pigmented: bool,
        }
        struct MiniatureTarget<'a> {
            basis: MiniatureBasis,
            coat: MiniatureCoat,
            out: &'a mut dyn MeshTarget,
        }
        impl MeshTarget for MiniatureTarget<'_> {
            fn push_quad(&mut self, quad: Quad) {
                let b = &self.basis;
                let s = self.coat.scale;
                let map_pt = |p: Vec3| b.origin + b.right * ((p.x - 0.25) * s) + b.up * (p.y * s) + b.back * ((p.z - 0.25) * s);
                let map_n = |n: Vec3| (b.right * n.x + b.up * n.y + b.back * n.z).normalize_or_zero();
                let mapped = quad.transfer(|v0, v1, v2, v3, n, col| {
                    let c = if self.coat.self_pigmented { col } else if n.y > 0.25 { self.coat.top_col } else { self.coat.side_col };
                    Quad::new(map_pt(v0), map_pt(v1), map_pt(v2), map_pt(v3), map_n(n), c)
                });
                self.out.push_quad(mapped);
            }
        }
        let unit_frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let shape = self.shape_for(Vec3::NEG_Z, &unit_frame);
        let self_pigmented = self.medium.self_pigmented() || self.full_height_flat();
        let lift = if self_pigmented || matches!(self.archetype, MoldArchetype::ConvexRidge | MoldArchetype::GentleLowerRidge) {
            Vec3::ZERO
        } else {
            let (r, lift) = (0.25 * scale, up * (0.08 * scale));
            let (b0, b1, b2, b3) = (origin - right * r + forward * r, origin + right * r + forward * r, origin + right * r - forward * r, origin - right * r - forward * r);
            target.push_quad(Quad::new(b0, b3, b2, b1, -up, side_color));
            target.push_quad(Quad::new(b0, b1, b1 + lift, b0 + lift, forward, side_color));
            target.push_quad(Quad::new(b1, b2, b2 + lift, b1 + lift, right, side_color));
            target.push_quad(Quad::new(b2, b3, b3 + lift, b2 + lift, -forward, side_color));
            target.push_quad(Quad::new(b3, b0, b0 + lift, b3 + lift, -right, side_color));
            lift
        };
        let mut mini = MiniatureTarget {
            basis: MiniatureBasis { origin: origin + lift, right, up, back: -forward },
            coat: MiniatureCoat { scale, side_col: side_color, top_col: top_color, self_pigmented },
            out: target,
        };
        shape.emit_shape(GridCell::new(0, 0), 0.0, &unit_frame, side_color, &mut mini);
    }
}
