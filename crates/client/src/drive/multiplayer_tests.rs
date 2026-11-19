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
        false,
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
