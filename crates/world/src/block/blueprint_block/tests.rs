use glam::Vec3;
use crate::block::{BlockMedium, BlockMold, GridCell, WedgeHeading};
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::BlueprintBlockMesh;

struct RecordedMesh {
    quads: Vec<Quad>,
}

impl RecordedMesh {
    const fn new() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for RecordedMesh {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

#[test]
fn medieval_blueprint_tablet_mounts_vertically_without_kickstand() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 16.0);
    let cell = GridCell::new(2, -3);
    let ((min_u, max_u), (min_v, max_v)) = (cell.u_bounds(), cell.v_bounds());
    let floor_elev = 1.5;
    let (uc, vc) = cell.center();
    let floor_center = frame.point_at(uc, vc, floor_elev);

    for heading in [
        WedgeHeading::North,
        WedgeHeading::South,
        WedgeHeading::East,
        WedgeHeading::West,
    ] {
        let mut mesh = RecordedMesh::new();
        BlueprintBlockMesh::new(heading).emit(cell, floor_elev, &frame, &mut mesh);
        assert!(mesh.quads.len() > 140);

        let fwd = heading.forward_vector(&frame);
        let up = frame.normal();
        let mut glowing_screen_quads = 0usize;
        let mut opaque_craft_quads = 0usize;
        let mut max_wall_depth = f32::NEG_INFINITY;

        for quad in &mesh.quads {
            let glow = quad.radiate(|g| g);
            quad.transfer(|v0, v1, v2, v3, n, _col| {
                assert!((n.length() - 1.0).abs() < 1e-3, "Quad normal must be unit length");
                for pt in [v0, v1, v2, v3] {
                    let (u, v) = frame.decompose(pt);
                    let elev = frame.elevation_of(pt);
                    assert!(u >= min_u - 1e-4 && u <= max_u + 1e-4, "u={u} outside [{min_u}, {max_u}]");
                    assert!(v >= min_v - 1e-4 && v <= max_v + 1e-4, "v={v} outside [{min_v}, {max_v}]");
                    assert!(
                        elev >= floor_elev + 0.05 && elev <= floor_elev + GridCell::SPAN - 0.05,
                        "Vertically mounted tablet must float without floor kickstand (elev={elev}, floor={floor_elev})",
                    );
                    let depth = (pt - floor_center).dot(fwd);
                    assert!(depth >= 0.17, "Tablet must sit flush near mounting wall without rear stand (depth={depth})");
                    max_wall_depth = max_wall_depth.max(depth);
                }
                if glow > 0.3 {
                    glowing_screen_quads += 1;
                    assert!(n.dot(-fwd) > 0.99, "Vertical tablet screen must face straight outward (-fwd), got {n:?}");
                    assert!(n.dot(up).abs() < 1e-4, "Vertical tablet screen must have zero tilt, got {n:?}");
                } else {
                    opaque_craft_quads += 1;
                }
                Quad::glowing(v0, v1, v2, v3, n, Vec3::ONE, glow)
            });
        }
        assert!(max_wall_depth > 0.245, "Rear wall cleats must sit flush against +0.25m cell face");
        assert!(opaque_craft_quads >= 100);
        assert!(glowing_screen_quads >= 30);
    }
}

#[test]
fn blueprint_mold_miniature_emits_vertical_guild_tablet_for_palm_and_satchel() {
    let mut mesh = RecordedMesh::new();
    BlockMold::cube()
        .in_medium(BlockMedium::VehicleBlueprint)
        .emit_miniature(
            Vec3::ZERO,
            Vec3::X,
            Vec3::Y,
            Vec3::Z,
            0.096,
            Vec3::ONE,
            Vec3::ONE,
            &mut mesh,
        );
    assert!(mesh.quads.len() > 140);
}
