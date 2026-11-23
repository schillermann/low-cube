use glam::Vec3;
use crate::block::{
    BlockMedium, BlockMold, CardinalTurn, GridCell, MoldArchetype, WedgeHeading,
};
use crate::frame::SurfaceFrame;
use crate::relief::vehicle_assembly::{PilotOrder, VehicleAxes, VehiclePart};
use super::super::super::CraftPose;
use super::super::CraftBlueprint;
use super::HullContour;
use super::super::hull_solid::HullSolid;

fn molded_solid(cu: i32, cv: i32, floor: f32, arch: MoldArchetype, wedge: WedgeHeading, med: BlockMedium) -> HullSolid {
    HullSolid::from_stamp(GridCell::new(cu, cv), floor, [arch.code(), wedge.code(), med.code()])
        .expect("structural or camera part must yield HullSolid")
}

#[test]
fn multi_block_rectangular_stack_merges_into_twelve_outer_edges() {
    let mut solids = Vec::new();
    for cv in [-1, 0] {
        for layer in [0.5, 1.0, 1.5] {
            solids.push(molded_solid(
                0,
                cv,
                layer,
                MoldArchetype::FullCube,
                WedgeHeading::South,
                BlockMedium::QuarriedStone,
            ));
        }
    }
    let contour = HullContour::from_solids(&solids);
    let mut segs = Vec::new();
    contour.visit_segments(|a, b| segs.push((a, b)));
    assert_eq!(segs.len(), 12, "2x3 stack of cubes must merge coplanar faces into 12 prism edges");
}

#[test]
fn l_shaped_winged_hull_preserves_concave_step_creases_without_inner_seams() {
    let solids = vec![
        molded_solid(0, 0, 0.0, MoldArchetype::FullCube, WedgeHeading::South, BlockMedium::QuarriedStone),
        molded_solid(0, 0, 0.5, MoldArchetype::FullCube, WedgeHeading::South, BlockMedium::QuarriedStone),
        molded_solid(1, 0, 0.0, MoldArchetype::FullCube, WedgeHeading::South, BlockMedium::QuarriedStone),
    ];
    let contour = HullContour::from_solids(&solids);
    let mut segs = Vec::new();
    contour.visit_segments(|a, b| segs.push((a, b)));
    assert_eq!(segs.len(), 18, "3-cube L-hull must emit 18 outer edges (6-sided L-profile * 2 + 6 cross edges)");
}

#[test]
fn gentle_two_block_ramp_fuses_coplanar_slope_seam() {
    let solids = vec![
        molded_solid(0, 0, 0.0, MoldArchetype::GentleLowerWedge, WedgeHeading::North, BlockMedium::PlanedPlanks),
        molded_solid(0, 1, 0.0, MoldArchetype::GentleUpperWedge, WedgeHeading::North, BlockMedium::PlanedPlanks),
    ];
    let contour = HullContour::from_solids(&solids);
    let mut diagonal_slopes = 0usize;
    let mut mid_seam = false;
    contour.visit_segments(|a, b| {
        let d = b - a;
        if d.y.abs() > 0.45 && d.z.abs() > 0.95 {
            diagonal_slopes += 1;
        }
        if (a.z - 0.5).abs() < 1e-3 && (b.z - 0.5).abs() < 1e-3 && (a.y - 0.25).abs() < 1e-3 {
            mid_seam = true;
        }
    });
    assert_eq!(diagonal_slopes, 2, "Lower + upper gentle wedges must fuse into two 1.0m slope edges");
    assert!(!mid_seam, "Coplanar horizontal seam between lower and upper gentle wedge must vanish");
}

#[test]
fn nose_cone_and_wedge_wing_fuse_onto_spacecraft_wireframe_without_base_crossbars() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let parts = vec![
        VehiclePart::new(
            (0, 0),
            0.0,
            BlockMold::cube().in_medium(BlockMedium::CabinSeat).shape_for(Vec3::NEG_Z, &frame),
        ),
        VehiclePart::new(
            (0, -1),
            0.0,
            BlockMold::cube().in_medium(BlockMedium::FlightConsole).shape_for(Vec3::Z, &frame),
        ),
        VehiclePart::new(
            (0, -1),
            0.5,
            BlockMold::cube().in_medium(BlockMedium::RocketCamera).shape_for(Vec3::NEG_Z, &frame),
        ),
        VehiclePart::new(
            (1, 0),
            0.0,
            BlockMold::with_medium(MoldArchetype::RampWedge, CardinalTurn::Left, BlockMedium::QuarriedStone)
                .shape_for(Vec3::NEG_Z, &frame),
        ),
    ];
    let bp = CraftBlueprint::commission(Vec3::ZERO, parts);
    let pose = CraftPose::parked(
        Vec3::new(0.25, 4.0, 0.25),
        VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y),
        Vec3::NEG_Z,
    );
    let wf = bp.sample_wireframe(&PilotOrder::idle(), pose);
    let mut slanted = 0usize;
    let mut total = 0usize;
    wf.visit_edges(|a, b| {
        total += 1;
        let d = (b - a).abs();
        let non_zero_axes = usize::from(d.x > 0.05) + usize::from(d.y > 0.05) + usize::from(d.z > 0.05);
        if non_zero_axes >= 2 {
            slanted += 1;
        }
    });
    assert!(total >= 18, "Sculpted nose-cone + wedge-wing hull must emit true outer contour edges, got {total}");
    assert!(slanted >= 6, "Nose cone + ramp wedge wing must contribute sloped crease edges, got {slanted}");
}
