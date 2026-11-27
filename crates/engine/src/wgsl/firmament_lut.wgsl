struct FirmamentGlobals {
    inv_view_proj: mat4x4<f32>,
    sun_dir_aurora: vec4<f32>,
    pole_time: vec4<f32>,
    cam_pos_half: vec4<f32>,
    canopy_params: vec4<f32>,
    eclipse_occluder: vec4<f32>,
    aurora_lower: vec4<f32>,
    aurora_upper: vec4<f32>,
    planet_cubes: array<vec4<f32>, 8>,
};

@group(0) @binding(0) var<uniform> u_firm: FirmamentGlobals;
@group(0) @binding(1) var t_sky_lut: texture_2d<f32>;
@group(0) @binding(2) var s_sky_lut: sampler;

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) ray_dir: vec3<f32>,
    @location(1) v_cam_pos: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    let corners = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let ndc = corners[vid];
    let far_world = u_firm.inv_view_proj * vec4<f32>(ndc, 1.0, 1.0);
    let near_world = u_firm.inv_view_proj * vec4<f32>(ndc, 0.0, 1.0);
    let dir = far_world.xyz / max(far_world.w, 1e-5) - near_world.xyz / max(near_world.w, 1e-5);

    var out: VsOut;
    out.clip_pos = vec4<f32>(ndc, 0.999999, 1.0);
    out.ray_dir = dir;
    out.v_cam_pos = u_firm.cam_pos_half.xyz;
    return out;
}

fn aabb_slab(ro: vec3<f32>, rd: vec3<f32>, bmin: vec3<f32>, bmax: vec3<f32>) -> vec2<f32> {
    let inv = 1.0 / select(rd, sign(rd) * 1e-5 + vec3<f32>(1e-6), abs(rd) < vec3<f32>(1e-5));
    let t0 = (bmin - ro) * inv;
    let t1 = (bmax - ro) * inv;
    let tmin = min(t0, t1);
    let tmax = max(t0, t1);
    let near_t = max(max(tmin.x, tmin.y), tmin.z);
    let far_t = min(min(tmax.x, tmax.y), tmax.z);
    return vec2<f32>(max(near_t, 0.0), far_t);
}

fn star_hash(p: vec3<f32>) -> f32 {
    let q = fract(p * vec3<f32>(443.897, 441.423, 437.195));
    let d = dot(q, q.yzx + 19.19);
    return fract((q.x + q.y) * q.z + d);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dir = normalize(in.ray_dir);
    let up = normalize(select(vec3<f32>(0.0, 1.0, 0.0), u_firm.canopy_params.xyz, dot(u_firm.canopy_params.xyz, u_firm.canopy_params.xyz) > 1e-4));
    let ref_ax = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(up.y) > 0.92);
    let tang = normalize(cross(ref_ax, up));
    let bitan = normalize(cross(up, tang));

    let sin_el = clamp(dot(dir, up), -1.0, 1.0);
    let az = atan2(dot(dir, bitan), dot(dir, tang));
    let u_lut = fract(az * 0.15915494);
    let v_lut = clamp(0.5 + 0.5 * sign(sin_el) * sqrt(abs(sin_el)), 0.008, 0.992);
    let lut = textureSampleLevel(t_sky_lut, s_sky_lut, vec2<f32>(u_lut, v_lut), 0.0);

    let space_bg = vec3<f32>(0.006, 0.010, 0.022);
    let cell = floor(dir * 210.0);
    let h = star_hash(cell);
    let star_vis = clamp(1.0 - lut.a * 1.35, 0.0, 1.0);
    var stars = vec3<f32>(0.0);
    if (h > 0.9965 && star_vis > 0.01) {
        let br = smoothstep(0.9965, 1.0, h) * star_vis;
        stars = vec3<f32>(0.88, 0.93, 1.0) * (br * 1.35);
    }

    let sun_dir = normalize(select(vec3<f32>(0.0, 1.0, 0.0), u_firm.sun_dir_aurora.xyz, dot(u_firm.sun_dir_aurora.xyz, u_firm.sun_dir_aurora.xyz) > 1e-6));
    let sol = u_firm.planet_cubes[7];
    let sol_dir = select(sun_dir, normalize(sol.xyz), sol.w > 0.0002);
    let sol_ang = select(0.012, sol.w, sol.w > 0.0002);
    let cos_sun = dot(dir, sol_dir);
    let disc = smoothstep(cos(sol_ang * 1.18), cos(sol_ang * 0.88), cos_sun);
    let corona = pow(max(cos_sun, 0.0), 96.0) * 0.32;
    var sun_rgb = vec3<f32>(1.0, 0.95, 0.84) * (disc * 1.3 + corona);

    let occ = u_firm.eclipse_occluder;
    if (occ.w > 0.0004) {
        let occ_hit = smoothstep(cos(occ.w * 1.05), cos(occ.w * 0.95), dot(dir, normalize(occ.xyz)));
        sun_rgb = sun_rgb * (1.0 - occ_hit * 0.96);
    }

    var planets_rgb = vec3<f32>(0.0);
    let tints = array<vec3<f32>, 7>(
        vec3<f32>(0.72, 0.70, 0.68),
        vec3<f32>(0.96, 0.88, 0.68),
        vec3<f32>(0.92, 0.44, 0.28),
        vec3<f32>(0.88, 0.76, 0.60),
        vec3<f32>(0.90, 0.82, 0.58),
        vec3<f32>(0.58, 0.86, 0.92),
        vec3<f32>(0.36, 0.54, 0.94),
    );
    for (var i = 0u; i < 7u; i = i + 1u) {
        let pc = u_firm.planet_cubes[i];
        if (pc.w > 0.00001) {
            let eff_r = max(pc.w, 0.0018);
            let hit = smoothstep(cos(eff_r * 1.25), cos(eff_r * 0.75), dot(dir, normalize(pc.xyz)));
            planets_rgb = planets_rgb + tints[i] * hit * clamp(1.0 - lut.a * 0.85, 0.12, 1.0);
        }
    }

    let u_aurora_factor = u_firm.sun_dir_aurora.w;
    let u_face_half = u_firm.cam_pos_half.w;
    let u_aurora_radius = u_firm.canopy_params.w;
    let u_ceiling_altitude = u_firm.pole_time.w;
    let u_aurora_lower = u_firm.aurora_lower;
    let u_aurora_upper = u_firm.aurora_upper;
    var aurora_rgb = vec3<f32>(0.0);
    if (u_aurora_factor > 0.01 && u_aurora_radius > 0.5) {
        let slab = aabb_slab(
            in.v_cam_pos,
            dir,
            vec3<f32>(-u_face_half - u_ceiling_altitude, -u_aurora_radius, -u_aurora_radius),
            vec3<f32>(-u_face_half - u_ceiling_altitude * 0.25, u_aurora_radius, u_aurora_radius),
        );
        if (slab.y > slab.x) {
            let mid = in.v_cam_pos + dir * ((slab.x + slab.y) * 0.5);
            let wave = 0.5 + 0.5 * sin(mid.z * 0.22 + u_firm.pole_time.z * 0.8 + cos(mid.y * 0.18));
            let curtain = clamp((slab.y - slab.x) / max(u_aurora_radius, 1.0), 0.0, 1.0) * wave * u_aurora_factor;
            aurora_rgb = mix(u_aurora_lower.xyz, u_aurora_upper.xyz, wave) * curtain * 0.55;
        }
    }

    let celestial = (space_bg + stars + sun_rgb + planets_rgb) * (1.0 - lut.a * 0.72) + lut.rgb + aurora_rgb;
    return vec4<f32>(clamp(celestial, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
