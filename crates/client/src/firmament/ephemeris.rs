use glam::Vec3;
use glow::HasContext;
use crate::rayleigh_shell::{
    AtmosphericEnvelope, AtmosphericShell, Magnetosphere, MagnetosphereEnvelope,
};

/// Focused contract for polymorphic planetary ephemeris state (`CadenceFallback` vs. `HelioActive`).
pub trait FirmamentEphemeris: Copy {
    fn for_pass_camera(&self, cam_pos: Vec3) -> VaultEphemeris;
    fn evaluate_illumination<R>(
        &self,
        helio_sink: impl FnOnce(Vec3, (f32, f32), AtmosphericShell) -> R,
        fallback_sink: impl FnOnce() -> R,
    ) -> R;
    fn bind_gl(
        &self,
        gl: &glow::Context,
        u_planets: &glow::UniformLocation,
        u_eclipse: &glow::UniformLocation,
        u_atmo_geom: &glow::UniformLocation,
        u_atmo_tint: &glow::UniformLocation,
    );
    fn bind_magnetosphere(
        &self,
        gl: &glow::Context,
        u_lower: &glow::UniformLocation,
        u_upper: &glow::UniformLocation,
    );
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

/// Polymorphic planetary ephemeris state (`<= 4` fields) without magic-zero sentinels.
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

    fn bind_gl(
        &self,
        gl: &glow::Context,
        u_planets: &glow::UniformLocation,
        u_eclipse: &glow::UniformLocation,
        u_atmo_geom: &glow::UniformLocation,
        u_atmo_tint: &glow::UniformLocation,
    ) {
        let (cubes, occ, shell) = match self.mode {
            EphemerisMode::CadenceFallback => ([[0.0; 4]; 8], [0.0; 4], AtmosphericShell::vacuum()),
            EphemerisMode::HelioActive { planet_cubes, occluder, atmosphere } => {
                (planet_cubes, occluder, atmosphere)
            }
        };
        let mut flat_planets = [0.0_f32; 32];
        for (i, cube) in cubes.iter().enumerate() {
            flat_planets[i * 4..i * 4 + 4].copy_from_slice(cube);
        }
        unsafe {
            gl.uniform_4_f32_slice(Some(u_planets), &flat_planets);
            gl.uniform_4_f32(Some(u_eclipse), occ[0], occ[1], occ[2], occ[3]);
        }
        shell.bind_gl(gl, u_atmo_geom, u_atmo_tint);
    }

    fn bind_magnetosphere(
        &self,
        gl: &glow::Context,
        u_lower: &glow::UniformLocation,
        u_upper: &glow::UniformLocation,
    ) {
        match self.mode {
            EphemerisMode::CadenceFallback => {
                Magnetosphere::terrestrial_arctic().transfer_bands(|low, high| unsafe {
                    gl.uniform_4_f32(Some(u_lower), low[0], low[1], low[2], low[3]);
                    gl.uniform_4_f32(Some(u_upper), high[0], high[1], high[2], high[3]);
                });
            }
            EphemerisMode::HelioActive { atmosphere, .. } => {
                atmosphere.bind_magnetosphere(gl, u_lower, u_upper);
            }
        }
    }
}
