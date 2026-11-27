use glam::{Mat4, Vec3, Vec4};

mod atmosphere_lut;
mod composite_mesh;
mod greedy_brick;
mod instanced_module;
mod lighting;
mod packed_quad;
mod patch_mesh;
mod power_profile;
mod quad_batch;
mod sector_mesh;
mod streaming_mesh;
mod upload_ring;
mod vram_ledger;
mod wgpu_backend;
mod wgsl;

#[cfg(test)]
mod tests;

pub use self::atmosphere_lut::{
    AtmosphereSkyLut, SkyLutMedium, SKY_LUT_HEIGHT, SKY_LUT_INTERVAL_SECS, SKY_LUT_WIDTH,
};
pub use self::composite_mesh::{CoarseMesh, CompositeMesh, PlanetaryMesh};
pub use self::greedy_brick::{ArenaNeighbors, BrickNeighborhood, GreedyBrickQuads, VacantNeighbors};
pub use self::instanced_module::{
    ArchetypeSlice, InstancedModuleStage, ModuleArchetypeBank, ModuleInstance, ModuleVertex,
    MAX_ARCHETYPE_TRIANGLES,
};
pub use self::lighting::{
    LuminanceHarvestTarget, PointLightCluster, PointLightEntry, SunShadowCascade, MAX_POINT_LIGHTS,
    SUN_SHADOW_MAP_DIM,
};
pub use self::packed_quad::{FreeQuad, PackedQuad, Rgb565Swatch};
pub use self::patch_mesh::{SlotPair, UploadedPatchMesh};
pub use self::power_profile::GpuPowerProfile;
pub use self::quad_batch::{AxisQuadPack, QuadBatch};
pub use self::sector_mesh::SectorGridMesh;
pub use self::streaming_mesh::StreamingMesh;
pub use self::upload_ring::{
    BufferSlot, StagingTarget, UploadBudgetRing, PCIE_FRAME_UPLOAD_CAP_BYTES,
};
pub use self::vram_ledger::{
    StrictVramGuard, VramAccount, VramLedger, THINKPAD_VRAM_BUDGET_BYTES,
};
pub use self::wgpu_backend::{
    FirmamentUniform, PassDraft, WgpuStage, MONITOR_TILE_H, MONITOR_TILE_W,
};
pub use self::wgsl::{
    FIRMAMENT_LUT_WGSL, FREE_QUAD_WGSL, MODULE_INSTANCE_WGSL, MONITOR_BLIT_WGSL, PACKED_QUAD_WGSL,
};

/// Alpha & depth state selector for `FramePass`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendPassMode {
    OpaqueDepth,
    TranslucentPlume,
    OverlayHud,
}

/// Focused contract reclaiming released GPU storage-buffer slots.
pub trait SlotReclamation {
    fn reclaim_slot(&mut self, slot: BufferSlot);
}

/// Pure-OOP command target for `wgpu` and headless rendering passes (`0` raw driver handles exposed).
pub trait FramePass {
    fn clear_sky(&mut self, sky_rgba: Vec4);
    fn clear_depth_layer(&mut self);
    fn bind_camera(&mut self, view_proj: Mat4, light_dir: Vec3);
    fn apply_model(&mut self, model: Mat4);
    fn select_blend(&mut self, mode: BlendPassMode);
    fn upload_sky_lut(&mut self, rgba8_texels: &[u8]);
    fn draw_sky_lut(&mut self, uniform: FirmamentUniform);
    fn draw_packed_quads(&mut self, slot: BufferSlot, quad_count: u32, patch_origin: [f32; 4]);
    fn draw_free_quads(&mut self, slot: BufferSlot, quad_count: u32);
    fn stream_quads(&mut self, batch: &QuadBatch);
    fn draw_module_instances(&mut self, instances: &[ModuleInstance]);
}

/// Interface for GPU-renderable mesh objects (`8-Byte` `PackedQuad`s, `ModuleInstance`s, sectors, and streams).
pub trait Mesh {
    fn draw(&self, pass: &mut dyn FramePass);
    fn release(&self, _reclaim: &mut dyn SlotReclamation) {}
    fn patch_at(&self, _slot: usize) -> Option<&dyn Mesh> {
        None
    }
    fn store_patch(&mut self, _slot: usize, _patch: Box<dyn Mesh>) {}
}

/// Interface for scene rendering pipelines (`128×64` `2 Hz` Sky-LUT + `1024²` Sun Shadow + `8-Byte` Quads).
pub trait GraphicPipeline {
    fn sync_ephemeris(
        &self,
        _planet_cubes: &[[f32; 4]; 8],
        _eclipse_occluder: [f32; 4],
        _atmo_shell: [[f32; 4]; 4],
    ) {
    }

    fn render(&self, pass: &mut dyn FramePass, mesh: &dyn Mesh, view_proj: &Mat4);
}
