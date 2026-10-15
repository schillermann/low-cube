use glam::Vec3;
use super::FlightPhase;

/// Outcome of a flight phase transition containing the subsequent phase and updated position.
pub struct FlightProgress {
    phase: FlightPhase,
    position: Vec3,
}

impl FlightProgress {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(phase: FlightPhase, position: Vec3) -> Self {
        Self { phase, position }
    }

    /// Mutates the caller's active phase reference and yields the updated spatial position.
    pub fn apply(self, flight: &mut FlightPhase) -> Vec3 {
        *flight = self.phase;
        self.position
    }
}
