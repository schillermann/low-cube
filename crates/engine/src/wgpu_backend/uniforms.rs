use glam::{Mat4, Vec3};
use crate::lighting::PointLightCluster;

/// `688-Byte` (`43 × vec4<f32>`) `@group(0) @binding(0)` scene globals uniform block (`3` private fields).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SceneUniformBlock {
    matrices: [[f32; 4]; 8],
    vectors: [[f32; 4]; 3],
    lights: [[f32; 4]; 32],
}

impl SceneUniformBlock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        matrices: [[f32; 4]; 8],
        vectors: [[f32; 4]; 3],
        lights: [[f32; 4]; 32],
    ) -> Self {
        Self { matrices, vectors, lights }
    }

    pub fn composed(
        (view_proj, light_vp): (Mat4, Mat4),
        (sun_dir, shadow_strength): (Vec3, f32),
        (cam_pos, up_axis, haze): (Vec3, Vec3, f32),
        cluster: &PointLightCluster,
    ) -> Self {
        let vp_c = view_proj.to_cols_array_2d();
        let lv_c = light_vp.to_cols_array_2d();
        let matrices = [vp_c[0], vp_c[1], vp_c[2], vp_c[3], lv_c[0], lv_c[1], lv_c[2], lv_c[3]];
        let vectors = [
            [sun_dir.x, sun_dir.y, sun_dir.z, shadow_strength],
            [cam_pos.x, cam_pos.y, cam_pos.z, cluster.active_count() as f32],
            [up_axis.x, up_axis.y, up_axis.z, haze],
        ];
        let mut lights = [[0.0; 4]; 32];
        for (i, entry) in cluster.entries().iter().enumerate() {
            let bytes: &[[f32; 4]; 2] = unsafe { &*(entry as *const _ as *const [[f32; 4]; 2]) };
            lights[i * 2] = bytes[0];
            lights[i * 2 + 1] = bytes[1];
        }
        Self::new(matrices, vectors, lights)
    }

    pub fn raw_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self).cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }
}

/// `256-Byte` dynamic-offset `@group(1) @binding(0)` patch transform & quantization step (`2` private fields).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AlignedPatchUniform {
    model_and_origin: [[f32; 4]; 5],
    alignment_pad: [[f32; 4]; 11],
}

impl AlignedPatchUniform {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(model_and_origin: [[f32; 4]; 5], alignment_pad: [[f32; 4]; 11]) -> Self {
        Self { model_and_origin, alignment_pad }
    }

    pub fn from_pose(model: Mat4, origin_step: [f32; 4]) -> Self {
        let m = model.to_cols_array_2d();
        Self::new([m[0], m[1], m[2], m[3], origin_step], [[0.0; 4]; 11])
    }
}

/// `304-Byte` (`19 × vec4<f32>`) `@group(0) @binding(0)` celestial firmament uniform block (`3` private fields).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FirmamentUniform {
    inv_view_proj: [[f32; 4]; 4],
    params: [[f32; 4]; 7],
    planet_cubes: [[f32; 4]; 8],
}

impl FirmamentUniform {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        inv_view_proj: [[f32; 4]; 4],
        params: [[f32; 4]; 7],
        planet_cubes: [[f32; 4]; 8],
    ) -> Self {
        Self { inv_view_proj, params, planet_cubes }
    }

    pub const fn dormant() -> Self {
        Self::new([[0.0; 4]; 4], [[0.0; 4]; 7], [[0.0; 4]; 8])
    }

    pub fn raw_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self).cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }
}

/// `256-Byte` dynamic-offset monitor blit uniform (`2` private fields).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AlignedMonitorUniform {
    rect_and_uv: [[f32; 4]; 2],
    alignment_pad: [[f32; 4]; 14],
}

impl AlignedMonitorUniform {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(rect_and_uv: [[f32; 4]; 2], alignment_pad: [[f32; 4]; 14]) -> Self {
        Self { rect_and_uv, alignment_pad }
    }

    pub const fn tile(dst_rect: [f32; 4], uv_span: [f32; 2], phosphor_mix: f32) -> Self {
        Self::new([dst_rect, [uv_span[0], uv_span[1], phosphor_mix, 0.0]], [[0.0; 4]; 14])
    }
}
