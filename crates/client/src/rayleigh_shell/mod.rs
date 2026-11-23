mod catalog;
mod medium;
mod optics;
mod planets;
mod strata;
#[cfg(test)]
mod tests;

pub use medium::{OpticalMedium, VacuumMedium};
#[cfg(test)]
pub use medium::{CubicAirVolume, HorizonHazeMedium, SolarFluxMedium};
pub use optics::CubicRayleighShell;
#[cfg(test)]
pub use optics::{CubicSolarBody, SolarDisc};
pub use planets::{AtlasSkyProjection, EphemerisSync, SkyProjection};
pub use strata::{PlateauZenith, ZenithField};
#[cfg(test)]
pub use strata::{
    AirSample, AirStratum, BarometricStratum, CobaltShadowArch, CrimsonDusk, TropopauseRim,
};

use glam::{Vec3, Vec4};
use glow::HasContext;

/// Focused contract for a planetary atmosphere envelope ("Interfaces Everywhere" + Tell-Don't-Ask).
pub trait AtmosphericEnvelope: Copy {
    fn airless(&self) -> bool;
    fn within_reach(&self, gap: f64) -> bool;
    fn sky_illumination(&self, sun_dir: Vec3) -> (Vec3, Vec4, f32);
    fn sky_illumination_with_flux(&self, sun_dir: Vec3, solar_half_ang: f32) -> (Vec3, Vec4, f32);
    fn for_pass_camera(&self, cam_pos: Vec3) -> AtmosphericShell;
    fn bind_gl(&self, gl: &glow::Context, u_geom: &glow::UniformLocation, u_tint: &glow::UniformLocation);
    fn transfer_uniforms<R>(&self, sink: impl FnOnce([f32; 4], [f32; 4]) -> R) -> R;
}

#[derive(Clone, Copy, Debug)]
enum ShellState {
    Vacuum,
    CubicBody {
        rel_cam: Vec3,
        body_half: f32,
        day_tint: Vec3,
        thickness: f32,
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

    pub const fn cubic(rel_cam: Vec3, body_half: f32, day_tint: Vec3, thickness: f32) -> Self {
        Self::from_state(ShellState::CubicBody { rel_cam, body_half, day_tint, thickness })
    }

    pub const fn terrestrial_at(cam_pos: Vec3) -> Self {
        Self::cubic(cam_pos, 16.0, Vec3::new(0.21, 0.44, 0.80), 12.0)
    }
}

/// Decodes boundary uniform pairs from `engine::GraphicPipeline::sync_ephemeris` into `AtmosphericShell`.
pub trait UniformEnvelopeSource {
    fn decode_shell(&self) -> AtmosphericShell;
}

/// Immutable uniform pair wrapper decoding `ShellState` outside constructors (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct SyncedShellPair {
    rel_cam_and_half: [f32; 4],
    tint_and_thick: [f32; 4],
}

impl SyncedShellPair {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(rel_cam_and_half: [f32; 4], tint_and_thick: [f32; 4]) -> Self {
        Self { rel_cam_and_half, tint_and_thick }
    }
}

impl UniformEnvelopeSource for SyncedShellPair {
    fn decode_shell(&self) -> AtmosphericShell {
        let body_half = self.rel_cam_and_half[3];
        let thickness = self.tint_and_thick[3];
        if thickness <= 0.01 || body_half <= 0.5 {
            AtmosphericShell::vacuum()
        } else {
            AtmosphericShell::cubic(
                Vec3::new(
                    self.rel_cam_and_half[0],
                    self.rel_cam_and_half[1],
                    self.rel_cam_and_half[2],
                ),
                body_half,
                Vec3::new(self.tint_and_thick[0], self.tint_and_thick[1], self.tint_and_thick[2]),
                thickness,
            )
        }
    }
}

impl AtmosphericEnvelope for AtmosphericShell {
    fn airless(&self) -> bool {
        matches!(self.state, ShellState::Vacuum)
    }

    fn within_reach(&self, gap: f64) -> bool {
        match self.state {
            ShellState::Vacuum => false,
            ShellState::CubicBody { body_half, thickness, .. } => {
                gap <= f64::from(body_half * 40.0 + thickness * 8.0)
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
            ShellState::CubicBody { rel_cam, body_half, day_tint, thickness } => {
                let body_up = PlateauZenith::new().zenith_at(rel_cam);
                if (rel_cam.abs().max_element() - body_half) >= thickness {
                    return (body_up, space_black.extend(1.0), 1.0);
                }
                let flux = (solar_half_ang / 0.00465).clamp(0.80, 1.30);
                let (scatter_rgb, scatter_alpha) =
                    CubicRayleighShell::new(body_half, thickness, day_tint, flux)
                        .scatter_along(rel_cam, body_up, sun_dir);
                let zenith_rgb =
                    (space_black * (1.0 - scatter_alpha) + scatter_rgb).clamp(Vec3::ZERO, Vec3::ONE);
                let sky_lum = scatter_rgb.x.max(scatter_rgb.y).max(scatter_rgb.z);
                let day_wash = (sky_lum / 0.36).clamp(0.0, 1.0);
                (body_up, zenith_rgb.extend(1.0), 1.0 - day_wash)
            }
        }
    }

    fn for_pass_camera(&self, cam_pos: Vec3) -> AtmosphericShell {
        match self.state {
            ShellState::Vacuum => Self::vacuum(),
            ShellState::CubicBody { rel_cam, body_half, day_tint, thickness } => {
                let earth_limit = body_half * 42.0;
                if rel_cam.length() < earth_limit && cam_pos.length() < earth_limit {
                    Self::cubic(cam_pos, body_half, day_tint, thickness)
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
        self.transfer_uniforms(|g, t| unsafe {
            gl.uniform_4_f32(Some(u_geom), g[0], g[1], g[2], g[3]);
            gl.uniform_4_f32(Some(u_tint), t[0], t[1], t[2], t[3]);
        });
    }

    fn transfer_uniforms<R>(&self, sink: impl FnOnce([f32; 4], [f32; 4]) -> R) -> R {
        match self.state {
            ShellState::Vacuum => sink([0.0; 4], [0.0; 4]),
            ShellState::CubicBody { rel_cam, body_half, day_tint, thickness } => sink(
                [rel_cam.x, rel_cam.y, rel_cam.z, body_half],
                [day_tint.x, day_tint.y, day_tint.z, thickness],
            ),
        }
    }
}
