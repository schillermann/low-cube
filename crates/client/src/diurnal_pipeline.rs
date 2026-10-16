use std::time::Instant;
use glam::{Mat4, Quat, Vec3};
use glow::HasContext;

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

/// Graphic pipeline dynamically driving terrain lighting and celestial firmament display.
pub struct DiurnalPipeline {
    cadence: Box<dyn CelestialCadence>,
    aurora: Box<dyn AuroraCadence>,
    clock: Box<dyn MonotonicTime>,
    epoch: Instant,
    sky: Box<dyn SkyFirmament>,
    sky_mesh: Box<dyn Mesh>,
}

impl DiurnalPipeline {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        cadence: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        clock: Box<dyn MonotonicTime>,
        epoch: Instant,
        sky: Box<dyn SkyFirmament>,
        sky_mesh: Box<dyn Mesh>,
    ) -> Self {
        Self {
            cadence,
            aurora,
            clock,
            epoch,
            sky,
            sky_mesh,
        }
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
    fn render(
        &self,
        gl: &glow::Context,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        view_proj: &Mat4,
        surface_rot: Quat,
    ) {
        let now = self.clock.current_instant();
        let elapsed = now.duration_since(self.epoch).as_secs_f64();
        let progress = self.cadence.day_progress(elapsed);
        let clear_color = self.cadence.sky_color(progress);
        let local_sun = self.cadence.sun_direction(progress);

        // Align sun trajectory and horizon with current planetary face orientation
        let light_direction = surface_rot * local_sun;
        let local_up = surface_rot * Vec3::Y;
        let celestial_pole = surface_rot * Vec3::Z;

        let night_factor = (-local_sun.y / 0.15).clamp(0.0, 1.0);
        let aurora_factor = self.aurora.activity(progress, elapsed, night_factor);

        unsafe {
            gl.clear_color(
                clear_color.x,
                clear_color.y,
                clear_color.z,
                clear_color.w,
            );
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        shader.bind(gl, view_proj, light_direction);
        mesh.draw(gl);

        let inv_view_proj = view_proj.inverse();
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
            self.sky_mesh.as_ref(),
        );
    }
}
