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
use super::valley::ValleyCorner;
use super::wedge::{RampWedge, WedgeHeading};

/// Four-way 90-degree horizontal rotation relative to the placement sight direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardinalTurn {
    Forward,
    Right,
    Backward,
    Left,
}

impl CardinalTurn {
    pub const fn turn_clockwise(self) -> Self {
        match self {
            Self::Forward => Self::Right,
            Self::Right => Self::Backward,
            Self::Backward => Self::Left,
            Self::Left => Self::Forward,
        }
    }

    pub const fn turn_counter_clockwise(self) -> Self {
        match self {
            Self::Forward => Self::Left,
            Self::Right => Self::Forward,
            Self::Backward => Self::Right,
            Self::Left => Self::Backward,
        }
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
}

/// Immutable palm-molded block specification pairing form archetype with 4-way directional turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockMold {
    archetype: MoldArchetype,
    turn: CardinalTurn,
}

impl BlockMold {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(archetype: MoldArchetype, turn: CardinalTurn) -> Self { Self { archetype, turn } }

    /// Secondary constructor initializing a forward-facing full cube mold.
    pub const fn cube() -> Self { Self::new(MoldArchetype::FullCube, CardinalTurn::Forward) }

    pub const fn next_form(self) -> Self { Self::new(self.archetype.next(), self.turn) }
    pub const fn prev_form(self) -> Self { Self::new(self.archetype.prev(), self.turn) }
    pub const fn turn_clockwise(self) -> Self { Self::new(self.archetype, self.turn.turn_clockwise()) }
    pub const fn turn_counter_clockwise(self) -> Self { Self::new(self.archetype, self.turn.turn_counter_clockwise()) }
    pub const fn full_height_flat(self) -> bool { self.archetype.full_height_flat() }

    /// Resolves the concrete VoxelShape oriented along the placement direction and current turn.
    pub fn shape_for(self, direction: Vec3, frame: &SurfaceFrame) -> Box<dyn VoxelShape> {
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
        match self.archetype {
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
        }
    }

    /// Emits a miniature 3D preview of this molded block oriented inside a hand basis.
    #[allow(clippy::too_many_arguments)]
    pub fn emit_miniature(
        self, origin: Vec3, right: Vec3, up: Vec3, forward: Vec3,
        scale: f32, side_color: Vec3, top_color: Vec3, target: &mut dyn MeshTarget,
    ) {
        struct MiniatureTarget<'a> {
            origin: Vec3, right: Vec3, up: Vec3, back: Vec3,
            scale: f32, side_col: Vec3, top_col: Vec3, out: &'a mut dyn MeshTarget,
        }
        impl MeshTarget for MiniatureTarget<'_> {
            fn push_quad(&mut self, quad: Quad) {
                let map_pt = |p: Vec3| self.origin + self.right * ((p.x - 0.25) * self.scale) + self.up * (p.y * self.scale) + self.back * ((p.z - 0.25) * self.scale);
                let map_n = |n: Vec3| (self.right * n.x + self.up * n.y + self.back * n.z).normalize_or_zero();
                let mapped = quad.transfer(|v0, v1, v2, v3, n, _| {
                    let col = if n.y > 0.25 { self.top_col } else { self.side_col };
                    Quad::new(map_pt(v0), map_pt(v1), map_pt(v2), map_pt(v3), map_n(n), col)
                });
                self.out.push_quad(mapped);
            }
        }
        let unit_frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let shape = self.shape_for(Vec3::NEG_Z, &unit_frame);
        let r = 0.25 * scale;
        let b0 = origin + right * (-r) + forward * r;
        let b1 = origin + right * r + forward * r;
        let b2 = origin + right * r - forward * r;
        let b3 = origin + right * (-r) - forward * r;
        let lift = up * (0.08 * scale);
        target.push_quad(Quad::new(b0, b3, b2, b1, -up, side_color));
        target.push_quad(Quad::new(b0, b1, b1 + lift, b0 + lift, forward, side_color));
        target.push_quad(Quad::new(b1, b2, b2 + lift, b1 + lift, right, side_color));
        target.push_quad(Quad::new(b2, b3, b3 + lift, b2 + lift, -forward, side_color));
        target.push_quad(Quad::new(b3, b0, b0 + lift, b3 + lift, -right, side_color));
        let mut mini = MiniatureTarget { origin: origin + lift, right, up, back: -forward, scale, side_col: side_color, top_col: top_color, out: target };
        shape.emit_shape(GridCell::new(0, 0), 0.0, &unit_frame, side_color, &mut mini);
    }
}
