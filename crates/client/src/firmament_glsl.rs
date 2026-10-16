pub const FIRMAMENT_VERTEX_SRC: &str = r#"#version 330 core
layout(location = 0) in vec2 in_position;

uniform mat4 u_inv_view_proj;

out vec3 v_ray_dir;

void main() {
    vec4 near_pt = u_inv_view_proj * vec4(in_position, -1.0, 1.0);
    vec4 far_pt = u_inv_view_proj * vec4(in_position, 1.0, 1.0);
    vec3 p_near = near_pt.xyz / near_pt.w;
    vec3 p_far = far_pt.xyz / far_pt.w;
    v_ray_dir = p_far - p_near;
    gl_Position = vec4(in_position, 1.0, 1.0);
}
"#;

pub const FIRMAMENT_FRAGMENT_SRC: &str = r#"#version 330 core
in vec3 v_ray_dir;

uniform vec3 u_sun_dir;
uniform vec3 u_local_up;
uniform vec4 u_sky_color;
uniform float u_night_factor;
uniform vec3 u_celestial_pole;

out vec4 frag_color;

float star_noise(vec3 p) {
    p = fract(p * vec3(443.897, 441.423, 437.195));
    p += dot(p, p.yzx + 19.19);
    return fract((p.x + p.y) * p.z);
}

float stars(vec3 dir) {
    vec3 cell = floor(dir * 140.0);
    float h = star_noise(cell);
    if (h > 0.982) {
        vec3 center = normalize(cell + 0.5);
        float d = length(dir - center);
        float size = 0.0035 * (0.5 + 0.5 * fract(h * 37.0));
        return (1.0 - smoothstep(0.0, size, d)) * (0.7 + 0.3 * fract(h * 111.0));
    }
    return 0.0;
}

float crater_field(vec2 uv, float grid_res, float r_factor, float rim_w, float depth) {
    vec2 g = uv * grid_res;
    vec2 id = floor(g);
    vec2 f = fract(g);
    float val = 0.0;
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            vec2 neighbor = vec2(float(x), float(y));
            vec2 cell = id + neighbor;
            vec2 h = fract(sin(vec2(dot(cell, vec2(127.1, 311.7)), dot(cell, vec2(269.5, 183.3)))) * 43758.5453);
            vec2 center = neighbor + h;
            float r = r_factor * (0.32 + 0.48 * fract(h.x * 19.19));
            float d = length(f - center) / r;
            if (d < 1.0) {
                float rim = smoothstep(1.0 - rim_w, 0.90, d) * (1.0 - smoothstep(0.90, 1.0, d));
                val += rim * (depth * 0.95) - (1.0 - smoothstep(0.0, 0.72, d)) * depth;
            }
        }
    }
    return val;
}

float lunar_basin(vec2 p, vec2 c, float r) {
    float d = length(p - c) / r;
    if (d > 1.0) return 0.0;
    return -(1.0 - smoothstep(0.15, 0.95, d)) * 0.22;
}

void main() {
    vec3 dir = normalize(v_ray_dir);

    // Fiery golden sun with warm corona and horizon cutoff
    float sun_dot = dot(dir, u_sun_dir);
    float sun_disc = smoothstep(0.9980, 0.9992, sun_dot);
    float sun_inner_glow = pow(max(sun_dot, 0.0), 32.0) * 0.50;
    float sun_outer_glow = pow(max(sun_dot, 0.0), 8.0) * 0.20;
    float sun_above = smoothstep(-0.02, 0.05, dot(u_sun_dir, u_local_up));
    vec3 sun_core = vec3(1.0, 0.95, 0.65) * sun_disc * 2.8;
    vec3 sun_halo = vec3(1.0, 0.60, 0.15) * sun_inner_glow + vec3(0.95, 0.35, 0.08) * sun_outer_glow;
    vec3 sun_color = (sun_core + sun_halo) * sun_above;

    // Full round lunar sphere with stable orbital celestial orientation
    vec3 moon_dir = -u_sun_dir;
    vec3 moon_right = normalize(cross(moon_dir, u_celestial_pole));
    vec3 moon_up = cross(moon_right, moon_dir);
    vec2 moon_uv = vec2(dot(dir - moon_dir, moon_right), dot(dir - moon_dir, moon_up)) * 28.0;

    float moon_disc = 1.0 - smoothstep(0.92, 1.0, length(moon_uv));
    float moon_above = smoothstep(-0.02, 0.05, dot(moon_dir, u_local_up));
    float moon_visible = moon_disc * moon_above;

    vec3 moon_body = vec3(0.0);
    if (moon_visible > 0.0) {
        float surface = 0.72;
        surface += lunar_basin(moon_uv, vec2(-0.30, 0.35), 0.42);
        surface += lunar_basin(moon_uv, vec2(-0.48, -0.15), 0.45);
        surface += lunar_basin(moon_uv, vec2(0.24, 0.32), 0.32);
        surface += lunar_basin(moon_uv, vec2(0.35, 0.05), 0.34);
        surface += lunar_basin(moon_uv, vec2(0.62, 0.18), 0.22);
        surface += crater_field(moon_uv, 4.5, 0.55, 0.25, 0.26);
        surface += crater_field(moon_uv, 9.5, 0.50, 0.20, 0.18);
        vec2 tycho_pos = vec2(0.12, -0.55);
        vec2 d_tycho = moon_uv - tycho_pos;
        float ray_angle = atan(d_tycho.y, d_tycho.x);
        float rays = pow(max(sin(ray_angle * 12.0), 0.0), 3.0) * exp(-length(d_tycho) * 1.6) * 0.28;
        surface += rays;

        moon_body = vec3(0.85, 0.88, 0.94) * clamp(surface, 0.32, 0.98);
    }

    // Occlude stars behind solid celestial bodies (moon and sun)
    float star_occ = (1.0 - moon_visible) * (1.0 - sun_disc * sun_above);
    vec3 night_sky = u_sky_color.rgb + vec3(stars(dir) * u_night_factor * star_occ * 1.5);

    // Completely opaque lunar body composited over firmament
    vec3 sky_with_moon = mix(night_sky, moon_body, moon_visible);

    frag_color = vec4(sky_with_moon + sun_color, 1.0);
}
"#;
