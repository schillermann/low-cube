use std::collections::HashMap;
use glam::Mat4;
use engine::{CpuMeshBatch, Mesh, ShaderProgram, StreamingMesh};
use world::{CraftBlueprint, CraftIdentity, CraftPose, VesselSink};
use crate::surface::PlatformSurface;

/// GPU-cached local-space vehicle hull registry transforming hulls via `u_model` `Mat4` uniforms.
pub struct VesselStage {
    hulls: HashMap<CraftIdentity, Box<dyn Mesh>>,
    instances: Vec<(CraftIdentity, Mat4)>,
    scratch: CpuMeshBatch,
}

impl VesselStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        hulls: HashMap<CraftIdentity, Box<dyn Mesh>>,
        instances: Vec<(CraftIdentity, Mat4)>,
        scratch: CpuMeshBatch,
    ) -> Self {
        Self { hulls, instances, scratch }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new(), Vec::with_capacity(16), CpuMeshBatch::with_quad_capacity(1_024))
    }

    pub fn clear_frame(&mut self) {
        self.instances.clear();
    }

    pub fn record_vessel(
        &mut self,
        surface: &dyn PlatformSurface,
        blueprint: &CraftBlueprint,
        pose: CraftPose,
        kinematic_out: &mut StreamingMesh,
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
    fn draw(&self, _gl: &glow::Context) {}

    fn draw_posed(&self, gl: &glow::Context, shader: &dyn ShaderProgram) {
        if self.instances.is_empty() {
            return;
        }
        for (id, model) in &self.instances {
            if let Some(hull) = self.hulls.get(id) {
                shader.apply_model(gl, model);
                hull.draw(gl);
            }
        }
        shader.apply_model(gl, &Mat4::IDENTITY);
    }

    fn release(&self, gl: &glow::Context) {
        for hull in self.hulls.values() {
            hull.release(gl);
        }
    }
}

/// Frame-scoped `VesselSink` routing GPU hulls to `VesselStage` and animated sub-parts to `StreamingMesh`.
pub(super) struct VesselFrameSink<'a> {
    stage: &'a mut VesselStage,
    surface: &'a dyn PlatformSurface,
    figures: &'a mut StreamingMesh,
}

impl<'a> VesselFrameSink<'a> {
    pub(super) fn new(
        stage: &'a mut VesselStage,
        surface: &'a dyn PlatformSurface,
        figures: &'a mut StreamingMesh,
    ) -> Self {
        Self { stage, surface, figures }
    }
}

impl VesselSink for VesselFrameSink<'_> {
    fn receive_vessel(&mut self, blueprint: &CraftBlueprint, pose: CraftPose) {
        self.stage.record_vessel(self.surface, blueprint, pose, self.figures);
    }
}

/// Combined dynamic actor stream and GPU-posed vessel stage for `ShadedScenePass`.
pub struct SceneActors {
    figures: StreamingMesh,
    vessels: VesselStage,
}

impl SceneActors {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(figures: StreamingMesh, vessels: VesselStage) -> Self {
        Self { figures, vessels }
    }

    pub(super) fn reset(&mut self) {
        self.figures.clear();
        self.vessels.clear_frame();
    }

    pub(super) fn figures_mut(&mut self) -> &mut StreamingMesh {
        &mut self.figures
    }

    pub(super) fn collect_vessels<F>(&mut self, surface: &dyn PlatformSurface, f: F)
    where
        F: FnOnce(&mut dyn VesselSink),
    {
        let mut sink = VesselFrameSink::new(&mut self.vessels, surface, &mut self.figures);
        f(&mut sink);
    }
}

impl Mesh for SceneActors {
    fn draw(&self, gl: &glow::Context) {
        self.figures.draw(gl);
    }

    fn draw_posed(&self, gl: &glow::Context, shader: &dyn ShaderProgram) {
        self.vessels.draw_posed(gl, shader);
        self.figures.draw_posed(gl, shader);
    }

    fn release(&self, gl: &glow::Context) {
        self.vessels.release(gl);
        self.figures.release(gl);
    }
}
