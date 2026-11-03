use glam::Vec3;
use crate::frame::SurfaceFrame;
use super::elevation::ReliefElevation;

#[derive(Clone, Copy, Debug)]
enum ContactFace {
    Top(f32),
    Underside(f32),
    Lateral((i32, i32), f32),
    CrustCeiling(f32),
}

impl ContactFace {
    const CELL_SPAN: f32 = 0.5;
    const LAYER_HALF: f32 = 0.25;
    const CRUST_SEAM_TOLERANCE: f32 = 0.05;
    const UNDERSIDE_MIN_CLEARANCE: f32 = 0.95;
    const UNDERSIDE_LAYER_OFFSET: f32 = 0.75;
    const UNDERSIDE_RAY_BELOW: f32 = 0.48;
    const UNDERSIDE_DROP: f32 = 1.0;
    const LATERAL_MIN_WALL: f32 = 0.9;
    const LATERAL_BAND_DEPTH: f32 = 0.55;

    #[allow(clippy::too_many_arguments)]
    fn classify(
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
        if !relief.breached_at(cu, cv) && hit_top < crust_h - Self::CRUST_SEAM_TOLERANCE {
            let snapped = ((hit_top + Self::LAYER_HALF) * 2.0).round() * Self::CELL_SPAN - Self::CELL_SPAN;
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
        let col_top = ground_top.max((hit_top * 2.0).ceil() * Self::CELL_SPAN);
        if (prev_cu, prev_cv) != (cu, cv)
            && prev_cu >= min_c
            && prev_cu <= max_c
            && prev_cv >= min_c
            && prev_cv <= max_c
        {
            let ground_prev = (relief.elevation_at(prev_cu, prev_cv) as f32) * Self::CELL_SPAN;
            if col_top >= ground_prev + Self::LATERAL_MIN_WALL
                && prior_elev >= col_top - Self::LATERAL_BAND_DEPTH
                && prior_elev <= col_top
            {
                return Self::Lateral((prev_cu, prev_cv), col_top - Self::CELL_SPAN);
            }
        }
        Self::Top(block_top)
    }
}

#[derive(Clone, Copy, Debug)]
struct SightContact {
    cell: (i32, i32),
    hit_top: f32,
    face: ContactFace,
}

impl SightContact {
    const CELL_SPAN: f32 = 0.5;
    const LAYER_HALF: f32 = 0.25;

    fn target_cell(&self) -> (i32, i32) {
        self.cell
    }

    fn target_layer(&self) -> (i32, i32, f32) {
        let (cu, cv) = self.cell;
        (cu, cv, self.hit_top - Self::LAYER_HALF)
    }

    fn resolve_placement(
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

/// Sight ray locating a discrete planetary surface cell for excavation or block placement.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceSightRay {
    eye: Vec3,
    direction: Vec3,
}

impl SurfaceSightRay {
    const CELL_SPAN: f32 = 0.5;
    const LAYER_HALF: f32 = 0.25;
    const BROW_FORWARD: f32 = 0.09;
    const BROW_PITCH_REACH: f32 = 0.07;
    const RAY_STEP: f32 = 0.04;
    const FRONT_PROBE_DIST: f32 = 0.35;
    const CLOSE_WALL_RETREAT: f32 = 0.28;
    const CLOSE_WALL_MAX_SLOPE: f32 = -0.10;
    const STEEP_DOWN_DOT: f32 = -0.96;
    const STEEP_UP_DOT: f32 = 0.95;
    const NADIR_DOT: f32 = -0.999;
    const NADIR_MAX_REACH: f32 = 3.4;
    const STAND_FLOOR_CLEARANCE: f32 = 0.90;
    const WALL_PROBE_LIFT: f32 = 0.15;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(eye: Vec3, direction: Vec3) -> Self {
        Self { eye, direction }
    }

    /// Shifts a cranium center forward to the brow camera origin along the surface normal and horizontal heading.
    pub fn brow_origin(&self, up: Vec3, horizontal_fwd: Vec3) -> Vec3 {
        let d_up = self.direction.normalize_or_zero().dot(up);
        self.eye + horizontal_fwd.normalize_or_zero() * self.brow_offset(d_up)
    }

    fn brow_offset(&self, d_up: f32) -> f32 {
        Self::BROW_FORWARD + (-d_up).clamp(0.0, 1.0) * Self::BROW_PITCH_REACH
    }

    /// Resolves the targeted discrete grid cell (u, v) along the sight ray within at most three blocks of planar reach.
    pub fn target_cell(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
    ) -> Option<(i32, i32)> {
        self.trace_contact(frame, relief, half_blocks, half_size)
            .map(|contact| contact.target_cell())
    }

    /// Resolves the targeted discrete grid cell (u, v) and vertical layer midpoint along the sight ray.
    pub fn target_layer(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
    ) -> Option<(i32, i32, f32)> {
        self.trace_contact(frame, relief, half_blocks, half_size)
            .map(|contact| contact.target_layer())
    }

    fn trace_contact(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
    ) -> Option<SightContact> {
        let (dir, up_norm) = (self.direction.normalize_or_zero(), frame.normal());
        let (d_up, (eu, ev)) = (dir.dot(up_norm), frame.decompose(self.eye));
        let flat_dir = (dir - up_norm * d_up).normalize_or_zero();
        let slanted = flat_dir.length_squared() > 1e-4;
        let (su, sv) = frame.decompose(self.eye - flat_dir * if slanted { self.brow_offset(d_up) } else { 0.0 });
        let (scu, scv) = ((su * 2.0).floor() as i32, (sv * 2.0).floor() as i32);
        let (ecu, ecv) = ((eu * 2.0).floor() as i32, (ev * 2.0).floor() as i32);
        let (fu, fv) = frame.decompose(self.eye + flat_dir * Self::FRONT_PROBE_DIST);
        let stand_steps = relief.elevation_at(scu, scv);
        let stand_top = (stand_steps as f32) * Self::CELL_SPAN;
        let front_steps = relief.elevation_at((fu * 2.0).floor() as i32, (fv * 2.0).floor() as i32);
        let close_wall = slanted && d_up < Self::CLOSE_WALL_MAX_SLOPE && d_up > Self::STEEP_DOWN_DOT && front_steps > stand_steps;
        let ray_origin = if close_wall { self.eye - flat_dir * Self::CLOSE_WALL_RETREAT } else { self.eye };
        let bounds @ (min_c, max_c) = (-half_blocks + 1, half_blocks - 2);
        let start_elev = frame.height_of(self.eye) - half_size;
        let (mut prev_cu, mut prev_cv, mut prev_elev) = (ecu, ecv, start_elev);
        let max_stand_top = prev_elev - Self::STAND_FLOOR_CLEARANCE;
        let (up, eye_steps, eye_crust) = (
            d_up > Self::STEEP_UP_DOT,
            relief.elevation_at(ecu, ecv),
            relief.crust_floor_at(ecu, ecv),
        );
        let weightless = relief.weightless_at(eu, ev, prev_elev);
        let (steps, max_span) = if weightless { (100, 4) } else { (85, 3) };

        for step in 1..=steps {
            let sample = ray_origin + dir * ((step as f32) * Self::RAY_STEP);
            let (u, v) = frame.decompose(sample);
            let (cu, cv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
            if (cu - ecu).abs().max((cv - ecv).abs()) > max_span {
                break;
            }
            let ray_elev = frame.height_of(sample) - half_size;
            if cu >= min_c && cu <= max_c && cv >= min_c && cv <= max_c {
                if up && (cu, cv) != (ecu, ecv) && relief.crust_floor_at(cu, cv) < eye_crust {
                    continue;
                }
                let wall_top = (relief.elevation_at(cu, cv) as f32) * Self::CELL_SPAN;
                let probe_elev = if (cu, cv) != (scu, scv)
                    && wall_top > stand_top + Self::LAYER_HALF
                    && ray_elev >= stand_top - Self::LAYER_HALF
                    && d_up > Self::STEEP_DOWN_DOT
                {
                    ray_elev.max(stand_top + Self::WALL_PROBE_LIFT)
                } else {
                    ray_elev
                };
                if let Some(hit_top) = relief.hit_top_at(u, v, probe_elev) {
                    if !weightless && slanted && d_up < 0.0 && (cu, cv) == (scu, scv) && hit_top <= max_stand_top {
                        if close_wall {
                            continue;
                        }
                        return None;
                    }
                    let face = ContactFace::classify((cu, cv), (prev_cu, prev_cv), prev_elev, hit_top, d_up, bounds, relief);
                    return Some(SightContact { cell: (cu, cv), hit_top, face });
                }
            }
            (prev_cu, prev_cv, prev_elev) = (cu, cv, ray_elev);
        }
        if d_up < Self::NADIR_DOT && ecu >= min_c && ecu <= max_c && ecv >= min_c && ecv <= max_c {
            let floor_h = (eye_steps as f32) * Self::CELL_SPAN;
            if (frame.height_of(self.eye) - half_size) - floor_h <= Self::NADIR_MAX_REACH {
                let face = ContactFace::classify((ecu, ecv), (ecu, ecv), floor_h + Self::CELL_SPAN, floor_h, d_up, bounds, relief);
                return Some(SightContact { cell: (ecu, ecv), hit_top: floor_h, face });
            }
        }
        None
    }

    /// Resolves a placement cell and base elevation within three-block reach, attaching laterally or on the underside of elevated blocks.
    pub fn placement_cell(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
        avatar_pos: Vec3,
    ) -> Option<(i32, i32, f32)> {
        let contact = self.trace_contact(frame, relief, half_blocks, half_size)?;
        let bounds = (-half_blocks + 1, half_blocks - 2);
        Some(contact.resolve_placement(self.direction, frame, relief, bounds, avatar_pos))
    }
}
