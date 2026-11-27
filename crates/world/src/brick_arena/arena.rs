use std::collections::HashMap;
use super::atom::RawAtom;
use super::brick::{Brick8, BrickCell};
use super::coord::{BrickCoord, CellCoord};
use super::inertia::{BrickVolume, DismantledSpan};
use super::module_spec::ModuleSpec;

/// Contiguous `Brick8` arena (`Vec<Brick8>`) with `u32` slot recycling and zero per-cell heap allocations (`3` fields).
#[derive(Clone)]
pub struct BrickArena {
    bricks: Vec<Brick8>,
    free_slots: Vec<u32>,
    lookup: HashMap<BrickCoord, u32>,
}

impl BrickArena {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        bricks: Vec<Brick8>,
        free_slots: Vec<u32>,
        lookup: HashMap<BrickCoord, u32>,
    ) -> Self {
        Self { bricks, free_slots, lookup }
    }

    pub fn untouched() -> Self {
        Self::new(Vec::new(), Vec::new(), HashMap::new())
    }

    pub fn active_brick_count(&self) -> usize {
        self.lookup.len()
    }

    pub fn allocated_brick_bytes(&self) -> usize {
        self.bricks.capacity() * core::mem::size_of::<Brick8>()
    }

    pub fn brick_at(&self, bcoord: BrickCoord) -> Option<&Brick8> {
        let &slot_idx = self.lookup.get(&bcoord)?;
        self.bricks.get(slot_idx as usize)
    }

    fn ensure_brick(&mut self, bcoord: BrickCoord) -> &mut Brick8 {
        if let Some(&slot_idx) = self.lookup.get(&bcoord) {
            return &mut self.bricks[slot_idx as usize];
        }
        let slot_idx = if let Some(reused) = self.free_slots.pop() {
            self.bricks[reused as usize] = Brick8::empty();
            reused
        } else {
            let idx = self.bricks.len() as u32;
            self.bricks.push(Brick8::empty());
            idx
        };
        self.lookup.insert(bcoord, slot_idx);
        &mut self.bricks[slot_idx as usize]
    }

    fn write_cell_raw(&mut self, coord: CellCoord, cell: BrickCell) -> BrickCell {
        let (bcoord, slot) = coord.brick_split();
        if matches!(cell, BrickCell::Empty) {
            let Some(&slot_idx) = self.lookup.get(&bcoord) else {
                return BrickCell::Empty;
            };
            let brick = &mut self.bricks[slot_idx as usize];
            let prior = brick.clear_slot(slot);
            if brick.vacant() {
                self.lookup.remove(&bcoord);
                self.free_slots.push(slot_idx);
            }
            return prior;
        }
        self.ensure_brick(bcoord).write_slot(slot, cell)
    }

    pub fn span_vacant(&self, anchor: CellCoord, spec: ModuleSpec) -> bool {
        let mut clear = true;
        spec.span().visit_offsets(|off| {
            if clear {
                let (du, dv, dw) = off.deltas();
                if self.cell_at(anchor.shifted(du as i32, dv as i32, dw as i32)).occupied() {
                    clear = false;
                }
            }
        });
        clear
    }

    pub fn mutate_atom(&mut self, coord: CellCoord, f: impl FnOnce(RawAtom) -> RawAtom) -> bool {
        let (bcoord, slot) = coord.brick_split();
        let Some(&slot_idx) = self.lookup.get(&bcoord) else {
            return false;
        };
        let brick = &mut self.bricks[slot_idx as usize];
        let BrickCell::RawAtom(atom) = brick.cell_at(slot) else {
            return false;
        };
        brick.write_slot(slot, BrickCell::RawAtom(f(atom)));
        true
    }

    pub fn visit_bricks(&self, mut visitor: impl FnMut(BrickCoord, &Brick8)) {
        for (&bcoord, &slot_idx) in &self.lookup {
            visitor(bcoord, &self.bricks[slot_idx as usize]);
        }
    }

    pub fn visit_occupied_cells(&self, mut visitor: impl FnMut(CellCoord, BrickCell)) {
        for (&bcoord, &slot_idx) in &self.lookup {
            self.bricks[slot_idx as usize].visit_occupied(|slot, cell| {
                visitor(bcoord.cell_at(slot), cell);
            });
        }
    }

    fn collect_column_bricks(&self, u: i32, v: i32) -> ([(BrickCoord, u32); 16], usize, u8, u8) {
        let (bx, by) = (u.div_euclid(8), v.div_euclid(8));
        let (lu, lv) = (u.rem_euclid(8) as u8, v.rem_euclid(8) as u8);
        let (mut stack, mut count) = ([(BrickCoord::new(0, 0, 0), 0u32); 16], 0usize);
        for (&bcoord, &slot_idx) in &self.lookup {
            if bcoord.planar_sector() == (bx, by)
                && self.bricks[slot_idx as usize].column_mask(lu, lv) != 0
                && count < stack.len()
            {
                stack[count] = (bcoord, slot_idx);
                count += 1;
            }
        }
        stack[..count].sort_unstable_by_key(|&(bc, _)| bc);
        (stack, count, lu, lv)
    }

    /// Zero-allocation stack traversal of occupied `CellCoord`s along `(u, v)` from lowest `w` to highest `w`.
    pub fn visit_column(&self, u: i32, v: i32, mut visitor: impl FnMut(CellCoord, BrickCell)) -> usize {
        let (stack, count, lu, lv) = self.collect_column_bricks(u, v);
        let mut cells = 0usize;
        for &(bcoord, slot_idx) in &stack[..count] {
            let brick = &self.bricks[slot_idx as usize];
            let mut col = brick.column_mask(lu, lv);
            while col != 0 {
                let lw = col.trailing_zeros() as u8;
                let slot = super::coord::LocalSlot::from_axes(lu, lv, lw);
                visitor(bcoord.cell_at(slot), brick.cell_at(slot));
                cells += 1;
                col &= col - 1;
            }
        }
        cells
    }

    /// Zero-allocation reverse stack traversal of occupied `CellCoord`s along `(u, v)` from highest `w` to lowest `w`.
    pub fn find_in_column_rev<T>(
        &self,
        u: i32,
        v: i32,
        mut pred: impl FnMut(CellCoord, BrickCell) -> Option<T>,
    ) -> Option<T> {
        let (stack, count, lu, lv) = self.collect_column_bricks(u, v);
        for &(bcoord, slot_idx) in stack[..count].iter().rev() {
            let brick = &self.bricks[slot_idx as usize];
            let mut col = brick.column_mask(lu, lv);
            while col != 0 {
                let lw = (7 - col.leading_zeros()) as u8;
                let slot = super::coord::LocalSlot::from_axes(lu, lv, lw);
                if let Some(found) = pred(bcoord.cell_at(slot), brick.cell_at(slot)) {
                    return Some(found);
                }
                col &= !(1u8 << lw);
            }
        }
        None
    }

    pub fn any_in_column(&self, u: i32, v: i32, mut pred: impl FnMut(CellCoord, BrickCell) -> bool) -> bool {
        self.find_in_column_rev(u, v, |c, cell| pred(c, cell).then_some(())).is_some()
    }

    pub fn column_present(&self, u: i32, v: i32) -> bool {
        let (bx, by) = (u.div_euclid(8), v.div_euclid(8));
        let (lu, lv) = (u.rem_euclid(8) as u8, v.rem_euclid(8) as u8);
        self.lookup.iter().any(|(&bcoord, &slot_idx)| {
            bcoord.planar_sector() == (bx, by) && self.bricks[slot_idx as usize].column_mask(lu, lv) != 0
        })
    }
}

impl BrickVolume for BrickArena {
    fn vacant(&self) -> bool {
        self.lookup.is_empty()
    }

    fn brick_count(&self) -> usize {
        self.lookup.len()
    }

    fn cell_at(&self, coord: CellCoord) -> BrickCell {
        let (bcoord, slot) = coord.brick_split();
        let Some(&slot_idx) = self.lookup.get(&bcoord) else {
            return BrickCell::Empty;
        };
        self.bricks[slot_idx as usize].cell_at(slot)
    }

    fn resolve_module(&self, coord: CellCoord) -> Option<(CellCoord, ModuleSpec)> {
        match self.cell_at(coord) {
            BrickCell::ModuleAnchor(spec) => Some((coord, spec)),
            BrickCell::ModuleProxy(off) => {
                let anchor = off.anchor_from(coord);
                match self.cell_at(anchor) {
                    BrickCell::ModuleAnchor(spec) => Some((anchor, spec)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn place_atom(&mut self, coord: CellCoord, atom: RawAtom) -> BrickCell {
        if matches!(self.cell_at(coord), BrickCell::ModuleAnchor(_) | BrickCell::ModuleProxy(_)) {
            let _ = self.dismantle_at(coord);
        }
        self.write_cell_raw(coord, BrickCell::RawAtom(atom))
    }

    fn place_module(&mut self, anchor: CellCoord, spec: ModuleSpec) -> bool {
        if !self.span_vacant(anchor, spec) {
            return false;
        }
        spec.span().visit_offsets(|off| {
            let (du, dv, dw) = off.deltas();
            let target = anchor.shifted(du as i32, dv as i32, dw as i32);
            let cell = if (du, dv, dw) == (0, 0, 0) {
                BrickCell::ModuleAnchor(spec)
            } else {
                BrickCell::ModuleProxy(off)
            };
            let _ = self.write_cell_raw(target, cell);
        });
        true
    }

    fn dismantle_at(&mut self, coord: CellCoord) -> DismantledSpan {
        match self.cell_at(coord) {
            BrickCell::Empty => DismantledSpan::Vacant,
            BrickCell::RawAtom(atom) => {
                let _ = self.write_cell_raw(coord, BrickCell::Empty);
                DismantledSpan::Atom { coord, atom }
            }
            BrickCell::ModuleAnchor(_) | BrickCell::ModuleProxy(_) => {
                let Some((anchor, spec)) = self.resolve_module(coord) else {
                    let _ = self.write_cell_raw(coord, BrickCell::Empty);
                    return DismantledSpan::Vacant;
                };
                spec.span().visit_offsets(|off| {
                    let (du, dv, dw) = off.deltas();
                    let _ = self.write_cell_raw(anchor.shifted(du as i32, dv as i32, dw as i32), BrickCell::Empty);
                });
                DismantledSpan::Module { anchor, spec }
            }
        }
    }
}
