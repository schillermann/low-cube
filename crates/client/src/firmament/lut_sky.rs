use std::sync::Mutex;
use glam::Vec3;
use engine::{AtmosphereSkyLut, FirmamentUniform, FramePass, SkyLutMedium};
use crate::aurora_canopy::AuroraCanopy;
use crate::rayleigh_shell::{AtmosphericEnvelope, AtmosphericShell, PlateauZenith, ZenithField};
use super::{FirmamentEphemeris, SkyFirmament, VaultEphemeris, VaultGlow, VaultView};

struct ShellLutProbe {
    shell: AtmosphericShell,
    sun_dir: Vec3,
}

impl SkyLutMedium for ShellLutProbe {
    fn sample_sky_ray(&self, ray_dir: Vec3) -> [f32; 4] {
        self.shell.sample_lut_ray(ray_dir, self.sun_dir)
    }
}

/// `wgpu` celestial firmament baking a `128 × 64` `2 Hz` `AtmosphereSkyLut` (`2` private fields).
pub struct LutSkyFirmament {
    canopy: Box<dyn AuroraCanopy>,
    lut: Mutex<AtmosphereSkyLut>,
}

impl LutSkyFirmament {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(canopy: Box<dyn AuroraCanopy>, lut: Mutex<AtmosphereSkyLut>) -> Self {
        Self { canopy, lut }
    }

    pub fn with_canopy(canopy: Box<dyn AuroraCanopy>) -> Self {
        Self::new(canopy, Mutex::new(AtmosphereSkyLut::cold()))
    }
}

impl SkyFirmament for LutSkyFirmament {
    fn draw_firmament(
        &self,
        pass: &mut dyn FramePass,
        cam_pos: Vec3,
        view: &VaultView,
        glow_state: &VaultGlow,
        ephemeris: &VaultEphemeris,
    ) {
        let [face_half, aurora_rad, ceiling_alt] = self.canopy.canopy_bounds();
        let (inv_vp, sun_dir, pole) = view.components();
        let (_, aurora_factor, elapsed_secs) = glow_state.components();

        ephemeris.transfer_sky_state(cam_pos, |planet_cubes, occluder, shell, (low_band, high_band)| {
            let up = if shell.airless() {
                Vec3::Y
            } else {
                PlateauZenith::new().zenith_at(cam_pos)
            };
            let sig = shell.transfer_shell_quad(|q| [
                q[0][3].to_bits(),
                q[1][0].to_bits() ^ q[1][3].to_bits(),
                (sun_dir.x * 64.0) as i32 as u32,
                (sun_dir.y * 64.0) as i32 as u32,
            ]);
            let probe = ShellLutProbe { shell, sun_dir };
            if let Ok(mut guard) = self.lut.lock() {
                if guard.refresh_at_2hz(elapsed_secs, sig, up, &probe) {
                    pass.upload_sky_lut(guard.texel_bytes());
                }
            }
            let uniform = FirmamentUniform::new(
                inv_vp.to_cols_array_2d(),
                [
                    [sun_dir.x, sun_dir.y, sun_dir.z, aurora_factor],
                    [pole.x, pole.y, pole.z, ceiling_alt],
                    [cam_pos.x, cam_pos.y, cam_pos.z, face_half],
                    [up.x, up.y, up.z, aurora_rad],
                    occluder,
                    low_band,
                    high_band,
                ],
                *planet_cubes,
            );
            pass.draw_sky_lut(uniform);
        });
    }
}
