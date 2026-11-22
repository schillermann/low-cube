use std::sync::Mutex;
use std::time::Instant;
use glam::{Mat4, Quat, Vec3, Vec4};
use glow::HasContext;
use world::{ApexSpin, PlanetarySpin};

use engine::{GraphicPipeline, Mesh, ShaderProgram};
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::AuroraCanopy;
use crate::celestial::CelestialCadence;
use crate::clock::{MonotonicTime, SystemClock};
use crate::firmament::{GlFirmamentQuad, SkyFirmament};
#[cfg(test)]
use crate::aurora_cadence::NightlyAuroraCadence;
#[cfg(test)]
use crate::firmament::{VoidFirmament, VoidMesh};
use crate::gl_firmament::GlFirmament;
use crate::rayleigh_shell::AtmosphericShell;

type EphemerisUniformState = ([[f32; 4]; 8], [f32; 4], ([f32; 4], [f32; 4]));

struct CelestialEpoch {
    cadence: Box<dyn CelestialCadence>,
    aurora: Box<dyn AuroraCadence>,
    clock: Box<dyn MonotonicTime>,
    epoch: Instant,
}

/// Graphic pipeline driving planetary atmosphere sky, world-space solar lighting, and firmament display.
pub struct DiurnalPipeline {
    celestial_epoch: CelestialEpoch,
    sky: Box<dyn SkyFirmament>,
    sky_mesh: Box<dyn Mesh>,
    ephemeris_state: Mutex<EphemerisUniformState>,
}

impl DiurnalPipeline {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn with_ephemeris(
        celestial_epoch: CelestialEpoch,
        sky: Box<dyn SkyFirmament>,
        sky_mesh: Box<dyn Mesh>,
        ephemeris_state: Mutex<EphemerisUniformState>,
    ) -> Self {
        Self { celestial_epoch, sky, sky_mesh, ephemeris_state }
    }

    pub fn new(
        cadence: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        clock: Box<dyn MonotonicTime>,
        epoch: Instant,
        sky: Box<dyn SkyFirmament>,
        sky_mesh: Box<dyn Mesh>,
    ) -> Self {
        Self::with_ephemeris(
            CelestialEpoch { cadence, aurora, clock, epoch },
            sky,
            sky_mesh,
            Mutex::new(([[0.0; 4]; 8], [0.0; 4], ([0.0; 4], [0.0; 4]))),
        )
    }

    /// Secondary constructor for headless operation and deterministic testing.
    #[cfg(test)]
    pub fn headless(
        cadence: Box<dyn CelestialCadence>,
        clock: Box<dyn MonotonicTime>,
        epoch: Instant,
    ) -> Self {
        Self::new(
            cadence,
            Box::new(NightlyAuroraCadence::new()),
            clock,
            epoch,
            Box::new(VoidFirmament::new()),
            Box::new(VoidMesh::new()),
        )
    }

    /// Secondary constructor compiling active firmament shader and uploading quad geometry.
    pub unsafe fn gl(
        cadence: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        canopy: Box<dyn AuroraCanopy>,
        gl: &glow::Context,
    ) -> Self {
        let clock = Box::new(SystemClock::new());
        let epoch = clock.current_instant();
        let sky = Box::new(GlFirmament::compiled(canopy, gl));
        let sky_mesh = GlFirmamentQuad::uploaded(gl);
        Self::new(cadence, aurora, clock, epoch, sky, sky_mesh)
    }
}

impl GraphicPipeline for DiurnalPipeline {
    fn sync_ephemeris(
        &self,
        planet_cubes: &[[f32; 4]; 8],
        eclipse_occluder: [f32; 4],
        atmo_shell: ([f32; 4], [f32; 4]),
    ) {
        if let Ok(mut guard) = self.ephemeris_state.lock() {
            *guard = (*planet_cubes, eclipse_occluder, atmo_shell);
        }
    }

    fn render(
        &self,
        gl: &glow::Context,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        view_proj: &Mat4,
        _surface_rot: Quat,
    ) {
        let now = self.celestial_epoch.clock.current_instant();
        let elapsed = now.duration_since(self.celestial_epoch.epoch).as_secs_f64();
        let progress = self.celestial_epoch.cadence.day_progress(elapsed);
        let (planets, occluder, (atmo_geom, atmo_tint)) = self
            .ephemeris_state
            .lock()
            .map(|g| *g)
            .unwrap_or(([[0.0; 4]; 8], [0.0; 4], ([0.0; 4], [0.0; 4])));

        let inv_view_proj = Mat4::from_cols(
            view_proj.x_axis.with_z(0.0),
            view_proj.y_axis.with_z(0.0),
            view_proj.z_axis.with_z(0.0),
            view_proj.w_axis.with_z(1.0),
        )
        .inverse();
        let cam_pos = (inv_view_proj * Vec4::new(0.0, 0.0, 1.0, 0.0)).truncate();

        let (light_direction, celestial_pole, local_up, clear_color, night_factor, local_progress) =
            if planets[7][3] > 0.0001 {
                let helio_sun = Vec3::new(planets[7][0], planets[7][1], planets[7][2]).normalize_or_zero();
                let (body_up, clear, night) = AtmosphericShell::new(atmo_geom, atmo_tint)
                    .sky_illumination_with_flux(helio_sun, planets[7][3]);
                let (pole, hour_prog) =
                    ApexSpin::terrestrial().diurnal_meridian(helio_sun, body_up, progress);
                (helio_sun, pole, body_up, clear, night, hour_prog)
            } else {
                let world_sun = self.celestial_epoch.cadence.sun_direction(progress);
                let (earth_up, clear, night) =
                    AtmosphericShell::terrestrial_at(cam_pos).sky_illumination(world_sun);
                (world_sun, Vec3::Z, earth_up, clear, night, progress)
            };
        let aurora_factor = self.celestial_epoch.aurora.activity(local_progress, elapsed, night_factor);

        unsafe {
            gl.clear_color(clear_color.x, clear_color.y, clear_color.z, clear_color.w);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        self.sky.draw_firmament(
            gl,
            &inv_view_proj,
            light_direction,
            celestial_pole,
            local_up,
            clear_color,
            night_factor,
            aurora_factor,
            elapsed as f32,
            (&planets, occluder, (atmo_geom, atmo_tint)),
            self.sky_mesh.as_ref(),
        );

        shader.bind(gl, view_proj, light_direction);
        mesh.draw_posed(gl, shader);
    }
}
