pub(super) const VERTEX_SHADER_SRC: &str = r#"#version 330 core
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_color;
layout(location = 3) in float in_glow;

uniform mat4 u_view_proj;
uniform mat4 u_model;

out vec3 v_normal;
out vec3 v_color;
out float v_glow;
out vec3 v_world;
out vec2 v_uv;
out float v_logz;
out float v_persp;

void main() {
    vec4 world_pos = u_model * vec4(in_position, 1.0);
    v_normal = (in_glow < 0.0 && in_glow > -1.5) ? in_normal : (mat3(u_model) * in_normal);
    v_color = in_color;
    v_glow = in_glow;
    v_world = in_position;
    int corner = gl_VertexID & 3;
    v_uv = vec2(float(corner == 1 || corner == 2), float(corner >= 2));
    vec4 clip_pos = u_view_proj * world_pos;
    v_persp = (u_view_proj[2][3] < -0.5) ? 1.0 : 0.0;
    if (v_persp > 0.5) {
        if (clip_pos.w >= 0.03) {
            v_logz = 1.0 + (clip_pos.w - 0.03);
            clip_pos.z = (log2(v_logz) * 0.060206 - 1.0) * clip_pos.w;
        } else {
            v_logz = 1.0;
            clip_pos.z = -2.0 * abs(clip_pos.w) - 1.0;
        }
    } else {
        v_logz = 1.0;
    }
    gl_Position = clip_pos;
}
"#;

pub(super) const FRAGMENT_SHADER_SRC: &str = r#"#version 330 core
in vec3 v_normal;
in vec3 v_color;
in float v_glow;
in vec3 v_world;
in vec2 v_uv;
in float v_logz;
in float v_persp;

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

vec3 world_plateau_zenith(vec3 p) {
    if (dot(p, p) <= 1e-6) {
        return vec3(0.0, 1.0, 0.0);
    }
    vec3 n = normalize(p);
    vec3 q2 = n * n;
    return normalize(n * q2 * q2);
}

void main() {
    gl_FragDepth = (v_persp > 0.5)
        ? clamp(log2(max(v_logz, 1.0)) * 0.030103, 0.0, 0.999999)
        : gl_FragCoord.z;
    if (v_glow < -5.5) {
        float base_a = clamp(-v_glow - 6.0, 0.02, 0.92);
        float edge_x = smoothstep(0.0, 0.22, min(v_uv.x, 1.0 - v_uv.x));
        float edge_y = smoothstep(0.0, 0.14, min(v_uv.y, 1.0 - v_uv.y));
        float mu = dot(world_plateau_zenith(v_world), normalize(u_light_dir));
        float day = smoothstep(-0.24, 0.22, mu);
        vec3 amb = mix(vec3(0.16, 0.20, 0.30), vec3(0.94, 0.97, 1.00), day);
        frag_color = vec4(v_color * amb, base_a * edge_x * edge_y);
        return;
    }
    if (v_glow < -3.5) {
        float base_a = clamp(-v_glow - 4.0, 0.02, 0.95);
        vec3 sun = normalize(u_light_dir);
        float l_dot = dot(normalize(v_normal), sun);
        float facet = clamp(l_dot * 0.32 + 0.70, 0.36, 1.04);
        float mu = dot(world_plateau_zenith(v_world), sun);
        float day = smoothstep(-0.24, 0.24, mu);
        float dusk = exp(-pow((mu - 0.02) / 0.18, 2.0)) * (1.0 - smoothstep(0.18, 0.45, mu));
        vec3 sky_env = mix(vec3(0.12, 0.16, 0.26), mix(vec3(1.0), vec3(1.06, 0.68, 0.44), dusk * 0.65), day);
        frag_color = vec4(min(v_color * facet * sky_env, vec3(1.0)), base_a);
        return;
    }
    if (v_glow < -1.5) {
        float base_alpha = clamp(-v_glow - 2.0, 0.0, 1.0);
        float axial = clamp(1.0 - 0.72 * v_uv.y * v_uv.y, 0.08, 1.0);
        float mach = 0.88 + 0.12 * cos(v_uv.y * 12.56637);
        vec3 hot_col = min(v_color * (1.10 + 0.22 * (1.0 - v_uv.y)) * mach, vec3(1.0));
        frag_color = vec4(hot_col, clamp(base_alpha * axial, 0.03, 0.90));
        return;
    }
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
    float l_dot = dot(norm, normalize(u_light_dir));
    float diff = max(l_dot, 0.0);
    vec3 light = vec3(0.35) + vec3(0.65) * diff;
    vec3 lit = base_col * light;
    float cam_dist = 1.0 / max(gl_FragCoord.w, 0.0001);
    float orb = smoothstep(55.0, 220.0, cam_dist) * (1.0 - smoothstep(450.0, 900.0, length(v_world)));
    if (orb > 0.005 && v_glow < 0.05) {
        float day_a = smoothstep(-0.08, 0.35, l_dot);
        float term_a = exp(-pow((l_dot - 0.02) / 0.16, 2.0)) * smoothstep(-0.25, 0.0, l_dot);
        lit = mix(lit, lit * 0.82 + vec3(0.14, 0.44, 0.92) * day_a * 0.34 + vec3(0.95, 0.36, 0.10) * term_a * 0.30, orb);
    }
    vec3 radiant = min(base_col * 1.15, vec3(1.0));
    float glow_mix = (v_glow < 0.05) ? 0.0 : clamp(v_glow, 0.0, 1.0);
    frag_color = vec4(mix(lit, radiant, glow_mix), 1.0);
}
"#;
