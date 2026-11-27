use world::{Brick8, BrickArena, BrickCell, BrickCoord, LocalSlot, MoldArchetype, AtomKind};

/// Polymorphic neighborhood view around a `Brick8` for seamless cross-brick face culling & corner AO.
pub trait BrickNeighborhood {
    fn neighbor_brick(&self, offset: [i32; 3]) -> Option<&Brick8>;
}

/// Isolated neighborhood treating all 6 adjacent brick slots as empty vacuum (`0` fields).
#[derive(Clone, Copy, Debug)]
pub struct VacantNeighbors;

impl VacantNeighbors {
    /// Primary constructor doing nothing except creating the instance.
    pub const fn new() -> Self {
        Self
    }
}

impl BrickNeighborhood for VacantNeighbors {
    fn neighbor_brick(&self, _offset: [i32; 3]) -> Option<&Brick8> {
        None
    }
}

/// Arena-backed neighborhood resolving adjacent `Brick8`s in `O(1)` (`2` private fields).
pub struct ArenaNeighbors<'a> {
    arena: &'a BrickArena,
    origin: BrickCoord,
}

impl<'a> ArenaNeighbors<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(arena: &'a BrickArena, origin: BrickCoord) -> Self {
        Self { arena, origin }
    }
}

impl BrickNeighborhood for ArenaNeighbors<'_> {
    fn neighbor_brick(&self, offset: [i32; 3]) -> Option<&Brick8> {
        let (bx, by) = self.origin.planar_sector();
        let bz = self.origin.origin_cell().layer_w().div_euclid(8);
        self.arena.brick_at(BrickCoord::new(bx + offset[0], by + offset[2], bz + offset[1]))
    }
}

/// Precomputed `10 × 10` (`-1..=8 × -1..=8`) ahead-plane bitmask evaluating `4 × 2-Bit` corner voxel AO (`1` field).
#[derive(Clone, Copy, Debug)]
pub struct CornerOcclusion {
    rows: [u16; 10],
}

impl CornerOcclusion {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(rows: [u16; 10]) -> Self {
        Self { rows }
    }

    const fn corner_ao(self, s1: u16, s2: u16, diag: u16) -> u8 {
        let _ = self;
        if (s1 & s2) != 0 {
            0
        } else {
            (3 - (s1 + s2 + diag)) as u8
        }
    }

    /// Evaluates packed `4 × 2-Bit` corner AO (`ao00 | ao10<<2 | ao11<<4 | ao01<<6`) at `(su, sv)` in `O(1)` bit ops.
    pub const fn evaluate_at(self, su: usize, sv: usize) -> u8 {
        let cu = su + 1;
        let r_lo = self.rows[sv];
        let r_mid = self.rows[sv + 1];
        let r_hi = self.rows[sv + 2];

        let s_um = (r_mid >> (cu - 1)) & 1;
        let s_up = (r_mid >> (cu + 1)) & 1;
        let s_vm = (r_lo >> cu) & 1;
        let s_vp = (r_hi >> cu) & 1;

        let d00 = (r_lo >> (cu - 1)) & 1;
        let d10 = (r_lo >> (cu + 1)) & 1;
        let d11 = (r_hi >> (cu + 1)) & 1;
        let d01 = (r_hi >> (cu - 1)) & 1;

        let ao00 = self.corner_ao(s_um, s_vm, d00);
        let ao10 = self.corner_ao(s_up, s_vm, d10);
        let ao11 = self.corner_ao(s_up, s_vp, d11);
        let ao01 = self.corner_ao(s_um, s_vp, d01);

        ao00 | (ao10 << 2) | (ao11 << 4) | (ao01 << 6)
    }

    pub fn from_y_plane(ahead: u64, top: bool) -> Self {
        let mut rows = [0u16; 10];
        if ahead != 0 {
            for lv in 0..8usize {
                let r = ((ahead >> (lv * 8)) & 0xFF) as u16;
                if top {
                    for lu in 0..8usize {
                        if ((r >> lu) & 1) != 0 {
                            rows[lu + 1] |= 1u16 << (lv + 1);
                        }
                    }
                } else {
                    rows[lv + 1] = r << 1;
                }
            }
        }
        Self::new(rows)
    }

    pub fn from_x_plane(planes: &[u64; 8], lu: usize, east: bool) -> Self {
        let mut rows = [0u16; 10];
        for lw in 0..8usize {
            let col = (planes[lw] >> lu) & 0x0101_0101_0101_0101;
            if col == 0 {
                continue;
            }
            for lv in 0..8usize {
                if ((col >> (lv * 8)) & 1) != 0 {
                    let (su, sv) = if east { (lw, lv) } else { (lv, lw) };
                    rows[sv + 1] |= 1u16 << (su + 1);
                }
            }
        }
        Self::new(rows)
    }

    pub fn from_z_plane(planes: &[u64; 8], lv: usize, south: bool) -> Self {
        let mut rows = [0u16; 10];
        for lw in 0..8usize {
            let r = ((planes[lw] >> (lv * 8)) & 0xFF) as u16;
            if r == 0 {
                continue;
            }
            if south {
                rows[lw + 1] = r << 1;
            } else {
                for lu in 0..8usize {
                    if ((r >> lu) & 1) != 0 {
                        rows[lu + 1] |= 1u16 << (lw + 1);
                    }
                }
            }
        }
        Self::new(rows)
    }
}

/// Extracts opaque full-cube `u64` horizontal masks (`[u64; 8]`) from a `Brick8` (`1` field).
pub struct SolidCubeMask<'a> {
    brick: &'a Brick8,
}

impl<'a> SolidCubeMask<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(brick: &'a Brick8) -> Self {
        Self { brick }
    }

    pub fn opaque_planes(&self) -> [u64; 8] {
        let mut masks = self.brick.occupancy_masks();
        for (lw, plane) in masks.iter_mut().enumerate() {
            let mut m = *plane;
            while m != 0 {
                let tz = m.trailing_zeros() as u16;
                let slot = LocalSlot::new(((lw as u16) << 6) | tz);
                let full_opaque = matches!(
                    self.brick.cell_at(slot),
                    BrickCell::RawAtom(a)
                        if matches!(a.kind(), AtomKind::Molded { archetype: MoldArchetype::FullCube, .. })
                );
                if !full_opaque {
                    *plane &= !(1u64 << tz);
                }
                m &= m - 1;
            }
        }
        masks
    }
}
