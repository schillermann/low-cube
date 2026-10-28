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
    u_face_half: glow::UniformLocation,
    u_aurora_radius: glow::UniformLocation,
    u_ceiling_altitude: glow::UniformLocation,
    canopy: Box<dyn AuroraCanopy>,
}

/// GPU shader program rendering the celestial vault with Sun, Moon, Stars, and Arctic Aurora.
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
        u_face_half: glow::UniformLocation,
        u_aurora_radius: glow::UniformLocation,
        u_ceiling_altitude: glow::UniformLocation,
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
                u_face_half,
                u_aurora_radius,
                u_ceiling_altitude,
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

        let u_inv_view_proj = gl.get_uniform_location(program, "u_inv_view_proj").expect("missing u_inv_view_proj");
        let u_sun_dir = gl.get_uniform_location(program, "u_sun_dir").expect("missing u_sun_dir");
        let u_celestial_pole = gl.get_uniform_location(program, "u_celestial_pole").expect("missing u_celestial_pole");
        let u_local_up = gl.get_uniform_location(program, "u_local_up").expect("missing u_local_up");
        let u_sky_color = gl.get_uniform_location(program, "u_sky_color").expect("missing u_sky_color");
        let u_night_factor = gl.get_uniform_location(program, "u_night_factor").expect("missing u_night_factor");
        let u_aurora_factor = gl.get_uniform_location(program, "u_aurora_factor").expect("missing u_aurora_factor");
        let u_time = gl.get_uniform_location(program, "u_time").expect("missing u_time");
        let u_face_half = gl.get_uniform_location(program, "u_face_half").expect("missing u_face_half");
        let u_aurora_radius = gl.get_uniform_location(program, "u_aurora_radius").expect("missing u_aurora_radius");
        let u_ceiling_altitude = gl.get_uniform_location(program, "u_ceiling_altitude").expect("missing u_ceiling_altitude");

        Self::new(
            program,
            u_inv_view_proj,
            u_sun_dir,
            u_celestial_pole,
            u_local_up,
            u_sky_color,
            u_night_factor,
            u_aurora_factor,
            u_time,
            u_face_half,
            u_aurora_radius,
            u_ceiling_altitude,
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
        mesh: &dyn Mesh,
    ) {
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
            self.canopy_uniforms.canopy.bind_canopy(
                gl,
                &self.canopy_uniforms.u_face_half,
                &self.canopy_uniforms.u_aurora_radius,
                &self.canopy_uniforms.u_ceiling_altitude,
            );
            gl.depth_mask(false);
            gl.depth_func(glow::LEQUAL);
            mesh.draw(gl);
            gl.depth_mask(true);
            gl.depth_func(glow::LESS);
        }
    }
}
