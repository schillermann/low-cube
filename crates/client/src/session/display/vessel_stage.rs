use std::collections::HashMap;
use glam::{Mat4, Vec3};
use engine::{
    BlendPassMode, FramePass, LuminanceHarvestTarget, Mesh, PointLightCluster, QuadBatch,
    StreamingMesh,
};
use world::{CraftBlueprint, CraftIdentity, CraftPose, MeshTarget, Quad, VesselSink};
use crate::surface::PlatformSurface;

/// Decorator splitting opaque actor/hull quads from semi-transparent rocket plume quads (`glow < -1.5`, `2` fields).
pub(super) struct PlumeSplitTarget<'a> {
    opaque: &'a mut StreamingMesh,
    plumes: &'a mut StreamingMesh,
}

impl<'a> PlumeSplitTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(opaque: &'a mut StreamingMesh, plumes: &'a mut StreamingMesh) -> Self {
        Self { opaque, plumes }
    }
}

impl MeshTarget for PlumeSplitTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        if quad.radiate(|g| g < -1.5) {
            self.plumes.push_quad(quad);
        } else {
            self.opaque.push_quad(quad);
        }
    }
}

/// GPU-cached local-space vehicle hull registry transforming hulls via `FramePass::apply_model` (`3` fields).
pub struct VesselStage {
    hulls: HashMap<CraftIdentity, Box<dyn Mesh>>,
    instances: Vec<(CraftIdentity, Mat4)>,
    scratch: QuadBatch,
}

impl VesselStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        hulls: HashMap<CraftIdentity, Box<dyn Mesh>>,
        instances: Vec<(CraftIdentity, Mat4)>,
        scratch: QuadBatch,
    ) -> Self {
        Self { hulls, instances, scratch }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new(), Vec::with_capacity(16), QuadBatch::with_quad_capacity(1_024))
    }

    pub fn clear_frame(&mut self) {
        self.instances.clear();
    }

    pub fn record_vessel(
        &mut self,
        surface: &dyn PlatformSurface,
        blueprint: &CraftBlueprint,
        pose: CraftPose,
        kinematic_out: &mut dyn MeshTarget,
    ) {
        let id = blueprint.identity();
        if id.assigned() {
            if !self.hulls.contains_key(&id) {
                self.scratch.clear();
                blueprint.emit_local_hull(&mut self.scratch);
                let uploaded = surface.upload_mesh(&self.scratch);
                self.hulls.insert(id, uploaded);
            }
            self.instances.push((id, pose.model_matrix()));
            blueprint.emit_kinematic_parts(pose, kinematic_out);
        } else {
            blueprint.emit_at(pose, kinematic_out);
        }
    }
}

impl Mesh for VesselStage {
    fn draw(&self, pass: &mut dyn FramePass) {
        if self.instances.is_empty() {
            return;
        }
        for (id, model) in &self.instances {
            if let Some(hull) = self.hulls.get(id) {
                pass.apply_model(*model);
                hull.draw(pass);
            }
        }
        pass.apply_model(Mat4::IDENTITY);
    }

    fn release(&self, reclaim: &mut dyn engine::SlotReclamation) {
        for hull in self.hulls.values() {
            hull.release(reclaim);
        }
    }
}

/// Frame-scoped `VesselSink` routing GPU hulls to `VesselStage` and animated sub-parts to `MeshTarget` (`3` fields).
pub(super) struct VesselFrameSink<'a> {
    stage: &'a mut VesselStage,
    surface: &'a dyn PlatformSurface,
    figures: &'a mut dyn MeshTarget,
}

impl<'a> VesselFrameSink<'a> {
    pub(super) fn new(
        stage: &'a mut VesselStage,
        surface: &'a dyn PlatformSurface,
        figures: &'a mut dyn MeshTarget,
    ) -> Self {
        Self { stage, surface, figures }
    }
}

impl VesselSink for VesselFrameSink<'_> {
    fn receive_vessel(&mut self, blueprint: &CraftBlueprint, pose: CraftPose) {
        self.stage.record_vessel(self.surface, blueprint, pose, self.figures);
    }
}

/// Combined dynamic actor stream, translucent plume stream, `VesselStage`, and `<= 16` `PointLightCluster` (`4` fields).
pub struct SceneActors {
    figures: StreamingMesh,
    plumes: StreamingMesh,
    vessels: VesselStage,
    lights: PointLightCluster,
}

impl SceneActors {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        figures: StreamingMesh,
        plumes: StreamingMesh,
        vessels: VesselStage,
        lights: PointLightCluster,
    ) -> Self {
        Self { figures, plumes, vessels, lights }
    }

    pub(super) fn reset(&mut self, viewer: Vec3) {
        self.figures.clear();
        self.plumes.clear();
        self.vessels.clear_frame();
        self.lights.reset(viewer);
    }

    pub(super) const fn lights(&self) -> &PointLightCluster {
        &self.lights
    }

    pub(super) fn emit_actors<F>(&mut self, f: F)
    where
        F: FnOnce(&mut dyn MeshTarget),
    {
        let mut split = PlumeSplitTarget::new(&mut self.figures, &mut self.plumes);
        let mut harvested = LuminanceHarvestTarget::new(&mut split, &mut self.lights);
        f(&mut harvested);
    }

    pub(super) fn collect_vessels<F>(&mut self, surface: &dyn PlatformSurface, f: F)
    where
        F: FnOnce(&mut dyn VesselSink),
    {
        let mut split = PlumeSplitTarget::new(&mut self.figures, &mut self.plumes);
        let mut harvested = LuminanceHarvestTarget::new(&mut split, &mut self.lights);
        let mut sink = VesselFrameSink::new(&mut self.vessels, surface, &mut harvested);
        f(&mut sink);
    }
}

impl Mesh for SceneActors {
    fn draw(&self, pass: &mut dyn FramePass) {
        self.vessels.draw(pass);
        self.figures.draw(pass);
        pass.select_blend(BlendPassMode::TranslucentPlume);
        self.plumes.draw(pass);
        pass.select_blend(BlendPassMode::OpaqueDepth);
    }

    fn release(&self, reclaim: &mut dyn engine::SlotReclamation) {
        self.vessels.release(reclaim);
        self.figures.release(reclaim);
        self.plumes.release(reclaim);
    }
}
