macro_rules! rayleigh_shell_glsl {
    () => {
        r#"
vec3 plateau_normal(vec3 p) {
    if (dot(p, p) <= 1e-6) {
        return vec3(0.0, 1.0, 0.0);
    }
    vec3 n = normalize(p);
    vec3 q2 = n * n;
    return normalize(n * q2 * q2);
}

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
    vec3 sgn = step(vec3(0.0), dir) * 2.0 - 1.0;
    vec3 inv_d = sgn / max(abs(dir), vec3(1e-6));
    vec3 t0s = (-vec3(half_ext) - orig) * inv_d;
    vec3 t1s = (vec3(half_ext) - orig) * inv_d;
    vec3 tsmaller = min(t0s, t1s);
    vec3 tbigger = max(t0s, t1s);
    float t0 = max(max(tsmaller.x, tsmaller.y), tsmaller.z);
    float t1 = min(min(tbigger.x, tbigger.y), tbigger.z);
    return vec2(t0, t1);
}

vec4 planetary_rayleigh_shell(vec3 rel_cam, vec3 dir, vec3 sun_dir, float body_h, float thick, vec3 day_tint, float flux, float low_sun_gate) {
    if (thick <= 0.01 || body_h <= 0.5) {
        return vec4(0.0);
    }
    float outer = body_h + thick;
    vec2 span = box_slab(rel_cam, dir, outer);
    if (span.y <= 0.0 || span.x > span.y) {
        return vec4(0.0);
    }
    float t_in = max(span.x, 0.0);
    float t_out = span.y;
    float core_h = max(body_h - 2.0, body_h * 0.85);
    vec2 inner = box_slab(rel_cam, dir, core_h);
    if (inner.x <= inner.y && inner.y > 0.0) {
        if (inner.x > 0.0) {
            t_out = min(t_out, inner.x);
        } else {
            t_in = max(t_in, inner.y);
        }
    }
    float air_len = max(t_out - t_in, 0.0);
    if (air_len <= 0.001) {
        return vec4(0.0);
    }
    float dt = air_len * 0.25;
    float inv_thick = 1.0 / thick;
    float sunward = smoothstep(-0.42, 0.25, dot(dir, sun_dir));
    float col_opt = 0.0;
    vec3 weighted_tint = vec3(0.0);
    float lit_weight = 0.0;
    for (int i = 0; i < 4; i++) {
        vec3 p = rel_cam + dir * (t_in + dt * (float(i) + 0.5));
        float cheb = max(max(abs(p.x), abs(p.y)), abs(p.z));
        float h = clamp((cheb - body_h) / thick, 0.0, 1.0);
        float rim_sheen = 0.22 * exp(-pow((h - 0.88) / 0.07, 2.0));
        float rho = (exp(-h * 1.35) + rim_sheen) * smoothstep(1.0, 0.92, h);
        float d_opt = rho * (dt * inv_thick);
        float mu = dot(plateau_normal(p), sun_dir);
        float day = smoothstep(-0.14, 0.30, mu);
        float arch = exp(-pow((mu + 0.04) / 0.18, 2.0)) * smoothstep(-0.36, -0.02, mu);
        float twilight = exp(-pow((mu - 0.02) / 0.18, 2.0)) * smoothstep(-0.42, 0.04, mu) * sunward * low_sun_gate;
        vec3 local_col = day_tint * day + vec3(0.05, 0.16, 0.44) * (arch * 0.52) + vec3(1.00, 0.38, 0.12) * (twilight * 1.05);
        col_opt += d_opt;
        weighted_tint += local_col * d_opt;
        lit_weight += max(day, max(arch * 0.35, twilight * 0.70)) * d_opt;
    }
    if (col_opt <= 0.0001) {
        return vec4(0.0);
    }
    float tau = 1.0 - exp(-col_opt * 2.5);
    vec3 mean_col = weighted_tint / col_opt;
    float mean_lit = clamp(lit_weight / col_opt, 0.0, 1.0);
    float lit_tau = clamp(tau * mean_lit, 0.0, 1.0);
    float horizon_lift = clamp(lit_tau - 0.62, 0.0, 0.38) * 0.45;
    vec3 col = (mean_col * tau + vec3(0.10, 0.14, 0.18) * (horizon_lift * lit_tau)) * (1.42 * flux);
    return vec4(min(col, vec3(1.0)), lit_tau);
}
"#
    };
}

pub(super) use rayleigh_shell_glsl;
