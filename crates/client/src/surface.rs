use std::cell::RefCell;
use std::num::NonZeroU32;
use glow::HasContext;
use glutin::context::PossiblyCurrentContext;
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};
use glam::Vec3;
use winit::window::Window;

use engine::{CpuMeshBatch, GraphicPipeline, Mesh, ShaderProgram, ShaderSource};

use crate::attire::SightMode;
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::AuroraCanopy;
use crate::avatar::Avatar;
use crate::celestial::CelestialCadence;
use crate::diurnal_pipeline::{DiurnalStageAssembly, GlDiurnalStage};
use crate::menu::{PreviewStripLayout, ScreenPoint, SessionModal, SystemClipboard, TextClipboard};
use crate::photo_plate::PhotoPlate;
use crate::preview_strip::PreviewAtlas;

/// Interface for platform window and display surface operations.
pub trait PlatformSurface {
    fn resize(&mut self, width: u32, height: u32);
    fn capture_cursor(&self);
    fn release_cursor(&self);
    fn normalize_cursor(&self, x: f64, y: f64) -> ScreenPoint;
    fn attach_clipboard(&self) -> Box<dyn TextClipboard> {
        SystemClipboard::new().detached()
    }
    fn upload_mesh(&self, batch: &CpuMeshBatch) -> Box<dyn Mesh>;
    fn replace_mesh(&self, _previous: &dyn Mesh, batch: &CpuMeshBatch) -> Box<dyn Mesh> {
        self.upload_mesh(batch)
    }
    fn compile_shader(&self, source: &dyn ShaderSource) -> Box<dyn ShaderProgram>;
    fn assemble_diurnal_pipeline(
        &self,
        cadence: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        canopy: Box<dyn AuroraCanopy>,
    ) -> Box<dyn GraphicPipeline>;
    fn render_frame(
        &self,
        pipeline: &dyn GraphicPipeline,
        shader: &dyn ShaderProgram,
        scenes: (&dyn Mesh, &dyn Mesh),
        avatar: &dyn Avatar,
        modal: &dyn SessionModal,
    );
    fn present_frame(&self) {}
    fn request_redraw(&self) {}
    fn capture_sight_frame(
        &self,
        _pipeline: &dyn GraphicPipeline,
        _shader: &dyn ShaderProgram,
        _mesh: &dyn Mesh,
        _avatar: &dyn Avatar,
        _sight: SightMode,
        _angle_label: &str,
    ) -> Vec<Vec3> {
        Vec::new()
    }
}

struct GlSwapSurface {
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
}

impl GlSwapSurface {
    const fn new(context: PossiblyCurrentContext, surface: Surface<WindowSurface>) -> Self {
        Self { context, surface }
    }

    fn resize(&self, width: NonZeroU32, height: NonZeroU32) {
        self.surface.resize(&self.context, width, height);
    }

    fn swap(&self) {
        self.surface.swap_buffers(&self.context).expect("Buffer swap failed");
    }
}

/// Encapsulated platform window, OpenGL context, swap surface, and coarse camera preview atlas.
pub struct GlPlatformSurface {
    window: Window,
    swap: GlSwapSurface,
    gl: glow::Context,
    previews: RefCell<PreviewAtlas>,
}

impl GlPlatformSurface {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn with_atlas(
        window: Window,
        swap: GlSwapSurface,
        gl: glow::Context,
        previews: RefCell<PreviewAtlas>,
    ) -> Self {
        Self { window, swap, gl, previews }
    }

    /// Secondary constructor assembling swap surface and dormant preview atlas.
    pub fn new(
        window: Window,
        gl_context: PossiblyCurrentContext,
        gl_surface: Surface<WindowSurface>,
        gl: glow::Context,
    ) -> Self {
        Self::with_atlas(
            window,
            GlSwapSurface::new(gl_context, gl_surface),
            gl,
            RefCell::new(PreviewAtlas::dormant()),
        )
    }

    fn aspect_ratio(&self) -> f32 {
        let size = self.window.inner_size();
        (size.width as f32) / (size.height.max(1) as f32)
    }
}

impl PlatformSurface for GlPlatformSurface {
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.swap.resize(NonZeroU32::new(width).unwrap(), NonZeroU32::new(height).unwrap());
            unsafe {
                self.gl.viewport(0, 0, width as i32, height as i32);
            }
        }
    }

    fn capture_cursor(&self) {
        let _ = self.window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_| self.window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
        self.window.set_cursor_visible(false);
    }

    fn release_cursor(&self) {
        let _ = self.window.set_cursor_grab(winit::window::CursorGrabMode::None);
        self.window.set_cursor_visible(true);
    }

    fn normalize_cursor(&self, x: f64, y: f64) -> ScreenPoint {
        let size = self.window.inner_size();
        let w = size.width.max(1) as f32;
        let h = size.height.max(1) as f32;
        ScreenPoint::new((x as f32 / w) * 2.0 - 1.0, 1.0 - (y as f32 / h) * 2.0)
    }

    fn attach_clipboard(&self) -> Box<dyn TextClipboard> {
        SystemClipboard::new().attach_window(&self.window)
    }

    fn upload_mesh(&self, batch: &CpuMeshBatch) -> Box<dyn Mesh> {
        unsafe { batch.upload(&self.gl) }
    }

    fn replace_mesh(&self, previous: &dyn Mesh, batch: &CpuMeshBatch) -> Box<dyn Mesh> {
        previous.release(&self.gl);
        unsafe { batch.upload(&self.gl) }
    }

    fn compile_shader(&self, source: &dyn ShaderSource) -> Box<dyn ShaderProgram> {
        unsafe { source.compile(&self.gl) }
    }

    fn assemble_diurnal_pipeline(
        &self,
        cadence: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        canopy: Box<dyn AuroraCanopy>,
    ) -> Box<dyn GraphicPipeline> {
        unsafe { GlDiurnalStage::new(cadence, aurora, canopy).assemble_gl(&self.gl) }
    }

    fn render_frame(
        &self,
        pipeline: &dyn GraphicPipeline,
        shader: &dyn ShaderProgram,
        (mesh, coarse_mesh): (&dyn Mesh, &dyn Mesh),
        avatar: &dyn Avatar,
        modal: &dyn SessionModal,
    ) {
        let size = self.window.inner_size();
        let aspect = self.aspect_ratio();
        let view_proj = avatar.view_projection_matrix(aspect);
        pipeline.render(&self.gl, shader, mesh, &view_proj);
        if !modal.open() {
            let mut views = Vec::new();
            let active_slot = avatar.collect_camera_previews(PreviewStripLayout::TILE_ASPECT, &mut views);
            let mut intercom_batch = CpuMeshBatch::empty();
            let intercom = avatar
                .emit_intercom_bust(PreviewStripLayout::TILE_ASPECT, &mut intercom_batch)
                .map(|vp| (vp, &intercom_batch));
            self.previews.borrow_mut().render_and_blit(
                &self.gl,
                (pipeline, shader),
                coarse_mesh,
                (&views, active_slot),
                intercom,
                (size.width, size.height),
            );
        }
        modal.draw_layer(&self.gl, shader, avatar, aspect);
    }

    fn present_frame(&self) {
        self.swap.swap();
    }

    fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn capture_sight_frame(
        &self,
        pipeline: &dyn GraphicPipeline,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        avatar: &dyn Avatar,
        sight: SightMode,
        angle_label: &str,
    ) -> Vec<Vec3> {
        let size = self.window.inner_size();
        let (width, height) = (size.width.max(1), size.height.max(1));
        let aspect = (width as f32) / (height as f32);
        let view_proj = avatar.view_projection_for_sight(aspect, sight);
        pipeline.render(&self.gl, shader, mesh, &view_proj);
        let mut raw_rgba = vec![0u8; (width as usize) * (height as usize) * 4];
        unsafe {
            self.gl.read_pixels(
                0,
                0,
                width as i32,
                height as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut raw_rgba)),
            );
        }
        let plate = PhotoPlate::new(width, height, raw_rgba);
        let _ = plate.develop_to_disk(angle_label);
        plate.sample_mosaic()
    }
}
