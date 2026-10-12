use glam::Vec3;
use world::PlanetaryWorld;

/// Interface for polymorphic vertical flight phases under planetary gravity.
pub trait FlightPhase: Send + Sync {
    fn jump(self: Box<Self>, strength: f32) -> Box<dyn FlightPhase>;
    fn advance(
        self: Box<Self>,
        dt: f32,
        position: Vec3,
        surface_up: Vec3,
        eye_height: f32,
        gravity: f32,
        world: &dyn PlanetaryWorld,
    ) -> (Box<dyn FlightPhase>, Vec3);
}

/// Grounded state resting on the planetary surface floor.
pub struct GroundedFlight;

impl FlightPhase for GroundedFlight {
    fn jump(self: Box<Self>, strength: f32) -> Box<dyn FlightPhase> {
        Box::new(AirborneFlight::new(strength))
    }

    fn advance(
        self: Box<Self>,
        _dt: f32,
        position: Vec3,
        surface_up: Vec3,
        eye_height: f32,
        _gravity: f32,
        world: &dyn PlanetaryWorld,
    ) -> (Box<dyn FlightPhase>, Vec3) {
        let supported = world.constrain_to_surface(position, eye_height);
        let floor_distance = (position - supported).dot(surface_up);
        if floor_distance > 1e-4 {
            (Box::new(AirborneFlight::new(0.0)), position)
        } else {
            (self, supported)
        }
    }
}

/// Airborne state subjected to gravitational acceleration.
pub struct AirborneFlight {
    vertical_speed: f32,
}

impl AirborneFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(vertical_speed: f32) -> Self {
        Self { vertical_speed }
    }
}

impl FlightPhase for AirborneFlight {
    fn jump(self: Box<Self>, _strength: f32) -> Box<dyn FlightPhase> {
        self
    }

    fn advance(
        self: Box<Self>,
        dt: f32,
        position: Vec3,
        surface_up: Vec3,
        eye_height: f32,
        gravity: f32,
        world: &dyn PlanetaryWorld,
    ) -> (Box<dyn FlightPhase>, Vec3) {
        let next_speed = self.vertical_speed - gravity * dt;
        let step = surface_up * (next_speed * dt);
        let candidate = position + step;
        let supported = world.constrain_to_surface(candidate, eye_height);
        let floor_distance = (candidate - supported).dot(surface_up);

        if floor_distance <= 1e-4 && next_speed <= 0.0 {
            (Box::new(GroundedFlight), supported)
        } else {
            (Box::new(AirborneFlight::new(next_speed)), candidate)
        }
    }
}

/// Interface representing vertical flight and gravitational physics on a planetary surface.
pub trait GravitationalFlight: Send + Sync {
    fn jump(&mut self);
    fn advance(
        &mut self,
        dt: f32,
        position: Vec3,
        surface_up: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Vec3;
}

/// Polymorphic planetary flight entity coordinating GroundedFlight and AirborneFlight phases.
pub struct SurfaceFlight {
    phase: Box<dyn FlightPhase>,
    jump_strength: f32,
    gravity: f32,
    eye_height: f32,
}

impl SurfaceFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        phase: Box<dyn FlightPhase>,
        jump_strength: f32,
        gravity: f32,
        eye_height: f32,
    ) -> Self {
        Self {
            phase,
            jump_strength,
            gravity,
            eye_height,
        }
    }

    /// Secondary constructor initializing in grounded state with authentic Earth gravity and realistic human leap.
    pub fn terrestrial(eye_height: f32) -> Self {
        Self::new(Box::new(GroundedFlight), 3.2, 9.81, eye_height)
    }
}

impl GravitationalFlight for SurfaceFlight {
    fn jump(&mut self) {
        let current = std::mem::replace(&mut self.phase, Box::new(GroundedFlight));
        self.phase = current.jump(self.jump_strength);
    }

    fn advance(
        &mut self,
        dt: f32,
        position: Vec3,
        surface_up: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Vec3 {
        let current = std::mem::replace(&mut self.phase, Box::new(GroundedFlight));
        let (next_phase, next_position) = current.advance(
            dt,
            position,
            surface_up,
            self.eye_height,
            self.gravity,
            world,
        );
        self.phase = next_phase;
        next_position
    }
}

