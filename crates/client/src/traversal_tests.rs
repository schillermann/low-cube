use glam::Vec3;
use world::{
    BiomedQuadSphere, BlockMedium, ClimateZone, FacetedReliefFace, GravityField, MoldArchetype,
    NullFace, PlanetaryWorld, QuadSphere, RawAtom, WedgeHeading,
};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::gait::{SneakGait, SprintGait, WalkGait};

fn world_with_atom(u: i32, v: i32, atom: RawAtom) -> QuadSphere {
    let face = Box::new(
        FacetedReliefFace::in_zone(ClimateZone::Temperate, 8.0, 16)
            .with_molded_block(u, v, 0.0, atom),
    );
    QuadSphere::new(vec![face], Box::new(NullFace))
}

#[test]
fn test_walk_into_solid_cube_blocked() {
    let world = world_with_atom(1, 0, RawAtom::turf_cube());
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let start_pos = world.constrain_to_surface(frame.point_at(0.45, 0.25, 1.0), 1.0);
    let up = world.surface_up_at(start_pos);
    let east = (Vec3::X - up * Vec3::X.dot(up)).normalize();

    let next_pos = flight.advance(1.0 / 60.0, start_pos, east * 0.1, &WalkGait::upright(), up, &world);

    assert!((next_pos - start_pos).length() < 1e-4);
}

#[test]
fn test_walk_up_ramp_wedge_smoothly() {
    let world = world_with_atom(0, 0, RawAtom::molded(BlockMedium::TurfEarth, MoldArchetype::RampWedge, WedgeHeading::North));
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let start_pos = world.constrain_to_surface(frame.point_at(0.25, 0.10, 1.0), 1.0);
    let up = world.surface_up_at(start_pos);
    let north = (Vec3::Z - up * Vec3::Z.dot(up)).normalize();

    let next_pos = flight.advance(1.0 / 60.0, start_pos, north * 0.1, &WalkGait::upright(), up, &world);

    assert!(next_pos.z > start_pos.z);
    assert!(next_pos.length() > start_pos.length());
}

#[test]
fn test_walk_into_wall_while_strafing_preserves_lateral_motion() {
    let world = world_with_atom(1, 0, RawAtom::turf_cube());
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let start_pos = world.constrain_to_surface(frame.point_at(0.45, 0.15, 1.0), 1.0);
    let up = world.surface_up_at(start_pos);
    let raw = Vec3::new(0.1, 0.0, 0.1);
    let stride = raw - up * raw.dot(up);

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &WalkGait::upright(), up, &world);

    let (u0, v0) = frame.decompose(start_pos);
    let (u1, v1) = frame.decompose(next_pos);
    assert!((u1 - u0).abs() < 1e-4);
    assert!(v1 > v0 + 0.05);
}

#[test]
fn test_walk_off_elevated_block_falls_when_not_sneaking() {
    let world = world_with_atom(1, 0, RawAtom::turf_cube());
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let start_pos = world.constrain_to_surface(frame.point_at(0.95, 0.25, 1.5), 1.0);
    assert!((start_pos.length() - 9.5).abs() < 1e-4);
    let up = world.surface_up_at(start_pos);
    let east = (Vec3::X - up * Vec3::X.dot(up)).normalize();

    let step_pos = flight.advance(1.0 / 60.0, start_pos, east * 0.1, &WalkGait::upright(), up, &world);
    assert!(frame.decompose(step_pos).0 > 1.0);

    let mut pos = step_pos;
    for _ in 0..50 {
        let cur_up = world.surface_up_at(pos);
        pos = flight.advance(1.0 / 60.0, pos, Vec3::ZERO, &WalkGait::upright(), cur_up, &world);
    }
    assert!((pos.length() - 9.0).abs() < 1e-4);
}

#[test]
fn test_sneak_on_elevated_block_prevents_falling() {
    let world = world_with_atom(1, 0, RawAtom::turf_cube());
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let start_pos = world.constrain_to_surface(frame.point_at(0.95, 0.25, 1.5), 1.0);
    assert!((start_pos.length() - 9.5).abs() < 1e-4);
    let up = world.surface_up_at(start_pos);
    let east = (Vec3::X - up * Vec3::X.dot(up)).normalize();

    let next_pos = flight.advance(1.0 / 60.0, start_pos, east * 0.1, &SneakGait::crouching(), up, &world);

    assert!((next_pos - start_pos).length() < 1e-4);
}

#[test]
fn test_sneak_on_elevated_block_diagonal_slides_along_edge() {
    let world = world_with_atom(1, 0, RawAtom::turf_cube());
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let start_pos = world.constrain_to_surface(frame.point_at(0.95, 0.15, 1.5), 1.0);
    let up = world.surface_up_at(start_pos);
    let raw = Vec3::new(0.1, 0.0, 0.1);
    let stride = raw - up * raw.dot(up);

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &SneakGait::crouching(), up, &world);

    let (u0, v0) = frame.decompose(start_pos);
    let (u1, v1) = frame.decompose(next_pos);
    assert!((u1 - u0).abs() < 1e-3 && u1 < 1.0);
    assert!(v1 > v0 + 0.05);
    assert!((next_pos.length() - 9.5).abs() < 1e-4);
}

#[test]
fn test_sneak_paces_stride_distance() {
    use crate::controls::LocomotionTarget;
    use crate::order::LocomotionOrder;

    let mut walk_order = LocomotionOrder::idle();
    walk_order.travel(Vec3::X);
    let walk_stride = walk_order.surface_stride(Vec3::Y, 1.0);

    let mut sneak_order = LocomotionOrder::idle();
    sneak_order.travel(Vec3::X);
    sneak_order.sneak();
    let sneak_stride = sneak_order.surface_stride(Vec3::Y, 1.0);

    assert_eq!(walk_stride.x, 1.0);
    assert!((sneak_stride.x - 0.35).abs() < 1e-5);
}

#[test]
fn test_sprint_jump_over_ramp_wedge_does_not_get_stuck() {
    let world = world_with_atom(0, 0, RawAtom::molded(BlockMedium::TurfEarth, MoldArchetype::RampWedge, WedgeHeading::North));
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let mut pos = world.constrain_to_surface(frame.point_at(0.25, -0.60, 1.0), 1.0);
    flight.jump();
    for _ in 0..35 {
        let prev_v = frame.decompose(pos).1;
        let up = world.surface_up_at(pos);
        let north = (Vec3::Z - up * Vec3::Z.dot(up)).normalize();
        pos = flight.advance(1.0 / 60.0, pos, north * 0.16, &SprintGait::swift(), up, &world);
        let next_v = frame.decompose(pos).1;
        assert!(next_v > prev_v + 0.05, "Sprint jump onto ramp slope must never stall, stalled at v = {next_v}");
    }
    assert!(frame.decompose(pos).1 > 2.0);
}

#[test]
fn test_sprint_jump_on_biomed_quad_sphere_slopes() {
    let world = BiomedQuadSphere::of_size(32);
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let mut pos = world.constrain_to_surface(frame.point_at(4.5, 4.5, 1.0), 1.0);
    flight.jump();
    for step in 0..40 {
        let up = world.surface_up_at(pos);
        let north = (Vec3::Z - up * Vec3::Z.dot(up)).normalize();
        pos = flight.advance(1.0 / 60.0, pos, north * 0.2, &SprintGait::swift(), up, &world);
        let ground = world.constrain_to_surface(pos, 1.0);
        assert!(
            pos.length() >= ground.length() - 0.01,
            "Frame {step}: radial r ({}) must not penetrate below ground ({})",
            pos.length(),
            ground.length()
        );
    }
    assert!(frame.decompose(pos).1 > 8.0);
}

#[test]
fn test_full_cube_staircase_jump_climbs_block_by_block_without_flying() {
    let face = Box::new(
        FacetedReliefFace::in_zone(ClimateZone::Temperate, 8.0, 16)
            .with_molded_block(0, 2, 0.0, RawAtom::turf_cube())
            .with_molded_block(0, 3, 0.0, RawAtom::turf_cube())
            .with_molded_block(0, 3, 0.5, RawAtom::turf_cube())
            .with_molded_block(0, 4, 0.0, RawAtom::turf_cube())
            .with_molded_block(0, 4, 0.5, RawAtom::turf_cube())
            .with_molded_block(0, 4, 1.0, RawAtom::turf_cube()),
    );
    let world = QuadSphere::new(vec![face], Box::new(NullFace));
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let mut pos = world.constrain_to_surface(frame.point_at(0.25, 0.75, 1.0), 1.0);
    let mut r_after_15 = 0.0;
    let mut landings = Vec::new();
    let mut prev_r = pos.length();
    for step in 0..85 {
        flight.jump();
        let up = world.surface_up_at(pos);
        let north = (Vec3::Z - up * Vec3::Z.dot(up)).normalize();
        pos = flight.advance(1.0 / 60.0, pos, north * 0.10, &WalkGait::upright(), up, &world);
        let cur_r = pos.length();
        if step == 14 { r_after_15 = cur_r; }
        for target_r in [9.5f32, 10.0, 10.5] {
            if prev_r < target_r - 1e-4 && (cur_r - target_r).abs() < 2e-3 && !landings.contains(&target_r) {
                landings.push(target_r);
            }
        }
        prev_r = cur_r;
    }
    assert!(r_after_15 < 9.60, "Held jump on full-cube stairs must not fly over multiple blocks in 15 frames, got {r_after_15}");
    assert_eq!(landings, vec![9.5, 10.0, 10.5], "Avatar must climb and land on each full-cube stair step sequentially, got {landings:?}");
}
