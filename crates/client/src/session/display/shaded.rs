use engine::{CompositeMesh, CpuMeshBatch, GraphicPipeline, PlanetaryMesh, ShaderProgram};
use fauna::WoodlandFauna;
use world::PlanetaryWorld;
use crate::attire::SightMode;
use crate::avatar::Avatar;
use crate::companion::CompanionFigure;
use crate::menu::SessionModal;
use crate::surface::PlatformSurface;
use super::pass::ScenePass;

/// Primary diurnal voxel rendering pass encapsulating its pipeline, shader, planetary mesh, and actor batch.
pub struct ShadedScenePass {
    pipeline: Box<dyn GraphicPipeline>,
    shader: Box<dyn ShaderProgram>,
    mesh: PlanetaryMesh,
    actors: CpuMeshBatch,
}

impl ShadedScenePass {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        pipeline: Box<dyn GraphicPipeline>,
        shader: Box<dyn ShaderProgram>,
        mesh: PlanetaryMesh,
        actors: CpuMeshBatch,
    ) -> Self {
        Self { pipeline, shader, mesh, actors }
    }

    fn populate_actors(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar, fauna: &WoodlandFauna, sight_scale: f32) {
        self.actors.clear();
        avatar.emit_appearance(&mut self.actors);
        CompanionFigure::emit_all(world, &mut self.actors);
        fauna.emit_visible(avatar.sight_cone().scaled(sight_scale), &mut self.actors);
    }

    fn populate_sight_actors(
        &mut self,
        world: &dyn PlanetaryWorld,
        avatar: &dyn Avatar,
        fauna: &WoodlandFauna,
        sight: SightMode,
        sight_scale: f32,
    ) {
        self.actors.clear();
        avatar.emit_appearance_for_sight(sight, &mut self.actors);
        CompanionFigure::emit_all(world, &mut self.actors);
        fauna.emit_visible(avatar.sight_cone_for_mode(sight).scaled(sight_scale), &mut self.actors);
    }
}

impl ScenePass for ShadedScenePass {
    fn select_visible(&mut self, mask: u32) {
        self.mesh.select_visible(mask);
    }

    fn refresh_face_slice(&mut self, surface: &dyn PlatformSurface, world: &dyn PlanetaryWorld, idx: usize) {
        let mut cpu_buf = CpuMeshBatch::with_quad_capacity(262_144);
        world.emit_face(idx, &mut cpu_buf);
        let next = surface.replace_mesh(self.mesh.slice_at(idx), &cpu_buf);
        self.mesh.store_slice(idx, next);
    }

    fn refresh_meadow_slice(&mut self, surface: &dyn PlatformSurface, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        let meadow_idx = world.face_count() * 2;
        let mut meadow_buf = CpuMeshBatch::with_quad_capacity(32_768);
        avatar.emit_surroundings(world, &mut meadow_buf);
        let next = surface.replace_mesh(self.mesh.slice_at(meadow_idx), &meadow_buf);
        self.mesh.store_slice(meadow_idx, next);
    }

    fn refresh_modified_face(&mut self, surface: &dyn PlatformSurface, world: &mut dyn PlanetaryWorld) {
        if !world.pending_slices() {
            let mut cpu_buf = CpuMeshBatch::with_quad_capacity(262_144);
            let idx = world.emit_modified_face(&mut cpu_buf);
            let next = surface.replace_mesh(self.mesh.slice_at(idx), &cpu_buf);
            self.mesh.store_slice(idx, next);
            return;
        }
        while world.pending_slices() {
            let mut cpu_buf = CpuMeshBatch::with_quad_capacity(262_144);
            if let Some(idx) = world.drain_modified_slice(&mut cpu_buf) {
                let next = surface.replace_mesh(self.mesh.slice_at(idx), &cpu_buf);
                self.mesh.store_slice(idx, next);
            }
        }
    }

    fn capture_sight(
        &mut self,
        surface: &dyn PlatformSurface,
        (world, avatar, fauna): (&dyn PlanetaryWorld, &dyn Avatar, &WoodlandFauna),
        sight: SightMode,
        sight_scale: f32,
    ) -> Vec<glam::Vec3> {
        self.populate_sight_actors(world, avatar, fauna, sight, sight_scale);
        let composed = CompositeMesh::new(&self.mesh, &self.actors);
        surface.capture_sight_frame(
            self.pipeline.as_ref(),
            self.shader.as_ref(),
            &composed,
            avatar,
            sight,
            sight.photo_label(),
        )
    }

    fn draw_frame(
        &mut self,
        surface: &dyn PlatformSurface,
        (world, avatar, fauna): (&dyn PlanetaryWorld, &dyn Avatar, &WoodlandFauna),
        modal: &dyn SessionModal,
        sight_scale: f32,
    ) {
        self.populate_actors(world, avatar, fauna, sight_scale);
        let composed = CompositeMesh::new(&self.mesh, &self.actors);
        let shader = self.shader.as_ref();
        surface.render_frame(self.pipeline.as_ref(), shader, &composed, avatar, shader, modal);
    }
}
