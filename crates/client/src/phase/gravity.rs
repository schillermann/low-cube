/// Interface defining gravitational acceleration for flight kinematics.
pub trait FlightGravity: Send + Sync {
    /// Computes downward gravitational velocity delta across time step dt.
    fn delta_velocity(&self, vertical_speed: f32, dt: f32) -> f32;
}

/// Uniform gravitational model applying constant acceleration regardless of trajectory phase.
#[derive(Clone, Copy, Debug)]
pub struct UniformGravity {
    magnitude: f32,
}

impl UniformGravity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(magnitude: f32) -> Self {
        Self { magnitude }
    }
}

impl FlightGravity for UniformGravity {
    fn delta_velocity(&self, _vertical_speed: f32, dt: f32) -> f32 {
        self.magnitude * dt
    }
}

/// Asymmetric gravitational decorator: decorates any base flight gravity with hang-time near apex and snappy descent.
#[derive(Clone, Copy, Debug)]
pub struct AsymmetricGravity<G: FlightGravity = UniformGravity> {
    base: G,
    ascent_ratio: f32,
    apex_ratio: f32,
    descent_ratio: f32,
    apex_threshold: f32,
}

impl<G: FlightGravity> AsymmetricGravity<G> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        base: G,
        ascent_ratio: f32,
        apex_ratio: f32,
        descent_ratio: f32,
        apex_threshold: f32,
    ) -> Self {
        Self {
            base,
            ascent_ratio,
            apex_ratio,
            descent_ratio,
            apex_threshold,
        }
    }
}

impl AsymmetricGravity<UniformGravity> {
    /// Secondary constructor decorating uniform gravity calibrated for terrestrial platforming feel.
    pub fn terrestrial(gravity: f32) -> Self {
        Self::new(UniformGravity::new(gravity), 1.0, 0.75, 1.50, 0.35)
    }
}

impl<G: FlightGravity> FlightGravity for AsymmetricGravity<G> {
    fn delta_velocity(&self, vertical_speed: f32, dt: f32) -> f32 {
        let base_delta = self.base.delta_velocity(vertical_speed, dt);
        let ratio = if vertical_speed > self.apex_threshold {
            self.ascent_ratio
        } else if vertical_speed < -self.apex_threshold {
            self.descent_ratio
        } else {
            self.apex_ratio
        };
        base_delta * ratio
    }
}
