use engine::{CompositeMesh, CpuMeshBatch, CpuTronBatch, GraphicPipeline, PlanetaryMesh, ShaderProgram};
use world::PlanetaryWorld;
use crate::attire::PhotoShutter;
use crate::avatar::Avatar;
use crate::clock::{MonotonicTime, SystemClock};
use crate::surface::PlatformSurface;
use super::stage::SessionStage;

/// Encapsulated monotonic frame delta tracker decoupling redraw steps from static clock calls.
pub(super) struct FramePace {
    clock: Box<dyn MonotonicTime>,
    last_tick: std::time::Instant,
}

impl FramePace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(clock: Box<dyn MonotonicTime>, last_tick: std::time::Instant) -> Self {
        Self { clock, last_tick }
    }

    /// Secondary constructor anchored at the current monotonic instant.
    pub(super) fn realtime() -> Self {
        let clock = Box::new(SystemClock::new());
        let now = clock.current_instant();
        Self::new(clock, now)
    }

    pub(super) fn advance(&mut self) -> f32 {
        let now = self.clock.current_instant();
        let dt = now.duration_since(self.last_tick).as_secs_f32().clamp(1.0 / 240.0, 0.10);
        self.last_tick = now;
        dt
    }
}

/// Encapsulated graphics pipeline, shader program, and GPU mesh slices for a render mode.
pub(super) struct RenderPass {
    pipeline: Box<dyn GraphicPipeline>,
    shader: Box<dyn ShaderProgram>,
    mesh: PlanetaryMesh,
}

impl RenderPass {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(pipeline: Box<dyn GraphicPipeline>, shader: Box<dyn ShaderProgram>, mesh: PlanetaryMesh) -> Self {
        Self { pipeline, shader, mesh }
    }
}

/// Encapsulated platform window surface and dual primary/Tron rendering passes.
pub(super) struct SessionDisplay {
    surface: Box<dyn PlatformSurface>,
    primary: RenderPass,
    tron: RenderPass,
    tron_active: bool,
}

impl SessionDisplay {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(surface: Box<dyn PlatformSurface>, primary: RenderPass, tron: RenderPass, tron_active: bool) -> Self {
        Self { surface, primary, tron, tron_active }
    }

    pub(super) fn surface(&self) -> &dyn PlatformSurface {
        self.surface.as_ref()
    }

    pub(super) fn resize(&mut self, width: u32, height: u32) {
        self.surface.resize(width, height);
    }

    pub(super) fn capture_cursor(&self) {
        self.surface.capture_cursor();
    }

    pub(super) fn refresh_face_slice(&mut self, world: &dyn PlanetaryWorld, idx: usize) {
        if self.tron_active {
            let mut tron_buf = CpuTronBatch::empty();
            world.emit_face(idx, &mut tron_buf);
            let next = self.surface.replace_tron_mesh(self.tron.mesh.slice_at(idx), &tron_buf);
            self.tron.mesh.store_slice(idx, next);
        } else {
            let mut cpu_buf = CpuMeshBatch::with_quad_capacity(65_536);
            world.emit_face(idx, &mut cpu_buf);
            let next = self.surface.replace_mesh(self.primary.mesh.slice_at(idx), &cpu_buf);
            self.primary.mesh.store_slice(idx, next);
        }
    }

    pub(super) fn refresh_meadow_slice(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        let (ax, ay, az) = avatar.meadow_anchor();
        let pos = glam::Vec3::new((ax as f32) * 0.5, (ay as f32) * 0.5, (az as f32) * 0.5);
        let mask = world.visible_slice_mask(pos);
        self.primary.mesh.select_visible(mask);
        self.tron.mesh.select_visible(mask);
        if !self.tron_active {
            let meadow_idx = world.face_count() * 2;
            let mut meadow_buf = CpuMeshBatch::with_quad_capacity(32_768);
            avatar.emit_surroundings(world, &mut meadow_buf);
            let next = self.surface.replace_mesh(self.primary.mesh.slice_at(meadow_idx), &meadow_buf);
            self.primary.mesh.store_slice(meadow_idx, next);
        }
    }

    pub(super) fn refresh_modified_face(&mut self, world: &mut dyn PlanetaryWorld) {
        if self.tron_active {
            let mut tron_buf = CpuTronBatch::empty();
            let idx = world.emit_modified_face(&mut tron_buf);
            let next = self.surface.replace_tron_mesh(self.tron.mesh.slice_at(idx), &tron_buf);
            self.tron.mesh.store_slice(idx, next);
        } else {
            let mut cpu_buf = CpuMeshBatch::with_quad_capacity(65_536);
            let idx = world.emit_modified_face(&mut cpu_buf);
            let next = self.surface.replace_mesh(self.primary.mesh.slice_at(idx), &cpu_buf);
            self.primary.mesh.store_slice(idx, next);
        }
    }

    pub(super) fn refresh_modified_and_meadow(&mut self, stage: &mut SessionStage) {
        self.refresh_modified_face(stage.world_mut());
        self.refresh_meadow_slice(stage.world(), stage.avatar());
    }

    pub(super) fn refresh_world_mesh(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        for idx in 0..(world.face_count() * 2) {
            self.refresh_face_slice(world, idx);
        }
        self.refresh_meadow_slice(world, avatar);
    }

    pub(super) fn toggle_tron(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        self.tron_active = !self.tron_active;
        self.refresh_world_mesh(world, avatar);
    }

    pub(super) fn render_stage(&self, stage: &mut SessionStage) {
        let shutter = stage.take_photo_shutter();
        if self.tron_active {
            if let PhotoShutter::Triggered(sight) = shutter {
                let mut shot_tron = CpuTronBatch::empty();
                stage.emit_avatar_for_sight(sight, &mut shot_tron);
                let composed = CompositeMesh::new(&self.tron.mesh, &shot_tron);
                let mosaic = self.surface.capture_sight_frame(
                    self.tron.pipeline.as_ref(),
                    self.tron.shader.as_ref(),
                    &composed,
                    stage.avatar(),
                    sight,
                    sight.photo_label(),
                );
                stage.apply_polaroid_mosaic(mosaic);
            }
            let mut avatar_tron = CpuTronBatch::empty();
            stage.emit_avatar(&mut avatar_tron);
            let composed = CompositeMesh::new(&self.tron.mesh, &avatar_tron);
            self.surface.render_frame(
                self.tron.pipeline.as_ref(),
                self.tron.shader.as_ref(),
                &composed,
                stage.avatar(),
                self.primary.shader.as_ref(),
                stage.modal(),
            );
        } else {
            if let PhotoShutter::Triggered(sight) = shutter {
                let mut shot_batch = CpuMeshBatch::with_quad_capacity(4_096);
                stage.emit_avatar_for_sight(sight, &mut shot_batch);
                let composed = CompositeMesh::new(&self.primary.mesh, &shot_batch);
                let mosaic = self.surface.capture_sight_frame(
                    self.primary.pipeline.as_ref(),
                    self.primary.shader.as_ref(),
                    &composed,
                    stage.avatar(),
                    sight,
                    sight.photo_label(),
                );
                stage.apply_polaroid_mosaic(mosaic);
            }
            let mut avatar_batch = CpuMeshBatch::with_quad_capacity(4_096);
            stage.emit_avatar(&mut avatar_batch);
            let composed = CompositeMesh::new(&self.primary.mesh, &avatar_batch);
            self.surface.render_frame(
                self.primary.pipeline.as_ref(),
                self.primary.shader.as_ref(),
                &composed,
                stage.avatar(),
                self.primary.shader.as_ref(),
                stage.modal(),
            );
        }
    }
}
