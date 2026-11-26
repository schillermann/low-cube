use winit::event_loop::ActiveEventLoop;

use engine::{CpuMeshBatch, PlanetaryMesh, VoxelShaderSource};
use world::{PlanetaryWorld, SolarProportions};
use crate::attire::{AttireProfile, FileAttire, HeldRelic};
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::CalibratedCanopy;
use crate::avatar::PlanetaryAvatar;
use crate::black_box::{BlackBoxAvatar, FileBlackBox};
use crate::celestial::CelestialCadence;
use crate::controls::{BindingProfile, FileBindings, SprintControls};
use crate::host_env::{GlHostEnvironment, HostEnvironment};
use crate::menu::{
    AutoPunchModal, EscapeMenu, InspectionModal, PortalInvite, ProximityChatModal, SolarChartModal,
};
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
    proportions: SolarProportions,
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
        proportions: SolarProportions,
        physics: Box<dyn PlanetaryPhysics>,
        celestial: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
    ) -> Self {
        Self {
            host_env,
            cosmos: LaunchCosmos { proportions, physics, celestial, aurora },
        }
    }

    /// Secondary constructor initializing with window dimensions and scalable solar proportions.
    pub fn windowed_cosmos(
        title: &'static str,
        width: f64,
        height: f64,
        proportions: SolarProportions,
    ) -> Self {
        let celestial = Box::new(ServerCelestial::assembled());
        let aurora = Box::new(ServerAurora::assembled(celestial.duration_seconds()));
        Self::new(
            Box::new(GlHostEnvironment::new(title, width, height)),
            proportions,
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

        let world_blocks = self.cosmos.proportions.earth_blocks();
        let planet: Box<dyn PlanetaryWorld> =
            PortalConfig::from_process().assemble_planet(self.cosmos.proportions);
        let initial_invite = PortalInvite::hosted(
            planet.portal_phrase().unwrap_or_default(),
            planet.portal_ticket().unwrap_or_default(),
        );

        let spawn_position = crate::avatar::BodySpan::landed_on(planet.as_ref()).current();
        let flight = self.cosmos.physics.surface_flight(1.0);
        let attire = FileAttire::profile()
            .load_or_squire()
            .with_hand_relic(1.0, HeldRelic::CantileverCrane);
        let bindings = FileBindings::profile().load_or_seeded();
        let mut embodied = PlanetaryAvatar::with_attire_and_bindings(
            spawn_position,
            flight,
            attire,
            bindings,
        )
        .with_starting_satchel(&ServerSatchel::assembled());
        if planet.assigned_survivor().is_some() {
            use crate::avatar::Avatar as _;
            embodied.land_on_world(planet.as_ref());
        }
        let avatar = Box::new(BlackBoxAvatar::wrapping(
            Box::new(embodied),
            FileBlackBox::profile().load_tape(),
        ));

        let slice_count = planet.face_count() * 2;
        let initial_mask = planet.visible_slice_mask(spawn_position);
        let mut primary_faces: Vec<Box<dyn engine::Mesh>> = Vec::with_capacity(slice_count + 1);
        let empty_cpu = CpuMeshBatch::empty();
        let mut patch_buf = CpuMeshBatch::with_quad_capacity(2_048);
        for idx in 0..slice_count {
            if (initial_mask >> idx.min(31)) & 1 == 0 {
                primary_faces.push(surface.upload_mesh(&empty_cpu));
                continue;
            }
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
        let mut meadow_patches: Vec<Box<dyn engine::Mesh>> =
            (0..world::MEADOW_RING_SLOTS).map(|_| surface.upload_mesh(&empty_cpu)).collect();
        while let Some(slot) = planet.drain_stale_meadow_patch(spawn_position, &mut initial_ring, &mut patch_buf) {
            meadow_patches[slot] = surface.upload_mesh(&patch_buf);
            patch_buf.clear();
        }
        primary_faces.push(Box::new(engine::SectorGridMesh::new(meadow_patches)));

        let primary_mesh = PlanetaryMesh::with_horizon(primary_faces, initial_mask, initial_mask);
        let primary_shader = surface.compile_shader(&VoxelShaderSource::embedded());
        let canopy = Box::new(CalibratedCanopy::planetary(world_blocks));
        let primary_pipeline = surface.assemble_diurnal_pipeline(
            self.cosmos.celestial.clone_cadence(),
            self.cosmos.aurora.clone_cadence(),
            canopy,
        );

        let controls = Box::new(SprintControls::with_bindings(bindings));
        let modal = Box::new(ProximityChatModal::wrapping(SolarChartModal::with_bindings(
            AutoPunchModal::wrapping(
                InspectionModal::new(EscapeMenu::with_clipboards(
                    initial_invite.clone(),
                    surface.attach_clipboard(),
                    surface.attach_clipboard(),
                )),
                initial_invite,
                surface.attach_clipboard(),
            ),
            bindings,
        )));
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
