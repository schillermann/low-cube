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
        let dir = self.direction.normalize_or_zero();
        let (eu, ev) = frame.decompose(self.eye);
        let (ecu, ecv) = ((eu * 2.0).floor() as i32, (ev * 2.0).floor() as i32);
        let min_c = -half_blocks + 1;
        let max_c = half_blocks - 2;

        let mut step = 1;
        while step <= 85 {
            let t = (step as f32) * 0.04;
            let sample = self.eye + dir * t;
            let (u, v) = frame.decompose(sample);
            let cu = (u * 2.0).floor() as i32;
            let cv = (v * 2.0).floor() as i32;
            if (cu - ecu).abs().max((cv - ecv).abs()) > 3 {
                break;
            }
            if cu >= min_c && cu <= max_c && cv >= min_c && cv <= max_c {
                let ray_elev = frame.height_of(sample) - half_size;
                let surf_elev = relief.surface_height_at(u, v);
                if ray_elev <= surf_elev + 0.02 {
                    return Some((cu, cv));
                }
            }
            step += 1;
        }
        None
    }

    /// Resolves a placement cell within three-block reach, stepping one cell forward if the target coincides with the avatar's stance cell.
    pub fn placement_cell(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
        avatar_pos: Vec3,
    ) -> Option<(i32, i32)> {
        let (mut cu, mut cv) = self.target_cell(frame, relief, half_blocks, half_size)?;
        let (au, av) = frame.decompose(avatar_pos);
        let (acu, acv) = ((au * 2.0).floor() as i32, (av * 2.0).floor() as i32);
        if cu == acu && cv == acv {
            let min_c = -half_blocks + 1;
            let max_c = half_blocks - 2;
            let (du, dv) = frame.decompose(self.direction);
            if du.abs() > dv.abs() {
                cu = (cu + if du >= 0.0 { 1 } else { -1 }).clamp(min_c, max_c);
            } else {
                cv = (cv + if dv >= 0.0 { 1 } else { -1 }).clamp(min_c, max_c);
            }
        }
        Some((cu, cv))
    }
}
