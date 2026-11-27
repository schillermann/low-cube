use glam::{Quat, Vec3};
use world::{
    BlockMedium, BlockMold, CardinalTurn, CelestialSystem, ClimateZone, CompanionPortal,
    CompanionPresence, CosmicFrame, CraftBlueprint, CraftPose, DrivenCraft, MeshTarget,
    MoldArchetype, PilotCraft, PlanetaryWorld, PresenceAttire, PresenceKinematics, Quad, SightCone,
    SignalKind, SkySignal, SolarEphemeris, SolarProportions, SurfaceFrame, TerrainMesh,
    VehicleAxes, VehiclePart, WorldEarthworks,
};
use crate::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryEnvelope, SanctuaryHost};

struct SignalMeshProbe {
    cyan_spikes: usize,
    emerald_beacons: usize,
}

impl MeshTarget for SignalMeshProbe {
    fn push_quad(&mut self, quad: Quad) {
        if !quad.radiate(|g| g > 0.9) {
            return;
        }
        quad.transfer(|_, _, _, _, _, col| {
            if col.z > 0.95 && col.y > 0.85 && col.x > 0.25 {
                self.cyan_spikes += 1;
            }
            if col.y > 0.95 && col.z > 0.50 && col.x < 0.65 {
                self.emerald_beacons += 1;
            }
        });
    }
}

fn make_pres(pos: Vec3) -> CompanionPresence {
    CompanionPresence::new(
        0,
        PresenceKinematics::new(pos, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::astronaut(),
    )
}

#[test]
fn test_sky_signal_wire_codec_zero_extra_bytes_when_idle() {
    let mut idle_bytes = Vec::new();
    SanctuaryEnvelope::GuestPulse {
        ack_seq: 1,
        presence: make_pres(Vec3::new(0.0, 64.0, 0.0)),
        craft: PilotCraft::Pedestrian,
        frame: CosmicFrame::earth(),
        berth: world::VesselBerth::planetary(),
        sky_watch: false,
    }
    .inscribe(&mut idle_bytes);

    let mut zoomed_bytes = Vec::new();
    let zoomed = SanctuaryEnvelope::GuestPulse {
        ack_seq: 1,
        presence: make_pres(Vec3::new(0.0, 64.0, 0.0)),
        craft: PilotCraft::Pedestrian,
        frame: CosmicFrame::earth(),
        berth: world::VesselBerth::planetary(),
        sky_watch: true,
    };
    zoomed.inscribe(&mut zoomed_bytes);
    assert_eq!(zoomed_bytes.len(), idle_bytes.len() + 4, "Zoomed Earth pulse adds only 3-byte frame + 1-byte watch tag");
    assert_eq!(SanctuaryEnvelope::decode(&zoomed_bytes), Some(zoomed));

    let sig = SkySignal::drive_star(Vec3::new(0.0, 0.8, -0.6), 42_500.0, 0.85);
    let mut raw = Vec::new();
    sig.inscribe(&mut raw);
    assert_eq!(raw.len(), SkySignal::WIRE_SPAN);
    let (dec, used) = SkySignal::decode(&raw).expect("decode SkySignal");
    assert_eq!(used, SkySignal::WIRE_SPAN);
    assert_eq!(dec.kind(), SignalKind::FusionDriveStar);
    assert!((dec.direction() - sig.direction()).length() < 0.001);
}

#[test]
fn test_drive_star_and_laser_beacon_horizon_and_telescope_culling_over_udp() {
    let prop = SolarProportions::true_scale(128);
    let earth_half = prop.earth_half();
    let eph = SolarEphemeris::new(prop, 0.0);

    // Find one sunlit ClimateZone (`dot > 0.2`) and one nightside ClimateZone (`dot < -0.2`) at epoch 0.0:
    let mut day_zone = ClimateZone::Temperate;
    let mut night_zone = ClimateZone::Volcanic;
    for zone in ClimateZone::all() {
        let spot = zone.normal() * (earth_half + 1.0);
        let sun_dir = eph.sun_direction_from(spot.as_dvec3(), 0.0);
        if zone.normal().dot(sun_dir) > 0.25 {
            day_zone = zone;
        } else if zone.normal().dot(sun_dir) < -0.25 {
            night_zone = zone;
        }
    }

    let host_world = CelestialSystem::barren_heliosphere(prop, 1.0);
    let raw_host = SanctuaryHost::bind_cosmos("127.0.0.1:0", host_world, prop).expect("bind host");
    let addr = raw_host.local_addr().expect("addr");
    let mut host = HostedPlanet::spawning(raw_host);

    // 1. Place a LaserBeacon on Temperate (`+Y`) and commission an Epstein FusionTorch ship for Pilot Alice:
    let g_temp = host.constrain_to_surface(Vec3::new(2.25, earth_half + 4.0, 0.25), 0.0);
    let beacon_mold = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::LaserBeacon);
    assert!(host.place_molded_surface_block(g_temp + Vec3::Y * 1.5, Vec3::NEG_Y, g_temp + Vec3::Z * 2.5, beacon_mold));

    let stand = host.constrain_to_surface(Vec3::new(0.25, earth_half + 4.0, 0.25), 0.0);
    let crane_eye = Vec3::new(0.25, stand.y + 1.5, 1.25);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    assert!(host.place_molded_surface_block(
        crane_eye,
        crane_dir,
        Vec3::new(0.25, stand.y, 1.75),
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
    ));
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let parts = vec![
        VehiclePart::new((0, 0), 0.0, BlockMold::cube().in_medium(BlockMedium::CabinSeat).atom_for(Vec3::NEG_Z, &frame)),
        VehiclePart::new((0, 1), 0.0, BlockMold::cube().in_medium(BlockMedium::FlightConsole).atom_for(Vec3::NEG_Z, &frame)),
        VehiclePart::new((0, -1), 0.0, BlockMold::cube().in_medium(BlockMedium::FusionTorch).atom_for(Vec3::NEG_Z, &frame)),
    ];
    let bp = CraftBlueprint::commission(Vec3::new(1.25, stand.y + 2.0, 1.25), parts);
    assert!(host.build_rig_blueprint(crane_eye, crane_dir, &bp));

    let mut alice = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("alice");
    let mut day_peer = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("day");
    let mut night_bare = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("night_bare");
    let mut night_scope = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("night_scope");
    let mut orbit_temp = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("orbit_temp");

    let seat_eye = Vec3::new(1.25, stand.y + 2.2, 0.25);
    alice.sync_presence(make_pres(seat_eye));
    for _ in 0..8 {
        let _ = alice.step_surface(0.03);
        let _ = host.step_surface(0.03);
    }
    let console_dir = (Vec3::new(1.25, stand.y + 1.4, 0.75) - seat_eye).normalize();
    let boarded = alice
        .board_vehicle(seat_eye, console_dir, true)
        .or_else(|| alice.board_vehicle(Vec3::new(1.25, stand.y + 2.6, 0.75), Vec3::NEG_Y, true))
        .expect("Alice claims helm");
    let ship_id = boarded.identity();
    for _ in 0..8 {
        let _ = alice.step_surface(0.03);
        let _ = host.step_surface(0.03);
    }

    // Move Alice into DeepSpace ~90,000 m directly above `night_zone` firing her FusionTorch at 100% burn!
    let ship_dir = night_zone.normal();
    let deep_pos = ship_dir * 90_000.0;
    let _ = alice.reconcile_cosmic_frame(deep_pos, ship_dir * 8_000.0);
    assert!(matches!(alice.cosmic_frame(), CosmicFrame::DeepSpace(_)));
    let burning_pose = CraftPose::new(
        Vec3::ZERO,
        VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y),
        Vec3::NEG_Z,
        (1.0, 0.0),
    );
    alice.sync_pilot_craft(PilotCraft::Driving(DrivenCraft::new(0, ship_id, burning_pose)));
    alice.sync_berth_presence(make_pres(Vec3::ZERO), world::VesselBerth::aboard(ship_id));

    let day_pos = host.constrain_to_surface(day_zone.normal() * (earth_half + 4.0), 1.0);
    let night_pos = host.constrain_to_surface(night_zone.normal() * (earth_half + 4.0), 1.0);
    let orbit_pos = Vec3::new(0.0, earth_half + 1_800.0, 0.0);

    day_peer.sync_sky_watch(true);
    day_peer.sync_presence(make_pres(day_pos));
    night_bare.sync_sky_watch(false);
    night_bare.sync_presence(make_pres(night_pos));
    night_scope.sync_sky_watch(true);
    night_scope.sync_presence(make_pres(night_pos));
    orbit_temp.sync_sky_watch(false);
    orbit_temp.sync_presence(make_pres(orbit_pos));

    for _ in 0..12 {
        let _ = alice.step_surface(0.03);
        let _ = day_peer.step_surface(0.03);
        let _ = night_bare.step_surface(0.03);
        let _ = night_scope.step_surface(0.03);
        let _ = orbit_temp.step_surface(0.03);
        let _ = host.step_surface(0.03);
    }

    let mut day_probe = SignalMeshProbe { cyan_spikes: 0, emerald_beacons: 0 };
    day_peer.emit_cosmic_field(SightCone::omnidirectional(day_pos), &mut day_probe);
    assert_eq!(day_probe.cyan_spikes, 0, "Daytime ground observer must receive 0 FusionDriveStar signals even with 24x zoom");
    assert_eq!(day_probe.emerald_beacons, 0, "Ground observer outside orbital LoS must receive 0 OpticalBeacon signals");

    let mut bare_probe = SignalMeshProbe { cyan_spikes: 0, emerald_beacons: 0 };
    night_bare.emit_cosmic_field(SightCone::omnidirectional(night_pos), &mut bare_probe);
    assert_eq!(bare_probe.cyan_spikes, 0, "Unzoomed night ground observer must receive 0 FusionDriveStar signals");

    let mut scope_probe = SignalMeshProbe { cyan_spikes: 0, emerald_beacons: 0 };
    night_scope.emit_cosmic_field(SightCone::from_view(night_pos, ship_dir), &mut scope_probe);
    assert!(scope_probe.cyan_spikes >= 2, "Nighttime 6x/24x telescope observer must see Alice's distant FusionDriveStar!");

    let mut orbit_probe = SignalMeshProbe { cyan_spikes: 0, emerald_beacons: 0 };
    orbit_temp.emit_cosmic_field(SightCone::omnidirectional(orbit_pos), &mut orbit_probe);
    assert!(orbit_probe.emerald_beacons >= 2, "Orbital viewer above Temperate must see the surface LaserBeacon!");
    assert!(orbit_probe.cyan_spikes >= 2, "Orbital viewer above atmosphere (>180m) must also see distant FusionDriveStar in vacuum!");
}
