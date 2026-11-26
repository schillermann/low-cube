use engine::{
    CoarseMesh, CompositeMesh, CpuMeshBatch, GraphicPipeline, PlanetaryMesh, SectorGridMesh,
    ShaderProgram, StreamingMesh,
};
use world::{MeadowRing, PlanetaryWorld, MEADOW_RING_SLOTS};
use crate::attire::SightMode;
use crate::avatar::Avatar;
use crate::companion::CompanionFigure;
use crate::menu::SessionModal;
use crate::surface::PlatformSurface;
use super::pass::ScenePass;
use super::vessel_stage::{SceneActors, VesselStage};

/// Primary diurnal voxel rendering pass using `16×16` sector grids, toroidal `MeadowRing`, `VesselStage`, and `StreamingMesh`.
pub struct ShadedScenePass {
    program: (Box<dyn GraphicPipeline>, Box<dyn ShaderProgram>),
    mesh: PlanetaryMesh,
    actors: SceneActors,
    meadow_ring: MeadowRing,
}

impl ShadedScenePass {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_stage(
        program: (Box<dyn GraphicPipeline>, Box<dyn ShaderProgram>),
        mesh: PlanetaryMesh,
        actors: SceneActors,
        meadow_ring: MeadowRing,
    ) -> Self {
        Self { program, mesh, actors, meadow_ring }
    }

    /// Secondary constructor initializing an empty GPU `VesselStage` alongside actor streaming buffers.
    pub fn new(
        program: (Box<dyn GraphicPipeline>, Box<dyn ShaderProgram>),
        mesh: PlanetaryMesh,
        figures: StreamingMesh,
        meadow_ring: MeadowRing,
    ) -> Self {
        let actors = SceneActors::new(figures, StreamingMesh::with_quad_capacity(1_024), VesselStage::empty());
        Self::with_stage(program, mesh, actors, meadow_ring)
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

    fn populate_actors(
        &mut self,
        surface: &dyn PlatformSurface,
        world: &dyn PlanetaryWorld,
        avatar: &dyn Avatar,
        sight_scale: f32,
    ) {
        let cone = avatar.sight_cone().scaled(sight_scale);
        let viewer = cone.observe_from(|eye| eye);
        self.actors.reset();
        self.actors.collect_vessels(surface, |sink| {
            avatar.present_vessel(sink);
            world.present_vessels_near(viewer, 2_400.0, sink);
        });
        self.actors.emit_actors(|figures| {
            avatar.emit_figure(figures);
            avatar.emit_placement_preview(world, figures);
            CompanionFigure::emit_figures(world, figures);
            world.emit_wildlife(cone, figures);
            world.emit_cosmic_field(cone, figures);
        });
    }

    fn populate_sight_actors(
        &mut self,
        surface: &dyn PlatformSurface,
        world: &dyn PlanetaryWorld,
        avatar: &dyn Avatar,
        sight: SightMode,
        sight_scale: f32,
    ) {
        let cone = avatar.sight_cone_for_mode(sight).scaled(sight_scale);
        let viewer = cone.observe_from(|eye| eye);
        self.actors.reset();
        self.actors.collect_vessels(surface, |sink| {
            avatar.present_vessel(sink);
            world.present_vessels_near(viewer, 2_400.0, sink);
        });
        self.actors.emit_actors(|figures| {
            avatar.emit_figure_for_sight(sight, figures);
            CompanionFigure::emit_figures(world, figures);
            world.emit_wildlife(cone, figures);
            world.emit_cosmic_field(cone, figures);
        });
    }

    fn ensure_horizon_slices(&mut self, surface: &dyn PlatformSurface, world: &dyn PlanetaryWorld, mask: u32) {
        self.mesh.select_visible(mask);
        let mut missing = self.mesh.missing_slices(mask, world.face_count() * 2);
        while missing != 0 {
            let idx = missing.trailing_zeros() as usize;
            missing &= !(1u32 << idx);
            self.refresh_face_slice(surface, world, idx);
        }
    }
}

impl ScenePass for ShadedScenePass {
    fn select_visible(&mut self, mask: u32) {
        self.mesh.select_visible(mask);
    }

    fn reset_horizon(&mut self) {
        self.mesh.reset_uploaded();
    }

    fn refresh_face_slice(&mut self, surface: &dyn PlatformSurface, world: &dyn PlanetaryWorld, idx: usize) {
        self.mesh.mark_uploaded(idx);
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
        self.meadow_ring.reopen();
        let mut patch_buf = CpuMeshBatch::with_quad_capacity(2_048);
        while let Some(slot) = avatar.drain_meadow(world, &mut self.meadow_ring, &mut patch_buf) {
            if let Some(prev) = self.mesh.patch_at(meadow_idx, slot) {
                let next = surface.replace_mesh(prev, &patch_buf);
                self.mesh.store_patch(meadow_idx, slot, next);
            }
            patch_buf.clear();
        }
    }

    fn stream_meadow_slice(
        &mut self,
        surface: &dyn PlatformSurface,
        world: &dyn PlanetaryWorld,
        avatar: &dyn Avatar,
        reopen: bool,
    ) {
        if reopen {
            self.meadow_ring.reopen();
        }
        if !self.meadow_ring.streaming() {
            return;
        }
        let meadow_idx = world.face_count() * 2;
        self.ensure_meadow_grid(surface, meadow_idx);
        let mut patch_buf = CpuMeshBatch::with_quad_capacity(2_048);
        for _ in 0..4 {
            let Some(slot) = avatar.drain_meadow(world, &mut self.meadow_ring, &mut patch_buf) else {
                break;
            };
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
            if !updated_patch && self.mesh.patch_at(0, 0).is_none() {
                let mut cpu_buf = CpuMeshBatch::with_quad_capacity(65_536);
                let idx = world.emit_modified_face(&mut cpu_buf);
                self.mesh.mark_uploaded(idx);
                let next = surface.replace_mesh(self.mesh.slice_at(idx), &cpu_buf);
                self.mesh.store_slice(idx, next);
            }
            return;
        }
        while world.pending_slices() {
            let mut cpu_buf = CpuMeshBatch::with_quad_capacity(65_536);
            if let Some(idx) = world.drain_modified_slice(&mut cpu_buf) {
                self.mesh.mark_uploaded(idx);
                let next = surface.replace_mesh(self.mesh.slice_at(idx), &cpu_buf);
                self.mesh.store_slice(idx, next);
            }
        }
    }

    fn capture_sight(
        &mut self,
        surface: &dyn PlatformSurface,
        (world, avatar): (&dyn PlanetaryWorld, &dyn Avatar),
        sight: SightMode,
        sight_scale: f32,
    ) -> Vec<glam::Vec3> {
        let mask = avatar.visible_slices(world);
        self.ensure_horizon_slices(surface, world, mask);
        self.populate_sight_actors(surface, world, avatar, sight, sight_scale);
        let viewer = avatar.sight_cone_for_mode(sight).observe_from(|eye| eye);
        use crate::rayleigh_shell::{AtlasSkyProjection, EphemerisSync, SkyProjection};
        AtlasSkyProjection::solar(world.celestial_atlas())
            .project_from(viewer)
            .sync_pipeline(self.program.0.as_ref());
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
        (world, avatar): (&dyn PlanetaryWorld, &dyn Avatar),
        modal: &dyn SessionModal,
        sight_scale: f32,
    ) {
        let mask = avatar.visible_slices(world);
        self.ensure_horizon_slices(surface, world, mask);
        self.populate_actors(surface, world, avatar, sight_scale);
        let viewer = avatar.sight_cone().observe_from(|eye| eye);
        use crate::rayleigh_shell::{AtlasSkyProjection, EphemerisSync, SkyProjection};
        AtlasSkyProjection::solar(world.celestial_atlas())
            .project_from(viewer)
            .sync_pipeline(self.program.0.as_ref());
        let composed = CompositeMesh::new(&self.mesh, &self.actors);
        let coarse_terrain = CoarseMesh::new(&self.mesh, world.face_count() * 2);
        let coarse_composed = CompositeMesh::new(&coarse_terrain, &self.actors);
        surface.render_frame(
            self.program.0.as_ref(),
            self.program.1.as_ref(),
            (&composed, &coarse_composed),
            avatar,
            modal,
        );
    }
}
