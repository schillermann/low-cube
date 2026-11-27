mod epoch;

pub use epoch::{DiurnalEpoch, MeasuredEpoch, OvercastCanopyEpoch};

use std::sync::Mutex;
use glam::{Mat4, Vec4};

use engine::{FramePass, GraphicPipeline, Mesh};
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::AuroraCanopy;
use crate::celestial::CelestialCadence;
use crate::clock::{MonotonicTime, SystemClock};
use crate::firmament::{
    FirmamentEphemeris, LutSkyFirmament, SkyFirmament, VaultEphemeris, VaultLuminance,
};
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

/// `wgpu` graphic pipeline driving `128×64` `2 Hz` Atmosphere Sky-LUT, `1024²` Sun Shadow, and `8-Byte` `PackedQuad`s (`3` fields).
pub struct DiurnalPipeline {
    epoch: Box<dyn DiurnalEpoch>,
    sky: Box<dyn SkyFirmament>,
    ephemeris_state: Mutex<VaultEphemeris>,
}

impl DiurnalPipeline {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        epoch: Box<dyn DiurnalEpoch>,
        sky: Box<dyn SkyFirmament>,
        ephemeris_state: Mutex<VaultEphemeris>,
    ) -> Self {
        Self { epoch, sky, ephemeris_state }
    }
}

impl GraphicPipeline for DiurnalPipeline {
    fn sync_ephemeris(
        &self,
        planet_cubes: &[[f32; 4]; 8],
        eclipse_occluder: [f32; 4],
        atmo_shell: [[f32; 4]; 4],
    ) {
        let mut guard = self.ephemeris_state.lock().expect("VaultEphemeris lock poisoned");
        let packet = SyncedEphemerisPacket::new(
            *planet_cubes,
            eclipse_occluder,
            SyncedShellPair::new(atmo_shell),
        );
        *guard = packet.decode_ephemeris();
    }

    fn render(&self, pass: &mut dyn FramePass, mesh: &dyn Mesh, view_proj: &Mat4) {
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
        vault_glow.clear_backdrop(pass);
        self.sky.draw_firmament(pass, cam_pos, &vault_view, &vault_glow, &pass_ephemeris);

        pass.bind_camera(*view_proj, light_direction);
        mesh.draw(pass);
    }
}

/// Focused contract assembling a `DiurnalPipeline` outside constructors.
pub trait DiurnalStageAssembly {
    fn assemble_stage(self) -> Box<dyn GraphicPipeline>;
}

/// Declarative specification of celestial cadence, aurora cadence, and aurora canopy (`3` private fields).
pub struct WgpuDiurnalStage {
    cadence: Box<dyn CelestialCadence>,
    aurora: Box<dyn AuroraCadence>,
    canopy: Box<dyn AuroraCanopy>,
}

impl WgpuDiurnalStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        cadence: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        canopy: Box<dyn AuroraCanopy>,
    ) -> Self {
        Self { cadence, aurora, canopy }
    }
}

impl DiurnalStageAssembly for WgpuDiurnalStage {
    fn assemble_stage(self) -> Box<dyn GraphicPipeline> {
        let clock = Box::new(SystemClock::new());
        let start = clock.current_instant();
        let epoch = Box::new(OvercastCanopyEpoch::new(MeasuredEpoch::new(
            self.cadence,
            self.aurora,
            clock,
            start,
        )));
        let sky = Box::new(LutSkyFirmament::with_canopy(self.canopy));
        Box::new(DiurnalPipeline::new(epoch, sky, Mutex::new(VaultEphemeris::dormant())))
    }
}
