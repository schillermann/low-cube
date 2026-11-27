use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell, OriginPlinthMesh};
use crate::frame::SurfaceFrame;

/// Zero-brick procedural sanctuary plinth state avoiding eager `Brick8` allocation on untouched planets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SanctuaryPlinth {
    Absent,
    OriginCrown,
}

impl SanctuaryPlinth {
    pub const fn absent() -> Self {
        Self::Absent
    }

    pub const fn origin_crown() -> Self {
        Self::OriginCrown
    }

    pub const fn active_at(self, u: i32, v: i32) -> bool {
        matches!(self, Self::OriginCrown) && u == -1 && v == -1
    }

    pub fn in_layer(self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.active_at(u, v) && (-0.55..-0.02).contains(&layer_elev)
    }

    pub fn medium_at(self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.in_layer(u, v, layer_elev).then_some(BlockMedium::OriginStone)
    }

    pub fn hit_top(self, u: i32, v: i32, ray_elev: f32) -> Option<f32> {
        (self.active_at(u, v) && (-0.55..=0.02).contains(&ray_elev)).then_some(0.0)
    }

    pub fn collect_site(self, medium: BlockMedium, frame: &SurfaceFrame, out: &mut Vec<(Vec3, Vec3)>) {
        if matches!(self, Self::OriginCrown) && medium == BlockMedium::OriginStone {
            let top = frame.point_at(-0.25, -0.25, 0.0);
            out.push((top, frame.up_at(top)));
        }
    }

    pub fn emit_at(self, cell: GridCell, frame: &SurfaceFrame, target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|cu, cv| (cu, cv));
        if !self.active_at(u, v) {
            return false;
        }
        OriginPlinthMesh::new(cell, -GridCell::SPAN).emit(frame, target);
        true
    }
}
