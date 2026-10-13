use glam::Vec3;
use crate::block::Block;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, OffsetMeshTarget};
use super::{VoxelCoord, VoxelNeighborhood, VoxelVolume};

/// Volume decorator adorning an underlying voxel volume with multi-form blocks (wedges, slabs, stairs).
pub struct FacetedVolume {
    underlying: Box<dyn VoxelVolume>,
    forms: Vec<Box<dyn Block>>,
}

impl FacetedVolume {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(underlying: Box<dyn VoxelVolume>, forms: Vec<Box<dyn Block>>) -> Self {
        Self { underlying, forms }
    }

    /// Secondary constructor wrapping an underlying volume with an empty form list.
    pub fn adorned(underlying: Box<dyn VoxelVolume>) -> Self {
        Self::new(underlying, Vec::new())
    }

    /// Decorates with an additional multi-form block.
    pub fn with_form(mut self, block: Box<dyn Block>) -> Self {
        self.forms.push(block);
        self
    }

    /// Constrains a surface position across all encapsulated multi-form blocks.
    pub fn constrain_surface(
        &self,
        base: Vec3,
        u: f32,
        v: f32,
        frame: &SurfaceFrame,
        distance: f32,
    ) -> Vec3 {
        let mut resolved = base;
        for form in &self.forms {
            resolved = form.constrain_surface(resolved, u, v, frame, distance);
        }
        resolved
    }
}

impl VoxelVolume for FacetedVolume {
    fn occluding_at(&self, coord: VoxelCoord) -> bool {
        self.underlying.occluding_at(coord)
    }

    fn inspect_voxel(&self, coord: VoxelCoord, consumer: &mut dyn FnMut(bool, Vec3)) {
        self.underlying.inspect_voxel(coord, consumer);
    }

    fn emit_mesh(
        &self,
        origin: Vec3,
        neighborhood: &dyn VoxelNeighborhood,
        target: &mut dyn MeshTarget,
    ) {
        self.underlying.emit_mesh(origin, neighborhood, target);

        // Frame oriented with identity surface normals for local volume blocks
        let local_frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let mut offset_target = OffsetMeshTarget::new(target, origin);
        for form in &self.forms {
            form.emit_surface(&local_frame, &mut offset_target);
        }
    }
}
