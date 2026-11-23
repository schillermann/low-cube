use glow::HasContext;
use engine::Mesh;
use crate::aurora_canopy::AuroraCanopy;
use crate::firmament::{
    FirmamentEphemeris, SkyFirmament, VaultEphemeris, VaultFrame, VaultGlow, VaultLuminance,
    VaultView,
};

trait ViewUniformGroup {
    fn bind_view(&self, gl: &glow::Context, view: &VaultView);
}

struct OrientationUniforms {
    u_inv_view_proj: glow::UniformLocation,
    u_sun_dir: glow::UniformLocation,
    u_celestial_pole: glow::UniformLocation,
    u_local_up: glow::UniformLocation,
}

impl OrientationUniforms {
    const fn new(
        u_inv_view_proj: glow::UniformLocation,
        u_sun_dir: glow::UniformLocation,
        u_celestial_pole: glow::UniformLocation,
        u_local_up: glow::UniformLocation,
    ) -> Self {
        Self { u_inv_view_proj, u_sun_dir, u_celestial_pole, u_local_up }
    }
}

impl ViewUniformGroup for OrientationUniforms {
    fn bind_view(&self, gl: &glow::Context, view: &VaultView) {
        view.bind_gl(
            gl,
            &self.u_inv_view_proj,
            &self.u_sun_dir,
            &self.u_celestial_pole,
            &self.u_local_up,
        );
    }
}

trait GlowUniformGroup {
    fn bind_glow(&self, gl: &glow::Context, glow_state: &VaultGlow);
}

struct AtmosphereUniforms {
    u_sky_color: glow::UniformLocation,
    u_night_factor: glow::UniformLocation,
    u_aurora_factor: glow::UniformLocation,
    u_time: glow::UniformLocation,
}

impl AtmosphereUniforms {
    const fn new(
        u_sky_color: glow::UniformLocation,
        u_night_factor: glow::UniformLocation,
        u_aurora_factor: glow::UniformLocation,
        u_time: glow::UniformLocation,
    ) -> Self {
        Self { u_sky_color, u_night_factor, u_aurora_factor, u_time }
    }
}

impl GlowUniformGroup for AtmosphereUniforms {
    fn bind_glow(&self, gl: &glow::Context, glow_state: &VaultGlow) {
        glow_state.bind_gl(
            gl,
            &self.u_sky_color,
            &self.u_night_factor,
            &self.u_aurora_factor,
            &self.u_time,
        );
    }
}

trait CanopyUniformGroup {
    fn bind_canopy(&self, gl: &glow::Context);
}

struct CanopyUniforms {
    u_face_half: glow::UniformLocation,
    u_aurora_radius: glow::UniformLocation,
    u_ceiling_altitude: glow::UniformLocation,
    canopy: Box<dyn AuroraCanopy>,
}

impl CanopyUniforms {
    fn new(
        u_face_half: glow::UniformLocation,
        u_aurora_radius: glow::UniformLocation,
        u_ceiling_altitude: glow::UniformLocation,
        canopy: Box<dyn AuroraCanopy>,
    ) -> Self {
        Self { u_face_half, u_aurora_radius, u_ceiling_altitude, canopy }
    }
}

impl CanopyUniformGroup for CanopyUniforms {
    fn bind_canopy(&self, gl: &glow::Context) {
        self.canopy.bind_canopy(gl, &self.u_face_half, &self.u_aurora_radius, &self.u_ceiling_altitude);
    }
}

trait EphemerisUniformGroup {
    fn bind_ephemeris(&self, gl: &glow::Context, ephemeris: &VaultEphemeris);
}

struct EphemerisUniforms {
    u_planets: glow::UniformLocation,
    u_eclipse: glow::UniformLocation,
    u_atmo_geom: glow::UniformLocation,
    u_atmo_tint: glow::UniformLocation,
}

impl EphemerisUniforms {
    const fn new(
        u_planets: glow::UniformLocation,
        u_eclipse: glow::UniformLocation,
        u_atmo_geom: glow::UniformLocation,
        u_atmo_tint: glow::UniformLocation,
    ) -> Self {
        Self { u_planets, u_eclipse, u_atmo_geom, u_atmo_tint }
    }
}

impl EphemerisUniformGroup for EphemerisUniforms {
    fn bind_ephemeris(&self, gl: &glow::Context, ephemeris: &VaultEphemeris) {
        ephemeris.bind_gl(gl, &self.u_planets, &self.u_eclipse, &self.u_atmo_geom, &self.u_atmo_tint);
    }
}

trait StageUniformGroup {
    fn bind_stage(&self, gl: &glow::Context, ephemeris: &VaultEphemeris);
}

struct StageUniforms {
    canopy: CanopyUniforms,
    ephemeris: EphemerisUniforms,
}

impl StageUniforms {
    fn new(canopy: CanopyUniforms, ephemeris: EphemerisUniforms) -> Self {
        Self { canopy, ephemeris }
    }
}

impl StageUniformGroup for StageUniforms {
    fn bind_stage(&self, gl: &glow::Context, ephemeris: &VaultEphemeris) {
        self.ephemeris.bind_ephemeris(gl, ephemeris);
        self.canopy.bind_canopy(gl);
    }
}

/// GPU shader program rendering the celestial vault with cubic Sun, real-time planets, eclipses, and Aurora.
pub struct GlFirmament {
    program: glow::Program,
    orientation: OrientationUniforms,
    atmosphere: AtmosphereUniforms,
    stage: StageUniforms,
}

impl GlFirmament {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(
        program: glow::Program,
        orientation: OrientationUniforms,
        atmosphere: AtmosphereUniforms,
        stage: StageUniforms,
    ) -> Self {
        Self { program, orientation, atmosphere, stage }
    }
}

impl SkyFirmament for GlFirmament {
    fn draw_firmament(
        &self,
        gl: &glow::Context,
        view: &VaultView,
        glow_state: &VaultGlow,
        ephemeris: &VaultEphemeris,
        mesh: &dyn Mesh,
    ) {
        unsafe {
            gl.use_program(Some(self.program));
            self.orientation.bind_view(gl, view);
            self.atmosphere.bind_glow(gl, glow_state);
            self.stage.bind_stage(gl, ephemeris);
            gl.depth_mask(false);
            gl.depth_func(glow::LEQUAL);
            mesh.draw(gl);
            gl.depth_mask(true);
            gl.depth_func(glow::LESS);
        }
    }
}

/// Focused contract compiling embedded firmament GLSL sources into a live `SkyFirmament`.
pub trait FirmamentProgramSource {
    /// # Safety
    /// Requires a valid, current OpenGL context.
    unsafe fn compile(&self, canopy: Box<dyn AuroraCanopy>, gl: &glow::Context) -> Box<dyn SkyFirmament>;
}

/// Immutable GLSL shader source wrapper for the celestial firmament (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct FirmamentShaderSource {
    sources: crate::firmament_glsl::FirmamentGlslPair,
}

impl FirmamentShaderSource {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sources: crate::firmament_glsl::FirmamentGlslPair) -> Self {
        Self { sources }
    }

    pub const fn embedded() -> Self {
        Self::new(crate::firmament_glsl::FirmamentGlslPair::embedded())
    }
}

impl FirmamentProgramSource for FirmamentShaderSource {
    unsafe fn compile(&self, canopy: Box<dyn AuroraCanopy>, gl: &glow::Context) -> Box<dyn SkyFirmament> {
        use crate::firmament_glsl::GlslSourcePair;
        let (vertex_src, fragment_src) = self.sources.transfer_sources(|v, f| (v, f));
        let vs = gl.create_shader(glow::VERTEX_SHADER).expect("VS create failed");
        gl.shader_source(vs, vertex_src);
        gl.compile_shader(vs);
        if !gl.get_shader_compile_status(vs) {
            panic!("Firmament VS error: {}", gl.get_shader_info_log(vs));
        }
        let fs = gl.create_shader(glow::FRAGMENT_SHADER).expect("FS create failed");
        gl.shader_source(fs, fragment_src);
        gl.compile_shader(fs);
        if !gl.get_shader_compile_status(fs) {
            panic!("Firmament FS error: {}", gl.get_shader_info_log(fs));
        }
        let program = gl.create_program().expect("Program create failed");
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("Firmament link error: {}", gl.get_program_info_log(program));
        }
        gl.delete_shader(vs);
        gl.delete_shader(fs);

        let loc = |name: &str| {
            gl.get_uniform_location(program, name)
                .unwrap_or_else(|| panic!("missing uniform {name}"))
        };
        let u_planets = gl
            .get_uniform_location(program, "u_planet_cubes[0]")
            .or_else(|| gl.get_uniform_location(program, "u_planet_cubes"))
            .expect("missing u_planet_cubes");

        Box::new(GlFirmament::new(
            program,
            OrientationUniforms::new(
                loc("u_inv_view_proj"),
                loc("u_sun_dir"),
                loc("u_celestial_pole"),
                loc("u_local_up"),
            ),
            AtmosphereUniforms::new(
                loc("u_sky_color"),
                loc("u_night_factor"),
                loc("u_aurora_factor"),
                loc("u_time"),
            ),
            StageUniforms::new(
                CanopyUniforms::new(
                    loc("u_face_half"),
                    loc("u_aurora_radius"),
                    loc("u_ceiling_altitude"),
                    canopy,
                ),
                EphemerisUniforms::new(
                    u_planets,
                    loc("u_eclipse_occluder"),
                    loc("u_atmo_geom"),
                    loc("u_atmo_tint"),
                ),
            ),
        ))
    }
}
