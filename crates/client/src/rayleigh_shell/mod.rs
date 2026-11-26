mod catalog;
mod ionosphere;
mod magnetosphere;
mod medium;
mod optics;
mod planets;
mod strata;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub use catalog::{AirCatalog, SolarAirCatalog};
pub use ionosphere::{IonosphereMedium, PolarIonosphere};
pub use magnetosphere::{Magnetosphere, MagnetosphereEnvelope};
pub use medium::{OpticalMedium, VacuumMedium};
#[cfg(test)]
pub use medium::{CubicAirVolume, HorizonHazeMedium, SolarFluxMedium};
pub use optics::CubicRayleighShell;
#[cfg(test)]
pub use optics::{CubicSolarBody, SolarDisc};
#[cfg(test)]
pub use planets::AtlasSkyProjection;
pub use planets::{EphemerisSync, SkyProjection, StormDarkenedProjection};
pub use strata::{BandCurve, PlateauZenith, SmoothBand, ZenithField};
#[cfg(test)]
pub use strata::{
    AirSample, AirStratum, BarometricStratum, CobaltShadowArch, CrimsonDusk, TropopauseRim,
};

use glam::{Vec3, Vec4};
use glow::HasContext;

/// Optical scatter tint and outer Rayleigh shell thickness (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct ShellOptics {
    day_tint: Vec3,
    thickness: f32,
}

impl ShellOptics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(day_tint: Vec3, thickness: f32) -> Self {
        Self { day_tint, thickness }
    }
}

/// Focused contract for a planetary atmosphere envelope ("Interfaces Everywhere" + Tell-Don't-Ask).
pub trait AtmosphericEnvelope: Copy {
    fn airless(&self) -> bool;
    fn magnetized(&self) -> bool;
    fn within_reach(&self, gap: f64) -> bool;
    fn sky_illumination(&self, sun_dir: Vec3) -> (Vec3, Vec4, f32);
    fn sky_illumination_with_flux(&self, sun_dir: Vec3, solar_half_ang: f32) -> (Vec3, Vec4, f32);
    fn darkened(&self, overcast: f32) -> AtmosphericShell;
    fn for_pass_camera(&self, cam_pos: Vec3) -> AtmosphericShell;
    fn bind_gl(&self, gl: &glow::Context, u_geom: &glow::UniformLocation, u_tint: &glow::UniformLocation);
    fn bind_magnetosphere(
        &self,
        gl: &glow::Context,
        u_lower: &glow::UniformLocation,
        u_upper: &glow::UniformLocation,
    );
    fn transfer_shell_quad<R>(&self, sink: impl FnOnce([[f32; 4]; 4]) -> R) -> R;
}

#[derive(Clone, Copy, Debug)]
enum ShellState {
    Vacuum,
    CubicBody {
        rel_cam: Vec3,
        body_half: f32,
        optics: ShellOptics,
        magnetosphere: Magnetosphere,
    },
}

/// Polymorphic planetary atmosphere envelope (`ShellState::Vacuum` vs. penetrable 3D `CubicBody`).
#[derive(Clone, Copy, Debug)]
pub struct AtmosphericShell {
    state: ShellState,
}

impl AtmosphericShell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn from_state(state: ShellState) -> Self {
        Self { state }
    }

    pub const fn vacuum() -> Self {
        Self::from_state(ShellState::Vacuum)
    }

    pub const fn cubic(
        rel_cam: Vec3,
        body_half: f32,
        optics: ShellOptics,
        magnetosphere: Magnetosphere,
    ) -> Self {
        Self::from_state(ShellState::CubicBody { rel_cam, body_half, optics, magnetosphere })
    }

    pub const fn terrestrial_at(cam_pos: Vec3) -> Self {
        Self::cubic(
            cam_pos,
            16.0,
            ShellOptics::new(Vec3::new(0.21, 0.44, 0.80), 12.0),
            Magnetosphere::terrestrial_arctic(),
        )
    }
}

/// Decodes boundary uniform quads from `engine::GraphicPipeline::sync_ephemeris` into `AtmosphericShell`.
pub trait UniformEnvelopeSource {
    fn decode_shell(&self) -> AtmosphericShell;
}

/// Immutable uniform quad wrapper decoding `ShellState` outside constructors (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct SyncedShellPair {
    quad: [[f32; 4]; 4],
}

impl SyncedShellPair {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(quad: [[f32; 4]; 4]) -> Self {
        Self { quad }
    }
}

impl UniformEnvelopeSource for SyncedShellPair {
    fn decode_shell(&self) -> AtmosphericShell {
        let geom = self.quad[0];
        let tint = self.quad[1];
        if tint[3] <= 0.01 || geom[3] <= 0.5 {
            AtmosphericShell::vacuum()
        } else {
            AtmosphericShell::cubic(
                Vec3::new(geom[0], geom[1], geom[2]),
                geom[3],
                ShellOptics::new(Vec3::new(tint[0], tint[1], tint[2]), tint[3]),
                Magnetosphere::from_bands(self.quad[2], self.quad[3]),
            )
        }
    }
}

impl AtmosphericEnvelope for AtmosphericShell {
    fn airless(&self) -> bool {
        matches!(self.state, ShellState::Vacuum)
    }

    fn magnetized(&self) -> bool {
        match self.state {
            ShellState::Vacuum => false,
            ShellState::CubicBody { magnetosphere, .. } => !magnetosphere.unmagnetized(),
        }
    }

    fn within_reach(&self, gap: f64) -> bool {
        match self.state {
            ShellState::Vacuum => false,
            ShellState::CubicBody { body_half, optics, .. } => {
                gap <= f64::from(body_half * 40.0 + optics.thickness * 8.0)
            }
        }
    }

    fn sky_illumination(&self, sun_dir: Vec3) -> (Vec3, Vec4, f32) {
        self.sky_illumination_with_flux(sun_dir, 0.00465)
    }

    fn sky_illumination_with_flux(&self, sun_dir: Vec3, solar_half_ang: f32) -> (Vec3, Vec4, f32) {
        let space_black = Vec3::new(0.008, 0.012, 0.025);
        match self.state {
            ShellState::Vacuum => {
                let (vac_rgb, _) = VacuumMedium::new().scatter_along(Vec3::ZERO, Vec3::Y, sun_dir);
                (Vec3::Y, (space_black + vac_rgb).extend(1.0), 1.0)
            }
            ShellState::CubicBody { rel_cam, body_half, optics, magnetosphere } => {
                let body_up = PlateauZenith::new().zenith_at(rel_cam);
                if (rel_cam.abs().max_element() - body_half) >= optics.thickness {
                    return (body_up, space_black.extend(1.0), 1.0);
                }
                let flux = (solar_half_ang / 0.00465).clamp(0.80, 1.30);
                let (scatter_rgb, scatter_alpha) =
                    CubicRayleighShell::new(body_half, optics.thickness, optics.day_tint, flux)
                        .scatter_along(rel_cam, body_up, sun_dir);
                let sky_lum = scatter_rgb.x.max(scatter_rgb.y).max(scatter_rgb.z);
                let lit_gate = 1.0 - SmoothBand::new(0.35, 0.62).sample(scatter_alpha);
                let night = (1.0 - (sky_lum / 0.36).clamp(0.0, 1.0)) * lit_gate;
                let iono_rgb = PolarIonosphere::planetary(body_half, magnetosphere, night, 0.0)
                    .radiance_along(rel_cam, body_up, sun_dir)
                    * 0.18;
                let zenith_rgb = (space_black * (1.0 - scatter_alpha) + scatter_rgb + iono_rgb)
                    .clamp(Vec3::ZERO, Vec3::ONE);
                (body_up, zenith_rgb.extend(1.0), night)
            }
        }
    }

    fn darkened(&self, overcast: f32) -> AtmosphericShell {
        if overcast <= 0.01 {
            return *self;
        }
        match self.state {
            ShellState::Vacuum => Self::vacuum(),
            ShellState::CubicBody { rel_cam, body_half, optics, magnetosphere } => {
                let storm_slate = Vec3::new(0.11, 0.14, 0.20);
                let k = (overcast * 0.86).clamp(0.0, 0.88);
                let dimmed = ShellOptics::new(optics.day_tint.lerp(storm_slate, k), optics.thickness);
                Self::cubic(rel_cam, body_half, dimmed, magnetosphere)
            }
        }
    }

    fn for_pass_camera(&self, cam_pos: Vec3) -> AtmosphericShell {
        match self.state {
            ShellState::Vacuum => Self::vacuum(),
            ShellState::CubicBody { rel_cam, body_half, optics, magnetosphere } => {
                let earth_limit = body_half * 42.0;
                if rel_cam.length() < earth_limit && cam_pos.length() < earth_limit {
                    Self::cubic(cam_pos, body_half, optics, magnetosphere)
                } else {
                    *self
                }
            }
        }
    }

    fn bind_gl(
        &self,
        gl: &glow::Context,
        u_geom: &glow::UniformLocation,
        u_tint: &glow::UniformLocation,
    ) {
        self.transfer_shell_quad(|q| unsafe {
            gl.uniform_4_f32(Some(u_geom), q[0][0], q[0][1], q[0][2], q[0][3]);
            gl.uniform_4_f32(Some(u_tint), q[1][0], q[1][1], q[1][2], q[1][3]);
        });
    }

    fn bind_magnetosphere(
        &self,
        gl: &glow::Context,
        u_lower: &glow::UniformLocation,
        u_upper: &glow::UniformLocation,
    ) {
        self.transfer_shell_quad(|q| unsafe {
            gl.uniform_4_f32(Some(u_lower), q[2][0], q[2][1], q[2][2], q[2][3]);
            gl.uniform_4_f32(Some(u_upper), q[3][0], q[3][1], q[3][2], q[3][3]);
        });
    }

    fn transfer_shell_quad<R>(&self, sink: impl FnOnce([[f32; 4]; 4]) -> R) -> R {
        match self.state {
            ShellState::Vacuum => sink([[0.0; 4]; 4]),
            ShellState::CubicBody { rel_cam, body_half, optics, magnetosphere } => {
                magnetosphere.transfer_bands(|low, high| {
                    sink([
                        [rel_cam.x, rel_cam.y, rel_cam.z, body_half],
                        [optics.day_tint.x, optics.day_tint.y, optics.day_tint.z, optics.thickness],
                        low,
                        high,
                    ])
                })
            }
        }
    }
}
