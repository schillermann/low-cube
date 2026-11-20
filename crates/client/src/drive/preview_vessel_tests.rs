use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, CraftBlueprint, CraftPose, MeshTarget, MountDomain,
    PlanetaryWorld, Quad, VesselSink, WorldEarthworks as _,
};
use crate::attire::{HeldRelic, YouthAttire};
use crate::avatar::{Avatar as _, AvatarPresence as _, HandGesture, PlanetaryAvatar};
use crate::controls::ActionBindings;
use crate::flight::SurfaceFlight;

struct GlowQuadCollector {
    quads: Vec<Quad>,
}

impl GlowQuadCollector {
    const fn empty() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for GlowQuadCollector {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

struct VesselProbe {
    captured: Vec<(CraftBlueprint, CraftPose)>,
}

impl VesselProbe {
    const fn empty() -> Self {
        Self { captured: Vec::new() }
    }
}

impl VesselSink for VesselProbe {
    fn receive_vessel(&mut self, blueprint: &CraftBlueprint, pose: CraftPose) {
        self.captured.push((blueprint.clone(), pose));
    }
}

#[test]
fn test_hologram_preview_distinguishes_craft_rig_and_world_cube_domains_and_matches_placement() {
    let mut world = BiomedCubeWorld::barren_beveled(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let far = Vec3::new(6.0, g0 + 1.0, 6.0);
    assert!(world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 0.25),
        Vec3::NEG_Y,
        far,
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
    ));

    let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.25), 1.0);
    let mut builder = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(SurfaceFlight::calibrated(0.522, 9.81, 1.0)),
        YouthAttire::squire().with_relics(HeldRelic::FlightConsole, HeldRelic::EmptyPalm),
        ActionBindings::seeded(),
    );
    builder.rotate_degrees(-90.0, 0.0);

    let rig_plan = builder.planned_mount(&world).expect("Crane jaw must produce a PlannedMount preview");
    assert_eq!(rig_plan.domain(), MountDomain::CraftRig);
    assert_eq!(rig_plan.cell_coordinates(), (2, 0, 1.5));

    let mut rig_holo = GlowQuadCollector::empty();
    builder.emit_placement_preview(&world, &mut rig_holo);
    assert!(rig_holo.quads.len() >= 30, "Hologram must emit sculpted shape plus 12 corner brackets");
    let cyan_glow = rig_holo.quads.iter().any(|q| {
        let g = q.radiate(|v| v);
        q.transfer(|_, _, _, _, _, col| g >= 0.78 && col.z > col.x * 1.4)
    });
    assert!(cyan_glow, "CraftRig hologram must glow in electric cyan");

    assert!(builder.dispatch_wardrobe_key(KeyCode::Tab));
    assert!(builder.planned_mount(&world).is_none(), "Opening satchel must suppress placement hologram");
    assert!(builder.dispatch_wardrobe_key(KeyCode::Tab));
    assert!(HandGesture::RightTap.perform(&mut builder, &mut world));
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 0.25), Vec3::NEG_Y, 1, false));

    builder.apply_attire(YouthAttire::squire().with_relics(HeldRelic::PlankBlock, HeldRelic::EmptyPalm));
    builder.rotate_degrees(180.0, 48.0);
    let cube_plan = builder.planned_mount(&world).expect("Ground aim must produce a WorldCube PlannedMount");
    assert_eq!(cube_plan.domain(), MountDomain::WorldCube);

    let mut cube_holo = GlowQuadCollector::empty();
    builder.emit_placement_preview(&world, &mut cube_holo);
    let emerald_glow = cube_holo.quads.iter().any(|q| {
        let g = q.radiate(|v| v);
        q.transfer(|_, _, _, _, _, col| g >= 0.70 && col.y > col.x * 1.25 && col.y > col.z)
    });
    assert!(emerald_glow, "WorldCube hologram must glow in warm emerald green");
}

#[test]
fn test_parked_vehicle_preserves_free_360_yaw_without_terrain_rebuilds_and_transforms_local_hull() {
    let mut world = BiomedCubeWorld::barren_beveled(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let far = Vec3::new(6.0, g0 + 1.0, 6.0);
    assert!(world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 0.25),
        Vec3::NEG_Y,
        far,
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 0.25),
        Vec3::NEG_Y,
        far,
        BlockMold::cube().in_medium(BlockMedium::SteeringWheel),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 0.75),
        Vec3::NEG_Z,
        far,
        BlockMold::cube().in_medium(BlockMedium::CabinSeat),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, -0.25),
        Vec3::Z,
        far,
        BlockMold::cube().in_medium(BlockMedium::CarWheel),
    ));

    let mut sink = GlowQuadCollector::empty();
    while world.drain_modified_patch(&mut sink).is_some() {}
    while world.drain_modified_slice(&mut sink).is_some() {}

    let mut car = world
        .board_vehicle(Vec3::new(1.25, g0 + 2.6, 0.25), Vec3::NEG_Y, true)
        .expect("Complete car must release from crane");
    while world.drain_modified_patch(&mut sink).is_some() {}
    while world.drain_modified_slice(&mut sink).is_some() {}
    assert!(!world.pending_slices());

    let g_park = world.constrain_to_surface(Vec3::new(4.37, 22.0, 3.63), 0.0).y;
    car.translate_to(Vec3::new(4.37, g_park, 3.63));
    car.steer(0.645);
    let angled_heading = car.heading();
    assert!(angled_heading.x.abs() > 0.35 && angled_heading.z.abs() > 0.35, "Heading must be non-orthogonal");

    let pose = car.pose().with_motion(0.4, -0.2);
    let model = pose.model_matrix();
    let (mut cpu_quads, mut local_quads, mut kin_quads) = (
        GlowQuadCollector::empty(),
        GlowQuadCollector::empty(),
        GlowQuadCollector::empty(),
    );
    car.blueprint().emit_at(pose, &mut cpu_quads);
    car.blueprint().emit_local_hull(&mut local_quads);
    car.blueprint().emit_kinematic_parts(pose, &mut kin_quads);
    assert!(!local_quads.quads.is_empty() && !kin_quads.quads.is_empty());
    assert_eq!(local_quads.quads.len() + kin_quads.quads.len(), cpu_quads.quads.len());

    let (mut gpu_min, mut gpu_max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for q in &local_quads.quads {
        q.transfer(|v0, v1, v2, v3, _, _| {
            for v in [v0, v1, v2, v3] {
                let w = model.transform_point3(v);
                gpu_min = gpu_min.min(w);
                gpu_max = gpu_max.max(w);
            }
        });
    }
    for q in &kin_quads.quads {
        q.transfer(|v0, v1, v2, v3, _, _| {
            for v in [v0, v1, v2, v3] {
                gpu_min = gpu_min.min(v);
                gpu_max = gpu_max.max(v);
            }
        });
    }
    let (mut cpu_min, mut cpu_max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for q in &cpu_quads.quads {
        q.transfer(|v0, v1, v2, v3, _, _| {
            for v in [v0, v1, v2, v3] {
                cpu_min = cpu_min.min(v);
                cpu_max = cpu_max.max(v);
            }
        });
    }
    assert!((gpu_min - cpu_min).length() < 1e-4 && (gpu_max - cpu_max).length() < 1e-4);

    assert!(world.park_vehicle(car));
    assert!(!world.pending_slices(), "Parking a vehicle must rebuild zero terrain sectors");

    let mut vessels = VesselProbe::empty();
    world.present_vessels(&mut vessels);
    assert_eq!(vessels.captured.len(), 1);
    assert!((vessels.captured[0].1.heading() - angled_heading).length() < 1e-4);
    assert!((vessels.captured[0].1.origin() - Vec3::new(4.37, g_park, 3.63)).length() < 1e-4);

    let helm_eye = Vec3::new(4.37, g_park + 1.15, 3.63) - angled_heading * 0.25;
    let reboarded = world
        .board_vehicle(helm_eye, (angled_heading - Vec3::Y * 0.45).normalize(), true)
        .expect("Angled parked car must re-board cleanly along its 3D helm axis");
    assert!(!world.pending_slices(), "Boarding a parked vehicle must rebuild zero terrain sectors");
    assert!(reboarded.heading().dot(angled_heading) > 0.9999, "Re-boarded vehicle must preserve exact 360-degree yaw");
}
