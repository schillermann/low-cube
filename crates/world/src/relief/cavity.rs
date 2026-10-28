use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::noise::SpatialFrequency;

/// Subterranean cavern cavity query result delivered to consumers ("Tell, Don't Ask").
#[derive(Clone, Copy, Debug)]
pub struct CavitySpan {
    floor: f32,
    ceiling: f32,
}

impl CavitySpan {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(floor: f32, ceiling: f32) -> Self {
        Self { floor, ceiling }
    }

    /// Evaluates if the given vertical position is inside this subterranean cavity.
    pub fn contains_height(&self, height: f32) -> bool {
        height >= self.floor && height <= self.ceiling
    }

    /// Feeds cavity span boundaries to a consumer closure.
    pub fn inspect<R>(&self, consumer: impl FnOnce(f32, f32) -> R) -> R {
        consumer(self.floor, self.ceiling)
    }
}

/// Behavioral interface for subterranean cave cavities and grottos beneath the planetary surface.
pub trait SubterraneanCavity: Send + Sync {
    /// Queries the subterranean cavity at discrete cell (u, v), delivering data to consumer if present.
    fn inspect_cavity(&self, u: i32, v: i32, consumer: &mut dyn FnMut(CavitySpan));

    /// Resolves avatar vertical constraint, selecting cave floor when inside or falling through entrance.
    fn constrain_depth(&self, u: f32, v: f32, height: f32, surface_height: f32) -> f32;
}

/// Null object pattern for planetary faces without subterranean cavities ("Tell, Don't Ask").
pub struct NullCavity;

impl SubterraneanCavity for NullCavity {
    fn inspect_cavity(&self, _u: i32, _v: i32, _consumer: &mut dyn FnMut(CavitySpan)) {}

    fn constrain_depth(&self, _u: f32, _v: f32, _height: f32, surface_height: f32) -> f32 {
        surface_height
    }
}

/// 3D coherent noise subterranean cave system generating tunnels and grottos.
pub struct ZonedCavity {
    frequency: Box<dyn SpatialFrequency>,
    frame: SurfaceFrame,
    half_size: f32,
    threshold: f32,
}

impl ZonedCavity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        frequency: Box<dyn SpatialFrequency>,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
        half_size: f32,
        threshold: f32,
    ) -> Self {
        Self {
            frequency,
            frame: SurfaceFrame::new(tangent, bitangent, normal, half_size),
            half_size,
            threshold,
        }
    }

    fn sample_at(&self, u: f32, v: f32, depth: f32) -> f32 {
        self.frequency.sample(self.frame.point_at(u, v, depth))
    }
}

impl SubterraneanCavity for ZonedCavity {
    fn inspect_cavity(&self, u: i32, v: i32, consumer: &mut dyn FnMut(CavitySpan)) {
        let wu = self.frame.step_elevation(u);
        let wv = self.frame.step_elevation(v);
        let in_spawn = if self.half_size >= 24.0 {
            wu.abs() <= 2.0 && wv.abs() <= 2.0
        } else {
            wu.abs() <= 1.0 || wv.abs() <= 1.0
        };
        if in_spawn {
            return;
        }
        let sample = self.sample_at(wu, wv, -1.0);
        if sample.abs() < self.threshold {
            consumer(CavitySpan::new(-2.0, -1.0));
        }
    }

    fn constrain_depth(&self, u: f32, v: f32, height: f32, surface_height: f32) -> f32 {
        let in_spawn = if self.half_size >= 24.0 {
            u.abs() <= 2.0 && v.abs() <= 2.0
        } else {
            u.abs() <= 1.0 || v.abs() <= 1.0
        };
        if in_spawn {
            return surface_height;
        }
        let sample = self.sample_at(u, v, -1.0);
        if sample.abs() < self.threshold {
            let span = CavitySpan::new(-2.0, -1.0);
            if span.contains_height(height) {
                return span.floor;
            }
        }
        surface_height
    }
}
