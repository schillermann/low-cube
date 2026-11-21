use glam::{Mat4, Quat, Vec3};
use glow::HasContext;
use engine::{CpuMeshBatch, GraphicPipeline, Mesh, ShaderProgram};
use crate::menu::PreviewStripLayout;

const TILE_W: i32 = 24;
const TILE_H: i32 = 16;
const ATLAS_W: i32 = TILE_W * (PreviewStripLayout::MAX_CAMERAS as i32);

const BLIT_VS: &str = r#"#version 330 core
uniform vec4 u_dst_rect;
uniform vec2 u_uv_span;
out vec2 v_uv;
void main() {
    vec2 c = vec2(
        float(gl_VertexID == 1 || gl_VertexID == 2 || gl_VertexID == 4),
        float(gl_VertexID == 2 || gl_VertexID == 4 || gl_VertexID == 5)
    );
    v_uv = vec2(mix(u_uv_span.x, u_uv_span.y, c.x), c.y);
    gl_Position = vec4(mix(u_dst_rect.xy, u_dst_rect.zw, c), 0.0, 1.0);
}
"#;

const BLIT_FS: &str = r#"#version 330 core
in vec2 v_uv;
uniform sampler2D u_tex;
out vec4 frag_color;
void main() {
    frag_color = vec4(texture(u_tex, v_uv).rgb, 1.0);
}
"#;

#[derive(Clone, Copy)]
struct AtlasTarget {
    fbo: glow::NativeFramebuffer,
    color: glow::NativeTexture,
    depth: glow::NativeRenderbuffer,
}

impl AtlasTarget {
    const fn new(
        fbo: glow::NativeFramebuffer,
        color: glow::NativeTexture,
        depth: glow::NativeRenderbuffer,
    ) -> Self {
        Self { fbo, color, depth }
    }
}

struct AtlasQuadShader {
    program: glow::Program,
    vao: glow::VertexArray,
    u_dst_rect: glow::UniformLocation,
    u_uv_span: glow::UniformLocation,
}

impl AtlasQuadShader {
    const fn new(
        program: glow::Program,
        vao: glow::VertexArray,
        u_dst_rect: glow::UniformLocation,
        u_uv_span: glow::UniformLocation,
    ) -> Self {
        Self { program, vao, u_dst_rect, u_uv_span }
    }

    unsafe fn compiled(gl: &glow::Context) -> Self {
        let vs = gl.create_shader(glow::VERTEX_SHADER).expect("Preview VS create failed");
        gl.shader_source(vs, BLIT_VS);
        gl.compile_shader(vs);
        assert!(gl.get_shader_compile_status(vs), "Preview VS: {}", gl.get_shader_info_log(vs));

        let fs = gl.create_shader(glow::FRAGMENT_SHADER).expect("Preview FS create failed");
        gl.shader_source(fs, BLIT_FS);
        gl.compile_shader(fs);
        assert!(gl.get_shader_compile_status(fs), "Preview FS: {}", gl.get_shader_info_log(fs));

        let program = gl.create_program().expect("Preview program create failed");
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        assert!(gl.get_program_link_status(program), "Preview link: {}", gl.get_program_info_log(program));
        gl.delete_shader(vs);
        gl.delete_shader(fs);

        gl.use_program(Some(program));
        if let Some(u_tex) = gl.get_uniform_location(program, "u_tex") {
            gl.uniform_1_i32(Some(&u_tex), 0);
        }
        let u_dst_rect = gl.get_uniform_location(program, "u_dst_rect").expect("missing u_dst_rect");
        let u_uv_span = gl.get_uniform_location(program, "u_uv_span").expect("missing u_uv_span");
        gl.use_program(None);

        let vao = gl.create_vertex_array().expect("Preview VAO create failed");
        Self::new(program, vao, u_dst_rect, u_uv_span)
    }
}

enum AtlasBuffer {
    Dormant,
    Live(AtlasTarget, AtlasQuadShader),
}

/// Live 3D Intercom Avatar renderer in Slot 0 + round-robin `24×16` coarse GPU camera preview strip in Slots `1..N`.
pub struct PreviewAtlas {
    buffer: AtlasBuffer,
    primed_cams: u8,
    cursor: u8,
}

impl PreviewAtlas {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(buffer: AtlasBuffer, primed_cams: u8, cursor: u8) -> Self {
        Self { buffer, primed_cams, cursor }
    }

    pub const fn dormant() -> Self {
        Self::new(AtlasBuffer::Dormant, 0, 0)
    }

    unsafe fn ensure_live(&mut self, gl: &glow::Context) -> (AtlasTarget, &AtlasQuadShader) {
        if matches!(self.buffer, AtlasBuffer::Dormant) {
            let color = gl.create_texture().expect("Preview atlas texture allocation failed");
            gl.bind_texture(glow::TEXTURE_2D, Some(color));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                ATLAS_W,
                TILE_H,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.bind_texture(glow::TEXTURE_2D, None);

            let depth = gl.create_renderbuffer().expect("Preview atlas depth allocation failed");
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH_COMPONENT24, ATLAS_W, TILE_H);
            gl.bind_renderbuffer(glow::RENDERBUFFER, None);

            let fbo = gl.create_framebuffer().expect("Preview atlas framebuffer allocation failed");
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(color), 0);
            gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, glow::RENDERBUFFER, Some(depth));
            assert_eq!(gl.check_framebuffer_status(glow::FRAMEBUFFER), glow::FRAMEBUFFER_COMPLETE);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            let target = AtlasTarget::new(fbo, color, depth);
            self.buffer = AtlasBuffer::Live(target, AtlasQuadShader::compiled(gl));
        }
        match &self.buffer {
            AtlasBuffer::Live(target, quad_shader) => (*target, quad_shader),
            AtlasBuffer::Dormant => unreachable!(),
        }
    }

    pub fn render_and_blit(
        &mut self,
        gl: &glow::Context,
        (pipeline, shader): (&dyn GraphicPipeline, &dyn ShaderProgram),
        coarse_mesh: &dyn Mesh,
        (cam_views, active_slot): (&[Mat4], u8),
        intercom: Option<(Mat4, &CpuMeshBatch)>,
        surface_rot: Quat,
        viewport: (u32, u32),
    ) {
        let cam_count = cam_views.len().min(PreviewStripLayout::MAX_CAMERAS as usize) as u8;
        if cam_count == 0 {
            self.primed_cams = 0;
            self.cursor = 0;
            return;
        }
        let (win_w, win_h) = (viewport.0.max(1) as i32, viewport.1.max(1) as i32);
        let aspect = (win_w as f32) / (win_h as f32);
        let layout = PreviewStripLayout::new(cam_count + 1, aspect);

        let (start, end) = if self.primed_cams != cam_count {
            self.primed_cams = cam_count;
            self.cursor = 0;
            (0, cam_count)
        } else {
            let mut idx = self.cursor % cam_count;
            if cam_count > 1 && idx + 1 == active_slot {
                idx = (idx + 1) % cam_count;
            }
            self.cursor = (idx + 1) % cam_count;
            (idx, idx + 1)
        };
        unsafe {
            let (target, quad_shader) = self.ensure_live(gl);
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target.fbo));
            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::SCISSOR_TEST);
            for cam_idx in start..end {
                if start + 1 == end && cam_idx + 1 == active_slot {
                    continue;
                }
                let sx = (cam_idx as i32) * TILE_W;
                gl.viewport(sx, 0, TILE_W, TILE_H);
                gl.scissor(sx, 0, TILE_W, TILE_H);
                pipeline.render(gl, shader, coarse_mesh, &cam_views[cam_idx as usize], surface_rot);
            }
            gl.disable(glow::SCISSOR_TEST);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.viewport(0, 0, win_w, win_h);

            let inv_cams = 1.0 / (PreviewStripLayout::MAX_CAMERAS as f32);
            let eps = 0.05 / (ATLAS_W as f32);
            gl.disable(glow::DEPTH_TEST);
            gl.use_program(Some(quad_shader.program));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(target.color));
            gl.bind_vertex_array(Some(quad_shader.vao));
            for cam_idx in 0..cam_count {
                let slot = cam_idx + 1;
                if slot == active_slot {
                    continue;
                }
                let (x0, y0, x1, y1) = layout.slot_bounds(slot);
                let u0 = f32::from(cam_idx) * inv_cams + eps;
                let u1 = f32::from(cam_idx + 1) * inv_cams - eps;
                gl.uniform_4_f32(Some(&quad_shader.u_dst_rect), x0, y0, x1, y1);
                gl.uniform_2_f32(Some(&quad_shader.u_uv_span), u0, u1);
                gl.draw_arrays(glow::TRIANGLES, 0, 6);
            }
            gl.bind_vertex_array(None);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.enable(glow::DEPTH_TEST);

            if let Some((bust_vp, bust_mesh)) = intercom {
                let (x0, y0, x1, y1) = layout.slot_bounds(0);
                let px0 = (((x0 + 1.0) * 0.5) * (win_w as f32)).round() as i32;
                let px1 = (((x1 + 1.0) * 0.5) * (win_w as f32)).round() as i32;
                let py0 = (((y0 + 1.0) * 0.5) * (win_h as f32)).round() as i32;
                let py1 = (((y1 + 1.0) * 0.5) * (win_h as f32)).round() as i32;
                let (pw, ph) = ((px1 - px0).max(1), (py1 - py0).max(1));
                gl.enable(glow::SCISSOR_TEST);
                gl.viewport(px0, py0, pw, ph);
                gl.scissor(px0, py0, pw, ph);
                gl.clear_color(0.085, 0.125, 0.175, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                shader.bind(gl, &bust_vp, Vec3::new(0.46, 0.52, 0.72).normalize());
                bust_mesh.draw(gl);
                gl.disable(glow::SCISSOR_TEST);
                gl.viewport(0, 0, win_w, win_h);
            }
            let _ = target.depth;
        }
    }
}
