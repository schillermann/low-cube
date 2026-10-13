use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::{FacetSlice, VoxelNeighborhood, VOLUME_EDGE_LENGTH};

/// Greedy facet merger constructing maximal coplanar quads along discrete volume slices.
pub struct GreedyMesh<'a> {
    palette: &'a [Vec3; 16],
}

impl<'a> GreedyMesh<'a> {
    /// Primary constructor assigning palette reference.
    pub fn new(palette: &'a [Vec3; 16]) -> Self {
        Self { palette }
    }

    fn color_for(&self, id: u8) -> Vec3 {
        let idx = (id as usize) & 0x0F;
        self.palette[idx]
    }

    /// Emits all merged exterior quads for a volume partition using slice buffers.
    pub fn emit_volume_mesh(
        &self,
        origin: Vec3,
        voxel_at: impl Fn(i32, i32, i32) -> u8,
        neighborhood: &dyn VoxelNeighborhood,
        target: &mut dyn MeshTarget,
    ) {
        let size = VOLUME_EDGE_LENGTH;
        let is_occluded = |x: i32, y: i32, z: i32| -> bool {
            let s = size as i32;
            if x >= 0 && x < s && y >= 0 && y < s && z >= 0 && z < s {
                voxel_at(x, y, z) != 0
            } else {
                neighborhood.occluding_at(x, y, z)
            }
        };

        self.emit_y_faces(origin, &voxel_at, &is_occluded, target);
        self.emit_x_faces(origin, &voxel_at, &is_occluded, target);
        self.emit_z_faces(origin, &voxel_at, &is_occluded, target);
    }

    fn emit_y_faces(
        &self,
        origin: Vec3,
        voxel_at: &impl Fn(i32, i32, i32) -> u8,
        is_occluded: &impl Fn(i32, i32, i32) -> bool,
        target: &mut dyn MeshTarget,
    ) {
        let size = VOLUME_EDGE_LENGTH;
        for y in 0..size {
            let yi = y as i32;
            let (mut top_slice, mut bottom_slice) = (FacetSlice::new(), FacetSlice::new());
            for z in 0..size {
                for x in 0..size {
                    let id = voxel_at(x as i32, yi, z as i32);
                    if id != 0 {
                        if !is_occluded(x as i32, yi + 1, z as i32) {
                            top_slice.record(x, z, id);
                        }
                        if !is_occluded(x as i32, yi - 1, z as i32) {
                            bottom_slice.record(x, z, id);
                        }
                    }
                }
            }
            top_slice.merge_into(&mut |u, v, w, h, id| {
                let (uf, vf, wf, hf, yf) = (u as f32, v as f32, w as f32, h as f32, (y + 1) as f32);
                target.push_quad(Quad::new(
                    origin + Vec3::new(uf, yf, vf + hf),
                    origin + Vec3::new(uf + wf, yf, vf + hf),
                    origin + Vec3::new(uf + wf, yf, vf),
                    origin + Vec3::new(uf, yf, vf),
                    Vec3::Y,
                    self.color_for(id),
                ));
            });
            bottom_slice.merge_into(&mut |u, v, w, h, id| {
                let (uf, vf, wf, hf, yf) = (u as f32, v as f32, w as f32, h as f32, y as f32);
                target.push_quad(Quad::new(
                    origin + Vec3::new(uf, yf, vf),
                    origin + Vec3::new(uf + wf, yf, vf),
                    origin + Vec3::new(uf + wf, yf, vf + hf),
                    origin + Vec3::new(uf, yf, vf + hf),
                    Vec3::NEG_Y,
                    self.color_for(id),
                ));
            });
        }
    }

    fn emit_x_faces(
        &self,
        origin: Vec3,
        voxel_at: &impl Fn(i32, i32, i32) -> u8,
        is_occluded: &impl Fn(i32, i32, i32) -> bool,
        target: &mut dyn MeshTarget,
    ) {
        let size = VOLUME_EDGE_LENGTH;
        for x in 0..size {
            let xi = x as i32;
            let (mut pos_slice, mut neg_slice) = (FacetSlice::new(), FacetSlice::new());
            for y in 0..size {
                for z in 0..size {
                    let id = voxel_at(xi, y as i32, z as i32);
                    if id != 0 {
                        if !is_occluded(xi + 1, y as i32, z as i32) {
                            pos_slice.record(z, y, id);
                        }
                        if !is_occluded(xi - 1, y as i32, z as i32) {
                            neg_slice.record(z, y, id);
                        }
                    }
                }
            }
            pos_slice.merge_into(&mut |u, v, w, h, id| {
                let (uf, vf, wf, hf, xf) = (u as f32, v as f32, w as f32, h as f32, (x + 1) as f32);
                target.push_quad(Quad::new(
                    origin + Vec3::new(xf, vf, uf),
                    origin + Vec3::new(xf, vf + hf, uf),
                    origin + Vec3::new(xf, vf + hf, uf + wf),
                    origin + Vec3::new(xf, vf, uf + wf),
                    Vec3::X,
                    self.color_for(id),
                ));
            });
            neg_slice.merge_into(&mut |u, v, w, h, id| {
                let (uf, vf, wf, hf, xf) = (u as f32, v as f32, w as f32, h as f32, x as f32);
                target.push_quad(Quad::new(
                    origin + Vec3::new(xf, vf, uf + wf),
                    origin + Vec3::new(xf, vf + hf, uf + wf),
                    origin + Vec3::new(xf, vf + hf, uf),
                    origin + Vec3::new(xf, vf, uf),
                    Vec3::NEG_X,
                    self.color_for(id),
                ));
            });
        }
    }

    fn emit_z_faces(
        &self,
        origin: Vec3,
        voxel_at: &impl Fn(i32, i32, i32) -> u8,
        is_occluded: &impl Fn(i32, i32, i32) -> bool,
        target: &mut dyn MeshTarget,
    ) {
        let size = VOLUME_EDGE_LENGTH;
        for z in 0..size {
            let zi = z as i32;
            let (mut pos_slice, mut neg_slice) = (FacetSlice::new(), FacetSlice::new());
            for y in 0..size {
                for x in 0..size {
                    let id = voxel_at(x as i32, y as i32, zi);
                    if id != 0 {
                        if !is_occluded(x as i32, y as i32, zi + 1) {
                            pos_slice.record(x, y, id);
                        }
                        if !is_occluded(x as i32, y as i32, zi - 1) {
                            neg_slice.record(x, y, id);
                        }
                    }
                }
            }
            pos_slice.merge_into(&mut |u, v, w, h, id| {
                let (uf, vf, wf, hf, zf) = (u as f32, v as f32, w as f32, h as f32, (z + 1) as f32);
                target.push_quad(Quad::new(
                    origin + Vec3::new(uf, vf, zf),
                    origin + Vec3::new(uf + wf, vf, zf),
                    origin + Vec3::new(uf + wf, vf + hf, zf),
                    origin + Vec3::new(uf, vf + hf, zf),
                    Vec3::Z,
                    self.color_for(id),
                ));
            });
            neg_slice.merge_into(&mut |u, v, w, h, id| {
                let (uf, vf, wf, hf, zf) = (u as f32, v as f32, w as f32, h as f32, z as f32);
                target.push_quad(Quad::new(
                    origin + Vec3::new(uf + wf, vf, zf),
                    origin + Vec3::new(uf, vf, zf),
                    origin + Vec3::new(uf, vf + hf, zf),
                    origin + Vec3::new(uf + wf, vf + hf, zf),
                    Vec3::NEG_Z,
                    self.color_for(id),
                ));
            });
        }
    }
}
