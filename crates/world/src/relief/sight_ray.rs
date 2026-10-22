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
        self.trace_hit(frame, relief, half_blocks, half_size).map(|(cu, cv, _, _, _, _)| (cu, cv))
    }

    fn trace_hit(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
    ) -> Option<(i32, i32, i32, i32, f32, f32)> {
        let dir = self.direction.normalize_or_zero();
        let (eu, ev) = frame.decompose(self.eye);
        let (ecu, ecv) = ((eu * 2.0).floor() as i32, (ev * 2.0).floor() as i32);
        let (min_c, max_c) = (-half_blocks + 1, half_blocks - 2);
        let (mut prev_cu, mut prev_cv, mut prev_elev) = (ecu, ecv, frame.height_of(self.eye) - half_size);

        for step in 1..=85 {
            let sample = self.eye + dir * ((step as f32) * 0.04);
            let (u, v) = frame.decompose(sample);
            let (cu, cv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
            if (cu - ecu).abs().max((cv - ecv).abs()) > 3 {
                break;
            }
            let ray_elev = frame.height_of(sample) - half_size;
            if cu >= min_c && cu <= max_c && cv >= min_c && cv <= max_c {
                let surf_elev = relief.surface_height_at(u, v);
                let ground_h = (relief.elevation_at(cu, cv) as f32) * 0.5;
                let float_base = ((surf_elev - 0.01) * 2.0).floor() * 0.5;
                let hit = if surf_elev > ground_h + 0.02 {
                    ray_elev <= ground_h + 0.02 || (ray_elev >= float_base - 0.02 && ray_elev <= surf_elev + 0.02)
                } else {
                    ray_elev <= surf_elev + 0.02
                };
                if hit {
                    return Some((cu, cv, prev_cu, prev_cv, prev_elev, surf_elev));
                }
            }
            (prev_cu, prev_cv, prev_elev) = (cu, cv, ray_elev);
        }
        None
    }

    /// Resolves a placement cell and base elevation within three-block reach, attaching laterally when aiming at an elevated wall.
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
        let col_top = ((relief.elevation_at(cu, cv) as f32) * 0.5).max((surf_elev * 2.0).ceil() * 0.5);
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
        }
        Some((cu, cv, (relief.elevation_at(cu, cv) as f32) * 0.5))
    }
}
