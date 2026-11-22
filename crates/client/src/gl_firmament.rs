use glam::{Mat4, Vec3, Vec4};
use glow::HasContext;
use engine::Mesh;
use crate::aurora_canopy::AuroraCanopy;
use crate::firmament::SkyFirmament;
use crate::firmament_glsl::{FIRMAMENT_FRAGMENT_SRC, FIRMAMENT_VERTEX_SRC};

struct FirmamentOrientationUniforms {
    u_inv_view_proj: glow::UniformLocation,
    u_sun_dir: glow::UniformLocation,
    u_celestial_pole: glow::UniformLocation,
    u_local_up: glow::UniformLocation,
}

struct FirmamentAtmosphereUniforms {
    u_sky_color: glow::UniformLocation,
    u_night_factor: glow::UniformLocation,
    u_aurora_factor: glow::UniformLocation,
    u_time: glow::UniformLocation,
}

struct FirmamentCanopyUniforms {
    aurora_locs: (glow::UniformLocation, glow::UniformLocation, glow::UniformLocation),
    ephemeris_locs: (glow::UniformLocation, glow::UniformLocation),
    canopy: Box<dyn AuroraCanopy>,
}

/// GPU shader program rendering the celestial vault with cubic Sun, real-time planets, eclipses, and Aurora.
pub struct GlFirmament {
    program: glow::Program,
    orientation: FirmamentOrientationUniforms,
    atmosphere: FirmamentAtmosphereUniforms,
    canopy_uniforms: FirmamentCanopyUniforms,
}

impl GlFirmament {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        program: glow::Program,
        u_inv_view_proj: glow::UniformLocation,
        u_sun_dir: glow::UniformLocation,
        u_celestial_pole: glow::UniformLocation,
        u_local_up: glow::UniformLocation,
        u_sky_color: glow::UniformLocation,
        u_night_factor: glow::UniformLocation,
        u_aurora_factor: glow::UniformLocation,
        u_time: glow::UniformLocation,
        aurora_locs: (glow::UniformLocation, glow::UniformLocation, glow::UniformLocation),
        ephemeris_locs: (glow::UniformLocation, glow::UniformLocation),
        canopy: Box<dyn AuroraCanopy>,
    ) -> Self {
        Self {
            program,
            orientation: FirmamentOrientationUniforms {
                u_inv_view_proj,
                u_sun_dir,
                u_celestial_pole,
                u_local_up,
            },
            atmosphere: FirmamentAtmosphereUniforms {
                u_sky_color,
                u_night_factor,
                u_aurora_factor,
                u_time,
            },
            canopy_uniforms: FirmamentCanopyUniforms {
                aurora_locs,
                ephemeris_locs,
                canopy,
            },
        }
    }

    /// Secondary constructor compiling embedded firmament shader sources.
    pub unsafe fn compiled(canopy: Box<dyn AuroraCanopy>, gl: &glow::Context) -> Self {
        let vs = gl.create_shader(glow::VERTEX_SHADER).expect("VS create failed");
        gl.shader_source(vs, FIRMAMENT_VERTEX_SRC);
        gl.compile_shader(vs);
        if !gl.get_shader_compile_status(vs) {
            panic!("Firmament VS error: {}", gl.get_shader_info_log(vs));
        }

        let fs = gl.create_shader(glow::FRAGMENT_SHADER).expect("FS create failed");
        gl.shader_source(fs, FIRMAMENT_FRAGMENT_SRC);
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
        let u_eclipse = loc("u_eclipse_occluder");

        Self::new(
            program,
            loc("u_inv_view_proj"),
            loc("u_sun_dir"),
            loc("u_celestial_pole"),
            loc("u_local_up"),
            loc("u_sky_color"),
            loc("u_night_factor"),
            loc("u_aurora_factor"),
            loc("u_time"),
            (loc("u_face_half"), loc("u_aurora_radius"), loc("u_ceiling_altitude")),
            (u_planets, u_eclipse),
            canopy,
        )
    }
}

impl SkyFirmament for GlFirmament {
    fn draw_firmament(
        &self,
        gl: &glow::Context,
        inv_view_proj: &Mat4,
        sun_dir: Vec3,
        celestial_pole: Vec3,
        local_up: Vec3,
        sky_color: Vec4,
        night_factor: f32,
        aurora_factor: f32,
        time: f32,
        (planets, occluder): (&[[f32; 4]; 7], [f32; 4]),
        mesh: &dyn Mesh,
    ) {
        let mut flat_planets = [0.0_f32; 28];
        for (i, cube) in planets.iter().enumerate() {
            flat_planets[i * 4..i * 4 + 4].copy_from_slice(cube);
        }
        let (ref u_face, ref u_rad, ref u_ceil) = self.canopy_uniforms.aurora_locs;
        let (ref u_planets, ref u_eclipse) = self.canopy_uniforms.ephemeris_locs;
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(Some(&self.orientation.u_inv_view_proj), false, &inv_view_proj.to_cols_array());
            gl.uniform_3_f32(Some(&self.orientation.u_sun_dir), sun_dir.x, sun_dir.y, sun_dir.z);
            gl.uniform_3_f32(Some(&self.orientation.u_celestial_pole), celestial_pole.x, celestial_pole.y, celestial_pole.z);
            gl.uniform_3_f32(Some(&self.orientation.u_local_up), local_up.x, local_up.y, local_up.z);
            gl.uniform_4_f32(Some(&self.atmosphere.u_sky_color), sky_color.x, sky_color.y, sky_color.z, sky_color.w);
            gl.uniform_1_f32(Some(&self.atmosphere.u_night_factor), night_factor);
            gl.uniform_1_f32(Some(&self.atmosphere.u_aurora_factor), aurora_factor);
            gl.uniform_1_f32(Some(&self.atmosphere.u_time), time);
            gl.uniform_4_f32_slice(Some(u_planets), &flat_planets);
            gl.uniform_4_f32(Some(u_eclipse), occluder[0], occluder[1], occluder[2], occluder[3]);
            self.canopy_uniforms.canopy.bind_canopy(gl, u_face, u_rad, u_ceil);
            gl.depth_mask(false);
            gl.depth_func(glow::LEQUAL);
            mesh.draw(gl);
            gl.depth_mask(true);
            gl.depth_func(glow::LESS);
        }
    }
}
