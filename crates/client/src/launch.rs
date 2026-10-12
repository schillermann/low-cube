use glam::Vec3;
use glow::HasContext;
use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

use world::{BiomedCubeWorld, PlanetaryWorld};
use engine::{CpuMeshBatch, ForwardPipeline, VoxelShaderSource};
use crate::avatar::PlanetaryAvatar;
use crate::controls::KeyMotion;
use crate::physics::PlanetaryPhysics;
use crate::server_physics::ServerPhysics;
use crate::session::{GameSession, Session};
use crate::surface::{GlPlatformSurface, PlatformSurface};

/// Interface for platform window and OpenGL environment setup.
pub trait HostEnvironment {
    fn attach(&self, event_loop: &ActiveEventLoop) -> Box<dyn PlatformSurface>;
}

/// Interface for assembling an active display session.
pub trait SessionAssembly {
    fn assemble(&self, event_loop: &ActiveEventLoop) -> Box<dyn Session>;
}

/// Encapsulated platform window and OpenGL display surface creation.
pub struct GlHostEnvironment {
    title: &'static str,
    width: f64,
    height: f64,
}

impl GlHostEnvironment {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(title: &'static str, width: f64, height: f64) -> Self {
        Self {
            title,
            width,
            height,
        }
    }
}

impl HostEnvironment for GlHostEnvironment {
    fn attach(&self, event_loop: &ActiveEventLoop) -> Box<dyn PlatformSurface> {
        let window_attributes = WindowAttributes::default()
            .with_title(self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));

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

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
        }

        Box::new(GlPlatformSurface::new(window, gl_context, gl_surface, gl))
    }
}

/// Environment assembly mechanism for constructing GameSession.
pub struct SessionLaunch {
    host_env: Box<dyn HostEnvironment>,
    world_blocks: u32,
    physics: Box<dyn PlanetaryPhysics>,
}

impl SessionLaunch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        host_env: Box<dyn HostEnvironment>,
        world_blocks: u32,
        physics: Box<dyn PlanetaryPhysics>,
    ) -> Self {
        Self {
            host_env,
            world_blocks,
            physics,
        }
    }

    /// Secondary constructor initializing with window dimensions and configured server physics.
    pub fn configured(title: &'static str, width: f64, height: f64, world_blocks: u32) -> Self {
        Self::new(
            Box::new(GlHostEnvironment::new(title, width, height)),
            world_blocks,
            Box::new(ServerPhysics::assembled()),
        )
    }
}

impl SessionAssembly for SessionLaunch {
    fn assemble(&self, event_loop: &ActiveEventLoop) -> Box<dyn Session> {
        let surface = self.host_env.attach(event_loop);
        surface.capture_cursor();

        let planet: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::beveled(self.world_blocks, 2.5));

        let mut cpu_buf = CpuMeshBatch::empty();
        planet.emit_surface(&mut cpu_buf);

        let mesh = surface.upload_mesh(&cpu_buf);

        let glsl = VoxelShaderSource::embedded();
        let shader = surface.compile_shader(&glsl);

        let pipeline = Box::new(ForwardPipeline::daylight());

        let half = (self.world_blocks as f32) * 0.5;
        let spawn_position = planet.constrain_to_surface(Vec3::new(0.0, half + 1.0, 0.0), 1.0);
        let flight = self.physics.surface_flight(1.0);
        let avatar = Box::new(PlanetaryAvatar::with_flight(spawn_position, flight));
        let controls = Box::new(KeyMotion::idle());

        Box::new(GameSession::new(
            surface,
            pipeline,
            shader,
            mesh,
            planet,
            avatar,
            controls,
        ))
    }
}
