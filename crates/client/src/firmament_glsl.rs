pub const FIRMAMENT_VERTEX_SRC: &str = r#"#version 330 core
layout(location = 0) in vec2 in_position;

uniform mat4 u_inv_view_proj;

out vec3 v_ray_dir;
out vec3 v_cam_pos;

void main() {
    v_ray_dir = (u_inv_view_proj * vec4(in_position, 0.0, 1.0)).xyz;
    v_cam_pos = (u_inv_view_proj * vec4(0.0, 0.0, 1.0, 0.0)).xyz;
    gl_Position = vec4(in_position, 1.0, 1.0);
}
"#;

pub const FIRMAMENT_FRAGMENT_SRC: &str = r#"#version 330 core
in vec3 v_ray_dir;
in vec3 v_cam_pos;

uniform vec3 u_sun_dir;
uniform vec3 u_local_up;
uniform vec4 u_sky_color;
uniform float u_night_factor;
uniform float u_aurora_factor;
uniform vec3 u_celestial_pole;
uniform float u_time;
uniform float u_face_half;
uniform float u_aurora_radius;
uniform float u_ceiling_altitude;
uniform vec4 u_planet_cubes[8];
uniform vec4 u_eclipse_occluder;
uniform vec4 u_atmo_geom;
uniform vec4 u_atmo_tint;

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

vec2 box_slab(vec3 orig, vec3 dir, float half_ext) {
    vec3 inv_d = 1.0 / sign(dir + vec3(1e-9)) / max(abs(dir), vec3(1e-6));
    vec3 t0s = (-vec3(half_ext) - orig) * inv_d;
    vec3 t1s = (vec3(half_ext) - orig) * inv_d;
    vec3 tsmaller = min(t0s, t1s);
    vec3 tbigger = max(t0s, t1s);
    float t0 = max(max(tsmaller.x, tsmaller.y), tsmaller.z);
    float t1 = min(min(tbigger.x, tbigger.y), tbigger.z);
    return vec2(t0, t1);
}

vec4 planetary_rayleigh_shell(vec3 rel_cam, vec3 dir, vec3 sun_dir, float body_h, float thick, vec3 day_tint) {
    if (thick <= 0.01 || body_h <= 0.5) return vec4(0.0);
    float outer = body_h + thick;
    vec2 span = box_slab(rel_cam, dir, outer);
    if (span.y <= 0.0 || span.x > span.y) return vec4(0.0);
    float t_in = max(span.x, 0.0);
    float t_out = span.y;
    vec2 inner = box_slab(rel_cam, dir, body_h);
    if (inner.x <= inner.y && inner.y > 0.0) {
        if (inner.x > 0.0) t_out = min(t_out, inner.x);
        else return vec4(0.0);
    }
    float air_len = max(t_out - t_in, 0.0);
    if (air_len <= 0.001) return vec4(0.0);
    vec3 mid = rel_cam + dir * (0.5 * (t_in + t_out));
    vec3 excess = max(abs(mid) - vec3(body_h), vec3(0.0));
    float alt = length(excess);
    if (alt >= thick) return vec4(0.0);
    float rho = exp(-alt / max(thick * 0.34, 1.0)) * smoothstep(thick, 0.0, alt);
    float tau = clamp(rho * (air_len / max(thick, 1.0)) * 0.90, 0.0, 1.0);
    float mu = dot(normalize(mid), sun_dir);
    float day = smoothstep(-0.12, 0.35, mu);
    float twilight = exp(-pow((mu - 0.02) / 0.18, 2.0)) * smoothstep(-0.32, -0.02, mu);
    vec3 col = (day_tint * day + vec3(1.00, 0.38, 0.12) * (twilight * 0.88)) * (tau * 1.35);
    return vec4(col, clamp(tau * (day + twilight * 0.6), 0.0, 1.0));
}

vec3 polar_aurora(vec3 dir, vec3 cam_pos, vec3 local_up, float aurora_factor, float time) {
    if (aurora_factor <= 0.01 || dir.x >= -0.01) return vec3(0.0);
    float above_horizon = smoothstep(0.0, 0.12, dot(dir, local_up));
    if (above_horizon <= 0.0) return vec3(0.0);
    float t = (-(u_face_half + u_ceiling_altitude) - cam_pos.x) / dir.x;
    if (t <= 0.0) return vec3(0.0);
    vec3 hit = cam_pos + t * dir;
    float face_dist = max(abs(hit.y), abs(hit.z));
    float face_cutoff = smoothstep(u_face_half, u_face_half * 0.85, face_dist);
    if (face_cutoff <= 0.001) return vec3(0.0);
    vec2 p = hit.yz / u_aurora_radius;
    float dist = length(p);
    float ring = smoothstep(0.12, 0.45, dist) * smoothstep(1.35, 0.80, dist);
    if (ring <= 0.005) return vec3(0.0);
    float angle = atan(p.y, p.x);
    float drift = time * 0.20;
    float r1 = abs(dist - 0.90 + sin(angle * 5.0 + drift) * 0.12);
    float c1 = exp(-r1 * 8.0) * (pow(sin(angle * 24.0 + dist * 14.0 + drift * 2.2) * 0.5 + 0.5, 3.0) * 0.8 + 0.2);
    float r2 = abs(dist - 1.20 + sin(angle * 7.0 - drift * 0.7) * 0.14);
    float c2 = exp(-r2 * 10.0) * 0.5;
    vec3 spectral = mix(vec3(0.08, 0.96, 0.46), vec3(0.62, 0.18, 0.92), clamp((dist - 0.75) * 1.5, 0.0, 1.0));
    return spectral * ((c1 + c2) * ring * face_cutoff) * (aurora_factor * above_horizon * 1.8);
}

vec3 subpixel_planet_beacon(vec3 dir, vec3 p_dir, vec3 pole, float half_ang, vec3 col) {
    if (half_ang <= 0.00001 || half_ang > 0.0045 || dot(dir, p_dir) < 0.95) return vec3(0.0);
    vec3 pole_ref = (abs(dot(p_dir, pole)) > 0.96) ? vec3(1.0, 0.0, 0.0) : pole;
    vec3 r_ax = normalize(cross(p_dir, pole_ref));
    vec3 u_ax = cross(r_ax, p_dir);
    vec2 uv = vec2(dot(dir - p_dir, r_ax), dot(dir - p_dir, u_ax));
    float fade_3d = 1.0 - smoothstep(0.0018, 0.0042, half_ang);
    float beacon = clamp(sqrt(half_ang * 1400.0), 0.35, 1.0) * fade_3d;
    float glow = exp(-length(uv) / 0.0018) * (0.88 * beacon);
    float phase = clamp(dot(p_dir, -u_sun_dir) * 0.45 + 0.65, 0.25, 1.0);
    return col * (phase * glow);
}

void main() {
    vec3 dir = normalize(v_ray_dir);
    bool helio_active = u_planet_cubes[7].w > 0.0001;
    vec3 rel_atmo = helio_active ? u_atmo_geom.xyz : v_cam_pos;
    float body_h = helio_active ? u_atmo_geom.w : u_face_half;
    float atmo_thick = helio_active ? u_atmo_tint.w : max(u_face_half * 0.30, 10.0);
    vec3 day_tint = helio_active ? u_atmo_tint.rgb : vec3(0.18, 0.56, 1.00);
    float cam_cheb = max(max(abs(rel_atmo.x), abs(rel_atmo.y)), abs(rel_atmo.z));
    float space_factor = (atmo_thick <= 0.01) ? 1.0 : smoothstep(atmo_thick * 0.25, atmo_thick * 1.15, cam_cheb - body_h);

    float eclipse_occ = 0.0;
    if (u_eclipse_occluder.w > 0.0005 && dot(dir, u_eclipse_occluder.xyz) > 0.92) {
        vec3 occ_d = normalize(u_eclipse_occluder.xyz);
        vec3 occ_r = normalize(cross(occ_d, u_celestial_pole));
        vec3 occ_u = cross(occ_r, occ_d);
        vec2 occ_uv = vec2(dot(dir - occ_d, occ_r), dot(dir - occ_d, occ_u));
        eclipse_occ = 1.0 - smoothstep(u_eclipse_occluder.w * 0.92, u_eclipse_occluder.w, max(abs(occ_uv.x), abs(occ_uv.y)));
    }

    vec3 active_sun = helio_active ? normalize(u_planet_cubes[7].xyz) : normalize(u_sun_dir);
    float s_half = helio_active ? u_planet_cubes[7].w : 0.026;
    vec3 pole_ref = (abs(dot(active_sun, u_celestial_pole)) > 0.96) ? vec3(1.0, 0.0, 0.0) : u_celestial_pole;
    vec3 sun_r = normalize(cross(active_sun, pole_ref));
    vec3 sun_u = cross(sun_r, active_sun);
    vec2 s_raw = vec2(dot(dir - active_sun, sun_r), dot(dir - active_sun, sun_u));
    float sun_cheb = max(abs(s_raw.x), abs(s_raw.y));
    float sun_cube = (!helio_active && dot(dir, active_sun) > 0.72) ? (1.0 - smoothstep(s_half * 0.90, s_half, sun_cheb)) : 0.0;
    float edge_bevel = smoothstep(s_half * 0.68, s_half * 0.90, sun_cheb) * sun_cube;
    float sun_dot = max(dot(dir, active_sun), 0.0);
    float sun_above = mix(smoothstep(-0.02, 0.05, dot(active_sun, u_local_up)), 1.0, space_factor);
    float corona_scale = clamp(s_half / 0.00465, 0.25, 4.5);
    vec3 sun_core = mix(vec3(1.0, 0.95, 0.62), vec3(1.0, 0.72, 0.25), edge_bevel) * sun_cube * 2.8 * (1.0 - eclipse_occ * 0.985);
    vec3 sun_halo = (vec3(1.0, 0.58, 0.14) * pow(sun_dot, 36.0 / sqrt(corona_scale)) * 0.44 + vec3(0.95, 0.34, 0.08) * pow(sun_dot, 9.0) * 0.16) * sqrt(corona_scale) * (1.0 - eclipse_occ * 0.85);
    vec3 sun_color = (sun_core + sun_halo) * sun_above;

    float belt_plane = abs(dot(dir, u_celestial_pole));
    float star_vis = max(u_night_factor, space_factor);
    vec3 belt_dust = vec3(0.42, 0.39, 0.36) * exp(-belt_plane * 26.0) * (0.04 + 0.03 * star_noise(floor(dir * 60.0))) * star_vis;
    vec3 beacons = vec3(0.0);
    if (helio_active && star_vis > 0.02) {
        beacons += subpixel_planet_beacon(dir, u_planet_cubes[0].xyz, u_celestial_pole, u_planet_cubes[0].w, vec3(0.68, 0.65, 0.62));
        beacons += subpixel_planet_beacon(dir, u_planet_cubes[1].xyz, u_celestial_pole, u_planet_cubes[1].w, vec3(0.92, 0.78, 0.46));
        beacons += subpixel_planet_beacon(dir, u_planet_cubes[2].xyz, u_celestial_pole, u_planet_cubes[2].w, vec3(0.85, 0.36, 0.20));
        beacons += subpixel_planet_beacon(dir, u_planet_cubes[3].xyz, u_celestial_pole, u_planet_cubes[3].w, vec3(0.88, 0.70, 0.48));
        beacons += subpixel_planet_beacon(dir, u_planet_cubes[4].xyz, u_celestial_pole, u_planet_cubes[4].w, vec3(0.90, 0.80, 0.56));
        beacons += subpixel_planet_beacon(dir, u_planet_cubes[5].xyz, u_celestial_pole, u_planet_cubes[5].w, vec3(0.56, 0.88, 0.92));
        beacons += subpixel_planet_beacon(dir, u_planet_cubes[6].xyz, u_celestial_pole, u_planet_cubes[6].w, vec3(0.24, 0.46, 0.96));
        beacons *= star_vis;
    }

    vec4 atmo = planetary_rayleigh_shell(rel_atmo, dir, active_sun, body_h, atmo_thick, day_tint) * space_factor;
    float star_occ = (1.0 - eclipse_occ) * (1.0 - sun_cube * sun_above) * (1.0 - atmo.a * 0.85);
    vec3 sky_bg = mix(u_sky_color.rgb, vec3(0.008, 0.012, 0.025), space_factor);
    vec3 cosmos = sky_bg + (vec3(stars(dir) * star_vis * 1.5) + belt_dust) * star_occ + beacons;
    bool on_earth = length(v_cam_pos) < u_face_half * 3.0;
    if (on_earth) {
        cosmos += polar_aurora(dir, v_cam_pos, u_local_up, u_aurora_factor * (1.0 - space_factor), u_time);
    }
    frag_color = vec4(cosmos + atmo.rgb + sun_color * (1.0 - atmo.a * 0.25), 1.0);
}
"#;
