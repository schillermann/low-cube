use glam::Vec3;
use crate::frame::SurfaceFrame;
use super::super::elevation::ReliefElevation;
use super::contact::PlacementTarget;

/// Decorator over a `PlacementTarget` rejecting placements inside or beneath the avatar's body volume.
#[derive(Clone, Copy, Debug)]
pub(super) struct StanceClearance<T> {
    origin: T,
    planar: (f32, f32),
    elevations: (f32, f32),
    bounds: (i32, i32),
}

impl<T> StanceClearance<T> {
    const CELL_SPAN: f32 = 0.5;
    const LAYER_HALF: f32 = 0.25;
    const STANCE_SEAM_MARGIN: f32 = 0.02;
    const EYE_SUPPORT_OFFSET: f32 = 1.0;
    const STAND_ORIGIN_THRESHOLD: f32 = 0.85;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        origin: T,
        planar: (f32, f32),
        elevations: (f32, f32),
        bounds: (i32, i32),
    ) -> Self {
        Self { origin, planar, elevations, bounds }
    }

    pub(super) fn around_avatar(
        origin: T,
        avatar_pos: Vec3,
        slope_up: f32,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        bounds: (i32, i32),
    ) -> Self {
        let (au, av) = frame.decompose(avatar_pos);
        let (acu, acv) = ((au * 2.0).floor() as i32, (av * 2.0).floor() as i32);
        let raw_elev = frame.elevation_of(avatar_pos);
        let stand_ground = (relief.elevation_at(acu, acv) as f32) * Self::CELL_SPAN;
        let foot_elev = if raw_elev >= stand_ground + Self::STAND_ORIGIN_THRESHOLD {
            raw_elev - Self::EYE_SUPPORT_OFFSET
        } else {
            raw_elev.max(stand_ground)
        };
        Self::new(origin, (au, av), (foot_elev, slope_up), bounds)
    }

    fn within_bounds(&self, u: i32, v: i32) -> bool {
        let (min_c, max_c) = self.bounds;
        u >= min_c && u <= max_c && v >= min_c && v <= max_c
    }

    fn overlaps_footprint(&self, u: i32, v: i32) -> bool {
        let (u0, u1) = ((u as f32) * Self::CELL_SPAN, ((u + 1) as f32) * Self::CELL_SPAN);
        let (v0, v1) = ((v as f32) * Self::CELL_SPAN, ((v + 1) as f32) * Self::CELL_SPAN);
        let du = self.planar.0 - self.planar.0.clamp(u0, u1);
        let dv = self.planar.1 - self.planar.1.clamp(v0, v1);
        du * du + dv * dv < Self::STANCE_SEAM_MARGIN * Self::STANCE_SEAM_MARGIN
    }

    fn stacked_floor(&self, u: i32, v: i32, raw_base: f32, relief: &dyn ReliefElevation) -> f32 {
        let _ = self.bounds;
        let ground_top = (relief.elevation_at(u, v) as f32) * Self::CELL_SPAN;
        let crust_top = (relief.crust_floor_at(u, v) as f32) * Self::CELL_SPAN;
        let sub = raw_base < crust_top + 0.1;
        let mut next_floor = if sub { raw_base } else { raw_base.max(ground_top) };
        while relief.layer_occupied_at(u, v, next_floor + Self::LAYER_HALF) {
            next_floor += if sub { -Self::CELL_SPAN } else { Self::CELL_SPAN };
        }
        next_floor
    }
}

impl<T: PlacementTarget> PlacementTarget for StanceClearance<T> {
    fn cell_and_base(&self, relief: &dyn ReliefElevation) -> Option<(i32, i32, f32)> {
        let (cu, cv, raw_base) = self.origin.cell_and_base(relief)?;
        if !self.within_bounds(cu, cv) {
            return None;
        }
        if !self.overlaps_footprint(cu, cv) {
            return Some((cu, cv, raw_base));
        }
        let (foot_elev, slope_up) = self.elevations;
        let actual_base = self.stacked_floor(cu, cv, raw_base, relief);
        let head_limit = self.origin.clearance_above(foot_elev, slope_up);
        (actual_base >= head_limit - 0.02).then_some((cu, cv, raw_base))
    }

    fn clearance_above(&self, foot_elev: f32, slope_up: f32) -> f32 {
        self.origin.clearance_above(foot_elev, slope_up)
    }
}
