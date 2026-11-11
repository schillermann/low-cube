mod contact;

use glam::Vec3;
use crate::frame::SurfaceFrame;
use super::elevation::ReliefElevation;
use self::contact::{ContactFace, SightContact};

#[derive(Clone, Copy, Debug)]
struct RayProbe {
    eye_cell: (i32, i32),
    stand_cell: (i32, i32),
    elevations: (f32, f32, bool),
}

impl RayProbe {
    fn from_ray(ray: &SurfaceSightRay, frame: &SurfaceFrame, relief: &dyn ReliefElevation, half_size: f32) -> Self {
        let (dir, up_norm) = (ray.direction.normalize_or_zero(), frame.normal());
        let (d_up, (eu, ev)) = (dir.dot(up_norm), frame.decompose(ray.eye));
        let flat_dir = (dir - up_norm * d_up).normalize_or_zero();
        let slanted = flat_dir.length_squared() > 1e-4;
        let (su, sv) = frame.decompose(ray.eye - flat_dir * if slanted { ray.brow_offset(d_up) } else { 0.0 });
        let (scu, scv) = ((su * 2.0).floor() as i32, (sv * 2.0).floor() as i32);
        let (ecu, ecv) = ((eu * 2.0).floor() as i32, (ev * 2.0).floor() as i32);
        let (fu, fv) = frame.decompose(ray.eye + flat_dir * SurfaceSightRay::FRONT_PROBE_DIST);
        let (fcu, fcv) = ((fu * 2.0).floor() as i32, (fv * 2.0).floor() as i32);
        let stand_steps = relief.elevation_at(scu, scv);
        let stand_top = (stand_steps as f32) * SurfaceSightRay::CELL_SPAN;
        let front_steps = relief.elevation_at(fcu, fcv);
        let start_elev = frame.height_of(ray.eye) - half_size;
        let front_wall = front_steps > stand_steps
            || relief
                .hit_top_at(fu, fv, start_elev - SurfaceSightRay::STAND_FLOOR_CLEARANCE)
                .is_some_and(|h| h > stand_top + SurfaceSightRay::LAYER_HALF);
        let close_wall = slanted && d_up < SurfaceSightRay::CLOSE_WALL_MAX_SLOPE && d_up > SurfaceSightRay::STEEP_DOWN_DOT && front_wall;
        Self { eye_cell: (ecu, ecv), stand_cell: (scu, scv), elevations: (stand_top, start_elev, close_wall) }
    }

    fn probe_elevation(&self, cu: i32, cv: i32, wall_top: f32, ray_elev: f32, d_up: f32) -> f32 {
        let stand_top = self.elevations.0;
        if (cu, cv) != self.stand_cell
            && wall_top > stand_top + SurfaceSightRay::LAYER_HALF
            && ray_elev >= stand_top - SurfaceSightRay::LAYER_HALF
            && d_up > SurfaceSightRay::STEEP_DOWN_DOT
        {
            ray_elev.max(stand_top + SurfaceSightRay::WALL_PROBE_LIFT)
        } else {
            ray_elev
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn step_hit_top(
        &self,
        relief: &dyn ReliefElevation,
        prev_cell: (i32, i32),
        cell: (i32, i32),
        prev_uv: (f32, f32),
        uv: (f32, f32),
        elevs: (f32, f32, f32, f32),
    ) -> Option<f32> {
        let ((prev_cu, prev_cv), (cu, cv), (prev_u, prev_v), (u, v)) = (prev_cell, cell, prev_uv, uv);
        let (prev_elev, ray_elev, wall_top, d_up) = elevs;
        if (cu, cv) != (prev_cu, prev_cv) {
            let alpha_u = if cu != prev_cu && (u - prev_u).abs() > 1e-6 {
                (((cu.max(prev_cu) as f32) * SurfaceSightRay::CELL_SPAN - prev_u) / (u - prev_u)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let alpha_v = if cv != prev_cv && (v - prev_v).abs() > 1e-6 {
                (((cv.max(prev_cv) as f32) * SurfaceSightRay::CELL_SPAN - prev_v) / (v - prev_v)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let alpha = (alpha_u.max(alpha_v) + 1e-4).min(1.0);
            let (entry_u, entry_v) = (prev_u + alpha * (u - prev_u), prev_v + alpha * (v - prev_v));
            let entry_elev = prev_elev + alpha * (ray_elev - prev_elev);
            let entry_probe = self.probe_elevation(cu, cv, wall_top, entry_elev, d_up);
            if let Some(hit) = relief.hit_top_at(entry_u, entry_v, entry_probe) {
                return Some(hit);
            }
        }
        relief.hit_top_at(u, v, self.probe_elevation(cu, cv, wall_top, ray_elev, d_up))
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
    const CLOSE_WALL_MAX_SLOPE: f32 = -0.10;
    const STEEP_DOWN_DOT: f32 = -0.96;
    const STEEP_UP_DOT: f32 = 0.75;
    const ZENITH_UP_DOT: f32 = 0.98;
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

    /// Evaluates a 3D point at `distance` meters along the normalized sight ray.
    pub fn sample_point(&self, distance: f32) -> Vec3 {
        self.eye + self.direction.normalize_or_zero() * distance
    }

    /// Selects the cardinal surface axis vector best aligned with the sight ray direction.
    pub fn align_heading(&self, frame: &SurfaceFrame) -> Vec3 {
        let candidates = [frame.bitangent(), frame.tangent(), -frame.bitangent(), -frame.tangent()];
        candidates
            .into_iter()
            .max_by(|a, b| a.dot(self.direction).total_cmp(&b.dot(self.direction)))
            .unwrap_or_else(|| frame.bitangent())
    }

    fn brow_offset(&self, d_up: f32) -> f32 {
        Self::BROW_FORWARD + (-d_up).clamp(0.0, 1.0) * Self::BROW_PITCH_REACH
    }

    /// Resolves the targeted discrete grid cell (u, v) and vertical layer midpoint along the sight ray.
    pub fn target_layer(
        &self, frame: &SurfaceFrame, relief: &dyn ReliefElevation, half_blocks: i32, half_size: f32,
    ) -> Option<(i32, i32, f32)> {
        self.trace_contact(frame, relief, half_blocks, half_size)
            .map(|contact| contact.target_layer())
    }

    fn trace_contact(
        &self, frame: &SurfaceFrame, relief: &dyn ReliefElevation, half_blocks: i32, half_size: f32,
    ) -> Option<SightContact> {
        let (dir, up_norm) = (self.direction.normalize_or_zero(), frame.normal());
        let (d_up, (eu, ev)) = (dir.dot(up_norm), frame.decompose(self.eye));
        let probe = RayProbe::from_ray(self, frame, relief, half_size);
        let ((ecu, ecv), (scu, scv), (_, start_elev, close_wall)) = (probe.eye_cell, probe.stand_cell, probe.elevations);
        let slanted = (dir - up_norm * d_up).length_squared() > 1e-4;
        let bounds @ (min_c, max_c) = (-half_blocks + 1, half_blocks - 2);
        let (mut prev_cu, mut prev_cv, mut prev_u, mut prev_v, mut prev_elev) = (ecu, ecv, eu, ev, start_elev);
        let max_stand_top = prev_elev - Self::STAND_FLOOR_CLEARANCE;
        let (up, eye_steps, eye_crust) = (d_up > Self::STEEP_UP_DOT, relief.elevation_at(ecu, ecv), relief.crust_floor_at(ecu, ecv));
        let start_occupied = relief.layer_occupied_at(ecu, ecv, start_elev);
        let start_layer_top = ((start_elev / Self::CELL_SPAN).floor() + 1.0) * Self::CELL_SPAN;
        let (steps, max_span) = if up { (100, 4) } else { (85, 3) };

        for step in 1..=steps {
            let sample = self.eye + dir * ((step as f32) * Self::RAY_STEP);
            let (u, v) = frame.decompose(sample);
            let (cu, cv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
            if (cu - ecu).abs().max((cv - ecv).abs()) > max_span {
                break;
            }
            let ray_elev = frame.height_of(sample) - half_size;
            if cu >= min_c && cu <= max_c && cv >= min_c && cv <= max_c {
                if d_up > Self::ZENITH_UP_DOT && (cu, cv) != (ecu, ecv) && relief.crust_floor_at(cu, cv) < eye_crust {
                    continue;
                }
                let wall_top = (relief.elevation_at(cu, cv) as f32) * Self::CELL_SPAN;
                if let Some(hit_top) = probe.step_hit_top(
                    relief, (prev_cu, prev_cv), (cu, cv), (prev_u, prev_v), (u, v), (prev_elev, ray_elev, wall_top, d_up),
                ) {
                    if start_occupied && (cu, cv) == (ecu, ecv) && (ecu, ecv) == (scu, scv) && (hit_top - start_layer_top).abs() < 0.05 {
                        (prev_cu, prev_cv, prev_u, prev_v, prev_elev) = (cu, cv, u, v, ray_elev);
                        continue;
                    }
                    if slanted
                        && d_up < 0.0
                        && (cu, cv) == (scu, scv)
                        && hit_top <= max_stand_top
                        && !relief.layer_occupied_at(cu, cv, hit_top - Self::LAYER_HALF)
                    {
                        if close_wall { continue; }
                        return None;
                    }
                    let face = ContactFace::classify((cu, cv), (prev_cu, prev_cv), prev_elev, hit_top, d_up, bounds, relief);
                    return Some(SightContact::new((cu, cv), hit_top, face));
                }
            }
            (prev_cu, prev_cv, prev_u, prev_v, prev_elev) = (cu, cv, u, v, ray_elev);
        }
        if d_up < Self::NADIR_DOT && ecu >= min_c && ecu <= max_c && ecv >= min_c && ecv <= max_c {
            let floor_h = (eye_steps as f32) * Self::CELL_SPAN;
            if start_elev - floor_h <= Self::NADIR_MAX_REACH {
                let face = ContactFace::classify((ecu, ecv), (ecu, ecv), floor_h + Self::CELL_SPAN, floor_h, d_up, bounds, relief);
                return Some(SightContact::new((ecu, ecv), floor_h, face));
            }
        }
        None
    }

    /// Resolves a placement cell and base elevation within three-block reach, attaching laterally or on the underside of elevated blocks.
    pub fn placement_cell(
        &self, frame: &SurfaceFrame, relief: &dyn ReliefElevation, half_blocks: i32, half_size: f32, avatar_pos: Vec3,
    ) -> Option<(i32, i32, f32)> {
        let contact = self.trace_contact(frame, relief, half_blocks, half_size)?;
        let bounds = (-half_blocks + 1, half_blocks - 2);
        Some(contact.resolve_placement(self.direction, frame, relief, bounds, avatar_pos))
    }
}
