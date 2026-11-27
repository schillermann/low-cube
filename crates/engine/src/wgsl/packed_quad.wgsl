struct PointLight {
    pos_radius: vec4<f32>,
    color_intensity: vec4<f32>,
};

struct SceneGlobals {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    sun_dir_shadow: vec4<f32>,
    cam_pos_lights: vec4<f32>,
    up_axis_haze: vec4<f32>,
    point_lights: array<PointLight, 16>,
};

struct PatchFrame {
    model: mat4x4<f32>,
    origin_step: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u_scene: SceneGlobals;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;
@group(0) @binding(3) var t_sky_lut: texture_2d<f32>;
@group(0) @binding(4) var s_sky_lut: sampler;

@group(1) @binding(0) var<uniform> u_patch: PatchFrame;
@group(1) @binding(1) var<storage, read> packed_quads: array<vec2<u32>>;

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_norm: vec3<f32>,
    @location(2) base_col: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) @interpolate(flat) ao_tag: u32,
};

fn decode_quad_vertex(vid: u32, out_norm: ptr<function, vec3<f32>>, out_uv: ptr<function, vec2<f32>>, out_col: ptr<function, vec3<f32>>, out_ao_tag: ptr<function, u32>) -> vec3<f32> {
    let q = packed_quads[vid / 6u];
    let corner_lut = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
    let corner = corner_lut[vid % 6u];
    let cu = select(0.0, 1.0, corner == 1u || corner == 2u);
    let cv = select(0.0, 1.0, corner == 2u || corner == 3u);
    *out_uv = vec2<f32>(cu, cv);

    let qx = f32(q.x & 0xFFu);
    let qy = f32((q.x >> 8u) & 0xFFu);
    let qz = f32((q.x >> 16u) & 0xFFu);
    let n_idx = (q.x >> 24u) & 7u;
    let w_steps = f32(((q.x >> 27u) & 31u) + 1u);
    let h_steps = f32((q.y & 31u) + 1u);
    *out_ao_tag = (q.y >> 5u) & 0x7FFu;

    let rgb = (q.y >> 16u) & 0xFFFFu;
    let r = f32((rgb >> 11u) & 31u) / 31.0;
    let g = f32((rgb >> 5u) & 63u) / 63.0;
    let b = f32(rgb & 31u) / 31.0;
    *out_col = vec3<f32>(r, g, b);

    var au = vec3<f32>(0.0, 1.0, 0.0);
    var av = vec3<f32>(1.0, 0.0, 0.0);
    var n = vec3<f32>(0.0, 0.0, -1.0);
    switch n_idx {
        case 0u: { au = vec3<f32>(0.0, 1.0, 0.0); av = vec3<f32>(0.0, 0.0, 1.0); n = vec3<f32>(1.0, 0.0, 0.0); }
        case 1u: { au = vec3<f32>(0.0, 0.0, 1.0); av = vec3<f32>(0.0, 1.0, 0.0); n = vec3<f32>(-1.0, 0.0, 0.0); }
        case 2u: { au = vec3<f32>(0.0, 0.0, 1.0); av = vec3<f32>(1.0, 0.0, 0.0); n = vec3<f32>(0.0, 1.0, 0.0); }
        case 3u: { au = vec3<f32>(1.0, 0.0, 0.0); av = vec3<f32>(0.0, 0.0, 1.0); n = vec3<f32>(0.0, -1.0, 0.0); }
        case 4u: { au = vec3<f32>(1.0, 0.0, 0.0); av = vec3<f32>(0.0, 1.0, 0.0); n = vec3<f32>(0.0, 0.0, 1.0); }
        default: { au = vec3<f32>(0.0, 1.0, 0.0); av = vec3<f32>(1.0, 0.0, 0.0); n = vec3<f32>(0.0, 0.0, -1.0); }
    }
    *out_norm = n;
    let step = u_patch.origin_step.w;
    let base_pos = u_patch.origin_step.xyz + vec3<f32>(qx, qy, qz) * step;
    return base_pos + au * (cu * w_steps * step) + av * (cv * h_steps * step);
}

fn apply_log_depth(clip_in: vec4<f32>) -> vec4<f32> {
    var clip = clip_in;
    if (u_scene.view_proj[2][3] < -0.5) {
        if (clip.w >= 0.03) {
            let logz = 1.0 + (clip.w - 0.03);
            clip.z = clamp(log2(logz) * 0.030103, 0.0, 0.999999) * clip.w;
        } else {
            clip.z = -1.0;
        }
    }
    return clip;
}

@vertex
fn vs_shadow(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4<f32> {
    var n = vec3<f32>(0.0);
    var uv = vec2<f32>(0.0);
    var col = vec3<f32>(0.0);
    var ao_tag = 0u;
    let local_pos = decode_quad_vertex(vid, &n, &uv, &col, &ao_tag);
    let world = u_patch.model * vec4<f32>(local_pos, 1.0);
    return u_scene.light_view_proj * world;
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var n = vec3<f32>(0.0);
    var uv = vec2<f32>(0.0);
    var col = vec3<f32>(0.0);
    var ao_tag = 0u;
    let local_pos = decode_quad_vertex(vid, &n, &uv, &col, &ao_tag);
    let world = u_patch.model * vec4<f32>(local_pos, 1.0);
    let world_n = normalize((u_patch.model * vec4<f32>(n, 0.0)).xyz);

    var out: VsOut;
    out.clip_pos = apply_log_depth(u_scene.view_proj * world);
    out.world_pos = world.xyz;
    out.world_norm = world_n;
    out.base_col = col;
    out.uv = uv;
    out.ao_tag = ao_tag;
    return out;
}

fn ao_level(code: u32) -> f32 {
    return 0.44 + f32(code & 3u) * 0.1866667;
}

fn sample_sun_shadow(world_pos: vec3<f32>, world_norm: vec3<f32>) -> f32 {
    let strength = u_scene.sun_dir_shadow.w;
    if (strength <= 0.01) {
        return 1.0;
    }
    let biased = vec4<f32>(world_pos + world_norm * 0.045, 1.0);
    let ls = u_scene.light_view_proj * biased;
    let ndc = ls.xyz / max(ls.w, 0.0001);
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    let in_bounds = uv.x >= 0.01 && uv.x <= 0.99 && uv.y >= 0.01 && uv.y <= 0.99 && ndc.z >= 0.0 && ndc.z <= 0.995;
    let raw = textureSampleCompareLevel(t_shadow, s_shadow, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)), clamp(ndc.z - 0.0018, 0.0, 1.0));
    let edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    let fade = smoothstep(0.01, 0.08, edge) * strength;
    return select(1.0, mix(1.0, raw, fade), in_bounds);
}

fn evaluate_point_lights(world_pos: vec3<f32>, norm: vec3<f32>) -> vec3<f32> {
    let count = min(u32(u_scene.cam_pos_lights.w), 16u);
    var acc = vec3<f32>(0.0);
    for (var i = 0u; i < count; i = i + 1u) {
        let pl = u_scene.point_lights[i];
        let to_l = pl.pos_radius.xyz - world_pos;
        let dist_sq = dot(to_l, to_l);
        let rad = max(pl.pos_radius.w, 0.1);
        if (dist_sq < rad * rad) {
            let dist = sqrt(max(dist_sq, 0.0004));
            let falloff = clamp(1.0 - dist / rad, 0.0, 1.0);
            let ndotl = max(dot(norm, to_l / dist), 0.15);
            acc = acc + pl.color_intensity.xyz * (pl.color_intensity.w * falloff * falloff * ndotl);
        }
    }
    return acc;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let ao_pack = in.ao_tag & 0xFFu;
    let tag = (in.ao_tag >> 8u) & 7u;
    let ao00 = ao_level(ao_pack);
    let ao10 = ao_level(ao_pack >> 2u);
    let ao11 = ao_level(ao_pack >> 4u);
    let ao01 = ao_level(ao_pack >> 6u);
    let ao = mix(mix(ao00, ao10, in.uv.x), mix(ao01, ao11, in.uv.x), in.uv.y);

    var albedo = in.base_col;
    let norm = normalize(in.world_norm);
    if (tag == 1u) {
        let wp = in.world_pos.xz;
        let w = sin(wp.x * 3.8 + wp.y * 2.7) + cos(wp.x * 2.1 - wp.y * 4.3);
        let scale = select(0.92, 1.08, w > 0.15);
        albedo = clamp(albedo * scale, vec3<f32>(0.0), vec3<f32>(1.0));
    }

    let raw_sun = u_scene.sun_dir_shadow.xyz;
    let sun_beam = clamp(length(raw_sun), 0.25, 1.0);
    let sun_dir = normalize(select(vec3<f32>(0.0, 1.0, 0.0), raw_sun, dot(raw_sun, raw_sun) > 1e-6));
    let ndotl = max(dot(norm, sun_dir), 0.0);
    let shadow = sample_sun_shadow(in.world_pos, norm);
    let shade = 1.0 - sun_beam;
    let amb = mix(vec3<f32>(0.34), vec3<f32>(0.21, 0.24, 0.30), shade) * ao;
    let direct = vec3<f32>(0.66 * sun_beam * ndotl * shadow);
    let pt_light = evaluate_point_lights(in.world_pos, norm);
    var lit = albedo * (amb + direct) + albedo * pt_light;

    let cam_vec = in.world_pos - u_scene.cam_pos_lights.xyz;
    let cam_dist = length(cam_vec);
    let haze_k = u_scene.up_axis_haze.w * smoothstep(55.0, 260.0, cam_dist) * 0.42;
    if (haze_k > 0.004 && tag < 2u) {
        let up = normalize(select(vec3<f32>(0.0, 1.0, 0.0), u_scene.up_axis_haze.xyz, dot(u_scene.up_axis_haze.xyz, u_scene.up_axis_haze.xyz) > 1e-4));
        let view_dir = cam_vec / max(cam_dist, 0.001);
        let sin_el = clamp(dot(view_dir, up), -1.0, 1.0);
        let lut_v = clamp(0.5 + 0.5 * sign(sin_el) * sqrt(abs(sin_el)), 0.01, 0.99);
        let sky_sample = textureSampleLevel(t_sky_lut, s_sky_lut, vec2<f32>(0.5, lut_v), 0.0);
        lit = mix(lit, sky_sample.rgb, haze_k * sky_sample.a);
    }

    if (tag == 3u) {
        return vec4<f32>(min(albedo * 1.16, vec3<f32>(1.0)), 1.0);
    }
    if (tag == 2u) {
        return vec4<f32>(mix(lit, min(albedo * 1.12, vec3<f32>(1.0)), 0.55), 1.0);
    }
    return vec4<f32>(min(lit, vec3<f32>(1.0)), 1.0);
}
