use glam::{Vec3, Vec4};
use crate::argument_celestial::ArgumentCelestial;
use crate::celestial::{CelestialCadence, TerrestrialCadence};
use crate::environment_celestial::EnvironmentCelestial;
use crate::file_celestial::FileCelestial;

/// Server launch celestial cadence decorator coordinating CLI arguments, environment variables, config file, and Terrestrial baseline.
pub struct ServerCelestial {
    cadence: Box<dyn CelestialCadence>,
}

impl ServerCelestial {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cadence: Box<dyn CelestialCadence>) -> Self {
        Self { cadence }
    }

    /// Secondary constructor assembling the full decorator chain in priority order.
    pub fn assembled() -> Self {
        Self::new(Box::new(
            ArgumentCelestial::process(Box::new(
                EnvironmentCelestial::new(Box::new(
                    FileCelestial::server(Box::new(
                        TerrestrialCadence::new(),
                    )),
                )),
            )),
        ))
    }
}

impl CelestialCadence for ServerCelestial {
    fn duration_seconds(&self) -> f32 {
        self.cadence.duration_seconds()
    }

    fn day_progress(&self, elapsed_seconds: f64) -> f32 {
        self.cadence.day_progress(elapsed_seconds)
    }

    fn sun_direction(&self, progress: f32) -> Vec3 {
        self.cadence.sun_direction(progress)
    }

    fn sky_color(&self, progress: f32) -> Vec4 {
        self.cadence.sky_color(progress)
    }

    fn clone_cadence(&self) -> Box<dyn CelestialCadence> {
        Box::new(Self::new(self.cadence.clone_cadence()))
    }
}
