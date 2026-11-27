use std::collections::{HashMap, HashSet};

pub const SECTOR_SPAN: usize = 16;
const CONTINENTAL_DIVISION_THRESHOLD: u32 = 512;
const CONTINENTAL_PATCH_SIDE: usize = 16;

/// Per-face sector invalidation queue, sparse meadow revision map, and continental touched-cell set (`<= 4` fields).
pub struct DirtySectors {
    queue: Vec<usize>,
    revisions: HashMap<(i32, i32), u32>,
    touched: HashSet<(i32, i32)>,
    patch_count: usize,
}

impl DirtySectors {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        queue: Vec<usize>,
        revisions: HashMap<(i32, i32), u32>,
        touched: HashSet<(i32, i32)>,
        patch_count: usize,
    ) -> Self {
        Self { queue, revisions, touched, patch_count }
    }

    pub fn sized(divisions: u32) -> Self {
        let side = Self::sectors_per_side(divisions);
        Self::new(Vec::new(), HashMap::new(), HashSet::new(), (side * side).max(1))
    }

    pub const fn continental(divisions: u32) -> bool {
        divisions > CONTINENTAL_DIVISION_THRESHOLD
    }

    pub const fn sectors_per_side(divisions: u32) -> usize {
        if Self::continental(divisions) {
            CONTINENTAL_PATCH_SIDE
        } else {
            let d = divisions as usize;
            let s = d.div_ceil(SECTOR_SPAN);
            if s == 0 { 1 } else { s }
        }
    }

    pub const fn patch_cell_span(divisions: u32) -> usize {
        let side = Self::sectors_per_side(divisions);
        let span = (divisions as usize).div_ceil(side);
        if span == 0 { 1 } else { span }
    }

    pub const fn sector_count(&self) -> usize {
        self.patch_count
    }

    pub const fn touched_cells(&self) -> &HashSet<(i32, i32)> {
        &self.touched
    }

    pub fn record_touched(&mut self, u: i32, v: i32) {
        self.touched.insert((u, v));
    }

    pub fn mark_cell_dirty(&mut self, u: i32, v: i32, half: i32, divisions: u32) {
        self.record_touched(u, v);
        let gsu = u.div_euclid(SECTOR_SPAN as i32);
        let gsv = v.div_euclid(SECTOR_SPAN as i32);
        let rev = self.revisions.entry((gsu, gsv)).or_insert(1);
        *rev = rev.wrapping_add(1).max(1);

        let side = Self::sectors_per_side(divisions);
        let span = Self::patch_cell_span(divisions);
        let su = (((u + half).max(0) as usize) / span).min(side - 1);
        let sv = (((v + half).max(0) as usize) / span).min(side - 1);
        let slot = sv * side + su;
        if !self.queue.contains(&slot) {
            self.queue.push(slot);
        }
    }

    pub fn meadow_revision_at(&self, gsu: i32, gsv: i32, half: i32) -> u32 {
        let half_sec = half.div_euclid(SECTOR_SPAN as i32).max(1);
        if gsu < -half_sec || gsu >= half_sec || gsv < -half_sec || gsv >= half_sec {
            return 0;
        }
        self.revisions.get(&(gsu, gsv)).copied().unwrap_or(1)
    }

    pub fn pending(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn pop_next(&mut self) -> Option<usize> {
        self.queue.pop()
    }
}
