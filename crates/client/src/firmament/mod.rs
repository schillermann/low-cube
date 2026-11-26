mod ephemeris;

pub use self::ephemeris::{FirmamentEphemeris, VaultEphemeris};

use glam::{Mat4, Vec3, Vec4};
use glow::HasContext;
use engine::Mesh;

/// Focused contract binding camera projection and solar frame uniforms to OpenGL.
pub trait VaultFrame: Copy {
    fn bind_gl(
        &self,
        gl: &glow::Context,
        u_inv_vp: &glow::UniformLocation,
        u_sun: &glow::UniformLocation,
        u_pole: &glow::UniformLocation,
    );
}

/// Camera projection and solar orientation frame for a celestial vault draw pass (`<= 4` fields).
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
}

impl VaultFrame for VaultView {
    fn bind_gl(
        &self,
        gl: &glow::Context,
        u_inv_vp: &glow::UniformLocation,
        u_sun: &glow::UniformLocation,
        u_pole: &glow::UniformLocation,
    ) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(u_inv_vp), false, &self.inv_view_proj.to_cols_array());
            gl.uniform_3_f32(Some(u_sun), self.sun_dir.x, self.sun_dir.y, self.sun_dir.z);
            gl.uniform_3_f32(Some(u_pole), self.celestial_pole.x, self.celestial_pole.y, self.celestial_pole.z);
        }
    }
}

/// Focused contract clearing the framebuffer and binding aurora activity and monotonic phase uniforms.
pub trait VaultLuminance: Copy {
    fn clear_backdrop(&self, gl: &glow::Context);
    fn bind_gl(
        &self,
        gl: &glow::Context,
        u_aurora: &glow::UniformLocation,
        u_time: &glow::UniformLocation,
    );
}

/// Backdrop clear color, aurora activity, and monotonic phase for a celestial vault pass (`<= 4` fields).
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
}

impl VaultLuminance for VaultGlow {
    fn clear_backdrop(&self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(self.sky_color.x, self.sky_color.y, self.sky_color.z, self.sky_color.w);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }

    fn bind_gl(
        &self,
        gl: &glow::Context,
        u_aurora: &glow::UniformLocation,
        u_time: &glow::UniformLocation,
    ) {
        unsafe {
            gl.uniform_1_f32(Some(u_aurora), self.aurora_factor);
            gl.uniform_1_f32(Some(u_time), self.elapsed_secs);
        }
    }
}

/// Focused contract for rendering the celestial sky vault without wide parameter lists.
pub trait SkyFirmament: Send + Sync {
    fn draw_firmament(
        &self,
        gl: &glow::Context,
        view: &VaultView,
        glow_state: &VaultGlow,
        ephemeris: &VaultEphemeris,
        mesh: &dyn Mesh,
    );
}

/// Null-object firmament performing no GPU operations.
#[cfg(test)]
pub struct VoidFirmament;

#[cfg(test)]
impl VoidFirmament {
    pub const fn new() -> Self { Self }
}

#[cfg(test)]
impl SkyFirmament for VoidFirmament {
    fn draw_firmament(
        &self,
        _gl: &glow::Context,
        _view: &VaultView,
        _glow_state: &VaultGlow,
        _ephemeris: &VaultEphemeris,
        _mesh: &dyn Mesh,
    ) {}
}

/// Null-object mesh performing no draw operations.
#[cfg(test)]
pub struct VoidMesh;

#[cfg(test)]
impl VoidMesh {
    pub const fn new() -> Self { Self }
}

#[cfg(test)]
impl Mesh for VoidMesh {
    fn draw(&self, _gl: &glow::Context) {}
}
