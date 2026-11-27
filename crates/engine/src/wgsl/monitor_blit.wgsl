struct MonitorTile {
    dst_rect: vec4<f32>,
    uv_span_phosphor: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u_tile: MonitorTile;
@group(0) @binding(1) var t_atlas: texture_2d<f32>;
@group(0) @binding(2) var s_atlas: sampler;

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) local_uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    let cx = select(0.0, 1.0, vid == 1u || vid == 2u || vid == 4u);
    let cy = select(0.0, 1.0, vid == 2u || vid == 4u || vid == 5u);
    let c = vec2<f32>(cx, cy);

    var out: VsOut;
    out.clip_pos = vec4<f32>(mix(u_tile.dst_rect.xy, u_tile.dst_rect.zw, c), 0.0, 1.0);
    out.uv = vec2<f32>(mix(u_tile.uv_span_phosphor.x, u_tile.uv_span_phosphor.y, cx), 1.0 - cy);
    out.local_uv = c;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let raw = textureSampleLevel(t_atlas, s_atlas, in.uv, 0.0).rgb;
    let scanline = 0.93 + 0.07 * cos(in.local_uv.y * 160.0 * 3.14159265);
    let lum = dot(raw, vec3<f32>(0.299, 0.587, 0.114));
    let phosphor = vec3<f32>(0.18, 0.96, 0.68) * (lum * 0.92 + 0.04);
    let mixed = mix(raw, phosphor, u_tile.uv_span_phosphor.z) * scanline;
    let border = min(min(in.local_uv.x, 1.0 - in.local_uv.x), min(in.local_uv.y, 1.0 - in.local_uv.y));
    let bezel = select(vec3<f32>(0.12, 0.42, 0.34), mixed, border > 0.018);
    return vec4<f32>(bezel, 1.0);
}
