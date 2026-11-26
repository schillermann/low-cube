use std::time::Instant;
use glam::{Mat4, Vec3};
use world::{ApexSpin, ClimateZone, PlanetarySpin};
use crate::aurora_cadence::AuroraCadence;
use crate::celestial::CelestialCadence;
use crate::clock::MonotonicTime;
use crate::firmament::{FirmamentEphemeris, VaultEphemeris, VaultGlow, VaultView};
use crate::rayleigh_shell::{AtmosphericEnvelope, AtmosphericShell, BandCurve, SmoothBand};

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
        let spin = ApexSpin::terrestrial();
        let magnetic_pole = ClimateZone::Arctic.normal();

        let ((sun_dir, pole), (clear_color, mag_night, arctic_hour)) = ephemeris
            .evaluate_illumination(
                |helio_sun, (sun_ang, eclipse_gate), shell| {
                    let (_, clear, _) = shell.sky_illumination_with_flux(helio_sun, sun_ang);
                    let (axis, polar_prog) =
                        spin.diurnal_meridian(helio_sun, magnetic_pole, progress);
                    let polar_dark = SmoothBand::new(0.12, -0.22)
                        .sample(helio_sun.dot(magnetic_pole))
                        .max(eclipse_gate);
                    let active_dark = if shell.magnetized() { polar_dark } else { 0.0 };
                    ((helio_sun, axis), (clear, active_dark, polar_prog))
                },
                || {
                    let world_sun = self.cadence.sun_direction(progress);
                    let (_, clear, _) =
                        AtmosphericShell::terrestrial_at(cam_pos).sky_illumination(world_sun);
                    let (axis, polar_prog) =
                        spin.diurnal_meridian(world_sun, magnetic_pole, progress);
                    let polar_dark =
                        SmoothBand::new(0.12, -0.22).sample(world_sun.dot(magnetic_pole));
                    ((world_sun, axis), (clear, polar_dark, polar_prog))
                },
            );
        let aurora_factor = self.aurora.activity(arctic_hour, elapsed, mag_night);
        let view = VaultView::new(inv_view_proj, sun_dir, pole);
        let glow = VaultGlow::new(clear_color, aurora_factor, elapsed as f32);
        (sun_dir, view, glow)
    }
}
