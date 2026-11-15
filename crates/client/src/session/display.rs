use engine::{CompositeMesh, CpuMeshBatch, CpuTronBatch, GraphicPipeline, PlanetaryMesh, ShaderProgram};
use fauna::WoodlandFauna;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{PhotoShutter, SightMode};
use crate::avatar::Avatar;
use crate::companion::CompanionFigure;
use crate::menu::SessionModal;
use crate::surface::PlatformSurface;

/// Encapsulated graphics pipeline, shader program, static slice mesh, and reusable actor scratch batch.
pub(super) struct RenderPass<B> {
    pipeline: Box<dyn GraphicPipeline>,
    shader: Box<dyn ShaderProgram>,
    mesh: PlanetaryMesh,
    actors: B,
}

impl<B> RenderPass<B> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        pipeline: Box<dyn GraphicPipeline>,
        shader: Box<dyn ShaderProgram>,
        mesh: PlanetaryMesh,
        actors: B,
    ) -> Self {
        Self { pipeline, shader, mesh, actors }
    }
}

/// Encapsulated platform window surface and dual primary/Tron rendering passes.
pub(super) struct SessionDisplay {
    surface: Box<dyn PlatformSurface>,
    primary: RenderPass<CpuMeshBatch>,
    tron: RenderPass<CpuTronBatch>,
    tron_active: bool,
}

impl SessionDisplay {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        surface: Box<dyn PlatformSurface>,
        primary: RenderPass<CpuMeshBatch>,
        tron: RenderPass<CpuTronBatch>,
        tron_active: bool,
    ) -> Self {
        Self { surface, primary, tron, tron_active }
    }

    pub(super) fn surface(&self) -> &dyn PlatformSurface {
        self.surface.as_ref()
    }

    pub(super) const fn tron_enabled(&self) -> bool {
        self.tron_active
    }

    pub(super) fn resize(&mut self, width: u32, height: u32) {
        self.surface.resize(width, height);
    }

    pub(super) fn capture_cursor(&self) {
        self.surface.capture_cursor();
    }

    pub(super) fn present(&self) {
        self.surface.present_frame();
    }

    pub(super) fn request_redraw(&self) {
        self.surface.request_redraw();
    }

    fn emit_actors(
        world: &dyn PlanetaryWorld,
        avatar: &dyn Avatar,
        fauna: &WoodlandFauna,
        sight_scale: f32,
        target: &mut dyn MeshTarget,
    ) {
        avatar.emit_appearance(target);
        CompanionFigure::emit_all(world, target);
        fauna.emit_visible(avatar.sight_cone().scaled(sight_scale), target);
    }

    fn emit_actors_for_sight(
        world: &dyn PlanetaryWorld,
        avatar: &dyn Avatar,
        fauna: &WoodlandFauna,
        sight: SightMode,
        sight_scale: f32,
        target: &mut dyn MeshTarget,
    ) {
        avatar.emit_appearance_for_sight(sight, target);
        CompanionFigure::emit_all(world, target);
        fauna.emit_visible(avatar.sight_cone_for_mode(sight).scaled(sight_scale), target);
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
        if !world.pending_slices() {
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
            return;
        }
        while world.pending_slices() {
            if self.tron_active {
                let mut tron_buf = CpuTronBatch::empty();
                if let Some(idx) = world.drain_modified_slice(&mut tron_buf) {
                    let next = self.surface.replace_tron_mesh(self.tron.mesh.slice_at(idx), &tron_buf);
                    self.tron.mesh.store_slice(idx, next);
                }
            } else {
                let mut cpu_buf = CpuMeshBatch::with_quad_capacity(65_536);
                if let Some(idx) = world.drain_modified_slice(&mut cpu_buf) {
                    let next = self.surface.replace_mesh(self.primary.mesh.slice_at(idx), &cpu_buf);
                    self.primary.mesh.store_slice(idx, next);
                }
            }
        }
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

    pub(super) fn render_scene(
        &mut self,
        world: &dyn PlanetaryWorld,
        avatar: &mut dyn Avatar,
        fauna: &WoodlandFauna,
        modal: &dyn SessionModal,
        sight_scale: f32,
    ) {
        let shutter = avatar.take_photo_shutter();
        if self.tron_active {
            if let PhotoShutter::Triggered(sight) = shutter {
                self.tron.actors.clear();
                Self::emit_actors_for_sight(world, avatar, fauna, sight, sight_scale, &mut self.tron.actors);
                let composed = CompositeMesh::new(&self.tron.mesh, &self.tron.actors);
                let mosaic = self.surface.capture_sight_frame(
                    self.tron.pipeline.as_ref(),
                    self.tron.shader.as_ref(),
                    &composed,
                    avatar,
                    sight,
                    sight.photo_label(),
                );
                avatar.apply_polaroid_mosaic(mosaic);
            }
            self.tron.actors.clear();
            Self::emit_actors(world, avatar, fauna, sight_scale, &mut self.tron.actors);
            let composed = CompositeMesh::new(&self.tron.mesh, &self.tron.actors);
            self.surface.render_frame(
                self.tron.pipeline.as_ref(),
                self.tron.shader.as_ref(),
                &composed,
                avatar,
                self.primary.shader.as_ref(),
                modal,
            );
        } else {
            if let PhotoShutter::Triggered(sight) = shutter {
                self.primary.actors.clear();
                Self::emit_actors_for_sight(world, avatar, fauna, sight, sight_scale, &mut self.primary.actors);
                let composed = CompositeMesh::new(&self.primary.mesh, &self.primary.actors);
                let mosaic = self.surface.capture_sight_frame(
                    self.primary.pipeline.as_ref(),
                    self.primary.shader.as_ref(),
                    &composed,
                    avatar,
                    sight,
                    sight.photo_label(),
                );
                avatar.apply_polaroid_mosaic(mosaic);
            }
            self.primary.actors.clear();
            Self::emit_actors(world, avatar, fauna, sight_scale, &mut self.primary.actors);
            let composed = CompositeMesh::new(&self.primary.mesh, &self.primary.actors);
            self.surface.render_frame(
                self.primary.pipeline.as_ref(),
                self.primary.shader.as_ref(),
                &composed,
                avatar,
                self.primary.shader.as_ref(),
                modal,
            );
        }
    }
}
