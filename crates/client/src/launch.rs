use glam::Vec3;
use winit::event_loop::ActiveEventLoop;

use engine::{CpuMeshBatch, PlanetaryMesh, StarryPipeline, VoxelShaderSource};
use world::{PlanetaryWorld, StarVault};
use crate::attire::{AttireProfile, FileAttire, HeldRelic};
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::CalibratedCanopy;
use crate::avatar::PlanetaryAvatar;
use crate::black_box::{BlackBoxAvatar, FileBlackBox};
use crate::celestial::CelestialCadence;
use crate::controls::{BindingProfile, FileBindings, KeyMotion};
use crate::host_env::{GlHostEnvironment, HostEnvironment};
use crate::menu::{AutoPunchModal, EscapeMenu, PortalInvite};
use crate::physics::PlanetaryPhysics;
use crate::portal_config::PortalConfig;
use crate::server_aurora::ServerAurora;
use crate::server_celestial::ServerCelestial;
use crate::server_physics::ServerPhysics;
use crate::server_satchel::ServerSatchel;
use crate::session::{
    GameSession, PolaroidScenePass, Session, SessionDisplay, SessionStage, ShadedScenePass,
};

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
}

impl SessionLaunch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        host_env: Box<dyn HostEnvironment>,
        world_blocks: u32,
        physics: Box<dyn PlanetaryPhysics>,
        celestial: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
    ) -> Self {
        Self {
            host_env,
            cosmos: LaunchCosmos { world_blocks, physics, celestial, aurora },
        }
    }

    /// Secondary constructor initializing with window dimensions and planetary scale.
    pub fn windowed(
        title: &'static str,
        width: f64,
        height: f64,
        world_blocks: u32,
    ) -> Self {
        let celestial = Box::new(ServerCelestial::assembled());
        let aurora = Box::new(ServerAurora::assembled(celestial.duration_seconds()));
        Self::new(
            Box::new(GlHostEnvironment::new(title, width, height)),
            world_blocks,
            Box::new(ServerPhysics::assembled()),
            celestial,
            aurora,
        )
    }
}

impl SessionAssembly for SessionLaunch {
    fn assemble(&self, event_loop: &ActiveEventLoop) -> Box<dyn Session> {
        let surface = self.host_env.attach(event_loop);
        surface.capture_cursor();
        surface.request_redraw();

        let planet: Box<dyn PlanetaryWorld> =
            PortalConfig::from_process().assemble_planet(self.cosmos.world_blocks);
        let initial_invite = PortalInvite::hosted(
            planet.portal_phrase().unwrap_or_default(),
            planet.portal_ticket().unwrap_or_default(),
        );

        let half = (self.cosmos.world_blocks as f32) * 0.5;
        let spawn_position = planet.constrain_to_surface(Vec3::new(0.25, half + 1.0, 0.25), 1.0);
        let flight = self.cosmos.physics.surface_flight(1.0);
        let attire = FileAttire::profile()
            .load_or_squire()
            .with_hand_relic(1.0, HeldRelic::CantileverCrane);
        let bindings = FileBindings::profile().load_or_seeded();
        let embodied = PlanetaryAvatar::with_attire_and_bindings(
            spawn_position,
            flight,
            attire,
            bindings,
        )
        .with_starting_satchel(&ServerSatchel::assembled());
        let avatar = Box::new(BlackBoxAvatar::wrapping(
            Box::new(embodied),
            FileBlackBox::profile().load_tape(),
        ));

        let slice_count = planet.face_count() * 2;
        let mut primary_faces: Vec<Box<dyn engine::Mesh>> = Vec::with_capacity(slice_count + 1);
        let mut patch_buf = CpuMeshBatch::with_quad_capacity(2_048);
        for idx in 0..slice_count {
            let patch_count = planet.face_patch_count(idx);
            if patch_count > 0 {
                let mut patches = Vec::with_capacity(patch_count);
                for slot in 0..patch_count {
                    patch_buf.clear();
                    planet.emit_face_patch(idx, slot, &mut patch_buf);
                    patches.push(surface.upload_mesh(&patch_buf));
                }
                primary_faces.push(Box::new(engine::SectorGridMesh::new(patches)));
            } else {
                let mut cpu_buf = CpuMeshBatch::empty();
                planet.emit_face(idx, &mut cpu_buf);
                primary_faces.push(surface.upload_mesh(&cpu_buf));
            }
        }
        let mut initial_ring = world::MeadowRing::cold();
        let empty_cpu = CpuMeshBatch::empty();
        let mut meadow_patches: Vec<Box<dyn engine::Mesh>> =
            (0..world::MEADOW_RING_SLOTS).map(|_| surface.upload_mesh(&empty_cpu)).collect();
        while let Some(slot) = planet.drain_stale_meadow_patch(spawn_position, &mut initial_ring, &mut patch_buf) {
            meadow_patches[slot] = surface.upload_mesh(&patch_buf);
            patch_buf.clear();
        }
        primary_faces.push(Box::new(engine::SectorGridMesh::new(meadow_patches)));

        let initial_mask = planet.visible_slice_mask(spawn_position);
        let primary_mesh = PlanetaryMesh::with_mask(primary_faces, initial_mask);
        let primary_shader = surface.compile_shader(&VoxelShaderSource::embedded());
        let canopy = Box::new(CalibratedCanopy::planetary(self.cosmos.world_blocks));
        let mut star_cpu = CpuMeshBatch::with_quad_capacity(384);
        StarVault::cosmic().emit(&mut star_cpu);
        let star_mesh = surface.upload_mesh(&star_cpu);
        let primary_pipeline = Box::new(StarryPipeline::new(
            surface.assemble_diurnal_pipeline(
                self.cosmos.celestial.clone_cadence(),
                self.cosmos.aurora.clone_cadence(),
                canopy,
            ),
            star_mesh,
        ));

        let controls = Box::new(KeyMotion::with_bindings(bindings));
        let modal = Box::new(AutoPunchModal::wrapping(
            EscapeMenu::with_clipboards(
                initial_invite.clone(),
                surface.attach_clipboard(),
                surface.attach_clipboard(),
            ),
            initial_invite,
            surface.attach_clipboard(),
        ));
        let shaded_pass = PolaroidScenePass::new(ShadedScenePass::new(
            (primary_pipeline, primary_shader),
            primary_mesh,
            engine::StreamingMesh::with_quad_capacity(4_096),
            initial_ring,
        ));
        let display = SessionDisplay::new(surface, Box::new(shaded_pass));
        let stage = SessionStage::new(planet, avatar, controls, modal);

        Box::new(GameSession::new(stage, display))
    }
}
