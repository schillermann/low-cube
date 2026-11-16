use super::super::elevation::ReliefElevation;

/// Focused interface for a resolved voxel block placement target.
pub(super) trait PlacementTarget {
    fn cell_and_base(&self, relief: &dyn ReliefElevation) -> Option<(i32, i32, f32)>;
}

#[derive(Clone, Copy, Debug)]
enum ContactFace {
    Top(f32),
    Underside(f32),
    Lateral((i32, i32), f32),
}

/// Encapsulated ray-step boundary crossing between two terrain cells.
#[derive(Clone, Copy, Debug)]
pub(super) struct StepCrossing {
    cells: ((i32, i32), (i32, i32)),
    elevations: (f32, f32),
    slope_up: f32,
    bounds: (i32, i32),
}

impl StepCrossing {
    const CELL_SPAN: f32 = 0.5;
    const CRUST_SEAM_TOLERANCE: f32 = 0.05;
    const UNDERSIDE_MIN_CLEARANCE: f32 = 0.95;
    const UNDERSIDE_LAYER_OFFSET: f32 = 0.75;
    const UNDERSIDE_RAY_BELOW: f32 = 0.48;
    const UNDERSIDE_DROP: f32 = 1.0;
    const LATERAL_MIN_WALL: f32 = 0.9;
    const LATERAL_BAND_DEPTH: f32 = 0.55;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        cells: ((i32, i32), (i32, i32)),
        elevations: (f32, f32),
        slope_up: f32,
        bounds: (i32, i32),
    ) -> Self {
        Self { cells, elevations, slope_up, bounds }
    }

    fn prior_in_bounds(&self) -> bool {
        let ((cu, cv), (prev_cu, prev_cv)) = self.cells;
        let (min_c, max_c) = self.bounds;
        (prev_cu, prev_cv) != (cu, cv)
            && prev_cu >= min_c
            && prev_cu <= max_c
            && prev_cv >= min_c
            && prev_cv <= max_c
    }

    fn subterranean_lateral(&self, raw_top: f32, ground_top: f32, relief: &dyn ReliefElevation) -> Option<ContactFace> {
        if raw_top >= ground_top - Self::CRUST_SEAM_TOLERANCE || !self.prior_in_bounds() {
            return None;
        }
        let (_, (prev_cu, prev_cv)) = self.cells;
        let ground_prev = (relief.elevation_at(prev_cu, prev_cv) as f32) * Self::CELL_SPAN;
        let exposed = raw_top >= ground_prev + Self::CELL_SPAN - Self::CRUST_SEAM_TOLERANCE
            && !relief.layer_occupied_at(prev_cu, prev_cv, raw_top - Self::CELL_SPAN * 0.5);
        exposed.then_some(ContactFace::Lateral((prev_cu, prev_cv), raw_top - Self::CELL_SPAN))
    }

    fn surface_lateral(&self, block_top: f32, prev_free: bool, relief: &dyn ReliefElevation) -> Option<ContactFace> {
        if !prev_free || !self.prior_in_bounds() {
            return None;
        }
        let ((cu, cv), (prev_cu, prev_cv)) = self.cells;
        let (prior_elev, _) = self.elevations;
        let layer_mid = block_top - Self::CELL_SPAN * 0.5;
        let ground_prev = (relief.elevation_at(prev_cu, prev_cv) as f32) * Self::CELL_SPAN;
        let min_wall = if relief.layer_occupied_at(cu, cv, layer_mid) && self.slope_up > -0.65 {
            Self::CELL_SPAN - Self::CRUST_SEAM_TOLERANCE
        } else {
            Self::LATERAL_MIN_WALL
        };
        if block_top >= ground_prev + min_wall
            && prior_elev >= block_top - Self::LATERAL_BAND_DEPTH
            && prior_elev <= block_top
        {
            return Some(ContactFace::Lateral((prev_cu, prev_cv), block_top - Self::CELL_SPAN));
        }
        let lower_occupied = relief.layer_occupied_at(cu, cv, block_top - Self::UNDERSIDE_LAYER_OFFSET);
        if lower_occupied
            && block_top - Self::CELL_SPAN >= ground_prev + min_wall
            && prior_elev >= block_top - Self::CELL_SPAN - Self::LATERAL_BAND_DEPTH
            && prior_elev <= block_top - Self::CELL_SPAN + Self::CRUST_SEAM_TOLERANCE
        {
            return Some(ContactFace::Lateral((prev_cu, prev_cv), block_top - 2.0 * Self::CELL_SPAN));
        }
        None
    }

    pub(super) fn contact(&self, relief: &dyn ReliefElevation) -> SightContact {
        let ((cu, cv), (prev_cu, prev_cv)) = self.cells;
        let (prior_elev, hit_top) = self.elevations;
        let (min_c, max_c) = self.bounds;
        let ground_top = (relief.elevation_at(cu, cv) as f32) * Self::CELL_SPAN;
        let raw_top = (hit_top * 2.0).round() * Self::CELL_SPAN;
        if let Some(sub_lat) = self.subterranean_lateral(raw_top, ground_top, relief) {
            return SightContact::new((cu, cv), hit_top, sub_lat);
        }
        let block_top = raw_top.max(ground_top);
        let layer_mid = block_top - Self::CELL_SPAN * 0.5;
        let backrest = relief.seat_backrest_cell(cu, cv, layer_mid);
        if block_top >= ground_top + Self::UNDERSIDE_MIN_CLEARANCE
            && !relief.layer_occupied_at(cu, cv, block_top - Self::UNDERSIDE_LAYER_OFFSET)
            && self.slope_up > 0.0
            && (prior_elev < block_top - Self::UNDERSIDE_RAY_BELOW || (prev_cu, prev_cv) == (cu, cv))
        {
            return SightContact::new((cu, cv), hit_top, ContactFace::Underside(block_top - Self::UNDERSIDE_DROP));
        }
        let prev_free = backrest.is_none() || !relief.layer_occupied_at(prev_cu, prev_cv, layer_mid);
        if let Some(lat) = self.surface_lateral(block_top, prev_free, relief) {
            return SightContact::new((cu, cv), hit_top, lat);
        }
        let face = backrest
            .filter(|&(bu, bv)| bu >= min_c && bu <= max_c && bv >= min_c && bv <= max_c)
            .map_or(ContactFace::Top(block_top), |(bu, bv)| ContactFace::Lateral((bu, bv), block_top - Self::CELL_SPAN));
        SightContact::new((cu, cv), hit_top, face)
    }
}

/// Encapsulated voxel ray contact on a terrain cell face.
#[derive(Clone, Copy, Debug)]
pub(super) struct SightContact {
    cell: (i32, i32),
    hit_top: f32,
    face: ContactFace,
}

impl SightContact {
    const CELL_SPAN: f32 = 0.5;
    const LAYER_HALF: f32 = 0.25;
    const UPRIGHT_HEADROOM: f32 = 1.75;
    const UNDERSIDE_HEADROOM: f32 = 0.95;

    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(cell: (i32, i32), hit_top: f32, face: ContactFace) -> Self {
        Self { cell, hit_top, face }
    }

    pub(super) fn target_layer(&self) -> (i32, i32, f32) {
        let (cu, cv) = self.cell;
        (cu, cv, self.hit_top - Self::LAYER_HALF)
    }

    pub(super) fn clearance_above(&self, foot_elev: f32, slope_up: f32) -> f32 {
        match self.face {
            ContactFace::Underside(_) if slope_up > 0.5 => foot_elev + Self::UNDERSIDE_HEADROOM,
            _ => foot_elev + Self::UPRIGHT_HEADROOM,
        }
    }
}

impl PlacementTarget for SightContact {
    fn cell_and_base(&self, relief: &dyn ReliefElevation) -> Option<(i32, i32, f32)> {
        let (cu, cv) = self.cell;
        match self.face {
            ContactFace::Underside(base) => Some((cu, cv, base)),
            ContactFace::Lateral((ncu, ncv), base) => Some((ncu, ncv, base)),
            ContactFace::Top(block_top) => {
                let ground_top = (relief.elevation_at(cu, cv) as f32) * Self::CELL_SPAN;
                (ground_top <= self.hit_top + 0.05).then_some((cu, cv, block_top))
            }
        }
    }
}
