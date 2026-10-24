use glam::Vec3;
use winit::event_loop::ActiveEventLoop;

use engine::{
    CpuMeshBatch, CpuTronBatch, PlanetaryMesh, TronPipeline, TronShaderSource,
    VoxelShaderSource,
};
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::attire::{AttireProfile, FileAttire};
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::CalibratedCanopy;
use crate::avatar::PlanetaryAvatar;
use crate::celestial::CelestialCadence;
use crate::controls::{BindingProfile, FileBindings, KeyMotion};
use crate::host_env::{GlHostEnvironment, HostEnvironment};
use crate::menu::EscapeMenu;
use crate::physics::PlanetaryPhysics;
use crate::server_aurora::ServerAurora;
use crate::server_celestial::ServerCelestial;
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
    celestial: Box<dyn CelestialCadence>,
    aurora: Box<dyn AuroraCadence>,
    tron_active: bool,
}

impl SessionLaunch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        host_env: Box<dyn HostEnvironment>,
        world_blocks: u32,
        physics: Box<dyn PlanetaryPhysics>,
        celestial: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        tron_active: bool,
    ) -> Self {
        Self {
            host_env,
            world_blocks,
            physics,
            celestial,
            aurora,
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
        let celestial = Box::new(ServerCelestial::assembled());
        let aurora = Box::new(ServerAurora::assembled(celestial.duration_seconds()));
        Self::new(
            Box::new(GlHostEnvironment::new(title, width, height)),
            world_blocks,
            Box::new(ServerPhysics::assembled()),
            celestial,
            aurora,
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

        // Primary diurnal daylight and nocturnal rendering pipeline (1 GPU mesh per planetary face)
        let face_count = planet.face_count();
        let mut primary_faces = Vec::with_capacity(face_count);
        let mut tron_faces = Vec::with_capacity(face_count);
        for idx in 0..face_count {
            let mut cpu_buf = CpuMeshBatch::empty();
            if !self.tron_active {
                planet.emit_face(idx, &mut cpu_buf);
            }
            primary_faces.push(surface.upload_mesh(&cpu_buf));

            let mut tron_buf = CpuTronBatch::empty();
            if self.tron_active {
                planet.emit_face(idx, &mut tron_buf);
            }
            tron_faces.push(surface.upload_tron_mesh(&tron_buf));
        }
        let primary_mesh = PlanetaryMesh::new(primary_faces);
        let primary_shader = surface.compile_shader(&VoxelShaderSource::embedded());
        let canopy = Box::new(CalibratedCanopy::planetary(self.world_blocks));
        let primary_pipeline = surface.assemble_diurnal_pipeline(
            self.celestial.clone_cadence(),
            self.aurora.clone_cadence(),
            canopy,
        );

        let tron_mesh = PlanetaryMesh::new(tron_faces);
        let tron_shader = surface.compile_shader(&TronShaderSource::embedded());
        let tron_pipeline = Box::new(TronPipeline::void());

        let half = (self.world_blocks as f32) * 0.5;
        let spawn_position = planet.constrain_to_surface(Vec3::new(0.25, half + 1.0, 0.25), 1.0);
        let flight = self.physics.surface_flight(1.0);
        let attire = FileAttire::profile().load_or_squire();
        let bindings = FileBindings::profile().load_or_seeded();
        let avatar = Box::new(PlanetaryAvatar::with_attire_and_bindings(
            spawn_position,
            flight,
            attire,
            bindings,
        ));
        let controls = Box::new(KeyMotion::with_bindings(bindings));
        let modal = Box::new(EscapeMenu::closed());

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
            modal,
        ))
    }
}
