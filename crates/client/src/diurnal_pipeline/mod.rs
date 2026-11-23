mod epoch;
mod quad;

pub use epoch::{DiurnalEpoch, MeasuredEpoch};
pub use quad::{FirmamentQuadSource, QuadUpload};

use std::sync::Mutex;
use glam::{Mat4, Quat, Vec4};

use engine::{GraphicPipeline, Mesh, ShaderProgram};
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::AuroraCanopy;
use crate::celestial::CelestialCadence;
use crate::clock::{MonotonicTime, SystemClock};
use crate::firmament::{FirmamentEphemeris, SkyFirmament, VaultEphemeris, VaultLuminance};
use crate::gl_firmament::{FirmamentProgramSource, FirmamentShaderSource};
use crate::rayleigh_shell::{SyncedShellPair, UniformEnvelopeSource};

trait EphemerisPacketSource {
    fn decode_ephemeris(&self) -> VaultEphemeris;
}

struct SyncedEphemerisPacket {
    planet_cubes: [[f32; 4]; 8],
    occluder: [f32; 4],
    shell_pair: SyncedShellPair,
}

impl SyncedEphemerisPacket {
    const fn new(
        planet_cubes: [[f32; 4]; 8],
        occluder: [f32; 4],
        shell_pair: SyncedShellPair,
    ) -> Self {
        Self { planet_cubes, occluder, shell_pair }
    }
}

impl EphemerisPacketSource for SyncedEphemerisPacket {
    fn decode_ephemeris(&self) -> VaultEphemeris {
        if self.planet_cubes[7][3] > 0.0001 {
            VaultEphemeris::active(
                self.planet_cubes,
                self.occluder,
                self.shell_pair.decode_shell(),
            )
        } else {
            VaultEphemeris::dormant()
        }
    }
}

/// Graphic pipeline driving planetary atmosphere sky, world-space solar lighting, and firmament display.
pub struct DiurnalPipeline {
    epoch: Box<dyn DiurnalEpoch>,
    sky: Box<dyn SkyFirmament>,
    sky_mesh: Box<dyn Mesh>,
    ephemeris_state: Mutex<VaultEphemeris>,
}

impl DiurnalPipeline {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        epoch: Box<dyn DiurnalEpoch>,
        sky: Box<dyn SkyFirmament>,
        sky_mesh: Box<dyn Mesh>,
        ephemeris_state: Mutex<VaultEphemeris>,
    ) -> Self {
        Self { epoch, sky, sky_mesh, ephemeris_state }
    }
}

impl GraphicPipeline for DiurnalPipeline {
    fn sync_ephemeris(
        &self,
        planet_cubes: &[[f32; 4]; 8],
        eclipse_occluder: [f32; 4],
        (atmo_geom, atmo_tint): ([f32; 4], [f32; 4]),
    ) {
        let mut guard = self.ephemeris_state.lock().expect("VaultEphemeris lock poisoned");
        let packet = SyncedEphemerisPacket::new(
            *planet_cubes,
            eclipse_occluder,
            SyncedShellPair::new(atmo_geom, atmo_tint),
        );
        *guard = packet.decode_ephemeris();
    }

    fn render(
        &self,
        gl: &glow::Context,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        view_proj: &Mat4,
        _surface_rot: Quat,
    ) {
        let inv_view_proj = Mat4::from_cols(
            view_proj.x_axis.with_z(0.0),
            view_proj.y_axis.with_z(0.0),
            view_proj.z_axis.with_z(0.0),
            view_proj.w_axis.with_z(1.0),
        )
        .inverse();
        let cam_pos = (inv_view_proj * Vec4::new(0.0, 0.0, 1.0, 0.0)).truncate();
        let pass_ephemeris = self
            .ephemeris_state
            .lock()
            .expect("VaultEphemeris lock poisoned")
            .for_pass_camera(cam_pos);

        let (light_direction, vault_view, vault_glow) =
            self.epoch.evaluate_pass(inv_view_proj, cam_pos, &pass_ephemeris);
        vault_glow.clear_backdrop(gl);

        self.sky.draw_firmament(
            gl,
            &vault_view,
            &vault_glow,
            &pass_ephemeris,
            self.sky_mesh.as_ref(),
        );

        shader.bind(gl, view_proj, light_direction);
        mesh.draw_posed(gl, shader);
    }
}

/// Focused contract assembling a live OpenGL `DiurnalPipeline` outside constructors.
pub trait DiurnalStageAssembly {
    /// # Safety
    /// Requires a valid, current OpenGL context.
    unsafe fn assemble_gl(self, gl: &glow::Context) -> Box<dyn GraphicPipeline>;
}

/// Declarative specification of celestial cadence, aurora cadence, and aurora canopy (`<= 4` fields).
pub struct GlDiurnalStage {
    cadence: Box<dyn CelestialCadence>,
    aurora: Box<dyn AuroraCadence>,
    canopy: Box<dyn AuroraCanopy>,
}

impl GlDiurnalStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        cadence: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        canopy: Box<dyn AuroraCanopy>,
    ) -> Self {
        Self { cadence, aurora, canopy }
    }
}

impl DiurnalStageAssembly for GlDiurnalStage {
    unsafe fn assemble_gl(self, gl: &glow::Context) -> Box<dyn GraphicPipeline> {
        let clock = Box::new(SystemClock::new());
        let start = clock.current_instant();
        let epoch = Box::new(MeasuredEpoch::new(self.cadence, self.aurora, clock, start));
        let sky = FirmamentShaderSource::embedded().compile(self.canopy, gl);
        let sky_mesh = FirmamentQuadSource::new().upload(gl);
        Box::new(DiurnalPipeline::new(epoch, sky, sky_mesh, Mutex::new(VaultEphemeris::dormant())))
    }
}
