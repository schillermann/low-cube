mod ionosphere;
mod rayleigh;

use self::ionosphere::ionosphere_aurora_glsl;
use self::rayleigh::rayleigh_shell_glsl;

const FIRMAMENT_VERTEX_SRC: &str = r#"#version 330 core
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

const FIRMAMENT_FRAGMENT_SRC: &str = concat!(
    r#"#version 330 core
in vec3 v_ray_dir;
in vec3 v_cam_pos;

uniform vec3 u_sun_dir;
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
uniform vec4 u_aurora_lower;
uniform vec4 u_aurora_upper;

out vec4 frag_color;
"#,
    rayleigh_shell_glsl!(),
    ionosphere_aurora_glsl!(),
    r#"
vec3 subpixel_planet_beacon(vec3 dir, vec3 p_dir, vec3 pole, float half_ang, vec3 col) {
    if (half_ang <= 0.00001 || half_ang > 0.0045 || dot(dir, p_dir) < 0.95) {
        return vec3(0.0);
    }
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
    vec2 vault_hit = box_slab(rel_atmo, dir, body_h * 0.50);
    if (body_h > 0.5 && cam_cheb < body_h + atmo_thick && vault_hit.x > 0.0 && vault_hit.x <= vault_hit.y) {
        frag_color = vec4(0.008, 0.012, 0.025, 1.0);
        return;
    }
    float atmo_fade = (atmo_thick <= 0.01) ? 1.0 : smoothstep(atmo_thick * 0.55, atmo_thick * 1.02, cam_cheb - body_h);
    vec3 cam_up = plateau_normal(rel_atmo);

    float eclipse_occ = 0.0;
    if (u_eclipse_occluder.w > 0.0005 && dot(dir, u_eclipse_occluder.xyz) > 0.92) {
        vec3 occ_d = normalize(u_eclipse_occluder.xyz);
        vec3 occ_pole = (abs(dot(occ_d, u_celestial_pole)) > 0.96) ? vec3(1.0, 0.0, 0.0) : u_celestial_pole;
        vec3 occ_r = normalize(cross(occ_d, occ_pole));
        vec3 occ_u = cross(occ_r, occ_d);
        vec2 occ_uv = vec2(dot(dir - occ_d, occ_r), dot(dir - occ_d, occ_u));
        eclipse_occ = 1.0 - smoothstep(u_eclipse_occluder.w * 0.92, u_eclipse_occluder.w, max(abs(occ_uv.x), abs(occ_uv.y)));
    }

    vec3 active_sun = helio_active ? normalize(u_planet_cubes[7].xyz) : normalize(u_sun_dir);
    float s_half = helio_active ? u_planet_cubes[7].w : 0.026;
    float flux = helio_active ? clamp(s_half / 0.00465, 0.55, 1.45) : 1.0;
    float space_blend = (atmo_thick <= 0.01) ? 1.0 : smoothstep(0.35, 0.95, (cam_cheb - body_h) / atmo_thick);
    float low_sun_gate = mix(1.0 - smoothstep(0.12, 0.42, dot(cam_up, active_sun)), 1.0, space_blend);
    vec4 atmo = planetary_rayleigh_shell(rel_atmo, dir, active_sun, body_h, atmo_thick, day_tint, flux, low_sun_gate);

    vec3 pole_ref = (abs(dot(active_sun, u_celestial_pole)) > 0.96) ? vec3(1.0, 0.0, 0.0) : u_celestial_pole;
    vec3 sun_r = normalize(cross(active_sun, pole_ref));
    vec3 sun_u = cross(sun_r, active_sun);
    vec2 s_raw = vec2(dot(dir - active_sun, sun_r), dot(dir - active_sun, sun_u));
    float sun_cheb = max(abs(s_raw.x), abs(s_raw.y));
    float sun_cube = (!helio_active && dot(dir, active_sun) > 0.72) ? (1.0 - smoothstep(s_half * 0.90, s_half, sun_cheb)) : 0.0;
    float edge_bevel = smoothstep(s_half * 0.68, s_half * 0.90, sun_cheb) * sun_cube;
    float sun_dot = max(dot(dir, active_sun), 0.0);
    float core_h = max(body_h - 2.0, body_h * 0.85);
    vec2 core_hit = box_slab(rel_atmo, dir, core_h);
    float core_open = (body_h > 0.5 && core_hit.x > 0.0 && core_hit.x <= core_hit.y) ? 0.0 : 1.0;
    float halo_horizon = mix(smoothstep(-0.38, -0.04, dot(active_sun, cam_up)), 1.0, atmo_fade);
    float corona_scale = clamp(s_half / 0.00465, 0.25, 4.5);
    float clear_sky = helio_active ? clamp(max(day_tint.r, max(day_tint.g, day_tint.b)) / 0.55, 0.14, 1.0) : 1.0;
    float halo_att = mix(1.0, clear_sky, atmo.a);
    vec3 sun_core = mix(vec3(1.0, 0.95, 0.62), vec3(1.0, 0.72, 0.25), edge_bevel) * sun_cube * 2.8 * (1.0 - eclipse_occ * 0.985);
    vec3 sun_halo = (vec3(1.0, 0.58, 0.14) * pow(sun_dot, 36.0 / sqrt(corona_scale)) * 0.44 + vec3(0.95, 0.34, 0.08) * pow(sun_dot, 9.0) * 0.16) * sqrt(corona_scale) * halo_att * (1.0 - eclipse_occ * 0.85);
    vec3 sun_color = sun_core * core_open + sun_halo * (core_open * halo_horizon);

    float sky_lum = max(atmo.r, max(atmo.g, atmo.b));
    float lit_day_gate = 1.0 - smoothstep(0.35, 0.62, atmo.a);
    float star_vis = (1.0 - clamp(sky_lum / 0.36, 0.0, 1.0)) * lit_day_gate;
    float belt_plane = helio_active ? abs(dir.y) : abs(dot(dir, u_celestial_pole));
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

    float star_occ = (1.0 - eclipse_occ) * (1.0 - sun_cube * core_open) * (1.0 - atmo.a * 0.88);
    vec3 cosmos = vec3(0.008, 0.012, 0.025) * (1.0 - atmo.a) + (vec3(stars(dir) * star_vis * 1.5) + belt_dust) * star_occ + beacons;
    float peri_surge = helio_active ? clamp(s_half / 0.00465, 0.75, 1.55) : 1.0;
    float storm_boost = peri_surge * (1.0 + 0.45 * eclipse_occ);
    cosmos += polar_aurora(
        dir,
        rel_atmo,
        cam_up,
        active_sun,
        body_h,
        u_aurora_factor * (1.0 - atmo.a * 0.85),
        storm_boost,
        eclipse_occ,
        u_time
    );
    frag_color = vec4(cosmos + atmo.rgb + sun_color * (1.0 - atmo.a * 0.25), 1.0);
}
"#
);

/// Focused contract delivering vertex and fragment GLSL sources to a shader consumer.
pub trait GlslSourcePair {
    fn transfer_sources<R>(&self, sink: impl FnOnce(&'static str, &'static str) -> R) -> R;
}

/// Embedded GLSL source pair for the celestial firmament vault (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct FirmamentGlslPair {
    vertex: &'static str,
    fragment: &'static str,
}

impl FirmamentGlslPair {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(vertex: &'static str, fragment: &'static str) -> Self {
        Self { vertex, fragment }
    }

    pub const fn embedded() -> Self {
        Self::new(FIRMAMENT_VERTEX_SRC, FIRMAMENT_FRAGMENT_SRC)
    }
}

impl GlslSourcePair for FirmamentGlslPair {
    fn transfer_sources<R>(&self, sink: impl FnOnce(&'static str, &'static str) -> R) -> R {
        sink(self.vertex, self.fragment)
    }
}
