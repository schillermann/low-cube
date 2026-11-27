use crate::block::{BlockMedium, BlockMold, CardinalTurn};
use super::coord::{AnchorOffset, CellCoord};
use super::module::{MachineKind, ScalableKind, SpaceModule, MAX_MODULE_AXIS_CELLS};

/// 3D rectangular `0.5 m` cell footprint `(width_u, depth_v, height_w)` of a multi-block or scalable module (`3` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModuleSpan {
    width_u: u8,
    depth_v: u8,
    height_w: u8,
}

impl ModuleSpan {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(width_u: u8, depth_v: u8, height_w: u8) -> Self {
        Self { width_u, depth_v, height_w }
    }

    const fn clamp_axis(x: u8) -> u8 {
        if x < 1 {
            1
        } else if x > MAX_MODULE_AXIS_CELLS {
            MAX_MODULE_AXIS_CELLS
        } else {
            x
        }
    }

    pub const fn clamped(width_u: u8, depth_v: u8, height_w: u8) -> Self {
        Self::new(
            Self::clamp_axis(width_u),
            Self::clamp_axis(depth_v),
            Self::clamp_axis(height_w),
        )
    }

    /// Computes dragged `(anchor, ModuleSpan)` between two corner cells `(a, b)` plus extra vertical steps `extra_h`.
    pub fn dragged_between(a: CellCoord, b: CellCoord, extra_h: i32) -> (CellCoord, Self) {
        let ((au, av, aw), (bu, bv, bw)) = (a.components(), b.components());
        let (u0, u1) = (au.min(bu), au.max(bu));
        let (v0, v1) = (av.min(bv), av.max(bv));
        let w0 = aw.min(bw);
        let raw_h = (aw.max(bw) - w0 + 1 + extra_h).clamp(1, i32::from(MAX_MODULE_AXIS_CELLS)) as u8;
        let du = (u1 - u0 + 1).clamp(1, i32::from(MAX_MODULE_AXIS_CELLS)) as u8;
        let dv = (v1 - v0 + 1).clamp(1, i32::from(MAX_MODULE_AXIS_CELLS)) as u8;
        (CellCoord::new(u0, v0, w0), Self::clamped(du, dv, raw_h))
    }

    pub const fn dimensions(self) -> (u8, u8, u8) {
        (self.width_u, self.depth_v, self.height_w)
    }

    pub const fn cell_volume(self) -> usize {
        (self.width_u as usize) * (self.depth_v as usize) * (self.height_w as usize)
    }

    pub fn visit_offsets(self, mut visitor: impl FnMut(AnchorOffset)) {
        for dw in 0..self.height_w {
            for dv in 0..self.depth_v {
                for du in 0..self.width_u {
                    visitor(AnchorOffset::new(du, dv, dw));
                }
            }
        }
    }
}

/// Complete specification stored in a `BrickCell::ModuleAnchor` cell (`3` fields, `Copy`, `0` heap pointers).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModuleSpec {
    module: SpaceModule,
    span: ModuleSpan,
    turn: CardinalTurn,
}

impl ModuleSpec {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(module: SpaceModule, span: ModuleSpan, turn: CardinalTurn) -> Self {
        Self { module, span, turn }
    }

    pub const fn machine(kind: MachineKind, turn: CardinalTurn) -> Self {
        Self::new(SpaceModule::Machine(kind), kind.footprint(turn), turn)
    }

    pub const fn scalable(kind: ScalableKind, span: ModuleSpan, turn: CardinalTurn) -> Self {
        let (b, t, h) = span.dimensions();
        Self::new(SpaceModule::Scalable(kind), ModuleSpan::clamped(b, t, h), turn)
    }

    pub const fn from_medium_turn(medium: BlockMedium, turn: CardinalTurn) -> Option<Self> {
        match SpaceModule::from_medium(medium) {
            Some(SpaceModule::Machine(k)) => Some(Self::machine(k, turn)),
            Some(SpaceModule::Scalable(k)) => Some(Self::scalable(k, k.initial_span(turn), turn)),
            None => None,
        }
    }

    pub const fn from_mold(mold: BlockMold) -> Option<Self> {
        let bytes = mold.chronicle_bytes();
        let turn = CardinalTurn::from_code(bytes[1]);
        match SpaceModule::from_medium(mold.medium()) {
            Some(SpaceModule::Machine(k)) => Some(Self::machine(k, turn)),
            Some(SpaceModule::Scalable(k)) => {
                let (b, t, h) = match bytes[0] & 0x0F {
                    0 => match k {
                        ScalableKind::PropulsionBell => (2, 2, 2),
                        ScalableKind::PropellantTank => (2, 2, 3),
                        ScalableKind::ThermalFin => (3, 1, 2),
                    },
                    1 => (1, 1, 2),
                    2 => (2, 2, 3),
                    3 => (2, 2, 4),
                    4 => (3, 3, 3),
                    5 => (3, 3, 4),
                    6 => (4, 4, 4),
                    7 => (4, 2, 3),
                    8 => (6, 1, 3),
                    9 => (4, 4, 6),
                    _ => (2, 2, 2),
                };
                let span = if (turn.code() & 1) == 1 {
                    ModuleSpan::clamped(t, b, h)
                } else {
                    ModuleSpan::clamped(b, t, h)
                };
                Some(Self::scalable(k, span, turn))
            }
            None => None,
        }
    }

    pub const fn rotated_quarter(self) -> Self {
        let next_turn = CardinalTurn::from_code(self.turn.code().wrapping_add(1));
        let (b, t, h) = self.span.dimensions();
        Self::new(self.module, ModuleSpan::clamped(t, b, h), next_turn)
    }

    pub const fn module(self) -> SpaceModule { self.module }
    pub const fn span(self) -> ModuleSpan { self.span }
    pub const fn turn(self) -> CardinalTurn { self.turn }
    pub const fn medium(self) -> BlockMedium { self.module.medium() }
    pub const fn cell_mass_kg(self) -> f64 { self.module.cell_mass_kg() }

    pub const fn wire_bytes(self) -> [u8; 5] {
        let (b, t, h) = self.span.dimensions();
        [self.module.code(), b, t, h, self.turn.code()]
    }

    pub const fn from_wire_bytes(bytes: [u8; 5]) -> Option<Self> {
        let Some(module) = SpaceModule::from_code(bytes[0]) else {
            return None;
        };
        let turn = CardinalTurn::from_code(bytes[4]);
        let span = match module {
            SpaceModule::Machine(k) => k.footprint(turn),
            SpaceModule::Scalable(_) => ModuleSpan::clamped(bytes[1], bytes[2], bytes[3]),
        };
        Some(Self::new(module, span, turn))
    }
}
