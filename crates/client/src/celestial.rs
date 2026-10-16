use std::f32::consts::TAU;
use glam::{Vec3, Vec4};

/// Celestial timing and firmament appearance interface.
pub trait CelestialCadence: Send + Sync {
    fn duration_seconds(&self) -> f32;
    fn day_progress(&self, elapsed_seconds: f64) -> f32;
    fn sun_direction(&self, progress: f32) -> Vec3;
    fn sky_color(&self, progress: f32) -> Vec4;
    fn clone_cadence(&self) -> Box<dyn CelestialCadence>;
}

/// Baseline planetary cadence with standard 20-minute (1200 seconds) planetary rotation.
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

    fn sky_color(&self, progress: f32) -> Vec4 {
        let sun = self.sun_direction(progress);
        let height = sun.y;

        let day_blue = Vec4::new(0.24, 0.48, 0.90, 1.0);
        let sunset_gold = Vec4::new(0.85, 0.38, 0.12, 1.0);
        let space_black = Vec4::new(0.01, 0.015, 0.03, 1.0);

        if height > 0.15 {
            day_blue
        } else if height >= 0.0 {
            let t = (height / 0.15).clamp(0.0, 1.0);
            sunset_gold.lerp(day_blue, t)
        } else if height >= -0.15 {
            let t = ((height + 0.15) / 0.15).clamp(0.0, 1.0);
            space_black.lerp(sunset_gold, t)
        } else {
            space_black
        }
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

    /// Secondary constructor decorating standard terrestrial cadence.
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
            0.25 // Stationary midday when time is frozen
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

    fn sky_color(&self, progress: f32) -> Vec4 {
        self.origin.sky_color(progress)
    }

    fn clone_cadence(&self) -> Box<dyn CelestialCadence> {
        Box::new(Self::new(self.duration, self.origin.clone_cadence()))
    }
}
