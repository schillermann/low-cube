use glam::Vec3;
use crate::frame::SurfaceFrame;
use super::super::elevation::ReliefElevation;

#[derive(Clone, Copy, Debug)]
pub(super) enum ContactFace {
    Top(f32),
    Underside(f32),
    Lateral((i32, i32), f32),
    CrustCeiling(f32),
}

impl ContactFace {
    const CELL_SPAN: f32 = 0.5;
    const CRUST_SEAM_TOLERANCE: f32 = 0.05;
    const UNDERSIDE_MIN_CLEARANCE: f32 = 0.95;
    const UNDERSIDE_LAYER_OFFSET: f32 = 0.75;
    const UNDERSIDE_RAY_BELOW: f32 = 0.48;
    const UNDERSIDE_DROP: f32 = 1.0;
    const LATERAL_MIN_WALL: f32 = 0.9;
    const LATERAL_BAND_DEPTH: f32 = 0.55;

    #[allow(clippy::too_many_arguments)]
    pub(super) fn classify(
        cell: (i32, i32),
        prior_cell: (i32, i32),
        prior_elev: f32,
        hit_top: f32,
        d_up: f32,
        bounds: (i32, i32),
        relief: &dyn ReliefElevation,
    ) -> Self {
        let ((cu, cv), (prev_cu, prev_cv), (min_c, max_c)) = (cell, prior_cell, bounds);
        let crust_h = (relief.crust_floor_at(cu, cv) as f32) * Self::CELL_SPAN;
        if !relief.breached_at(cu, cv)
            && hit_top <= crust_h + Self::CELL_SPAN + Self::CRUST_SEAM_TOLERANCE
            && (hit_top < crust_h - Self::CRUST_SEAM_TOLERANCE || (d_up > 0.0 && prior_elev < crust_h))
        {
            let snapped = (hit_top * 2.0).round() * Self::CELL_SPAN - 2.0 * Self::CELL_SPAN;
            return Self::CrustCeiling(snapped);
        }
        let ground_top = (relief.elevation_at(cu, cv) as f32) * Self::CELL_SPAN;
        let block_top = ((hit_top * 2.0).round() * Self::CELL_SPAN).max(ground_top);
        if block_top >= ground_top + Self::UNDERSIDE_MIN_CLEARANCE
            && !relief.layer_occupied_at(cu, cv, block_top - Self::UNDERSIDE_LAYER_OFFSET)
            && d_up > 0.0
            && (prior_elev < block_top - Self::UNDERSIDE_RAY_BELOW || (prev_cu, prev_cv) == (cu, cv))
        {
            return Self::Underside(block_top - Self::UNDERSIDE_DROP);
        }
        if (prev_cu, prev_cv) != (cu, cv) && prev_cu >= min_c && prev_cu <= max_c && prev_cv >= min_c && prev_cv <= max_c {
            let ground_prev = (relief.elevation_at(prev_cu, prev_cv) as f32) * Self::CELL_SPAN;
            if block_top >= ground_prev + Self::LATERAL_MIN_WALL
                && prior_elev >= block_top - Self::LATERAL_BAND_DEPTH
                && prior_elev <= block_top
            {
                return Self::Lateral((prev_cu, prev_cv), block_top - Self::CELL_SPAN);
            }
        }
        Self::Top(block_top)
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

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(cell: (i32, i32), hit_top: f32, face: ContactFace) -> Self {
        Self { cell, hit_top, face }
    }

    pub(super) fn target_layer(&self) -> (i32, i32, f32) {
        let (cu, cv) = self.cell;
        (cu, cv, self.hit_top - Self::LAYER_HALF)
    }

    pub(super) fn resolve_placement(
        &self,
        direction: Vec3,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        bounds: (i32, i32),
        avatar_pos: Vec3,
    ) -> (i32, i32, f32) {
        let ((mut cu, mut cv), (min_c, max_c)) = (self.cell, bounds);
        match self.face {
            ContactFace::CrustCeiling(base) | ContactFace::Underside(base) => (cu, cv, base),
            ContactFace::Lateral((ncu, ncv), base) => (ncu, ncv, base),
            ContactFace::Top(block_top) => {
                let (au, av) = frame.decompose(avatar_pos);
                let (acu, acv) = ((au * 2.0).floor() as i32, (av * 2.0).floor() as i32);
                if cu == acu && cv == acv {
                    let (du, dv) = frame.decompose(direction);
                    if du.abs() > dv.abs() {
                        cu = (cu + if du >= 0.0 { 1 } else { -1 }).clamp(min_c, max_c);
                    } else {
                        cv = (cv + if dv >= 0.0 { 1 } else { -1 }).clamp(min_c, max_c);
                    }
                    (cu, cv, (relief.elevation_at(cu, cv) as f32) * Self::CELL_SPAN)
                } else {
                    (cu, cv, block_top)
                }
            }
        }
    }
}
