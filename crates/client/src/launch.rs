use glam::Vec3;
use winit::event_loop::ActiveEventLoop;

use engine::{
    CpuMeshBatch, CpuTronBatch, ForwardPipeline, TronPipeline, TronShaderSource,
    VoxelShaderSource,
};
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::avatar::PlanetaryAvatar;
use crate::controls::KeyMotion;
use crate::host_env::{GlHostEnvironment, HostEnvironment};
use crate::physics::PlanetaryPhysics;
use crate::server_physics::ServerPhysics;
use crate::session::{GameSession, Session};

/// Interface for assembling an active display session.
pub trait SessionAssembly {
    fn assemble(&self, event_loop: &ActiveEventLoop) -> Box<dyn Session>;
}

/// Environment assembly mechanism for constructing GameSession.
pub struct SessionLaunch {
    host_env: Box<dyn HostEnvironment>,
    world_blocks: u32,
    physics: Box<dyn PlanetaryPhysics>,
    tron_active: bool,
}

impl SessionLaunch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        host_env: Box<dyn HostEnvironment>,
        world_blocks: u32,
        physics: Box<dyn PlanetaryPhysics>,
        tron_active: bool,
    ) -> Self {
        Self {
            host_env,
            world_blocks,
            physics,
            tron_active,
        }
    }

    /// Secondary constructor initializing with window dimensions, physics, and Tron mode state.
    pub fn with_tron(
        title: &'static str,
        width: f64,
        height: f64,
        world_blocks: u32,
        tron_active: bool,
    ) -> Self {
        Self::new(
            Box::new(GlHostEnvironment::new(title, width, height)),
            world_blocks,
            Box::new(ServerPhysics::assembled()),
            tron_active,
        )
    }
}

impl SessionAssembly for SessionLaunch {
    fn assemble(&self, event_loop: &ActiveEventLoop) -> Box<dyn Session> {
        let surface = self.host_env.attach(event_loop);
        surface.capture_cursor();

        let planet: Box<dyn PlanetaryWorld> = if self.tron_active {
            Box::new(BiomedCubeWorld::beveled_tron(self.world_blocks, 1.0))
        } else {
            Box::new(BiomedCubeWorld::beveled(self.world_blocks, 1.0))
        };

        // Primary daylight rendering assets
        let mut cpu_buf = CpuMeshBatch::empty();
        planet.emit_surface(&mut cpu_buf);
        let primary_mesh = surface.upload_mesh(&cpu_buf);
        let primary_shader = surface.compile_shader(&VoxelShaderSource::embedded());
        let primary_pipeline = Box::new(ForwardPipeline::daylight());

        // Tron wireframe rendering assets
        let tron_world = BiomedCubeWorld::beveled_tron(self.world_blocks, 1.0);
        let mut tron_buf = CpuTronBatch::empty();
        tron_world.emit_surface(&mut tron_buf);
        let tron_mesh = surface.upload_tron_mesh(&tron_buf);
        let tron_shader = surface.compile_shader(&TronShaderSource::embedded());
        let tron_pipeline = Box::new(TronPipeline::void());

        let half = (self.world_blocks as f32) * 0.5;
        let spawn_position = planet.constrain_to_surface(Vec3::new(0.0, half + 1.0, 0.0), 1.0);
        let flight = self.physics.surface_flight(1.0);
        let avatar = Box::new(PlanetaryAvatar::with_flight(spawn_position, flight));
        let controls = Box::new(KeyMotion::idle());

        Box::new(GameSession::new(
            surface,
            primary_pipeline,
            primary_shader,
            primary_mesh,
            tron_pipeline,
            tron_shader,
            tron_mesh,
            self.tron_active,
            planet,
            avatar,
            controls,
        ))
    }
}
