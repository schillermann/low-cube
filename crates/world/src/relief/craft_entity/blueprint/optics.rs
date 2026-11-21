use glam::Vec3;
use crate::block::{GridCell, SensorPodAxis};
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::CraftBlueprint;
use super::super::pose::CraftPose;

pub(in crate::relief::craft_entity) struct ShiftedMeshTarget<'a> {
    shift: Vec3,
    inner: &'a mut dyn MeshTarget,
}

impl<'a> ShiftedMeshTarget<'a> {
    pub(in crate::relief::craft_entity) const fn new(shift: Vec3, inner: &'a mut dyn MeshTarget) -> Self {
        Self { shift, inner }
    }
}

impl MeshTarget for ShiftedMeshTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let (s, glow) = (self.shift, quad.radiate(|g| g));
        self.inner.push_quad(quad.transfer(|v0, v1, v2, v3, norm, col| {
            Quad::glowing(v0 + s, v1 + s, v2 + s, v3 + s, norm, col, glow)
        }));
    }
}

/// Collaborator projecting hull-mounted field and rocket sensor-pod camera rays for a `CraftBlueprint`.
pub(super) struct BlueprintOptics<'a> {
    blueprint: &'a CraftBlueprint,
}

impl<'a> BlueprintOptics<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(blueprint: &'a CraftBlueprint) -> Self {
        Self { blueprint }
    }

    pub(super) fn push_views(&self, pose: CraftPose, out: &mut Vec<(Vec3, Vec3, Vec3)>) {
        let bp = self.blueprint;
        let car_frame = pose.surface_frame();
        let shift = pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        let lift = bp.wheel_ground_lift();
        let ship_up = pose.normal().normalize_or_zero();
        let raw_fwd = pose.heading() - ship_up * pose.heading().dot(ship_up);
        let ship_fwd = if raw_fwd.length_squared() > 1e-6 {
            raw_fwd.normalize()
        } else {
            car_frame.tangent().normalize_or_zero()
        };
        if bp.spacefaring() {
            let mut pods: Vec<_> = bp.parts().iter().filter_map(|p| p.rocket_pod_at(lift)).collect();
            pods.sort_by(|a, b| b.0.channel_rank().cmp(&a.0.channel_rank()).then_with(|| b.2.total_cmp(&a.2)));
            let dedicated_belly = pods.iter().any(|(axis, _, _)| matches!(axis, SensorPodAxis::BellyPod));
            let mut ventral_emitted = false;
            for (axis, (ox, oy), part_floor) in &pods {
                let (col_bottom, col_top) = self.column_span(*ox, *oy, *part_floor, lift);
                let (local_eye, fwd, up) =
                    axis.lens_ray(GridCell::new(*ox, *oy), *part_floor, col_top, col_bottom, ship_fwd, &car_frame);
                out.push((shift + local_eye, fwd, up));
                if matches!(axis, SensorPodAxis::NoseCone) && !dedicated_belly && !ventral_emitted {
                    let (belly_eye, bfwd, bup) =
                        self.synthetic_ventral_ray((*ox, *oy), col_bottom, lift, ship_fwd, &car_frame);
                    out.push((shift + belly_eye, bfwd, bup));
                    ventral_emitted = true;
                }
            }
            return;
        }
        for part in bp.parts() {
            if let Some((hub, lens_fwd, _, _)) = part.camera_lens_at(lift, &car_frame) {
                out.push((shift + hub + lens_fwd * 0.22 + ship_up * 0.02, lens_fwd, ship_up));
            }
        }
    }

    fn column_span(&self, ox: i32, oy: i32, part_floor: f32, lift: f32) -> (f32, f32) {
        self.blueprint
            .parts()
            .iter()
            .filter_map(|p| p.cell_top_at(ox, oy, lift))
            .fold((part_floor, part_floor + 0.5), |(lo, hi), (f, c)| (lo.min(f), hi.max(c)))
    }

    fn synthetic_ventral_ray(
        &self,
        cell: (i32, i32),
        col_bottom: f32,
        lift: f32,
        ship_fwd: Vec3,
        frame: &SurfaceFrame,
    ) -> (Vec3, Vec3, Vec3) {
        let ship_up = frame.normal().normalize_or_zero();
        if col_bottom >= 0.45 {
            let (uc, vc) = GridCell::new(cell.0, cell.1).center();
            return (frame.point_at(uc, vc, (col_bottom - 0.25).max(0.20)), -ship_up, ship_fwd);
        }
        for (nx, ny) in [(cell.0, cell.1 + 1), (cell.0, cell.1 - 1), (cell.0 + 1, cell.1), (cell.0 - 1, cell.1)] {
            let clear = self
                .blueprint
                .parts()
                .iter()
                .filter_map(|p| p.cell_top_at(nx, ny, lift))
                .all(|(f, _)| f >= 0.45);
            if clear {
                let (uc, vc) = GridCell::new(nx, ny).center();
                return (frame.point_at(uc, vc, 0.20), -ship_up, ship_fwd);
            }
        }
        let (uc, vc) = GridCell::new(cell.0, cell.1).center();
        (frame.point_at(uc, vc, 0.20) + ship_fwd * 0.32, -ship_up, ship_fwd)
    }
}
