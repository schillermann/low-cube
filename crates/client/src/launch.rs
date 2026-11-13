use glam::Vec3;
use winit::event_loop::ActiveEventLoop;

use engine::{
    CpuMeshBatch, CpuTronBatch, PlanetaryMesh, TronPipeline, TronShaderSource,
    VoxelShaderSource,
};
use world::PlanetaryWorld;
use crate::attire::{AttireProfile, FileAttire, HeldRelic};
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::CalibratedCanopy;
use crate::avatar::PlanetaryAvatar;
use crate::celestial::CelestialCadence;
use crate::controls::{BindingProfile, FileBindings, KeyMotion};
use crate::host_env::{GlHostEnvironment, HostEnvironment};
use crate::menu::EscapeMenu;
use crate::physics::PlanetaryPhysics;
use crate::portal_config::PortalConfig;
use crate::server_aurora::ServerAurora;
use crate::server_celestial::ServerCelestial;
use crate::server_physics::ServerPhysics;
use crate::server_satchel::ServerSatchel;
use crate::session::{GameSession, Session};

/// Interface for assembling an active display session.
pub trait SessionAssembly {
    fn assemble(&self, event_loop: &ActiveEventLoop) -> Box<dyn Session>;
}

struct LaunchCosmos {
    world_blocks: u32,
    physics: Box<dyn PlanetaryPhysics>,
    celestial: Box<dyn CelestialCadence>,
    aurora: Box<dyn AuroraCadence>,
}

/// Environment assembly mechanism for constructing GameSession.
pub struct SessionLaunch {
    host_env: Box<dyn HostEnvironment>,
    cosmos: LaunchCosmos,
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
            cosmos: LaunchCosmos { world_blocks, physics, celestial, aurora },
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

        let planet: Box<dyn PlanetaryWorld> =
            PortalConfig::from_process().assemble_planet(self.cosmos.world_blocks, self.tron_active);
        let initial_ticket = planet.portal_ticket().unwrap_or_default();

        let half = (self.cosmos.world_blocks as f32) * 0.5;
        let spawn_position = planet.constrain_to_surface(Vec3::new(0.25, half + 1.0, 0.25), 1.0);
        let flight = self.cosmos.physics.surface_flight(1.0);
        let attire = FileAttire::profile()
            .load_or_squire()
            .with_hand_relic(1.0, HeldRelic::CantileverCrane);
        let bindings = FileBindings::profile().load_or_seeded();
        let avatar = Box::new(
            PlanetaryAvatar::with_attire_and_bindings(
                spawn_position,
                flight,
                attire,
                bindings,
            )
            .with_starting_satchel(&ServerSatchel::assembled()),
        );

        let slice_count = planet.face_count() * 2;
        let mut primary_faces = Vec::with_capacity(slice_count + 1);
        let mut tron_faces = Vec::with_capacity(slice_count + 1);
        for idx in 0..slice_count {
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
        let mut meadow_buf = CpuMeshBatch::empty();
        if !self.tron_active {
            use crate::avatar::Avatar;
            avatar.emit_surroundings(planet.as_ref(), &mut meadow_buf);
        }
        primary_faces.push(surface.upload_mesh(&meadow_buf));
        tron_faces.push(surface.upload_tron_mesh(&CpuTronBatch::empty()));

        let initial_mask = planet.visible_slice_mask(spawn_position);
        let primary_mesh = PlanetaryMesh::with_mask(primary_faces, initial_mask);
        let primary_shader = surface.compile_shader(&VoxelShaderSource::embedded());
        let canopy = Box::new(CalibratedCanopy::planetary(self.cosmos.world_blocks));
        let primary_pipeline = surface.assemble_diurnal_pipeline(
            self.cosmos.celestial.clone_cadence(),
            self.cosmos.aurora.clone_cadence(),
            canopy,
        );

        let tron_mesh = PlanetaryMesh::with_mask(tron_faces, initial_mask);
        let tron_shader = surface.compile_shader(&TronShaderSource::embedded());
        let tron_pipeline = Box::new(TronPipeline::void());
        let controls = Box::new(KeyMotion::with_bindings(bindings));
        let modal = Box::new(EscapeMenu::with_portal(initial_ticket));

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
