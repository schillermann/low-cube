use glam::Vec3;
use sanctuary::{HostedPlanet, JoinTarget, RemoteLink, RemotePlanet, SanctuaryHost};
use winit::keyboard::KeyCode;
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, MeshTarget, PlanetaryWorld, Quad, WorldEarthworks as _,
};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarPresence as _, HandGesture, PlanetaryAvatar,
};
use crate::companion::CompanionFigure;
use crate::controls::{KeyMotion, KeyboardControls};

struct CentroidProbe {
    sum_x: f32,
    count: usize,
}

impl CentroidProbe {
    const fn empty() -> Self {
        Self { sum_x: 0.0, count: 0 }
    }

    fn mean_x(&self) -> f32 {
        self.sum_x / (self.count.max(1) as f32)
    }
}

impl MeshTarget for CentroidProbe {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, v1, v2, v3, _, _| {
            self.sum_x += v0.x + v1.x + v2.x + v3.x;
            self.count += 4;
        });
    }
}

#[test]
fn test_multiplayer_companion_car_renders_and_interpolates_subtick_without_teleporting() {
    let host = SanctuaryHost::bind(
        "127.0.0.1:0",
        Box::new(BiomedCubeWorld::barren_beveled(32, 1.0)),
        32,
    )
    .expect("bind host");
    let addr = host.local_addr().expect("addr");
    let mut host_world = HostedPlanet::new(host);
    let mut guest_world = RemotePlanet::new(
        Box::new(BiomedCubeWorld::barren_beveled(32, 1.0)),
        RemoteLink::bind(JoinTarget::Direct(addr)).expect("guest link"),
        std::collections::VecDeque::new(),
    );
    for _ in 0..8 {
        let _ = guest_world.step_surface(0.03);
        let _ = host_world.step_surface(0.03);
    }

    let g0 = host_world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av_pos = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    assert!(host_world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 1.25),
        crane_dir,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
    ));
    assert!(host_world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 1.25),
        crane_dir,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::PedalCluster),
    ));
    assert!(host_world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    let wheel_mold = BlockMold::cube().next_form().in_medium(BlockMedium::CarWheel);
    for (eye, dir, mold) in [
        (Vec3::new(1.25, g0 + 2.40, 0.25), Vec3::NEG_Y, BlockMold::cube().turn_clockwise().in_medium(BlockMedium::SteeringWheel)),
        (Vec3::new(1.25, g0 + 1.25, 0.75), Vec3::NEG_Z, wheel_mold),
        (Vec3::new(1.25, g0 + 1.75, 1.25), Vec3::NEG_Z, BlockMold::cube().in_medium(BlockMedium::CabinSeat)),
        (Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, wheel_mold),
        (Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, wheel_mold.next_form()),
        (Vec3::new(1.25, g0 + 2.35, 0.75), Vec3::NEG_Y, BlockMold::cube().in_medium(BlockMedium::PlanedPlanks)),
    ] {
        assert!(host_world.place_molded_surface_block(eye, dir, av_pos, mold));
    }

    let mut host_driver = PlanetaryAvatar::at(host_world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0));
    host_driver.rotate_degrees(-90.0, 12.0);
    assert!(HandGesture::RightTap.perform(&mut host_driver, &mut host_world));
    assert!(host_driver.use_right_hand(&mut host_world));
    for _ in 0..6 {
        let _ = host_world.step_surface(0.03);
        let _ = guest_world.step_surface(0.03);
    }
    assert!(
        guest_world.constrain_to_surface(Vec3::new(1.25, 22.0, 1.25), 0.0).y < g0 + 0.5,
        "Car must not remain standing at the crane on the other player's screen once boarded"
    );

    let mut gas = KeyMotion::idle();
    gas.register_key(KeyCode::KeyW, true);
    let dt = 1.0 / 120.0;
    let mut guest_xs = Vec::new();
    for tick in 0..40 {
        host_driver.step_motion(dt, &gas, &host_world);
        host_driver.sync_world_presence(&mut host_world);
        let _ = host_world.step_surface(dt);
        let _ = guest_world.step_surface(dt);

        if tick == 20 {
            let (mut p0, mut p_mid, mut p1) = (CentroidProbe::empty(), CentroidProbe::empty(), CentroidProbe::empty());
            host_driver.blend_subtick(0.0);
            host_driver.emit_appearance(&mut p0);
            host_driver.blend_subtick(0.5);
            host_driver.emit_appearance(&mut p_mid);
            host_driver.blend_subtick(1.0);
            host_driver.emit_appearance(&mut p1);
            assert!(
                p0.mean_x() < p_mid.mean_x() && p_mid.mean_x() < p1.mean_x(),
                "Local DrivenCar must interpolate smoothly across sub-ticks ({} < {} < {})",
                p0.mean_x(),
                p_mid.mean_x(),
                p1.mean_x()
            );
        }

        if tick >= 10 {
            let mut ext = CentroidProbe::empty();
            CompanionFigure::emit_all(&guest_world, &mut ext);
            assert!(ext.count > 200, "Guest must render both companion avatar and driven car mesh");
            guest_xs.push(ext.mean_x());
        }
    }
    let smooth_deltas = guest_xs.windows(2).filter(|w| w[1] > w[0] + 1e-4 && w[1] - w[0] < 0.25).count();
    assert!(smooth_deltas >= 15, "Guest must observe continuous car motion rather than discrete teleports");

    assert!(host_driver.use_left_hand(&mut host_world));
    host_driver.sync_world_presence(&mut host_world);
    for _ in 0..6 {
        let _ = host_world.step_surface(0.03);
        let _ = guest_world.step_surface(0.03);
    }
    let dest_x = *guest_xs.last().expect("drove forward");
    let guest_parked = (0..8).any(|dx| {
        let probe_x = dest_x - 1.0 + (dx as f32) * 0.5;
        guest_world.constrain_to_surface(Vec3::new(probe_x, 22.0, 0.75), 0.0).y > g0 + 0.5
    });
    assert!(guest_parked, "Parked car must appear at its driven destination on the other player's world");
}

#[test]
fn test_unseated_deck_crew_stays_aboard_heliosphere_ship_across_liftoff_and_orbit() {
    use world::{CelestialSystem, CompanionPortal, CraftBlueprint, DrivenCraft, PilotCraft, SolarProportions, SurfaceFrame, VehicleAxes, VehiclePart};

    let prop = SolarProportions::compressed(128, 0.02);
    let earth_half = prop.earth_half();
    let mut raw_host = SanctuaryHost::bind_cosmos(
        "127.0.0.1:0",
        CelestialSystem::barren_heliosphere(prop, 1.0),
        prop,
    )
    .expect("bind heliosphere host");
    let addr = raw_host.local_addr().expect("addr");

    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let mut parts = Vec::new();
    for gx in -1..=1 {
        for gz in -1..=1 {
            parts.push(VehiclePart::new(
                (gx, gz),
                0.0,
                BlockMold::cube().in_medium(BlockMedium::PlanedPlanks).shape_for(Vec3::NEG_Z, &frame),
            ));
        }
    }
    parts.push(VehiclePart::new((0, 1), 0.5, BlockMold::cube().in_medium(BlockMedium::FlightConsole).shape_for(Vec3::NEG_Z, &frame)));
    parts.push(VehiclePart::new((0, -1), 0.0, BlockMold::cube().in_medium(BlockMedium::RocketNozzle).shape_for(Vec3::NEG_Z, &frame)));
    let bp = CraftBlueprint::commission(Vec3::new(0.25, earth_half + 2.0, 0.25), parts);
    let ship_id = bp.identity();

    raw_host.with_chronicled(|w| {
        let _ = w.register_craft_blueprint(bp.clone());
    });
    raw_host.record_local_fact(world::WorldFact::CraftCommissioned { blueprint: bp });
    let mut host_world = HostedPlanet::spawning(raw_host);
    let mut crew_world = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("crew connect");

    let pad_origin = host_world.constrain_to_surface(Vec3::new(0.25, earth_half + 4.0, 0.25), 0.0) + Vec3::Y * 0.5;
    let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
    let pad_pose = world::CraftPose::new(pad_origin, axes, Vec3::Z, (0.0, 0.0));
    host_world.sync_presence(world::CompanionPresence::new(
        1,
        world::PresenceKinematics::new(pad_origin + Vec3::Y, glam::Quat::IDENTITY, glam::Quat::IDENTITY, (1.5, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        world::PresenceAttire::squire(),
    ));
    host_world.sync_pilot_craft(PilotCraft::Driving(DrivenCraft::new(1, ship_id, pad_pose)));
    crew_world.sync_presence(world::CompanionPresence::new(
        0,
        world::PresenceKinematics::new(pad_origin + Vec3::Y * 1.55, glam::Quat::IDENTITY, glam::Quat::IDENTITY, (1.5, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        world::PresenceAttire::squire(),
    ));
    for _ in 0..8 {
        let _ = host_world.step_surface(0.02);
        let _ = crew_world.step_surface(0.02);
    }

    // Unseated crewmate boards the stationary launchpad deck first, then ship lifts off into HeliosphereWorld orbit!
    let mut crew_avatar = PlanetaryAvatar::at(pad_origin + Vec3::Y * 1.55);
    let idle = KeyMotion::idle();
    let dt = 1.0 / 120.0;
    for _ in 0..8 {
        let _ = host_world.step_surface(0.02);
        let _ = crew_world.step_surface(0.02);
        crew_avatar.step_motion(dt, &idle, &crew_world);
        crew_avatar.sync_world_presence(&mut crew_world);
    }
    let mut ship_y = pad_origin.y;
    for tick in 0..120 {
        ship_y += 2.0; // +240 m vertical liftoff into HeliosphereWorld orbit (where solar + Earth gravity > 1e-4!)
        let burn = if tick < 90 { 1.2 } else { 0.0 }; // 90 ticks powered burn, 30 ticks 0.0g coast!
        let next_pose = world::CraftPose::new(Vec3::new(pad_origin.x, ship_y, pad_origin.z), axes, Vec3::Z, (burn, 0.0));
        host_world.sync_pilot_craft(PilotCraft::Driving(DrivenCraft::new(1, ship_id, next_pose)));
        host_world.sync_berth_presence(
            world::CompanionPresence::new(
                1,
                world::PresenceKinematics::new(
                    next_pose.world_point_at(Vec3::new(0.0, 1.12, 0.5)),
                    glam::Quat::IDENTITY,
                    glam::Quat::IDENTITY,
                    (1.5, 0.0),
                ),
                (0.0, 0.0, 0.0, 0.0),
                world::PresenceAttire::squire(),
            )
            .localized_to_hull(next_pose),
            world::VesselBerth::aboard(ship_id),
        );
        let _ = host_world.step_surface(dt);
        let _ = crew_world.step_surface(dt);
        crew_avatar.step_motion(dt, &idle, &crew_world);
        crew_avatar.sync_world_presence(&mut crew_world);
    }

    for _ in 0..8 {
        let _ = host_world.step_surface(0.02);
        let _ = crew_world.step_surface(0.02);
        crew_avatar.step_motion(dt, &idle, &crew_world);
        crew_avatar.sync_world_presence(&mut crew_world);
    }
    let mut host_seen = Vec::new();
    host_world.active_companions(&mut host_seen);
    assert_eq!(host_seen.len(), 1, "Host pilot must see unseated deck crewmate aboard the ascending ship");
    let crew_pos = host_seen[0].transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
    assert!(
        (crew_pos.y - (ship_y + 1.5)).abs() < 1.0,
        "Unseated crewmate must ride the ship deck into HeliosphereWorld orbit with < 1m deck precision, got y={} vs ship_y={}",
        crew_pos.y,
        ship_y
    );
}
