use crate::{CountedMeshTarget, MeshTarget, Quad};
use crate::block::{BlockMedium, BlockMold, CuboidShape, GridCell, MediumShape};
use crate::topology::ClimateZone;
use super::{ReliefElevation, ReliefPalette, ReliefStrata, SurfaceSightRay, TerracedRelief};

struct FlatMeadow;

impl ReliefElevation for FlatMeadow {
    fn natural_elevation_at(&self, _u: i32, _v: i32) -> i32 { 0 }
    fn elevation_at(&self, _u: i32, _v: i32) -> i32 { 0 }
    fn surface_height_at(&self, _u: f32, _v: f32) -> f32 { 0.0 }
}

struct NullTarget;

impl MeshTarget for NullTarget {
    fn push_quad(&mut self, _quad: Quad) {}
}

#[test]
fn test_cantilever_rig_mounts_any_initial_block_hoists_rotates_and_packs_up() {
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);

    let rig_shape = Box::new(MediumShape::new(Box::new(CuboidShape::new(0.5)), BlockMedium::CantileverRig, true, false));
    assert!(relief.stack_molded_cell(0, 0, 0.0, rig_shape));
    assert!(!relief.layer_occupied_at(2, 0, 1.75));
    let (claw_eye, claw_dir) = (frame.point_at(1.25, -1.25, 1.75), frame.bitangent());
    assert_eq!(SurfaceSightRay::new(claw_eye, claw_dir).target_layer(&frame, &relief, 64, 32.0), Some((2, 0, 1.75)));

    assert!(
        relief.stack_molded_cell(0, 0, 0.0, Box::new(CuboidShape::new(0.5))),
        "Any block medium must mount directly onto an empty crane without requiring a steering wheel first"
    );
    assert!(relief.layer_occupied_at(2, 0, 1.75));

    let mut null_out = NullTarget;
    let mut rig_counter = CountedMeshTarget::new(&mut null_out);
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &relief, &palette, 64, &mut rig_counter);
    assert!(rig_counter.count() >= 40);

    let eng_shape = Box::new(MediumShape::new(Box::new(CuboidShape::new(0.5)), BlockMedium::SteeringWheel, true, false));
    assert!(relief.stack_molded_cell(1, 0, 1.5, eng_shape));
    let wheel_shape = BlockMold::cube().in_medium(BlockMedium::CarWheel).shape_for(frame.tangent(), &frame);
    assert!(relief.stack_molded_cell(2, 1, 1.5, wheel_shape));

    let mut s0 = NullTarget;
    let mut s0_count = CountedMeshTarget::new(&mut s0);
    ReliefStrata::new().emit_cell(GridCell::new(2, 1), &frame, &relief, &palette, 64, &mut s0_count);
    for stage in 1..=3 {
        assert_eq!(relief.mark_dig_layer(2, 1, 1.75, stage).1, Some(BlockMedium::CarWheel));
        let mut sd = NullTarget;
        let mut sd_count = CountedMeshTarget::new(&mut sd);
        ReliefStrata::new().emit_cell(GridCell::new(2, 1), &frame, &relief, &palette, 64, &mut sd_count);
        assert_eq!(sd_count.count(), s0_count.count());
    }
    assert_eq!(relief.harvest_layer(2, 1, 1.75).1, Some(BlockMedium::CarWheel));
    assert!(!relief.layer_occupied_at(2, 1, 1.75));
    assert!(relief.adjust_rig_at(0, 0, 1, false).0);
    assert!(relief.layer_occupied_at(1, 0, 2.25) && relief.layer_occupied_at(2, 0, 2.25));
    assert!(relief.adjust_rig_at(2, 0, 0, true).0);
    assert_eq!(relief.harvest_layer(0, 0, 0.25).1, Some(BlockMedium::CantileverRig));
    assert!(!relief.layer_occupied_at(1, 0, 0.25));
    assert!(relief.layer_occupied_at(1, 1, 0.25));
}

#[test]
fn test_crane_object_requires_seat_to_sit_and_steering_wheel_to_leave_crane() {
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);

    let rig_shape = Box::new(MediumShape::new(Box::new(CuboidShape::new(0.5)), BlockMedium::CantileverRig, true, false));
    assert!(relief.stack_molded_cell(0, 0, 0.0, rig_shape));
    assert!(relief.stack_molded_cell(0, 0, 0.0, BlockMold::cube().in_medium(BlockMedium::PlanedPlanks).shape_for(frame.tangent(), &frame)));
    assert!(relief.stack_molded_cell(2, 1, 1.5, BlockMold::cube().in_medium(BlockMedium::CarWheel).shape_for(frame.tangent(), &frame)));

    let ray = SurfaceSightRay::new(frame.point_at(1.25, 0.25, 2.6), -frame.normal());
    assert!(relief.board_vehicle_at(&frame, 2, 0, 1.75, ray, false).is_none(), "Without a seat, player must not be able to sit");
    assert!(relief.board_vehicle_at(&frame, 2, 0, 1.75, ray, true).is_none(), "Without seat and steering wheel, object stays on crane");

    assert!(relief.stack_molded_cell(2, -1, 1.5, BlockMold::cube().in_medium(BlockMedium::CabinSeat).shape_for(frame.bitangent(), &frame)));
    let perch = relief.board_vehicle_at(&frame, 2, -1, 1.75, ray, false).expect("With a seat installed, player must be able to sit");
    assert!(!perch.drivable());
    assert!(relief.board_vehicle_at(&frame, 2, 0, 1.75, ray, true).is_none(), "Without a steering wheel, object must stay on the crane");
    assert!(relief.lower_rig_at(0, 0));
    assert!(relief.board_vehicle_at(&frame, 2, -1, 0.45, ray, true).is_none(), "Even lowered, object without steering wheel stays on crane");
    assert!(relief.adjust_rig_at(0, 0, 2, false).0, "Crane must still hoist the steerable-less object back up");
    assert!(relief.layer_occupied_at(2, 0, 1.25));
}

#[test]
fn test_decorative_wheels_seat_and_helm_on_world_cube_house_remain_static_world() {
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut house = TerracedRelief::in_span(Box::new(FlatMeadow), 64);

    assert!(house.stack_molded_cell(4, 4, 0.0, BlockMold::cube().in_medium(BlockMedium::PlanedPlanks).shape_for(frame.tangent(), &frame)));
    assert!(house.stack_molded_cell(4, 5, 0.0, BlockMold::cube().in_medium(BlockMedium::CarWheel).shape_for(frame.tangent(), &frame)));
    assert!(house.stack_molded_cell(5, 4, 0.0, BlockMold::cube().in_medium(BlockMedium::CabinSeat).shape_for(frame.bitangent(), &frame)));
    assert!(house.stack_molded_cell(5, 5, 0.0, BlockMold::cube().in_medium(BlockMedium::SteeringWheel).shape_for(frame.bitangent(), &frame)));

    let ray = SurfaceSightRay::new(frame.point_at(2.75, 2.25, 1.5), frame.bitangent());
    assert!(house.board_vehicle_at(&frame, 5, 4, 0.25, ray, false).is_some(), "House seat must still allow sitting");
    assert!(house.board_vehicle_at(&frame, 5, 5, 0.25, ray, true).is_none(), "World-Cube house with decorative wheels must never become a car");
    assert_eq!(house.harvest_layer(4, 5, 0.25).1, Some(BlockMedium::CarWheel), "Decorative wheel on house must remain freely removable");

    let mut cam_relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    for h in [0.0, 0.5, 1.0] {
        assert!(cam_relief.stack_molded_cell(0, 0, h, Box::new(CuboidShape::new(0.5))));
    }
    let mut dirt_out = NullTarget;
    let mut three_dirt = CountedMeshTarget::new(&mut dirt_out);
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &cam_relief, &palette, 64, &mut three_dirt);
    assert!(cam_relief.stack_molded_cell(0, 0, 1.5, BlockMold::cube().in_medium(BlockMedium::FieldCamera).shape_for(frame.tangent(), &frame)));
    let mut cam_out = NullTarget;
    let mut with_cam = CountedMeshTarget::new(&mut cam_out);
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &cam_relief, &palette, 64, &mut with_cam);
    assert!(with_cam.count() >= three_dirt.count() + 30);
}
