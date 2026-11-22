use std::f32::consts::TAU;
use glam::Vec3;

/// Celestial timing and solar-angle cadence interface.
pub trait CelestialCadence: Send + Sync {
    fn duration_seconds(&self) -> f32;
    fn day_progress(&self, elapsed_seconds: f64) -> f32;
    fn sun_direction(&self, progress: f32) -> Vec3;
    fn clone_cadence(&self) -> Box<dyn CelestialCadence>;
}

/// Baseline planetary cadence with 20-minute (1200 seconds) planetary rotation.
pub struct TerrestrialCadence;

impl TerrestrialCadence {
    /// Primary constructor doing nothing except creating instance.
    pub fn new() -> Self {
        Self
    }
}

impl CelestialCadence for TerrestrialCadence {
    fn duration_seconds(&self) -> f32 {
        1200.0
    }

    fn day_progress(&self, elapsed_seconds: f64) -> f32 {
        let duration = self.duration_seconds() as f64;
        let cycle = elapsed_seconds % duration;
        let positive_cycle = if cycle < 0.0 { cycle + duration } else { cycle };
        (positive_cycle / duration) as f32
    }

    fn sun_direction(&self, progress: f32) -> Vec3 {
        let angle = progress * TAU;
        Vec3::new(angle.cos(), angle.sin(), 0.35).normalize()
    }

    fn clone_cadence(&self) -> Box<dyn CelestialCadence> {
        Box::new(Self::new())
    }
}

/// Custom configured celestial cadence decorator with explicit cycle duration.
pub struct TunedCadence {
    duration: f32,
    origin: Box<dyn CelestialCadence>,
}

impl TunedCadence {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(duration: f32, origin: Box<dyn CelestialCadence>) -> Self {
        Self { duration, origin }
    }

    /// Secondary constructor decorating terrestrial cadence.
    #[cfg(test)]
    pub fn terrestrial(duration: f32) -> Self {
        Self::new(duration, Box::new(TerrestrialCadence::new()))
    }
}

impl CelestialCadence for TunedCadence {
    fn duration_seconds(&self) -> f32 {
        self.duration
    }

    fn day_progress(&self, elapsed_seconds: f64) -> f32 {
        if self.duration <= 0.0 {
            0.25
        } else {
            let dur = self.duration as f64;
            let cycle = elapsed_seconds % dur;
            let positive_cycle = if cycle < 0.0 { cycle + dur } else { cycle };
            (positive_cycle / dur) as f32
        }
    }

    fn sun_direction(&self, progress: f32) -> Vec3 {
        self.origin.sun_direction(progress)
    }

    fn clone_cadence(&self) -> Box<dyn CelestialCadence> {
        Box::new(Self::new(self.duration, self.origin.clone_cadence()))
    }
}
