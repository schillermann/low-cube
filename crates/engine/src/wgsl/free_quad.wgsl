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
@group(1) @binding(1) var<storage, read> free_quads: array<vec4<f32>>;

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) local_pos: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) color: vec3<f32>,
    @location(4) uv_glow: vec3<f32>,
};

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
    let base = (vid / 6u) * 5u;
    let corner_lut = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
    let c = corner_lut[vid % 6u];
    let glow = free_quads[base + 3u].w;
    if (glow < 0.0 || glow >= 2.0) {
        return vec4<f32>(0.0, 0.0, -2.0, 1.0);
    }
    let pos = free_quads[base + c].xyz;
    return u_scene.light_view_proj * (u_patch.model * vec4<f32>(pos, 1.0));
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    let base = (vid / 6u) * 5u;
    let corner_lut = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
    let c = corner_lut[vid % 6u];
    let c0 = free_quads[base + 0u];
    let c1 = free_quads[base + 1u];
    let c2 = free_quads[base + 2u];
    let c3 = free_quads[base + 3u];
    let col = free_quads[base + 4u].xyz;

    var local_p = c0.xyz;
    if (c == 1u) { local_p = c1.xyz; }
    if (c == 2u) { local_p = c2.xyz; }
    if (c == 3u) { local_p = c3.xyz; }

    let raw_n = vec3<f32>(c0.w, c1.w, c2.w);
    let glow = c3.w;
    let world = u_patch.model * vec4<f32>(local_p, 1.0);
    let xform_n = (u_patch.model * vec4<f32>(raw_n, 0.0)).xyz;
    let eff_n = select(xform_n, raw_n, glow < 0.0 && glow > -1.5);
    let uv = vec2<f32>(
        select(0.0, 1.0, c == 1u || c == 2u),
        select(0.0, 1.0, c >= 2u),
    );

    var out: VsOut;
    out.clip_pos = apply_log_depth(u_scene.view_proj * world);
    out.local_pos = local_p;
    out.world_pos = world.xyz;
    out.normal = eff_n;
    out.color = col;
    out.uv_glow = vec3<f32>(uv.x, uv.y, glow);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let v_uv = in.uv_glow.xy;
    let v_glow = in.uv_glow.z;
    let raw_sun = u_scene.sun_dir_shadow.xyz;
    let sun_dir = normalize(select(vec3<f32>(0.0, 1.0, 0.0), raw_sun, dot(raw_sun, raw_sun) > 1e-6));

    if (v_glow < -7.5) {
        let base_a = clamp(-v_glow - 8.0, 0.01, 0.58);
        return vec4<f32>(in.color, base_a);
    }
    if (v_glow < -5.5) {
        let p = v_uv * 2.0 - 1.0;
        let r2 = dot(p, p);
        if (r2 >= 1.0) { discard; }
        let soft = 1.0 - r2;
        let alpha = clamp(-v_glow - 6.0, 0.02, 0.85) * soft * soft;
        if (alpha < 0.005) { discard; }
        return vec4<f32>(in.color, alpha);
    }
    if (v_glow < -3.5) {
        let p = v_uv * 2.0 - 1.0;
        let r = length(p);
        if (r >= 0.92) { discard; }
        let soft = 1.0 - (r / 0.92) * (r / 0.92);
        let alpha = clamp(-v_glow - 4.0, 0.01, 0.92) * soft * soft;
        if (alpha < 0.004) { discard; }
        return vec4<f32>(in.color, alpha);
    }
    if (v_glow < -1.5) {
        let base_alpha = clamp(-v_glow - 2.0, 0.0, 1.0);
        let axial = clamp(1.0 - 0.72 * v_uv.y * v_uv.y, 0.08, 1.0);
        let mach = 0.88 + 0.12 * cos(v_uv.y * 12.56637);
        let hot = min(in.color * (1.10 + 0.22 * (1.0 - v_uv.y)) * mach, vec3<f32>(1.0));
        return vec4<f32>(hot, clamp(base_alpha * axial, 0.03, 0.90));
    }
    if (v_glow < 0.0) {
        let ramp = clamp((length(in.local_pos.xy) - in.normal.x) / max(in.normal.y, 0.0001), 0.0, 1.0);
        let alpha = clamp(-v_glow * mix(in.normal.z, 1.0, ramp), 0.0, 1.0);
        return vec4<f32>(in.color, alpha);
    }
    if (v_glow >= 2.0) {
        let cam_dist = distance(in.world_pos, u_scene.cam_pos_lights.xyz);
        let dist_fade = clamp((42.0 - cam_dist) * 0.15, 0.0, 1.0);
        if (dist_fade <= 0.01) { discard; }
        let seed = v_glow - 2.0;
        let x = v_uv.x * 9.0 + seed;
        let cell = floor(x);
        let lu = x - cell;
        let h_hash = fract((cell * 0.618034 + seed * 0.381966) * 17.31);
        let max_h = (0.60 + 0.38 * h_hash) * dist_fade;
        if (v_uv.y > max_h) { discard; }
        let t = clamp(v_uv.y / max_h, 0.0, 1.0);
        let center = 0.5 + t * t * 0.44 * (h_hash - 0.5);
        let half_w = max(0.42 * (1.0 - t * (0.65 + 0.35 * t)), 0.04);
        if (abs(lu - center) > half_w) { discard; }
        let ridge = 1.0 - abs(lu - center) / half_w * 0.25;
        let sun_beam = clamp(length(raw_sun), 0.25, 1.0);
        let diff = abs(dot(normalize(in.normal), sun_dir)) * (0.45 * sun_beam) + 0.48;
        let blade = in.color * vec3<f32>(0.75 + 0.42 * t, 0.84 + 0.34 * t, 0.70 + 0.18 * t) * ridge * diff;
        return vec4<f32>(min(blade, vec3<f32>(1.0)), 1.0);
    }

    let norm = normalize(select(vec3<f32>(0.0, 1.0, 0.0), in.normal, dot(in.normal, in.normal) > 1e-6));
    let sun_beam = clamp(length(raw_sun), 0.25, 1.0);
    let ndotl = max(dot(norm, sun_dir), 0.0);
    let ls = u_scene.light_view_proj * vec4<f32>(in.world_pos + norm * 0.045, 1.0);
    let ndc = ls.xyz / max(ls.w, 0.0001);
    let suv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    let in_sb = suv.x >= 0.01 && suv.x <= 0.99 && suv.y >= 0.01 && suv.y <= 0.99 && ndc.z >= 0.0 && ndc.z <= 0.995;
    let raw_sh = textureSampleCompareLevel(t_shadow, s_shadow, clamp(suv, vec2<f32>(0.0), vec2<f32>(1.0)), clamp(ndc.z - 0.0018, 0.0, 1.0));
    let sh = select(1.0, mix(1.0, raw_sh, u_scene.sun_dir_shadow.w), in_sb);

    let count = min(u32(u_scene.cam_pos_lights.w), 16u);
    var pt_acc = vec3<f32>(0.0);
    for (var i = 0u; i < count; i = i + 1u) {
        let pl = u_scene.point_lights[i];
        let to_l = pl.pos_radius.xyz - in.world_pos;
        let rad = max(pl.pos_radius.w, 0.1);
        let d2 = dot(to_l, to_l);
        if (d2 < rad * rad) {
            let d = sqrt(max(d2, 0.0004));
            let f = clamp(1.0 - d / rad, 0.0, 1.0);
            pt_acc = pt_acc + pl.color_intensity.xyz * (pl.color_intensity.w * f * f * max(dot(norm, to_l / d), 0.15));
        }
    }

    let lit = in.color * (vec3<f32>(0.34) + vec3<f32>(0.66 * sun_beam * ndotl * sh)) + in.color * pt_acc;
    let radiant = min(in.color * 1.15, vec3<f32>(1.0));
    let glow_mix = select(0.0, clamp(v_glow, 0.0, 1.0), v_glow >= 0.05);
    return vec4<f32>(mix(lit, radiant, glow_mix), 1.0);
}
