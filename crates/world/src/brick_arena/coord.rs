use glam::DVec3;
use crate::block::GridCell;

/// Discrete `0.5 m` 3D voxel coordinate `(u, v, w)` inside a local grid body or planetary face (`3` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellCoord {
    u: i32,
    v: i32,
    w: i32,
}

impl CellCoord {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(u: i32, v: i32, w: i32) -> Self {
        Self { u, v, w }
    }

    /// Quantizes a planar `(u, v)` cell and metric floor elevation above `datum` into a `0.5 m` `CellCoord`.
    pub fn from_floor(u: i32, v: i32, floor_elev: f32, datum: f32) -> Self {
        let w = ((floor_elev - datum) / GridCell::SPAN).round() as i32;
        Self::new(u, v, w)
    }

    /// Quantizes an arbitrary interior sample elevation (`layer_elev`) above `datum` into its containing `0.5 m` `CellCoord`.
    pub fn from_sample(u: i32, v: i32, layer_elev: f32, datum: f32) -> Self {
        let w = (((layer_elev - datum) + 0.05) / GridCell::SPAN).floor() as i32;
        Self::new(u, v, w)
    }

    pub const fn components(self) -> (i32, i32, i32) {
        (self.u, self.v, self.w)
    }

    pub const fn planar(self) -> (i32, i32) {
        (self.u, self.v)
    }

    pub const fn layer_w(self) -> i32 {
        self.w
    }

    pub const fn shifted(self, du: i32, dv: i32, dw: i32) -> Self {
        Self::new(self.u + du, self.v + dv, self.w + dw)
    }

    pub const fn grid_cell(self) -> GridCell {
        GridCell::new(self.u, self.v)
    }

    pub fn floor_elev(self, datum: f32) -> f32 {
        (self.w as f32) * GridCell::SPAN + datum
    }

    pub fn ceiling_elev(self, datum: f32) -> f32 {
        ((self.w + 1) as f32) * GridCell::SPAN + datum
    }

    /// Exact double-precision `0.5 m` cell centroid in local metric space `(x, y_up, z)`.
    pub fn metric_center(self, datum: f64) -> DVec3 {
        let step = f64::from(GridCell::SPAN);
        DVec3::new(
            (f64::from(self.u) + 0.5) * step,
            f64::from(self.w) * step + datum + 0.5 * step,
            (f64::from(self.v) + 0.5) * step,
        )
    }

    /// Splits this `0.5 m` cell into its `4.0 m` `BrickCoord` (`8×8×8`) and `0..512` `LocalSlot`.
    pub const fn brick_split(self) -> (BrickCoord, LocalSlot) {
        let brick = BrickCoord::new(
            self.u.div_euclid(8),
            self.v.div_euclid(8),
            self.w.div_euclid(8),
        );
        let slot = LocalSlot::from_axes(
            self.u.rem_euclid(8) as u8,
            self.v.rem_euclid(8) as u8,
            self.w.rem_euclid(8) as u8,
        );
        (brick, slot)
    }
}

/// Discrete `4.0 m × 4.0 m × 4.0 m` brick coordinate addressing one `Brick8` (`8×8×8` cells, `3` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BrickCoord {
    bx: i32,
    by: i32,
    bz: i32,
}

impl BrickCoord {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bx: i32, by: i32, bz: i32) -> Self {
        Self { bx, by, bz }
    }

    pub const fn planar_sector(self) -> (i32, i32) {
        (self.bx, self.by)
    }

    pub const fn origin_cell(self) -> CellCoord {
        CellCoord::new(self.bx * 8, self.by * 8, self.bz * 8)
    }

    pub const fn cell_at(self, slot: LocalSlot) -> CellCoord {
        let (lu, lv, lw) = slot.axes();
        CellCoord::new(
            self.bx * 8 + (lu as i32),
            self.by * 8 + (lv as i32),
            self.bz * 8 + (lw as i32),
        )
    }
}

/// Encapsulated linear cell index `0..512` inside an `8×8×8` `Brick8` (`1` field).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocalSlot {
    linear: u16,
}

impl LocalSlot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(linear: u16) -> Self {
        Self { linear: linear & 511 }
    }

    pub const fn from_axes(lu: u8, lv: u8, lw: u8) -> Self {
        let idx = (((lw & 7) as u16) << 6) | (((lv & 7) as u16) << 3) | ((lu & 7) as u16);
        Self::new(idx)
    }

    pub const fn index(self) -> usize {
        (self.linear & 511) as usize
    }

    pub const fn axes(self) -> (u8, u8, u8) {
        let idx = self.linear & 511;
        ((idx & 7) as u8, ((idx >> 3) & 7) as u8, ((idx >> 6) & 7) as u8)
    }

    pub const fn plane_row(self) -> usize {
        ((self.linear >> 6) & 7) as usize
    }

    pub const fn plane_bit(self) -> u64 {
        1u64 << (self.linear & 63)
    }
}

/// Relative `0.5 m` cell offset from a `ModuleProxy` cell back to its owning `ModuleAnchor` (`3` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnchorOffset {
    du: u8,
    dv: u8,
    dw: u8,
}

impl AnchorOffset {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(du: u8, dv: u8, dw: u8) -> Self {
        Self { du, dv, dw }
    }

    pub const fn anchor_from(self, proxy: CellCoord) -> CellCoord {
        proxy.shifted(-(self.du as i32), -(self.dv as i32), -(self.dw as i32))
    }

    pub const fn deltas(self) -> (u8, u8, u8) {
        (self.du, self.dv, self.dw)
    }
}
