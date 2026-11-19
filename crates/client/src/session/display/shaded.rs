use engine::{
    CompositeMesh, CpuMeshBatch, GraphicPipeline, PlanetaryMesh, SectorGridMesh, ShaderProgram,
    StreamingMesh,
};
use fauna::WoodlandFauna;
use world::{MeadowRing, PlanetaryWorld, MEADOW_RING_SLOTS};
use crate::attire::SightMode;
use crate::avatar::Avatar;
use crate::companion::CompanionFigure;
use crate::menu::SessionModal;
use crate::surface::PlatformSurface;
use super::pass::ScenePass;

/// Primary diurnal voxel rendering pass using `16×16` sector grids, toroidal `MeadowRing`, and `StreamingMesh`.
pub struct ShadedScenePass {
    program: (Box<dyn GraphicPipeline>, Box<dyn ShaderProgram>),
    mesh: PlanetaryMesh,
    actors: StreamingMesh,
    meadow_ring: MeadowRing,
}

impl ShadedScenePass {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        program: (Box<dyn GraphicPipeline>, Box<dyn ShaderProgram>),
        mesh: PlanetaryMesh,
        actors: StreamingMesh,
        meadow_ring: MeadowRing,
    ) -> Self {
        Self { program, mesh, actors, meadow_ring }
    }

    fn ensure_meadow_grid(&mut self, surface: &dyn PlatformSurface, meadow_idx: usize) {
        let empty = CpuMeshBatch::empty();
        self.mesh.ensure_slice(meadow_idx, || surface.upload_mesh(&empty));
        if self.mesh.patch_at(meadow_idx, 0).is_some() {
            return;
        }
        let patches = (0..MEADOW_RING_SLOTS).map(|_| surface.upload_mesh(&empty)).collect();
        self.mesh.store_slice(meadow_idx, Box::new(SectorGridMesh::new(patches)));
        self.meadow_ring.invalidate_all();
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
        let empty_init = CpuMeshBatch::empty();
        self.mesh.ensure_slice(idx, || surface.upload_mesh(&empty_init));
        let patch_count = world.face_patch_count(idx);
        if patch_count > 0 {
            if idx == 0 {
                self.meadow_ring.invalidate_all();
            }
            let mut patch_buf = CpuMeshBatch::with_quad_capacity(2_048);
            let mut patches = Vec::with_capacity(patch_count);
            for slot in 0..patch_count {
                patch_buf.clear();
                world.emit_face_patch(idx, slot, &mut patch_buf);
                patches.push(surface.upload_mesh(&patch_buf));
            }
            let prev = self.mesh.slice_at(idx);
            let _ = surface.replace_mesh(prev, &CpuMeshBatch::empty());
            self.mesh.store_slice(idx, Box::new(SectorGridMesh::new(patches)));
            return;
        }
        let mut cpu_buf = CpuMeshBatch::with_quad_capacity(65_536);
        world.emit_face(idx, &mut cpu_buf);
        let next = surface.replace_mesh(self.mesh.slice_at(idx), &cpu_buf);
        self.mesh.store_slice(idx, next);
    }

    fn refresh_meadow_slice(&mut self, surface: &dyn PlatformSurface, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        let meadow_idx = world.face_count() * 2;
        self.ensure_meadow_grid(surface, meadow_idx);
        let mut patch_buf = CpuMeshBatch::with_quad_capacity(2_048);
        while let Some(slot) = avatar.drain_meadow(world, &mut self.meadow_ring, &mut patch_buf) {
            if let Some(prev) = self.mesh.patch_at(meadow_idx, slot) {
                let next = surface.replace_mesh(prev, &patch_buf);
                self.mesh.store_patch(meadow_idx, slot, next);
            }
            patch_buf.clear();
        }
    }

    fn refresh_modified_face(&mut self, surface: &dyn PlatformSurface, world: &mut dyn PlanetaryWorld) {
        let mut updated_patch = false;
        let mut patch_buf = CpuMeshBatch::with_quad_capacity(2_048);
        while let Some((slice_idx, patch_slot)) = world.drain_modified_patch(&mut patch_buf) {
            if let Some(prev) = self.mesh.patch_at(slice_idx, patch_slot) {
                let next = surface.replace_mesh(prev, &patch_buf);
                self.mesh.store_patch(slice_idx, patch_slot, next);
                updated_patch = true;
            }
            patch_buf.clear();
        }
        if !world.pending_slices() {
            if !updated_patch {
                let mut cpu_buf = CpuMeshBatch::with_quad_capacity(65_536);
                let idx = world.emit_modified_face(&mut cpu_buf);
                let next = surface.replace_mesh(self.mesh.slice_at(idx), &cpu_buf);
                self.mesh.store_slice(idx, next);
            }
            return;
        }
        while world.pending_slices() {
            let mut cpu_buf = CpuMeshBatch::with_quad_capacity(65_536);
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
            self.program.0.as_ref(),
            self.program.1.as_ref(),
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
        let shader = self.program.1.as_ref();
        surface.render_frame(self.program.0.as_ref(), shader, &composed, avatar, shader, modal);
    }
}
