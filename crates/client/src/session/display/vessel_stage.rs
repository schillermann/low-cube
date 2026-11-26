use std::collections::HashMap;
use glam::Mat4;
use glow::HasContext;
use engine::{CpuMeshBatch, Mesh, ShaderProgram, StreamingMesh};
use world::{CraftBlueprint, CraftIdentity, CraftPose, MeshTarget, Quad, VesselSink};
use crate::surface::PlatformSurface;

/// Decorator splitting opaque actor/hull quads from semi-transparent rocket plume quads (`glow < -1.5`).
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

/// Frame-scoped `VesselSink` routing GPU hulls to `VesselStage` and animated sub-parts to `MeshTarget`.
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

/// Combined dynamic actor stream, translucent exhaust plume stream, and GPU-posed vessel stage for `ShadedScenePass`.
pub struct SceneActors {
    figures: StreamingMesh,
    plumes: StreamingMesh,
    vessels: VesselStage,
}

impl SceneActors {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(figures: StreamingMesh, plumes: StreamingMesh, vessels: VesselStage) -> Self {
        Self { figures, plumes, vessels }
    }

    pub(super) fn reset(&mut self) {
        self.figures.clear();
        self.plumes.clear();
        self.vessels.clear_frame();
    }

    pub(super) fn emit_actors<F>(&mut self, f: F)
    where
        F: FnOnce(&mut dyn MeshTarget),
    {
        let mut split = PlumeSplitTarget::new(&mut self.figures, &mut self.plumes);
        f(&mut split);
    }

    pub(super) fn collect_vessels<F>(&mut self, surface: &dyn PlatformSurface, f: F)
    where
        F: FnOnce(&mut dyn VesselSink),
    {
        let mut split = PlumeSplitTarget::new(&mut self.figures, &mut self.plumes);
        let mut sink = VesselFrameSink::new(&mut self.vessels, surface, &mut split);
        f(&mut sink);
    }
}

impl Mesh for SceneActors {
    fn draw(&self, gl: &glow::Context) {
        self.figures.draw(gl);
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func_separate(
                glow::SRC_ALPHA,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
            );
            gl.depth_mask(false);
            self.plumes.draw(gl);
            gl.depth_mask(true);
            gl.disable(glow::BLEND);
        }
    }

    fn draw_posed(&self, gl: &glow::Context, shader: &dyn ShaderProgram) {
        self.vessels.draw_posed(gl, shader);
        self.figures.draw_posed(gl, shader);
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func_separate(
                glow::SRC_ALPHA,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
            );
            gl.depth_mask(false);
            self.plumes.draw_posed(gl, shader);
            gl.depth_mask(true);
            gl.disable(glow::BLEND);
        }
    }

    fn release(&self, gl: &glow::Context) {
        self.vessels.release(gl);
        self.figures.release(gl);
        self.plumes.release(gl);
    }
}
