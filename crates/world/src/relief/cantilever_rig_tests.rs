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

struct QuadCollector {
    quads: Vec<Quad>,
}

impl MeshTarget for QuadCollector {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

#[test]
fn test_crane_arm_advances_to_remaining_dirt_block_and_pedestal_fills_ground_cell() {
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);

    let rig_shape = Box::new(MediumShape::new(Box::new(CuboidShape::new(0.5)), BlockMedium::CantileverRig, true, false));
    assert!(relief.stack_molded_cell(0, 0, 0.0, rig_shape));

    let mut base_collector = QuadCollector { quads: Vec::new() };
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &relief, &palette, 64, &mut base_collector);
    let full_pedestal = base_collector.quads.iter().any(|q| {
        q.transfer(|v0, v1, v2, v3, _, _| {
            let (mut min_u, mut max_u, mut min_v, mut max_v, mut max_z) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN, f32::MIN);
            for pt in [v0, v1, v2, v3] {
                let (u, pv) = frame.decompose(pt);
                min_u = min_u.min(u);
                max_u = max_u.max(u);
                min_v = min_v.min(pv);
                max_v = max_v.max(pv);
                max_z = max_z.max(frame.elevation_of(pt));
            }
            (min_u - 0.0).abs() < 1e-4
                && (max_u - 0.5).abs() < 1e-4
                && (min_v - 0.0).abs() < 1e-4
                && (max_v - 0.5).abs() < 1e-4
                && (max_z - 0.08).abs() < 1e-4
        })
    });
    assert!(full_pedestal, "Crane ground pedestal must cover the full 0.5m x 0.5m block cell footprint");

    let max_arm_u = |rel: &TerracedRelief| -> f32 {
        let mut col = QuadCollector { quads: Vec::new() };
        ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, rel, &palette, 64, &mut col);
        col.quads.iter().fold(f32::MIN, |acc, q| {
            q.transfer(|v0, v1, v2, v3, _, _| [v0, v1, v2, v3].into_iter().fold(acc, |m, pt| m.max(frame.decompose(pt).0)))
        })
    };

    assert!(relief.stack_molded_cell(0, 0, 0.0, Box::new(CuboidShape::new(0.5))));
    assert!(relief.layer_occupied_at(2, 0, 1.75));
    let reach_at_two = max_arm_u(&relief);

    assert!(relief.stack_molded_cell(1, 0, 1.5, Box::new(CuboidShape::new(0.5))));
    assert!(relief.layer_occupied_at(1, 0, 1.75));
    let reach_at_one = max_arm_u(&relief);
    assert!(reach_at_one < reach_at_two - 0.4, "Placing a block in the claw slot must retract the crane arm by 1 block");

    assert_eq!(relief.harvest_layer(1, 0, 1.75).1, Some(BlockMedium::TurfEarth));
    assert!(!relief.layer_occupied_at(1, 0, 1.75) && relief.layer_occupied_at(2, 0, 1.75));
    assert!((max_arm_u(&relief) - reach_at_two).abs() < 1e-4, "Removing the block at the crane must advance the arm 1 block forward to (2,0)");

    assert!(relief.stack_molded_cell(3, 0, 1.5, Box::new(CuboidShape::new(0.5))));
    assert!(relief.layer_occupied_at(3, 0, 1.75));
    assert_eq!(relief.harvest_layer(2, 0, 1.75).1, Some(BlockMedium::TurfEarth));
    assert!(!relief.layer_occupied_at(2, 0, 1.75) && relief.layer_occupied_at(3, 0, 1.75));
    assert!((max_arm_u(&relief) - (reach_at_two + 0.5)).abs() < 1e-4, "Removing the first block at (2,0) must advance the arm 1 block forward to (3,0)");
}

#[test]
fn test_dirt_blocks_on_crane_and_parked_car_stay_bare_without_grass_blades() {
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);

    let rig_shape = Box::new(MediumShape::new(Box::new(CuboidShape::new(0.5)), BlockMedium::CantileverRig, true, false));
    assert!(relief.stack_molded_cell(0, 0, 0.0, rig_shape));
    assert!(relief.stack_molded_cell(0, 0, 0.0, Box::new(CuboidShape::new(0.5))));
    assert!(!relief.bare_dirt_at(2, 0), "Mounting dirt on crane must leave meadow ground underneath unscarred");

    let elevated_grass = |rel: &TerracedRelief, cell_u: i32, cell_v: i32| -> bool {
        let mut col = QuadCollector { quads: Vec::new() };
        ReliefStrata::new().emit_cell(GridCell::new(cell_u, cell_v), &frame, rel, &palette, 64, &mut col);
        col.quads.iter().any(|q| {
            let glow = q.radiate(|g| g);
            q.transfer(|v0, v1, v2, v3, _, color| {
                let z = [v0, v1, v2, v3].into_iter().fold(0.0f32, |m, p| m.max(frame.elevation_of(p)));
                z > 0.05 && (glow > 1.5 || (color.y > color.x * 1.15 && color.y > color.z * 1.15))
            })
        })
    };

    assert!(!elevated_grass(&relief, 2, 0), "Dirt block placed on crane arm must immediately lack grass layer and blades");
    assert!(relief.lower_rig_at(0, 0));
    assert!(relief.step_grass(1000.0).is_empty());
    assert!(!elevated_grass(&relief, 2, 0), "Dirt block on crane arm must never grow grass even when lowered");

    assert!(relief.stack_molded_cell(2, 1, 0.0, BlockMold::cube().in_medium(BlockMedium::CarWheel).shape_for(frame.tangent(), &frame)));
    assert!(relief.stack_molded_cell(2, -1, 0.0, BlockMold::cube().in_medium(BlockMedium::CabinSeat).shape_for(frame.bitangent(), &frame)));
    assert!(relief.stack_molded_cell(1, 0, 0.0, BlockMold::cube().in_medium(BlockMedium::SteeringWheel).shape_for(frame.bitangent(), &frame)));

    let ray = SurfaceSightRay::new(frame.point_at(0.75, 0.25, 1.5), -frame.normal());
    let car = relief.board_vehicle_at(&frame, 1, 0, 0.25, ray, true).expect("Complete vehicle must release from crane");
    let seat_cell = relief.park_vehicle_at(&frame, car);
    assert!(relief.step_grass(1000.0).is_empty());
    for dv in -2..=2 {
        for du in -2..=2 {
            assert!(!elevated_grass(&relief, seat_cell.0 + du, seat_cell.1 + dv), "Dirt block on parked car must never grow grass");
        }
    }

    assert!(relief.stack_molded_cell(8, 8, 0.0, Box::new(CuboidShape::new(0.5))) && relief.bare_dirt_at(8, 8));
    assert!(!elevated_grass(&relief, 8, 8), "Placed dirt block on ground must start bare without grass");
    assert!(!relief.step_grass(120.0).is_empty() && !relief.bare_dirt_at(8, 8));
    assert!(elevated_grass(&relief, 8, 8), "Ground-connected dirt block regrows grass after neighbor contact");
    assert_eq!(relief.harvest_layer(8, 8, 0.25).1, Some(BlockMedium::TurfEarth));
    assert!(relief.stack_molded_cell(8, 8, 0.0, Box::new(CuboidShape::new(0.5))) && relief.bare_dirt_at(8, 8));
    assert!(!elevated_grass(&relief, 8, 8), "Harvested grass block must be placed bare without grass again");
}

#[test]
fn test_dismantled_crane_adopts_bare_dirt_and_smothered_meadow_renders_loam() {
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);

    let rig_shape = Box::new(MediumShape::new(Box::new(CuboidShape::new(0.5)), BlockMedium::CantileverRig, true, false));
    assert!(relief.stack_molded_cell(0, 0, 0.0, rig_shape));
    assert!(relief.stack_molded_cell(0, 0, 0.0, Box::new(CuboidShape::new(0.5))));
    assert_eq!(relief.harvest_layer(0, 0, 0.25).1, Some(BlockMedium::CantileverRig));
    assert!(relief.bare_dirt_at(2, 0) && relief.ground_bare_at(2, 0), "Dismantled crane block must land bare and smother ground below");
    assert!(!relief.step_grass(120.0).is_empty() && !relief.bare_dirt_at(2, 0), "Dismantled dirt block on ground must regrow grass");
    assert!(relief.ground_bare_at(2, 0), "Ground beneath verdant block must stay bare");

    assert_eq!(relief.harvest_layer(2, 0, 0.25).1, Some(BlockMedium::TurfEarth));
    let mut col = QuadCollector { quads: Vec::new() };
    ReliefStrata::new().emit_cell(GridCell::new(2, 0), &frame, &relief, &palette, 64, &mut col);
    assert!(!col.quads.iter().any(|q| q.radiate(|g| g > 0.01)), "Vacated smothered meadow must render bare loam without grass glow");
}
