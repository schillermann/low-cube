use super::atom::RawAtom;
use super::coord::{AnchorOffset, LocalSlot};
use super::module_spec::ModuleSpec;

/// Unboxed `0.5 m` cell payload inside a `Brick8` (`0` heap pointers, `Copy`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BrickCell {
    Empty,
    RawAtom(RawAtom),
    ModuleAnchor(ModuleSpec),
    ModuleProxy(AnchorOffset),
}

impl BrickCell {
    pub const fn occupied(self) -> bool {
        !matches!(self, Self::Empty)
    }

    pub const fn granular(self) -> bool {
        match self {
            Self::RawAtom(atom) => atom.settles_under_gravity(),
            _ => false,
        }
    }

    pub const fn mass_kg(self) -> f64 {
        match self {
            Self::Empty => 0.0,
            Self::RawAtom(atom) => atom.mass_kg(),
            Self::ModuleAnchor(spec) => spec.cell_mass_kg(),
            Self::ModuleProxy(_) => 0.0,
        }
    }
}

/// Flat `8 × 8 × 8` (`512` cells at `0.5 m` = `4.0 m` cube) voxel brick with `8 × u64` occupancy & granular bitmasks (`3` fields).
#[derive(Clone)]
pub struct Brick8 {
    cells: [BrickCell; 512],
    occupancy: [u64; 8],
    granular: [u64; 8],
}

impl Brick8 {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cells: [BrickCell; 512], occupancy: [u64; 8], granular: [u64; 8]) -> Self {
        Self { cells, occupancy, granular }
    }

    /// Initializes a vacant `8×8×8` brick with cleared `u64` bitmasks.
    pub const fn empty() -> Self {
        Self::new([BrickCell::Empty; 512], [0; 8], [0; 8])
    }

    pub const fn vacant(&self) -> bool {
        (self.occupancy[0]
            | self.occupancy[1]
            | self.occupancy[2]
            | self.occupancy[3]
            | self.occupancy[4]
            | self.occupancy[5]
            | self.occupancy[6]
            | self.occupancy[7])
            == 0
    }

    pub fn occupied_count(&self) -> u32 {
        self.occupancy.iter().map(|m| m.count_ones()).sum()
    }

    pub const fn occupancy_masks(&self) -> [u64; 8] {
        self.occupancy
    }

    pub const fn granular_masks(&self) -> [u64; 8] {
        self.granular
    }

    pub const fn plane_mask(&self, lw: usize) -> u64 {
        self.occupancy[lw & 7]
    }

    pub const fn granular_plane_mask(&self, lw: usize) -> u64 {
        self.granular[lw & 7]
    }

    /// 8-bit vertical occupancy mask across `lw = 0..8` for planar slot `(lu, lv)`.
    pub const fn column_mask(&self, lu: u8, lv: u8) -> u8 {
        let bit = 1u64 << (((lv & 7) << 3) | (lu & 7));
        let mut out = 0u8;
        let mut lw = 0usize;
        while lw < 8 {
            if (self.occupancy[lw] & bit) != 0 {
                out |= 1u8 << lw;
            }
            lw += 1;
        }
        out
    }

    pub const fn cell_at(&self, slot: LocalSlot) -> BrickCell {
        self.cells[slot.index()]
    }

    pub const fn occupied_at(&self, slot: LocalSlot) -> bool {
        (self.occupancy[slot.plane_row()] & slot.plane_bit()) != 0
    }

    /// Writes `cell` into `slot`, updating `occupancy` and `granular` `u64` masks in `O(1)` and returning the displaced `BrickCell`.
    pub fn write_slot(&mut self, slot: LocalSlot, cell: BrickCell) -> BrickCell {
        let idx = slot.index();
        let prior = self.cells[idx];
        self.cells[idx] = cell;
        let row = slot.plane_row();
        let bit = slot.plane_bit();
        if cell.occupied() {
            self.occupancy[row] |= bit;
        } else {
            self.occupancy[row] &= !bit;
        }
        if cell.granular() {
            self.granular[row] |= bit;
        } else {
            self.granular[row] &= !bit;
        }
        prior
    }

    pub fn clear_slot(&mut self, slot: LocalSlot) -> BrickCell {
        self.write_slot(slot, BrickCell::Empty)
    }

    /// Iterates only occupied `LocalSlot`s using `u64::trailing_zeros` hardware bit-scanning.
    pub fn visit_occupied(&self, mut visitor: impl FnMut(LocalSlot, BrickCell)) {
        for lw in 0..8usize {
            let mut mask = self.occupancy[lw];
            while mask != 0 {
                let tz = mask.trailing_zeros() as u16;
                let slot = LocalSlot::new(((lw as u16) << 6) | tz);
                visitor(slot, self.cells[slot.index()]);
                mask &= mask - 1;
            }
        }
    }
}
