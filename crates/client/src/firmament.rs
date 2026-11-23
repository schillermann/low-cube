use glam::{Mat4, Vec3, Vec4};
use glow::HasContext;
use engine::Mesh;
use crate::rayleigh_shell::{AtmosphericEnvelope, AtmosphericShell};

/// Focused contract binding camera projection and solar frame uniforms to OpenGL.
pub trait VaultFrame: Copy {
    fn bind_gl(
        &self,
        gl: &glow::Context,
        u_inv_vp: &glow::UniformLocation,
        u_sun: &glow::UniformLocation,
        u_pole: &glow::UniformLocation,
        u_up: &glow::UniformLocation,
    );
}

/// Camera projection and solar orientation frame for a celestial vault draw pass (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct VaultView {
    inv_view_proj: Mat4,
    sun_dir: Vec3,
    celestial_pole: Vec3,
    local_up: Vec3,
}

impl VaultView {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        inv_view_proj: Mat4,
        sun_dir: Vec3,
        celestial_pole: Vec3,
        local_up: Vec3,
    ) -> Self {
        Self { inv_view_proj, sun_dir, celestial_pole, local_up }
    }
}

impl VaultFrame for VaultView {
    fn bind_gl(
        &self,
        gl: &glow::Context,
        u_inv_vp: &glow::UniformLocation,
        u_sun: &glow::UniformLocation,
        u_pole: &glow::UniformLocation,
        u_up: &glow::UniformLocation,
    ) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(u_inv_vp), false, &self.inv_view_proj.to_cols_array());
            gl.uniform_3_f32(Some(u_sun), self.sun_dir.x, self.sun_dir.y, self.sun_dir.z);
            gl.uniform_3_f32(Some(u_pole), self.celestial_pole.x, self.celestial_pole.y, self.celestial_pole.z);
            gl.uniform_3_f32(Some(u_up), self.local_up.x, self.local_up.y, self.local_up.z);
        }
    }
}

/// Focused contract clearing the framebuffer and binding sky luminance, night wash, aurora, and time uniforms.
pub trait VaultLuminance: Copy {
    fn clear_backdrop(&self, gl: &glow::Context);
    fn bind_gl(
        &self,
        gl: &glow::Context,
        u_sky: &glow::UniformLocation,
        u_night: &glow::UniformLocation,
        u_aurora: &glow::UniformLocation,
        u_time: &glow::UniformLocation,
    );
}

/// Radiance, night-wash, aurora activity, and monotonic phase for a celestial vault pass (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct VaultGlow {
    sky_color: Vec4,
    night_factor: f32,
    aurora_factor: f32,
    elapsed_secs: f32,
}

impl VaultGlow {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        sky_color: Vec4,
        night_factor: f32,
        aurora_factor: f32,
        elapsed_secs: f32,
    ) -> Self {
        Self { sky_color, night_factor, aurora_factor, elapsed_secs }
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
        u_sky: &glow::UniformLocation,
        u_night: &glow::UniformLocation,
        u_aurora: &glow::UniformLocation,
        u_time: &glow::UniformLocation,
    ) {
        unsafe {
            gl.uniform_4_f32(Some(u_sky), self.sky_color.x, self.sky_color.y, self.sky_color.z, self.sky_color.w);
            gl.uniform_1_f32(Some(u_night), self.night_factor);
            gl.uniform_1_f32(Some(u_aurora), self.aurora_factor);
            gl.uniform_1_f32(Some(u_time), self.elapsed_secs);
        }
    }
}

/// Focused contract for polymorphic planetary ephemeris state (`CadenceFallback` vs. `HelioActive`).
pub trait FirmamentEphemeris: Copy {
    fn for_pass_camera(&self, cam_pos: Vec3) -> VaultEphemeris;
    fn evaluate_illumination<R>(
        &self,
        helio_sink: impl FnOnce(Vec3, f32, AtmosphericShell) -> R,
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
        helio_sink: impl FnOnce(Vec3, f32, AtmosphericShell) -> R,
        fallback_sink: impl FnOnce() -> R,
    ) -> R {
        match self.mode {
            EphemerisMode::CadenceFallback => fallback_sink(),
            EphemerisMode::HelioActive { planet_cubes, atmosphere, .. } => {
                let sun = Vec3::new(planet_cubes[7][0], planet_cubes[7][1], planet_cubes[7][2])
                    .normalize_or_zero();
                helio_sink(sun, planet_cubes[7][3], atmosphere)
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
    pub const fn new() -> Self {
        Self
    }
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
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
impl Mesh for VoidMesh {
    fn draw(&self, _gl: &glow::Context) {}
}
