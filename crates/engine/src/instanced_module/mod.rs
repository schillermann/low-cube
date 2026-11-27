mod archetypes;

use glam::{Mat4, Vec3};
use world::{CellCoord, GridCell, ModuleSpec, SpaceModule, SurfaceFrame};
use crate::{FramePass, Mesh};

pub use self::archetypes::{ArchetypeSlice, ModuleArchetypeBank, ModuleVertex, MAX_ARCHETYPE_TRIANGLES};

/// Per-instance GPU payload scaling a `[0..1]³` archetype onto `B × H × T` metric bounds (`2` private fields).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModuleInstance {
    model_cols: [[f32; 4]; 4],
    code_tint: [f32; 4],
}

impl ModuleInstance {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(model_cols: [[f32; 4]; 4], code_tint: [f32; 4]) -> Self {
        Self { model_cols, code_tint }
    }

    /// Builds a `B × H × T` matrix-scaled instance from `anchor` and `ModuleSpec` in a local `[X, Y_up, Z]` grid.
    pub fn from_grid_anchor(anchor: CellCoord, spec: ModuleSpec, datum: f32) -> Self {
        let (u, v, _) = anchor.components();
        let (du, dv, dw) = spec.span().dimensions();
        let step = GridCell::SPAN;
        let min_pos = Vec3::new((u as f32) * step, anchor.floor_elev(datum), (v as f32) * step);
        let scale = Vec3::new((du as f32) * step, (dw as f32) * step, (dv as f32) * step);
        let center = min_pos + scale * 0.5;
        let yaw = -(spec.turn().code() as f32) * core::f32::consts::FRAC_PI_2;
        let rot = Mat4::from_rotation_y(yaw);
        let (b_can, t_can) = if (spec.turn().code() & 1) == 1 { (scale.z, scale.x) } else { (scale.x, scale.z) };
        let local_scale = Vec3::new(b_can, scale.y, t_can);
        let shift_unit = Mat4::from_translation(Vec3::splat(-0.5));
        let model = Mat4::from_translation(center) * rot * Mat4::from_scale(local_scale) * shift_unit;
        Self::new(model.to_cols_array_2d(), [spec.module().code() as f32, 1.0, 1.0, 1.0])
    }

    /// Builds a `B × H × T` matrix-scaled instance mapped onto a `SurfaceFrame`.
    pub fn on_surface_frame(anchor: CellCoord, spec: ModuleSpec, datum: f32, frame: &SurfaceFrame) -> Self {
        let (u, v, _) = anchor.components();
        let (du, dv, dw) = spec.span().dimensions();
        let step = GridCell::SPAN;
        let u_mid = ((u as f32) + (du as f32) * 0.5) * step;
        let v_mid = ((v as f32) + (dv as f32) * 0.5) * step;
        let h_mid = anchor.floor_elev(datum) + (dw as f32) * step * 0.5;
        let center = frame.point_at(u_mid, v_mid, h_mid);
        let up = frame.up_at(center);
        let (tang, bitan) = frame.tangent_at(center);
        let basis = Mat4::from_cols(
            tang.extend(0.0),
            up.extend(0.0),
            bitan.extend(0.0),
            center.extend(1.0),
        );
        let yaw = -(spec.turn().code() as f32) * core::f32::consts::FRAC_PI_2;
        let span_u = (du as f32) * step;
        let span_v = (dv as f32) * step;
        let span_h = (dw as f32) * step;
        let (b_can, t_can) = if (spec.turn().code() & 1) == 1 { (span_v, span_u) } else { (span_u, span_v) };
        let model = basis
            * Mat4::from_rotation_y(yaw)
            * Mat4::from_scale(Vec3::new(b_can, span_h, t_can))
            * Mat4::from_translation(Vec3::splat(-0.5));
        Self::new(model.to_cols_array_2d(), [spec.module().code() as f32, 1.0, 1.0, 1.0])
    }

    pub fn module(&self) -> SpaceModule {
        SpaceModule::from_code(self.code_tint[0] as u8).expect("valid module code in ModuleInstance")
    }

    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.model_cols)
    }

    pub fn recentered(self, shift: impl FnOnce(Vec3) -> Vec3) -> Self {
        let mut cols = self.model_cols;
        let rel = shift(Vec3::new(cols[3][0], cols[3][1], cols[3][2]));
        cols[3][0] = rel.x;
        cols[3][1] = rel.y;
        cols[3][2] = rel.z;
        Self::new(cols, self.code_tint)
    }
}

/// Hardware-instanced draw batch for placed `SpaceModule`s (`1` private field).
#[derive(Clone, Debug)]
pub struct InstancedModuleStage {
    instances: Vec<ModuleInstance>,
}

impl InstancedModuleStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(instances: Vec<ModuleInstance>) -> Self {
        Self { instances }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }

    pub fn push_instance(&mut self, instance: ModuleInstance) {
        self.instances.push(instance);
    }

    pub fn instance_slice(&self) -> &[ModuleInstance] {
        &self.instances
    }
}

impl Mesh for InstancedModuleStage {
    fn draw(&self, pass: &mut dyn FramePass) {
        if !self.instances.is_empty() {
            pass.draw_module_instances(&self.instances);
        }
    }
}
