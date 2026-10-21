use glam::Vec3;
use crate::noise::SpatialFrequency;

/// Behavioral interface for planetary relief elevation evaluation ("Tell, Don't Ask").
pub trait ReliefElevation: Send + Sync {
    /// Discretized integer block elevation relative to base face datum.
    fn elevation_at(&self, u: i32, v: i32) -> i32;

    /// Pristine natural block elevation prior to player earthworks.
    fn natural_elevation_at(&self, u: i32, v: i32) -> i32 {
        self.elevation_at(u, v)
    }

    /// Continuous surface elevation in meters for physical avatar constraint.
    fn surface_height_at(&self, u: f32, v: f32) -> f32;

    /// Indicates whether the discrete cell (u, v) was excavated or stacked by earthworks.
    fn reworked_at(&self, _u: i32, _v: i32) -> bool {
        false
    }

    /// Partial digging stage (0..=3) at discrete cell (u, v) before full block excavation.
    fn dig_stage_at(&self, _u: i32, _v: i32) -> u8 {
        0
    }

    /// Marks partial digging progress (0..=3) at discrete cell (u, v).
    fn mark_dig_stage(&mut self, _u: i32, _v: i32, _stage: u8) -> (i32, bool) {
        (0, false)
    }

    /// Excavates one discrete block layer at (u, v).
    fn excavate_cell(&mut self, _u: i32, _v: i32) -> (i32, bool) {
        (0, false)
    }

    /// Stacks one discrete block layer at (u, v).
    fn stack_cell(&mut self, _u: i32, _v: i32) -> bool {
        false
    }
}

/// Zoned planetary relief evaluating continuous spatial frequency into stepped voxel strata.
pub struct ZonedRelief {
    frequency: Box<dyn SpatialFrequency>,
    normal: Vec3,
    tangent: Vec3,
    bitangent: Vec3,
    half_size: f32,
    min_elevation: i32,
    max_elevation: i32,
}

impl ZonedRelief {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        frequency: Box<dyn SpatialFrequency>,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
        half_size: f32,
        min_elevation: i32,
        max_elevation: i32,
    ) -> Self {
        Self {
            frequency,
            normal,
            tangent,
            bitangent,
            half_size,
            min_elevation,
            max_elevation,
        }
    }

    fn sample_at(&self, u: f32, v: f32) -> f32 {
        let point = self.normal * self.half_size + self.tangent * u + self.bitangent * v;
        self.frequency.sample(point)
    }
}

impl ReliefElevation for ZonedRelief {
    fn elevation_at(&self, u: i32, v: i32) -> i32 {
        let wu = (u as f32) * 0.5;
        let wv = (v as f32) * 0.5;
        let r = wu.hypot(wv);
        let spawn_range = if self.half_size >= 48.0 { 16.0 } else if self.half_size >= 24.0 { 10.0 } else { 4.0 };
        let spawn_weight = ((r - 2.5) / spawn_range).clamp(0.0, 1.0);
        let spawn_smooth = spawn_weight * spawn_weight * (3.0 - 2.0 * spawn_weight);

        let edge_dist = (self.half_size - wu.abs()).min(self.half_size - wv.abs());
        let border_weight = ((edge_dist - 1.5) / 8.0).clamp(0.0, 1.0);
        let border_smooth = border_weight * border_weight * (3.0 - 2.0 * border_weight);

        let weight = spawn_smooth * border_smooth;
        if weight <= 0.0 {
            return 0;
        }

        let s = self.sample_at(wu, wv);
        let normalized = (s + 1.0) * 0.5;
        let range = (self.max_elevation - self.min_elevation) as f32;
        let raw = (normalized * range * weight).round() as i32;
        raw.clamp(self.min_elevation, self.max_elevation)
    }


    fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        let cell_u = (u * 2.0).floor() as i32;
        let cell_v = (v * 2.0).floor() as i32;
        let h = self.elevation_at(cell_u, cell_v);

        let frac_u = (u * 2.0 - cell_u as f32).clamp(0.0, 1.0);
        let frac_v = (v * 2.0 - cell_v as f32).clamp(0.0, 1.0);


        let hn = self.elevation_at(cell_u, cell_v + 1);
        let hs = self.elevation_at(cell_u, cell_v - 1);
        let he = self.elevation_at(cell_u + 1, cell_v);
        let hw = self.elevation_at(cell_u - 1, cell_v);

        let (dn, ds, de, dw) = (h - hn, h - hs, h - he, h - hw);
        if dn == 1 && ds <= 0 && de <= 0 && dw <= 0 {
            return ((h as f32) - frac_v) * 0.5;
        }
        if ds == 1 && dn <= 0 && de <= 0 && dw <= 0 {
            return ((h as f32) - (1.0 - frac_v)) * 0.5;
        }
        if de == 1 && dn <= 0 && ds <= 0 && dw <= 0 {
            return ((h as f32) - frac_u) * 0.5;
        }
        if dw == 1 && dn <= 0 && ds <= 0 && de <= 0 {
            return ((h as f32) - (1.0 - frac_u)) * 0.5;
        }

        if dn == 1 && de == 1 && ds <= 0 && dw <= 0 {
            return ((h as f32) - (frac_u + frac_v).min(1.0)) * 0.5;
        }
        if dn == 1 && dw == 1 && ds <= 0 && de <= 0 {
            return ((h as f32) - ((1.0 - frac_u) + frac_v).min(1.0)) * 0.5;
        }
        if ds == 1 && de == 1 && dn <= 0 && dw <= 0 {
            return ((h as f32) - (frac_u + (1.0 - frac_v)).min(1.0)) * 0.5;
        }
        if ds == 1 && dw == 1 && dn <= 0 && de <= 0 {
            return ((h as f32) - ((1.0 - frac_u) + (1.0 - frac_v)).min(1.0)) * 0.5;
        }

        if dn <= 0 && ds <= 0 && de <= 0 && dw <= 0 {
            let hse = self.elevation_at(cell_u + 1, cell_v - 1);
            let hsw = self.elevation_at(cell_u - 1, cell_v - 1);
            let hne = self.elevation_at(cell_u + 1, cell_v + 1);
            let hnw = self.elevation_at(cell_u - 1, cell_v + 1);
            if hse < h && hs == h && he == h {
                return ((h as f32) - frac_u.min(1.0 - frac_v)) * 0.5;
            }
            if hsw < h && hs == h && hw == h {
                return ((h as f32) - (1.0 - frac_u).min(1.0 - frac_v)) * 0.5;
            }
            if hne < h && hn == h && he == h {
                return ((h as f32) - frac_u.min(frac_v)) * 0.5;
            }
            if hnw < h && hn == h && hw == h {
                return ((h as f32) - (1.0 - frac_u).min(frac_v)) * 0.5;
            }
        }

        (h as f32) * 0.5
    }
}
