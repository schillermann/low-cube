use glam::Vec3;
use crate::frame::SurfaceFrame;
use super::elevation::ReliefElevation;

/// Sight ray locating a discrete planetary surface cell for excavation or block placement.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceSightRay {
    eye: Vec3,
    direction: Vec3,
}

impl SurfaceSightRay {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(eye: Vec3, direction: Vec3) -> Self {
        Self { eye, direction }
    }

    /// Resolves the targeted discrete grid cell (u, v) along the sight ray within at most three blocks of planar reach.
    pub fn target_cell(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
    ) -> Option<(i32, i32)> {
        self.target_layer(frame, relief, half_blocks, half_size).map(|(cu, cv, _)| (cu, cv))
    }

    /// Resolves the targeted discrete grid cell (u, v) and vertical layer midpoint along the sight ray.
    pub fn target_layer(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
    ) -> Option<(i32, i32, f32)> {
        self.trace_hit(frame, relief, half_blocks, half_size)
            .map(|(cu, cv, _, _, _, hit_top)| (cu, cv, hit_top - 0.25))
    }

    fn trace_hit(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
    ) -> Option<(i32, i32, i32, i32, f32, f32)> {
        let (dir, up_norm) = (self.direction.normalize_or_zero(), frame.normal());
        let (d_up, (eu, ev)) = (dir.dot(up_norm), frame.decompose(self.eye));
        let flat_dir = (dir - up_norm * d_up).normalize_or_zero();
        let slanted = flat_dir.length_squared() > 1e-4;
        let (su, sv) = frame.decompose(self.eye - flat_dir * if slanted { 0.09 + (-d_up).clamp(0.0, 1.0) * 0.07 } else { 0.0 });
        let (scu, scv) = ((su * 2.0).floor() as i32, (sv * 2.0).floor() as i32);
        let (ecu, ecv) = ((eu * 2.0).floor() as i32, (ev * 2.0).floor() as i32);
        let (min_c, max_c) = (-half_blocks + 1, half_blocks - 2);
        let (mut prev_cu, mut prev_cv, mut prev_elev) = (ecu, ecv, frame.height_of(self.eye) - half_size);
        let max_stand_top = prev_elev - 0.90;
        let (up, eye_steps, eye_crust) = (d_up > 0.75, relief.elevation_at(ecu, ecv), relief.crust_floor_at(ecu, ecv));
        let weightless = relief.weightless_at(eu, ev, prev_elev);
        let (steps, max_span) = if weightless { (100, 4) } else { (85, 3) };

        for step in 1..=steps {
            let sample = self.eye + dir * ((step as f32) * 0.04);
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
                if let Some(hit_top) = relief.hit_top_at(u, v, ray_elev) {
                    if !weightless && slanted && d_up < 0.0 && (cu, cv) == (scu, scv) && hit_top <= max_stand_top {
                        return None;
                    }
                    return Some((cu, cv, prev_cu, prev_cv, prev_elev, hit_top));
                }
            }
            (prev_cu, prev_cv, prev_elev) = (cu, cv, ray_elev);
        }
        if d_up < -0.999 && ecu >= min_c && ecu <= max_c && ecv >= min_c && ecv <= max_c {
            let floor_h = (eye_steps as f32) * 0.5;
            if (frame.height_of(self.eye) - half_size) - floor_h <= 3.4 {
                return Some((ecu, ecv, ecu, ecv, floor_h + 0.5, floor_h));
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
        let (mut cu, mut cv, prev_cu, prev_cv, prev_elev, surf_elev) =
            self.trace_hit(frame, relief, half_blocks, half_size)?;
        let (min_c, max_c) = (-half_blocks + 1, half_blocks - 2);
        let crust_h = (relief.crust_floor_at(cu, cv) as f32) * 0.5;
        if !relief.breached_at(cu, cv) && surf_elev < crust_h - 0.05 {
            return Some((cu, cv, ((surf_elev + 0.25) * 2.0).round() * 0.5 - 0.5));
        }
        let ground_top = (relief.elevation_at(cu, cv) as f32) * 0.5;
        let block_top = ((surf_elev * 2.0).round() * 0.5).max(ground_top);
        if block_top >= ground_top + 0.95
            && !relief.layer_occupied_at(cu, cv, block_top - 0.75)
            && self.direction.dot(frame.normal()) > 0.0
            && (prev_elev < block_top - 0.48 || (prev_cu, prev_cv) == (cu, cv))
        {
            return Some((cu, cv, block_top - 1.0));
        }
        let col_top = ground_top.max((surf_elev * 2.0).ceil() * 0.5);
        if (prev_cu, prev_cv) != (cu, cv) && prev_cu >= min_c && prev_cu <= max_c && prev_cv >= min_c && prev_cv <= max_c {
            let ground_prev = (relief.elevation_at(prev_cu, prev_cv) as f32) * 0.5;
            if col_top >= ground_prev + 0.9 && prev_elev >= col_top - 0.55 && prev_elev <= col_top {
                return Some((prev_cu, prev_cv, col_top - 0.5));
            }
        }
        let (au, av) = frame.decompose(avatar_pos);
        let (acu, acv) = ((au * 2.0).floor() as i32, (av * 2.0).floor() as i32);
        if cu == acu && cv == acv {
            let (du, dv) = frame.decompose(self.direction);
            if du.abs() > dv.abs() {
                cu = (cu + if du >= 0.0 { 1 } else { -1 }).clamp(min_c, max_c);
            } else {
                cv = (cv + if dv >= 0.0 { 1 } else { -1 }).clamp(min_c, max_c);
            }
            return Some((cu, cv, (relief.elevation_at(cu, cv) as f32) * 0.5));
        }
        Some((cu, cv, block_top))
    }
}
