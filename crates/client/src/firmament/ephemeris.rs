use glam::Vec3;
use crate::rayleigh_shell::{
    AtmosphericEnvelope, AtmosphericShell, Magnetosphere, MagnetosphereEnvelope,
};

/// Focused contract for polymorphic planetary ephemeris state (`CadenceFallback` vs. `HelioActive`).
pub trait FirmamentEphemeris: Copy {
    fn for_pass_camera(&self, cam_pos: Vec3) -> VaultEphemeris;
    fn sunbeam_transmittance(&self) -> f32;
    fn evaluate_illumination<R>(
        &self,
        helio_sink: impl FnOnce(Vec3, (f32, f32), AtmosphericShell) -> R,
        fallback_sink: impl FnOnce() -> R,
    ) -> R;
    fn transfer_sky_state<R>(
        &self,
        cam_pos: Vec3,
        sink: impl FnOnce(&[[f32; 4]; 8], [f32; 4], AtmosphericShell, ([f32; 4], [f32; 4])) -> R,
    ) -> R;
}

#[derive(Clone, Copy, Debug)]
enum EphemerisMode {
    CadenceFallback,
    HelioActive {
        planet_cubes: [[f32; 4]; 8],
        occluder: [f32; 4],
        atmosphere: AtmosphericShell,
    },
}

/// Polymorphic planetary ephemeris state (`1` private field) without magic-zero sentinels.
#[derive(Clone, Copy, Debug)]
pub struct VaultEphemeris {
    mode: EphemerisMode,
}

impl VaultEphemeris {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn from_mode(mode: EphemerisMode) -> Self {
        Self { mode }
    }

    pub const fn dormant() -> Self {
        Self::from_mode(EphemerisMode::CadenceFallback)
    }

    pub const fn active(
        planet_cubes: [[f32; 4]; 8],
        occluder: [f32; 4],
        atmosphere: AtmosphericShell,
    ) -> Self {
        Self::from_mode(EphemerisMode::HelioActive { planet_cubes, occluder, atmosphere })
    }
}

impl FirmamentEphemeris for VaultEphemeris {
    fn for_pass_camera(&self, cam_pos: Vec3) -> VaultEphemeris {
        match self.mode {
            EphemerisMode::CadenceFallback => Self::dormant(),
            EphemerisMode::HelioActive { planet_cubes, occluder, atmosphere } => {
                Self::active(planet_cubes, occluder, atmosphere.for_pass_camera(cam_pos))
            }
        }
    }

    fn sunbeam_transmittance(&self) -> f32 {
        match self.mode {
            EphemerisMode::CadenceFallback => 1.0,
            EphemerisMode::HelioActive { planet_cubes, occluder, .. } => {
                let raw_sun = Vec3::new(planet_cubes[7][0], planet_cubes[7][1], planet_cubes[7][2]);
                let sun = raw_sun.normalize_or_zero();
                let weather_beam = raw_sun.length().clamp(0.28, 1.0);
                let occ_dir = Vec3::new(occluder[0], occluder[1], occluder[2]).normalize_or_zero();
                let eclipse_gate = if occluder[3] > 0.001 && occ_dir.dot(sun) > 0.992 {
                    (occluder[3] / planet_cubes[7][3].max(0.001)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (weather_beam * (1.0 - 0.58 * eclipse_gate)).clamp(0.25, 1.0)
            }
        }
    }

    fn evaluate_illumination<R>(
        &self,
        helio_sink: impl FnOnce(Vec3, (f32, f32), AtmosphericShell) -> R,
        fallback_sink: impl FnOnce() -> R,
    ) -> R {
        match self.mode {
            EphemerisMode::CadenceFallback => fallback_sink(),
            EphemerisMode::HelioActive { planet_cubes, occluder, atmosphere } => {
                let sun = Vec3::new(planet_cubes[7][0], planet_cubes[7][1], planet_cubes[7][2])
                    .normalize_or_zero();
                let occ_dir = Vec3::new(occluder[0], occluder[1], occluder[2]).normalize_or_zero();
                let eclipse_gate = if occluder[3] > 0.001 && occ_dir.dot(sun) > 0.992 {
                    (occluder[3] / planet_cubes[7][3].max(0.001)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                helio_sink(sun, (planet_cubes[7][3], eclipse_gate), atmosphere)
            }
        }
    }

    fn transfer_sky_state<R>(
        &self,
        cam_pos: Vec3,
        sink: impl FnOnce(&[[f32; 4]; 8], [f32; 4], AtmosphericShell, ([f32; 4], [f32; 4])) -> R,
    ) -> R {
        match self.mode {
            EphemerisMode::CadenceFallback => {
                let shell = AtmosphericShell::terrestrial_at(cam_pos);
                Magnetosphere::terrestrial_arctic().transfer_bands(|low, high| {
                    sink(&[[0.0; 4]; 8], [0.0; 4], shell, (low, high))
                })
            }
            EphemerisMode::HelioActive { planet_cubes, occluder, atmosphere } => {
                atmosphere.transfer_shell_quad(|quad| {
                    sink(&planet_cubes, occluder, atmosphere, (quad[2], quad[3]))
                })
            }
        }
    }
}
