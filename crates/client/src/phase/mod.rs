use crate::threshold::StepThreshold;

mod context;

pub use context::{FlightContext, FlightProgress};

/// Polymorphic vertical flight phase under planetary gravity.
#[derive(Clone, Copy, Debug)]
pub enum FlightPhase {
    Grounded(GroundedFlight),
    Airborne(AirborneFlight),
}

impl FlightPhase {
    pub fn jump(&self, strength: f32) -> Self {
        match self {
            Self::Grounded(g) => g.jump(strength),
            Self::Airborne(a) => a.jump(strength),
        }
    }

    pub fn advance(&self, ctx: &FlightContext) -> FlightProgress {
        match self {
            Self::Grounded(g) => g.advance(ctx),
            Self::Airborne(a) => a.advance(ctx),
        }
    }
}

/// Grounded state resting on the planetary surface floor.
#[derive(Clone, Copy, Debug)]
pub struct GroundedFlight {
    threshold: StepThreshold,
}

impl GroundedFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(threshold: StepThreshold) -> Self {
        Self { threshold }
    }

    /// Secondary constructor with standard human stride calibration.
    pub fn human() -> Self {
        Self::new(StepThreshold::human_stride())
    }

    pub fn jump(&self, strength: f32) -> FlightPhase {
        FlightPhase::Airborne(AirborneFlight::new(strength, self.threshold))
    }

    pub fn advance(&self, ctx: &FlightContext) -> FlightProgress {
        let horizontal = ctx.candidate_horizontal();
        let surface_pos = ctx.constrain_surface(horizontal);
        let clearance = ctx.clearance(horizontal, surface_pos);

        if !clearance.passable_with(ctx.permits_seam_fold()) {
            ctx.resolve_precipice(self.threshold, horizontal)
        } else {
            let next_pos = ctx.resolve_ground_stride(self.threshold);
            FlightProgress::new(FlightPhase::Grounded(GroundedFlight::new(self.threshold)), next_pos)
        }
    }
}

/// Airborne state executing continuous parabolic trajectory under planetary gravity.
#[derive(Clone, Copy, Debug)]
pub struct AirborneFlight {
    vertical_speed: f32,
    threshold: StepThreshold,
}

impl AirborneFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(vertical_speed: f32, threshold: StepThreshold) -> Self {
        Self {
            vertical_speed,
            threshold,
        }
    }

    pub fn jump(&self, _strength: f32) -> FlightPhase {
        FlightPhase::Airborne(AirborneFlight::new(self.vertical_speed, self.threshold))
    }

    pub fn advance(&self, ctx: &FlightContext) -> FlightProgress {
        let (surface_up, vertical_speed, effective_stride) = ctx.reorient_edge_flight(self.vertical_speed);
        let next_speed = vertical_speed - ctx.gravity_velocity_delta();
        let vertical_step = surface_up * (next_speed * ctx.dt());

        let rebound = ctx.resolve_aerial_stride(effective_stride, surface_up, vertical_step, self.threshold);
        let supported = ctx.constrain_surface(rebound);
        let clearance = ctx.clearance(rebound, supported);

        if next_speed <= 0.0 && clearance.floor_contact(self.threshold) {
            FlightProgress::new(FlightPhase::Grounded(GroundedFlight::new(self.threshold)), supported)
        } else {
            FlightProgress::new(FlightPhase::Airborne(AirborneFlight::new(next_speed, self.threshold)), rebound)
        }
    }
}
