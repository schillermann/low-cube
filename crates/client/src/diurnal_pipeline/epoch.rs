use std::time::Instant;
use glam::{Mat4, Vec3};
use world::{ApexSpin, PlanetarySpin};
use crate::aurora_cadence::AuroraCadence;
use crate::celestial::CelestialCadence;
use crate::clock::MonotonicTime;
use crate::firmament::{FirmamentEphemeris, VaultEphemeris, VaultGlow, VaultView};
use crate::rayleigh_shell::{AtmosphericEnvelope, AtmosphericShell};

/// Focused contract evaluating solar direction, `VaultView`, and `VaultGlow` for a render pass.
pub trait DiurnalEpoch: Send + Sync {
    fn evaluate_pass(
        &self,
        inv_view_proj: Mat4,
        cam_pos: Vec3,
        ephemeris: &VaultEphemeris,
    ) -> (Vec3, VaultView, VaultGlow);
}

/// Encapsulated monotonic solar & aurora epoch (`<= 4` strictly private fields).
pub struct MeasuredEpoch {
    cadence: Box<dyn CelestialCadence>,
    aurora: Box<dyn AuroraCadence>,
    clock: Box<dyn MonotonicTime>,
    start_instant: Instant,
}

impl MeasuredEpoch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        cadence: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        clock: Box<dyn MonotonicTime>,
        start_instant: Instant,
    ) -> Self {
        Self { cadence, aurora, clock, start_instant }
    }
}

impl DiurnalEpoch for MeasuredEpoch {
    fn evaluate_pass(
        &self,
        inv_view_proj: Mat4,
        cam_pos: Vec3,
        ephemeris: &VaultEphemeris,
    ) -> (Vec3, VaultView, VaultGlow) {
        let now = self.clock.current_instant();
        let elapsed = now.duration_since(self.start_instant).as_secs_f64();
        let progress = self.cadence.day_progress(elapsed);

        let ((sun_dir, pole, local_up), (clear_color, night_factor, local_progress)) = ephemeris
            .evaluate_illumination(
                |helio_sun, sun_ang, shell| {
                    let (body_up, clear, night) =
                        shell.sky_illumination_with_flux(helio_sun, sun_ang);
                    let (axis, hour_prog) =
                        ApexSpin::terrestrial().diurnal_meridian(helio_sun, body_up, progress);
                    ((helio_sun, axis, body_up), (clear, night, hour_prog))
                },
                || {
                    let world_sun = self.cadence.sun_direction(progress);
                    let (earth_up, clear, night) =
                        AtmosphericShell::terrestrial_at(cam_pos).sky_illumination(world_sun);
                    ((world_sun, Vec3::Z, earth_up), (clear, night, progress))
                },
            );
        let aurora_factor = self.aurora.activity(local_progress, elapsed, night_factor);
        let view = VaultView::new(inv_view_proj, sun_dir, pole, local_up);
        let glow = VaultGlow::new(clear_color, night_factor, aurora_factor, elapsed as f32);
        (sun_dir, view, glow)
    }
}
