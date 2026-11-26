use crate::aurora_canopy::AuroraCanopy;
use crate::firmament::{
    FirmamentEphemeris, VaultEphemeris, VaultFrame, VaultGlow, VaultLuminance, VaultView,
};

pub(super) trait ViewUniformGroup {
    fn bind_view(&self, gl: &glow::Context, view: &VaultView);
}

pub(super) struct OrientationUniforms {
    u_inv_view_proj: glow::UniformLocation,
    u_sun_dir: glow::UniformLocation,
    u_celestial_pole: glow::UniformLocation,
}

impl OrientationUniforms {
    pub(super) const fn new(
        u_inv_view_proj: glow::UniformLocation,
        u_sun_dir: glow::UniformLocation,
        u_celestial_pole: glow::UniformLocation,
    ) -> Self {
        Self { u_inv_view_proj, u_sun_dir, u_celestial_pole }
    }
}

impl ViewUniformGroup for OrientationUniforms {
    fn bind_view(&self, gl: &glow::Context, view: &VaultView) {
        view.bind_gl(gl, &self.u_inv_view_proj, &self.u_sun_dir, &self.u_celestial_pole);
    }
}

pub(super) trait GlowUniformGroup {
    fn bind_glow(&self, gl: &glow::Context, glow_state: &VaultGlow);
}

pub(super) struct AtmosphereUniforms {
    u_aurora_factor: glow::UniformLocation,
    u_time: glow::UniformLocation,
}

impl AtmosphereUniforms {
    pub(super) const fn new(
        u_aurora_factor: glow::UniformLocation,
        u_time: glow::UniformLocation,
    ) -> Self {
        Self { u_aurora_factor, u_time }
    }
}

impl GlowUniformGroup for AtmosphereUniforms {
    fn bind_glow(&self, gl: &glow::Context, glow_state: &VaultGlow) {
        glow_state.bind_gl(gl, &self.u_aurora_factor, &self.u_time);
    }
}

pub(super) struct CanopyUniforms {
    u_face_half: glow::UniformLocation,
    u_aurora_radius: glow::UniformLocation,
    u_ceiling_altitude: glow::UniformLocation,
    canopy: Box<dyn AuroraCanopy>,
}

impl CanopyUniforms {
    pub(super) fn new(
        u_face_half: glow::UniformLocation,
        u_aurora_radius: glow::UniformLocation,
        u_ceiling_altitude: glow::UniformLocation,
        canopy: Box<dyn AuroraCanopy>,
    ) -> Self {
        Self { u_face_half, u_aurora_radius, u_ceiling_altitude, canopy }
    }

    fn bind_canopy(&self, gl: &glow::Context) {
        self.canopy.bind_canopy(
            gl,
            &self.u_face_half,
            &self.u_aurora_radius,
            &self.u_ceiling_altitude,
        );
    }
}

pub(super) struct EphemerisUniforms {
    u_planets: glow::UniformLocation,
    u_eclipse: glow::UniformLocation,
    u_atmo_geom: glow::UniformLocation,
    u_atmo_tint: glow::UniformLocation,
}

impl EphemerisUniforms {
    pub(super) const fn new(
        u_planets: glow::UniformLocation,
        u_eclipse: glow::UniformLocation,
        u_atmo_geom: glow::UniformLocation,
        u_atmo_tint: glow::UniformLocation,
    ) -> Self {
        Self { u_planets, u_eclipse, u_atmo_geom, u_atmo_tint }
    }

    fn bind_ephemeris(&self, gl: &glow::Context, ephemeris: &VaultEphemeris) {
        ephemeris.bind_gl(
            gl,
            &self.u_planets,
            &self.u_eclipse,
            &self.u_atmo_geom,
            &self.u_atmo_tint,
        );
    }
}

pub(super) struct MagnetosphereUniforms {
    u_aurora_lower: glow::UniformLocation,
    u_aurora_upper: glow::UniformLocation,
}

impl MagnetosphereUniforms {
    pub(super) const fn new(
        u_aurora_lower: glow::UniformLocation,
        u_aurora_upper: glow::UniformLocation,
    ) -> Self {
        Self { u_aurora_lower, u_aurora_upper }
    }

    fn bind_magnetosphere(&self, gl: &glow::Context, ephemeris: &VaultEphemeris) {
        ephemeris.bind_magnetosphere(gl, &self.u_aurora_lower, &self.u_aurora_upper);
    }
}

pub(super) trait StageUniformGroup {
    fn bind_stage(&self, gl: &glow::Context, ephemeris: &VaultEphemeris);
}

pub(super) struct StageUniforms {
    canopy: CanopyUniforms,
    ephemeris: EphemerisUniforms,
    magnetosphere: MagnetosphereUniforms,
}

impl StageUniforms {
    pub(super) fn new(
        canopy: CanopyUniforms,
        ephemeris: EphemerisUniforms,
        magnetosphere: MagnetosphereUniforms,
    ) -> Self {
        Self { canopy, ephemeris, magnetosphere }
    }
}

impl StageUniformGroup for StageUniforms {
    fn bind_stage(&self, gl: &glow::Context, ephemeris: &VaultEphemeris) {
        self.ephemeris.bind_ephemeris(gl, ephemeris);
        self.magnetosphere.bind_magnetosphere(gl, ephemeris);
        self.canopy.bind_canopy(gl);
    }
}
