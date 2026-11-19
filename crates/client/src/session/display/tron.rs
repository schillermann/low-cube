use engine::{CompositeMesh, CpuTronBatch, GraphicPipeline, PlanetaryMesh, ShaderProgram};
use fauna::WoodlandFauna;
use world::PlanetaryWorld;
use crate::attire::SightMode;
use crate::avatar::Avatar;
use crate::companion::CompanionFigure;
use crate::menu::SessionModal;
use crate::surface::PlatformSurface;
use super::pass::ScenePass;

/// Wireframe Tron rendering pass encapsulating its pipeline, wireframe/HUD shaders, planetary mesh, and actor batch.
pub struct TronScenePass {
    pipeline: Box<dyn GraphicPipeline>,
    shaders: (Box<dyn ShaderProgram>, Box<dyn ShaderProgram>),
    mesh: PlanetaryMesh,
    actors: CpuTronBatch,
}

impl TronScenePass {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        pipeline: Box<dyn GraphicPipeline>,
        shaders: (Box<dyn ShaderProgram>, Box<dyn ShaderProgram>),
        mesh: PlanetaryMesh,
        actors: CpuTronBatch,
    ) -> Self {
        Self { pipeline, shaders, mesh, actors }
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

impl ScenePass for TronScenePass {
    fn tron_wireframe(&self) -> bool {
        true
    }

    fn select_visible(&mut self, mask: u32) {
        self.mesh.select_visible(mask);
    }

    fn refresh_face_slice(&mut self, surface: &dyn PlatformSurface, world: &dyn PlanetaryWorld, idx: usize) {
        let empty_init = CpuTronBatch::empty();
        self.mesh.ensure_slice(idx, || surface.upload_tron_mesh(&empty_init));
        let mut tron_buf = CpuTronBatch::empty();
        world.emit_face(idx, &mut tron_buf);
        let next = surface.replace_tron_mesh(self.mesh.slice_at(idx), &tron_buf);
        self.mesh.store_slice(idx, next);
    }

    fn refresh_meadow_slice(&mut self, _surface: &dyn PlatformSurface, _world: &dyn PlanetaryWorld, _avatar: &dyn Avatar) {}

    fn refresh_modified_face(&mut self, surface: &dyn PlatformSurface, world: &mut dyn PlanetaryWorld) {
        if !world.pending_slices() {
            let mut tron_buf = CpuTronBatch::empty();
            let idx = world.emit_modified_face(&mut tron_buf);
            let next = surface.replace_tron_mesh(self.mesh.slice_at(idx), &tron_buf);
            self.mesh.store_slice(idx, next);
            return;
        }
        while world.pending_slices() {
            let mut tron_buf = CpuTronBatch::empty();
            if let Some(idx) = world.drain_modified_slice(&mut tron_buf) {
                let next = surface.replace_tron_mesh(self.mesh.slice_at(idx), &tron_buf);
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
            self.shaders.0.as_ref(),
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
        let (wire_shader, hud_shader) = (self.shaders.0.as_ref(), self.shaders.1.as_ref());
        surface.render_frame(self.pipeline.as_ref(), wire_shader, &composed, avatar, hud_shader, modal);
    }
}
