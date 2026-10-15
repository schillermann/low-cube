use glam::Vec3;
use world::PlanetaryWorld;
use crate::threshold::{StepThreshold, SurfaceClearance, SurfaceStep};

/// Kinematic pacing entity modulating stride across terrain slope incline and decline.
#[derive(Clone, Copy, Debug)]
pub struct InclinePace {
    uphill_penalty: f32,
    downhill_bonus: f32,
    threshold: StepThreshold,
}

impl InclinePace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(uphill_penalty: f32, downhill_bonus: f32, threshold: StepThreshold) -> Self {
        Self {
            uphill_penalty,
            downhill_bonus,
            threshold,
        }
    }

    /// Secondary constructor calibrated for realistic terrestrial slope traversal.
    pub fn calibrated() -> Self {
        Self::new(0.25, 0.35, StepThreshold::human_stride())
    }

    /// Modulates commanded stride based on terrain gradient ahead ("Tell, Don't Ask").
    pub fn pace_stride(
        &self,
        stride: Vec3,
        position: Vec3,
        surface_up: Vec3,
        eye: f32,
        world: &dyn PlanetaryWorld,
    ) -> Vec3 {
        if stride.length_squared() <= 1e-6 {
            return stride;
        }
        let current_ground = world.constrain_to_surface(position, eye);
        let target_ground = world.constrain_to_surface(position + stride, eye);
        let clearance = SurfaceClearance::between(position, position + stride, target_ground, surface_up, world);

        match clearance {
            SurfaceClearance::Planar(_) => {
                let step = SurfaceStep::new(current_ground, target_ground, surface_up);
                if !self.threshold.permits_slope(step.elevation_gain()) {
                    return stride;
                }
                let gradient = step.gradient();
                let factor = if gradient > 0.0 {
                    1.0 - gradient * self.uphill_penalty
                } else {
                    1.0 + (-gradient) * self.downhill_bonus
                };
                stride * factor
            }
            SurfaceClearance::FoldedSeam => stride,
        }
    }
}
