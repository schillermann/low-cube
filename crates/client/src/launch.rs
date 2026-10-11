use glam::{Vec3, Vec4};
use glow::HasContext;
use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

use world::{PlanetaryWorld, StandardCubeWorld};
use engine_render::{CpuMeshBatch, ForwardPipeline, GlslPipeline, GpuAllocation};
use crate::avatar::PlanetaryAvatar;
use crate::controls::KeyState;
use crate::session::GameSession;

/// Environment assembly mechanism for constructing GameSession.
pub struct SessionLaunch;

impl SessionLaunch {
    pub fn new() -> Self {
        Self
    }

    pub fn assemble(&self, event_loop: &ActiveEventLoop) -> GameSession {
        let window_attributes = WindowAttributes::default()
            .with_title("Low-Cube [WASD: Move, Mouse: Look, I: Invert Y, O: Invert X, Esc: Exit]")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

        let template = ConfigTemplateBuilder::new().with_alpha_size(8);
        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .expect("Failed to build GL config");

        let window = window.expect("Failed to get window");
        let raw_window_handle = window.window_handle().ok().map(|h| h.as_raw());

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version::new(3, 3))))
            .build(raw_window_handle);

        let not_current_gl_context = unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attributes)
                .expect("Failed to create context")
        };

        let attrs = window.build_surface_attributes(Default::default()).expect("Failed to build surface attributes");
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .expect("Failed to create surface")
        };

        let gl_context = not_current_gl_context
            .make_current(&gl_surface)
            .expect("Failed to make context current");

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|s| {
                gl_config.display().get_proc_address(s)
            })
        };

        let planet: Box<dyn PlanetaryWorld> = Box::new(StandardCubeWorld::build(16));

        let mut cpu_buf = CpuMeshBatch::new();
        planet.emit_surface(&mut cpu_buf);

        let allocation = GpuAllocation::new();
        let mesh = unsafe { Box::new(allocation.upload(&gl, &cpu_buf)) };

        let glsl = GlslPipeline::new();
        let shader = unsafe { Box::new(glsl.compile_voxel_program(&gl)) };

        let pipeline = Box::new(ForwardPipeline::new(
            Vec4::new(0.04, 0.05, 0.08, 1.0),
            Vec3::new(0.6, 0.8, 0.4),
        ));

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
        }

        let _ = window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
        window.set_cursor_visible(false);

        let avatar = Box::new(PlanetaryAvatar::new(Vec3::new(0.0, 9.0, 0.0)));
        let controls = Box::new(KeyState::new());

        GameSession::new(
            window,
            gl_context,
            gl_surface,
            gl,
            pipeline,
            shader,
            mesh,
            planet,
            avatar,
            controls,
        )
    }
}
