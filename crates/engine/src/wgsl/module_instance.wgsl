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

@group(0) @binding(0) var<uniform> u_scene: SceneGlobals;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;
@group(0) @binding(3) var t_sky_lut: texture_2d<f32>;
@group(0) @binding(4) var s_sky_lut: sampler;

struct VertexIn {
    @location(0) pos_glow: vec4<f32>,
    @location(1) norm_pad: vec4<f32>,
    @location(2) color_pad: vec4<f32>,
    @location(3) inst_col0: vec4<f32>,
    @location(4) inst_col1: vec4<f32>,
    @location(5) inst_col2: vec4<f32>,
    @location(6) inst_col3: vec4<f32>,
    @location(7) inst_meta: vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_norm: vec3<f32>,
    @location(2) color_glow: vec4<f32>,
};

@vertex
fn vs_shadow(in: VertexIn) -> @builtin(position) vec4<f32> {
    let model = mat4x4<f32>(in.inst_col0, in.inst_col1, in.inst_col2, in.inst_col3);
    let world = model * vec4<f32>(in.pos_glow.xyz, 1.0);
    return u_scene.light_view_proj * world;
}

@vertex
fn vs_main(in: VertexIn) -> VsOut {
    let model = mat4x4<f32>(in.inst_col0, in.inst_col1, in.inst_col2, in.inst_col3);
    let world = model * vec4<f32>(in.pos_glow.xyz, 1.0);
    let n_mat = mat3x3<f32>(in.inst_col0.xyz, in.inst_col1.xyz, in.inst_col2.xyz);
    var clip = u_scene.view_proj * world;
    if (u_scene.view_proj[2][3] < -0.5) {
        if (clip.w >= 0.03) {
            let logz = 1.0 + (clip.w - 0.03);
            clip.z = clamp(log2(logz) * 0.030103, 0.0, 0.999999) * clip.w;
        } else {
            clip.z = -1.0;
        }
    }
    var out: VsOut;
    out.clip_pos = clip;
    out.world_pos = world.xyz;
    out.world_norm = normalize(n_mat * in.norm_pad.xyz);
    out.color_glow = vec4<f32>(in.color_pad.xyz * in.inst_meta.yzw, in.pos_glow.w);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let norm = normalize(in.world_norm);
    let raw_sun = u_scene.sun_dir_shadow.xyz;
    let sun_beam = clamp(length(raw_sun), 0.25, 1.0);
    let sun_dir = normalize(select(vec3<f32>(0.0, 1.0, 0.0), raw_sun, dot(raw_sun, raw_sun) > 1e-6));
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

    let col = in.color_glow.xyz;
    let glow = in.color_glow.w;
    let lit = col * (vec3<f32>(0.34) + vec3<f32>(0.66 * sun_beam * ndotl * sh)) + col * pt_acc;
    let radiant = min(col * 1.16, vec3<f32>(1.0));
    return vec4<f32>(mix(lit, radiant, clamp(glow, 0.0, 1.0)), 1.0);
}
