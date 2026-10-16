use std::num::NonZeroU32;
use glow::HasContext;
use glutin::context::PossiblyCurrentContext;
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};
use winit::window::Window;

use engine::{CpuMeshBatch, CpuTronBatch, GraphicPipeline, Mesh, ShaderProgram, ShaderSource};

use crate::avatar::Avatar;
use crate::celestial::CelestialCadence;
use crate::diurnal_pipeline::DiurnalPipeline;

/// Interface for platform window and display surface operations.
pub trait PlatformSurface {
    fn resize(&mut self, width: u32, height: u32);
    fn capture_cursor(&self);
    fn upload_mesh(&self, batch: &CpuMeshBatch) -> Box<dyn Mesh>;
    fn upload_tron_mesh(&self, batch: &CpuTronBatch) -> Box<dyn Mesh>;
    fn compile_shader(&self, source: &dyn ShaderSource) -> Box<dyn ShaderProgram>;
    fn assemble_diurnal_pipeline(&self, cadence: Box<dyn CelestialCadence>) -> Box<dyn GraphicPipeline>;
    fn render_frame(
        &self,
        pipeline: &dyn GraphicPipeline,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        avatar: &dyn Avatar,
    );
}

/// Encapsulated platform window, OpenGL context, and surface implementation.
pub struct GlPlatformSurface {
    window: Window,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    gl: glow::Context,
}

impl GlPlatformSurface {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        window: Window,
        gl_context: PossiblyCurrentContext,
        gl_surface: Surface<WindowSurface>,
        gl: glow::Context,
    ) -> Self {
        Self {
            window,
            gl_context,
            gl_surface,
            gl,
        }
    }

    fn aspect_ratio(&self) -> f32 {
        let size = self.window.inner_size();
        (size.width as f32) / (size.height.max(1) as f32)
    }
}

impl PlatformSurface for GlPlatformSurface {
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.gl_surface.resize(
                &self.gl_context,
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            );
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

    fn upload_mesh(&self, batch: &CpuMeshBatch) -> Box<dyn Mesh> {
        unsafe { batch.upload(&self.gl) }
    }

    fn upload_tron_mesh(&self, batch: &CpuTronBatch) -> Box<dyn Mesh> {
        unsafe { batch.upload(&self.gl) }
    }

    fn compile_shader(&self, source: &dyn ShaderSource) -> Box<dyn ShaderProgram> {
        unsafe { source.compile(&self.gl) }
    }

    fn assemble_diurnal_pipeline(&self, cadence: Box<dyn CelestialCadence>) -> Box<dyn GraphicPipeline> {
        unsafe { Box::new(DiurnalPipeline::gl(cadence, &self.gl)) }
    }

    fn render_frame(
        &self,
        pipeline: &dyn GraphicPipeline,
        shader: &dyn ShaderProgram,
        mesh: &dyn Mesh,
        avatar: &dyn Avatar,
    ) {
        let view_proj = avatar.view_projection_matrix(self.aspect_ratio());
        pipeline.render(&self.gl, shader, mesh, &view_proj, avatar.surface_rotation());
        self.gl_surface.swap_buffers(&self.gl_context).expect("Buffer swap failed");
        self.window.request_redraw();
    }
}

