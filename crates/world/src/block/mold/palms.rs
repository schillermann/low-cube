use super::BlockMold;

/// Immutable dual-palm block and field-camera orientation state for the right and left hands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PalmMolds {
    right: BlockMold,
    left: BlockMold,
}

impl PalmMolds {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(right: BlockMold, left: BlockMold) -> Self {
        Self { right, left }
    }

    /// Secondary constructor initializing both palms with forward-facing full cubes.
    pub const fn cubes() -> Self {
        Self::new(BlockMold::cube(), BlockMold::cube())
    }

    pub const fn on_side(self, side: f32) -> BlockMold {
        if side > 0.0 {
            self.right
        } else {
            self.left
        }
    }

    pub const fn with_side(self, side: f32, mold: BlockMold) -> Self {
        if side > 0.0 {
            Self::new(mold, self.left)
        } else {
            Self::new(self.right, mold)
        }
    }

    pub const fn synced_from_opposite(self, side: f32) -> Self {
        if side > 0.0 {
            Self::new(self.left, self.left)
        } else {
            Self::new(self.right, self.right)
        }
    }

    pub const fn wire_bytes(self) -> [u8; 6] {
        let [r0, r1, r2] = self.right.chronicle_bytes();
        let [l0, l1, l2] = self.left.chronicle_bytes();
        [r0, r1, r2, l0, l1, l2]
    }

    pub const fn from_wire_bytes(bytes: [u8; 6]) -> Self {
        Self::new(
            BlockMold::from_chronicle_bytes([bytes[0], bytes[1], bytes[2]]),
            BlockMold::from_chronicle_bytes([bytes[3], bytes[4], bytes[5]]),
        )
    }
}
