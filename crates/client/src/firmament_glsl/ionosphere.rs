macro_rules! ionosphere_aurora_glsl {
    () => {
        r#"
vec2 aabb_slab(vec3 orig, vec3 dir, vec3 b_min, vec3 b_max) {
    vec3 sgn = step(vec3(0.0), dir) * 2.0 - 1.0;
    vec3 inv_d = sgn / max(abs(dir), vec3(1e-6));
    vec3 t0s = (b_min - orig) * inv_d;
    vec3 t1s = (b_max - orig) * inv_d;
    vec3 tsmaller = min(t0s, t1s);
    vec3 tbigger = max(t0s, t1s);
    float t0 = max(max(tsmaller.x, tsmaller.y), tsmaller.z);
    float t1 = min(min(tbigger.x, tbigger.y), tbigger.z);
    return vec2(t0, t1);
}

vec3 polar_aurora(
    vec3 dir,
    vec3 cam_pos,
    vec3 cam_up,
    vec3 active_sun,
    float body_h,
    float aurora_factor,
    float storm_boost,
    float eclipse_occ,
    float time
) {
    float strength = u_aurora_lower.w;
    if (strength <= 0.001 || aurora_factor <= 0.01 || body_h <= 0.5) {
        return vec3(0.0);
    }
    float scale_ratio = (u_face_half > 0.5) ? (body_h / u_face_half) : 1.0;
    float flare = u_aurora_upper.w;
    float oval_half = max(u_aurora_radius * scale_ratio * 1.10, body_h * 0.66);
    float iono_base = max(u_ceiling_altitude * scale_ratio * 1.15, body_h * 0.44);
    float iono_thick = max(iono_base * 1.10, body_h * 0.52);
    float iono_top = iono_base + iono_thick;

    float lat_bound = body_h * (1.0 + (iono_top / body_h) * flare);
    vec3 b_min = vec3(-(body_h + iono_top), -lat_bound, -lat_bound);
    vec3 b_max = vec3(-(body_h + iono_base * 0.85), lat_bound, lat_bound);
    vec2 span = aabb_slab(cam_pos, dir, b_min, b_max);
    if (span.y <= 0.0 || span.x > span.y) {
        return vec3(0.0);
    }
    float t_in = max(span.x, 0.0);
    float t_out = span.y;
    float core_h = max(body_h - 1.5, body_h * 0.92);
    vec2 crust = box_slab(cam_pos, dir, core_h);
    if (crust.x <= crust.y && crust.y > 0.0) {
        if (crust.x > 0.0) {
            t_out = min(t_out, crust.x);
        } else {
            t_in = max(t_in, crust.y);
        }
    }
    float seg_len = t_out - t_in;
    if (seg_len <= 0.01) {
        return vec3(0.0);
    }

    float cam_cheb = max(max(abs(cam_pos.x), abs(cam_pos.y)), abs(cam_pos.z));
    float ground_prox = 1.0 - smoothstep(body_h + 2.0, body_h + iono_base * 0.60, cam_cheb);
    float horizon_gate = mix(1.0, smoothstep(-0.04, 0.08, dot(dir, cam_up)), ground_prox);
    if (horizon_gate <= 0.001) {
        return vec3(0.0);
    }

    float midnight_tail = clamp(active_sun.x, 0.0, 1.0);
    float drift = time * (0.18 + 0.06 * midnight_tail);
    float dt = seg_len * 0.20;
    float d_weight = min(dt / iono_thick, 0.35);
    float crimson_gain = clamp(storm_boost - 1.0, 0.0, 0.85);
    vec3 accum = vec3(0.0);

    for (int i = 0; i < 5; i++) {
        vec3 p = cam_pos + dir * (t_in + dt * (float(i) + 0.5));
        float alt = -p.x - body_h;
        float h_norm = clamp((alt - iono_base) / iono_thick, 0.0, 1.0);
        float vert_env = smoothstep(0.0, 0.16, h_norm) * smoothstep(1.0, 0.55, h_norm);
        float mu_sun = dot(plateau_normal(p), active_sun) * (1.0 - eclipse_occ * 0.95);
        float local_dark = 1.0 - smoothstep(-0.22, 0.08, mu_sun);
        if (vert_env * local_dark <= 0.002) {
            continue;
        }
        float flare_scale = 1.0 + (max(alt, 0.0) / body_h) * flare;
        vec2 foot_yz = p.yz / flare_scale;
        float face_cheb = max(abs(foot_yz.x), abs(foot_yz.y)) / body_h;
        float face_gate = 1.0 - smoothstep(0.80, 0.98, face_cheb);
        if (face_gate <= 0.001) {
            continue;
        }
        vec2 q = foot_yz / oval_half;
        vec2 q2 = q * q;
        float dist_l4 = pow(q2.x * q2.x + q2.y * q2.y, 0.25);
        float oval_band = smoothstep(0.20, 0.46, dist_l4) * (1.0 - smoothstep(1.02, 1.44, dist_l4));
        if (oval_band <= 0.002) {
            continue;
        }
        float angle = atan(q.y, q.x);
        float wave1 = sin(angle * 4.0 + drift) * 0.09 + cos(angle * 8.0 - drift * 0.65) * 0.04;
        float wave2 = sin(angle * 4.0 - drift * 0.80 + 1.1) * 0.10;
        float pillars = pow(sin(angle * 24.0 + dist_l4 * 14.0 + drift * 2.2) * 0.5 + 0.5, 2.5) * 0.78 + 0.22;
        float c1 = exp(-abs(dist_l4 - 0.72 + wave1) * 8.5) * pillars;
        float c2 = exp(-abs(dist_l4 - 1.12 + wave2) * 6.8) * (0.58 * pillars + 0.20);
        float alt_mix = smoothstep(0.18, 0.78, h_norm + (dist_l4 - 0.75) * 0.22);
        vec3 storm_tips = vec3(0.95, 0.16, 0.44) * (smoothstep(0.68, 0.98, h_norm) * crimson_gain);
        vec3 spectral = mix(u_aurora_lower.rgb, u_aurora_upper.rgb, alt_mix) + storm_tips;
        accum += spectral * ((c1 + c2) * oval_band * face_gate * vert_env * local_dark * d_weight);
    }

    float tail_surge = 1.0 + 0.28 * midnight_tail;
    return accum * (aurora_factor * strength * storm_boost * tail_surge * horizon_gate * 2.6);
}
"#
    };
}

pub(super) use ionosphere_aurora_glsl;
