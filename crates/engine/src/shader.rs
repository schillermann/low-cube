use glam::{Mat4, Vec3};
use glow::HasContext;
use crate::{ShaderProgram, ShaderSource};

const VERTEX_SHADER_SRC: &str = r#"#version 330 core
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_color;
layout(location = 3) in float in_glow;

uniform mat4 u_view_proj;

out vec3 v_normal;
out vec3 v_color;
out float v_glow;
out vec3 v_world;
out vec2 v_uv;

void main() {
    v_normal = in_normal;
    v_color = in_color;
    v_glow = in_glow;
    v_world = in_position;
    int corner = gl_VertexID & 3;
    v_uv = vec2(float(corner == 1 || corner == 2), float(corner >= 2));
    gl_Position = u_view_proj * vec4(in_position, 1.0);
}
"#;

const FRAGMENT_SHADER_SRC: &str = r#"#version 330 core
in vec3 v_normal;
in vec3 v_color;
in float v_glow;
in vec3 v_world;
in vec2 v_uv;

uniform vec3 u_light_dir;

out vec4 frag_color;

int contour_tier(vec2 p) {
    float w1 = sin(p.x * 3.8 + p.y * 2.7) + cos(p.x * 2.1 - p.y * 4.3);
    float w2 = sin(p.x * 6.2 - p.y * 5.1) * 0.6 + cos(p.x * 4.7 + p.y * 6.4) * 0.5;
    float val = w1 * 0.55 + w2 * 0.45;
    return (val < -0.65) ? 0 : (val < -0.10) ? 1 : (val < 0.42) ? 2 : (val < 0.92) ? 3 : 4;
}

vec3 tier_scale(int t) {
    return (t == 0) ? vec3(0.78, 0.82, 0.76)
         : (t == 1) ? vec3(0.90, 0.92, 0.88)
         : (t == 2) ? vec3(1.02, 1.00, 0.98)
         : (t == 3) ? vec3(1.22, 1.12, 1.10)
         : vec3(1.44, 1.24, 1.25);
}

void main() {
    if (v_glow < 0.0) {
        float ramp = clamp((length(v_world.xy) - v_normal.x) / max(v_normal.y, 0.0001), 0.0, 1.0);
        float alpha = clamp(-v_glow * mix(v_normal.z, 1.0, ramp), 0.0, 1.0);
        frag_color = vec4(v_color, alpha);
        return;
    }
    if (v_glow > 1.5) {
        float cam_dist = 1.0 / max(gl_FragCoord.w, 0.0001);
        float dist_fade = clamp((42.0 - cam_dist) * 0.15, 0.0, 1.0);
        if (dist_fade <= 0.01) {
            discard;
        }
        float seed = v_glow - 2.0;
        float x = v_uv.x * 9.0 + seed;
        float cell = floor(x);
        float lu = x - cell;
        float h_hash = fract((cell * 0.618034 + seed * 0.381966) * 17.31);
        float max_h = (0.60 + 0.38 * h_hash) * dist_fade;
        if (v_uv.y > max_h) {
            discard;
        }
        float t = clamp(v_uv.y / max_h, 0.0, 1.0);
        float center = 0.5 + t * t * 0.44 * (h_hash - 0.5);
        float half_w = max(0.42 * (1.0 - t * (0.65 + 0.35 * t)), 0.04);
        if (abs(lu - center) > half_w) {
            discard;
        }
        float ridge = 1.0 - abs(lu - center) / half_w * 0.25;
        vec3 blade_col = v_color * vec3(0.75 + 0.42 * t, 0.84 + 0.34 * t, 0.70 + 0.18 * t) * ridge;
        float diff = abs(dot(normalize(v_normal), normalize(u_light_dir))) * 0.45 + 0.55;
        frag_color = vec4(min(blade_col * diff, vec3(1.0)), 1.0);
        return;
    }
    vec3 norm = normalize(v_normal);
    vec3 base_col = v_color;
    if (v_glow > 0.005 && v_glow < 0.05) {
        vec3 an = abs(norm);
        vec2 wp = (an.y >= max(an.x, an.z)) ? v_world.xz : ((an.x >= an.z) ? v_world.yz : v_world.xy);
        vec2 grid_p = floor(wp * 8.0) * 0.125 + vec2(0.0625);
        vec2 cell_uv = fract(wp * 8.0);
        int t = contour_tier(grid_p);
        bool rx = (cell_uv.x > 0.86) && (t != contour_tier(grid_p + vec2(0.125, 0.0)));
        bool ry = !rx && (cell_uv.y > 0.86) && (t != contour_tier(grid_p + vec2(0.0, 0.125)));
        float riser = (rx || ry) ? 0.88 : 1.0;
        base_col = clamp(v_color * tier_scale(t) * riser, vec3(0.0), vec3(1.0));
    }
    float diff = max(dot(norm, normalize(u_light_dir)), 0.0);
    vec3 light = vec3(0.35) + vec3(0.65) * diff;
    vec3 radiant = min(base_col * 1.15, vec3(1.0));
    float glow_mix = (v_glow < 0.05) ? 0.0 : clamp(v_glow, 0.0, 1.0);
    frag_color = vec4(mix(base_col * light, radiant, glow_mix), 1.0);
}
"#;

/// Encapsulated GPU program with clean, code-free primary constructor.
pub struct VoxelProgram {
    program: glow::Program,
    u_view_proj: glow::UniformLocation,
    u_light_dir: glow::UniformLocation,
}

impl VoxelProgram {
    pub fn new(
        program: glow::Program,
        u_view_proj: glow::UniformLocation,
        u_light_dir: glow::UniformLocation,
    ) -> Self {
        Self {
            program,
            u_view_proj,
            u_light_dir,
        }
    }
}

impl ShaderProgram for VoxelProgram {
    fn bind(&self, gl: &glow::Context, view_proj: &Mat4, light_dir: Vec3) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(Some(&self.u_view_proj), false, &view_proj.to_cols_array());
            gl.uniform_3_f32(Some(&self.u_light_dir), light_dir.x, light_dir.y, light_dir.z);
        }
    }
}

/// Encapsulated GLSL source code for voxel rendering.
pub struct VoxelShaderSource {
    vertex: &'static str,
    fragment: &'static str,
}

impl VoxelShaderSource {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(vertex: &'static str, fragment: &'static str) -> Self {
        Self { vertex, fragment }
    }

    /// Embedded voxel shader sources compiled into the binary.
    pub fn embedded() -> Self {
        Self::new(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC)
    }
}

impl ShaderSource for VoxelShaderSource {
    unsafe fn compile(&self, gl: &glow::Context) -> Box<dyn ShaderProgram> {
        let vs = gl.create_shader(glow::VERTEX_SHADER).expect("VS create failed");
        gl.shader_source(vs, self.vertex);
        gl.compile_shader(vs);
        if !gl.get_shader_compile_status(vs) {
            panic!("VS compile error: {}", gl.get_shader_info_log(vs));
        }

        let fs = gl.create_shader(glow::FRAGMENT_SHADER).expect("FS create failed");
        gl.shader_source(fs, self.fragment);
        gl.compile_shader(fs);
        if !gl.get_shader_compile_status(fs) {
            panic!("FS compile error: {}", gl.get_shader_info_log(fs));
        }

        let program = gl.create_program().expect("Program create failed");
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("Program link error: {}", gl.get_program_info_log(program));
        }

        gl.delete_shader(vs);
        gl.delete_shader(fs);

        let u_view_proj = gl.get_uniform_location(program, "u_view_proj").expect("u_view_proj missing");
        let u_light_dir = gl.get_uniform_location(program, "u_light_dir").expect("u_light_dir missing");

        Box::new(VoxelProgram::new(program, u_view_proj, u_light_dir))
    }
}
