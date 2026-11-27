mod ephemeris;
mod lut_sky;

pub use self::ephemeris::{FirmamentEphemeris, VaultEphemeris};
pub use self::lut_sky::LutSkyFirmament;

use glam::{Mat4, Vec3, Vec4};
use engine::FramePass;

/// Camera projection and solar orientation frame for a celestial vault draw pass (`3` private fields).
#[derive(Clone, Copy, Debug)]
pub struct VaultView {
    inv_view_proj: Mat4,
    sun_dir: Vec3,
    celestial_pole: Vec3,
}

impl VaultView {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inv_view_proj: Mat4, sun_dir: Vec3, celestial_pole: Vec3) -> Self {
        Self { inv_view_proj, sun_dir, celestial_pole }
    }

    pub const fn components(self) -> (Mat4, Vec3, Vec3) {
        (self.inv_view_proj, self.sun_dir, self.celestial_pole)
    }
}

/// Focused contract clearing the framebuffer backdrop (`Tell, Don't Ask`).
pub trait VaultLuminance: Copy {
    fn clear_backdrop(&self, pass: &mut dyn FramePass);
}

/// Backdrop clear color, aurora activity, and monotonic phase for a celestial vault pass (`3` private fields).
#[derive(Clone, Copy, Debug)]
pub struct VaultGlow {
    sky_color: Vec4,
    aurora_factor: f32,
    elapsed_secs: f32,
}

impl VaultGlow {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sky_color: Vec4, aurora_factor: f32, elapsed_secs: f32) -> Self {
        Self { sky_color, aurora_factor, elapsed_secs }
    }

    pub const fn components(self) -> (Vec4, f32, f32) {
        (self.sky_color, self.aurora_factor, self.elapsed_secs)
    }
}

impl VaultLuminance for VaultGlow {
    fn clear_backdrop(&self, pass: &mut dyn FramePass) {
        pass.clear_sky(self.sky_color);
    }
}

/// Focused contract for rendering the celestial sky vault via `128×64` `2 Hz` `AtmosphereSkyLut`.
pub trait SkyFirmament: Send + Sync {
    fn draw_firmament(
        &self,
        pass: &mut dyn FramePass,
        cam_pos: Vec3,
        view: &VaultView,
        glow_state: &VaultGlow,
        ephemeris: &VaultEphemeris,
    );
}

/// Null-object firmament performing no GPU operations (`0` fields).
#[cfg(test)]
pub struct VoidFirmament;

#[cfg(test)]
impl VoidFirmament {
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
impl SkyFirmament for VoidFirmament {
    fn draw_firmament(
        &self,
        _pass: &mut dyn FramePass,
        _cam_pos: Vec3,
        _view: &VaultView,
        _glow_state: &VaultGlow,
        _ephemeris: &VaultEphemeris,
    ) {
    }
}

/// Null-object mesh performing no draw operations (`0` fields).
#[cfg(test)]
pub struct VoidMesh;

#[cfg(test)]
impl VoidMesh {
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
impl engine::Mesh for VoidMesh {
    fn draw(&self, _pass: &mut dyn FramePass) {}
}
