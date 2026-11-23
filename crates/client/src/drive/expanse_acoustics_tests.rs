use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BlockMedium, BlockMold, CelestialSystem, SurfaceFrame, VehicleAssembly, VehicleAxes,
    VehiclePart, WedgeHeading,
};
use acoustics::{
    AcousticWave as _, AttitudeHiss, FusionHum, HullChorus, HullWaveTrace, NozzleRoar,
    PropulsionCue,
};
use crate::controls::{KeyMotion, KeyboardControls as _};
use super::rocket::RocketDrive;
use super::CraftDrive as _;

fn commission_expanse_ship(fusion_drive: bool) -> VehicleAssembly {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let rcs_pod = |heading: WedgeHeading| {
        BlockMold::shape_from_stamp([1, heading.code(), BlockMedium::RocketNozzle.code()])
    };
    let main_med = if fusion_drive {
        BlockMedium::FusionTorch
    } else {
        BlockMedium::RocketNozzle
    };
    let parts = vec![
        VehiclePart::new(
            (0, 0),
            0.5,
            BlockMold::cube().in_medium(BlockMedium::CabinSeat).shape_for(Vec3::NEG_Z, &frame),
        ),
        VehiclePart::new(
            (0, -1),
            0.5,
            BlockMold::cube().in_medium(BlockMedium::FlightConsole).shape_for(Vec3::Z, &frame),
        ),
        VehiclePart::new(
            (0, 0),
            0.0,
            BlockMold::cube().in_medium(main_med).shape_for(Vec3::NEG_Z, &frame),
        ),
        VehiclePart::new((-1, 0), 0.5, rcs_pod(WedgeHeading::East)),
        VehiclePart::new((1, 0), 0.5, rcs_pod(WedgeHeading::West)),
        VehiclePart::new((0, -2), 0.5, rcs_pod(WedgeHeading::North)),
        VehiclePart::new((0, 1), 0.5, rcs_pod(WedgeHeading::South)),
    ];
    VehicleAssembly::new(
        Vec3::new(0.0, 64.0, 0.0),
        VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y),
        Vec3::NEG_Z,
        parts,
    )
}

#[test]
fn test_expanse_epstein_torch_bell_roar_and_rcs_teakettle_waveforms() {
    let port_rcs = AttitudeHiss::valve_burst(0.90, -0.55, 0);
    let starboard_rcs = AttitudeHiss::valve_burst(0.90, 0.55, 1);
    let (pl, pr) = port_rcs.channel_gains();
    let (sl, sr) = starboard_rcs.channel_gains();
    assert!(pl > pr * 1.35, "Port RCS pod must bias stereo energy toward left bulkhead");
    assert!(sr > sl * 1.35, "Starboard RCS pod must bias stereo energy toward right bulkhead");

    let open_energy = (0..12)
        .map(|k| port_rcs.amplitude_at(0.001 + (k as f32) * 0.001).abs())
        .fold(0.0_f32, f32::max);
    let steam_energy = (0..20)
        .map(|k| port_rcs.amplitude_at(0.05 + (k as f32) * 0.003).abs())
        .fold(0.0_f32, f32::max);
    let shutoff_energy = (0..12)
        .map(|k| port_rcs.amplitude_at(0.176 + (k as f32) * 0.0015).abs())
        .fold(0.0_f32, f32::max);
    assert!(open_energy > 0.12, "Solenoid valve opening must produce crisp mechanical snap");
    assert!(steam_energy > 0.06, "Superheated steam vent must sustain pressurized hiss");
    assert!(shutoff_energy > 0.05, "Valve shutoff must produce audible arrest clack");

    let rate = port_rcs.sample_rate() as f32;
    let total = (0.24 * rate) as usize;
    let mut prev = port_rcs.amplitude_at(0.0);
    let mut zc = 0usize;
    for i in 1..total {
        let cur = port_rcs.amplitude_at((i as f32) / rate);
        if (prev >= 0.0) != (cur >= 0.0) {
            zc += 1;
        }
        prev = cur;
    }
    let zc_hz = (zc as f32) / (2.0 * 0.24);
    assert!(
        zc_hz < 500.0,
        "Hull-muffled RCS burst must stay low-mid weighted (<500 Hz zero-crossing), got {zc_hz} Hz"
    );
    assert_eq!(port_rcs.stereo_riff_wave()[..4], *b"RIFF");

    let epstein = FusionHum::torch();
    let bell = NozzleRoar::ignition();
    let chorus = HullChorus::cockpit(FusionHum::sustained(), port_rcs, -0.45);
    let ep_kick = (0..12)
        .map(|k| epstein.amplitude_at(0.003 + (k as f32) * 0.0012).abs())
        .fold(0.0_f32, f32::max);
    let bell_kick = (0..12)
        .map(|k| bell.amplitude_at(0.003 + (k as f32) * 0.0012).abs())
        .fold(0.0_f32, f32::max);
    assert!(ep_kick > 0.10, "Embedded NASA Epstein ignition must kick in without lag");
    assert!(bell_kick > 0.10, "Embedded NASA main-bell ignition must kick in without lag");
    assert!(chorus.amplitude_at(0.08).abs() <= 1.0, "HullChorus superposition must never clip");
}

#[test]
fn test_rcs_maneuvers_emit_teakettle_valve_hiss_instead_of_main_engine_roar() {
    let world = CelestialSystem::barren_authoritative(32, 1.0);
    let mut ship = commission_expanse_ship(true);
    let mut drive = RocketDrive::boarded(&ship, world.as_ref());

    let mut roll_input = KeyMotion::idle();
    roll_input.register_key(KeyCode::KeyQ, true);
    let cues = HullWaveTrace::new(1).capture(|| {
        drive.advance(&mut ship, 1.0 / 120.0, &roll_input, world.as_ref());
    });
    assert_eq!(
        cues,
        vec![PropulsionCue::AttitudeValve],
        "Steering with RCS on an Epstein ship must fire crisp teakettle AttitudeValve on tick 1, never FusionHum"
    );
}

#[test]
fn test_main_burn_ignites_immediately_and_layers_hull_chorus_when_steering() {
    let world = CelestialSystem::barren_authoritative(32, 1.0);
    let mut fusion_ship = commission_expanse_ship(true);
    let mut fusion_drive = RocketDrive::boarded(&fusion_ship, world.as_ref());

    let mut burn = KeyMotion::idle();
    burn.register_key(KeyCode::Space, true);
    let ignite_cues = HullWaveTrace::new(1).capture(|| {
        fusion_drive.advance(&mut fusion_ship, 1.0 / 120.0, &burn, world.as_ref());
    });
    assert_eq!(ignite_cues, vec![PropulsionCue::FusionIgnition]);

    let sustain_cues = HullWaveTrace::new(1).capture(|| {
        for _ in 0..48 {
            fusion_drive.advance(&mut fusion_ship, 1.0 / 120.0, &burn, world.as_ref());
        }
    });
    assert!(sustain_cues.contains(&PropulsionCue::FusionSustained));

    let mut burn_and_pitch = KeyMotion::idle();
    burn_and_pitch.register_key(KeyCode::Space, true);
    burn_and_pitch.register_key(KeyCode::KeyW, true);
    let chorus_cues = HullWaveTrace::new(1).capture(|| {
        fusion_drive.advance(&mut fusion_ship, 1.0 / 120.0, &burn_and_pitch, world.as_ref());
    });
    assert_eq!(
        chorus_cues,
        vec![PropulsionCue::HullCombined],
        "Pitching with RCS mid-burn must superpose Epstein roar and RCS burst via HullChorus"
    );

    let mut chem_ship = commission_expanse_ship(false);
    let mut chem_drive = RocketDrive::boarded(&chem_ship, world.as_ref());
    let bell_cues = HullWaveTrace::new(1).capture(|| {
        chem_drive.advance(&mut chem_ship, 1.0 / 120.0, &burn, world.as_ref());
    });
    assert_eq!(bell_cues, vec![PropulsionCue::BellIgnition]);
}

#[test]
fn test_retro_flip_emits_rcs_slew_and_retrograde_arrest_valve_burst() {
    let world = CelestialSystem::barren_authoritative(32, 1.0);
    let mut ship = commission_expanse_ship(true);
    let mut drive = RocketDrive::boarded(&ship, world.as_ref());

    let mut burn = KeyMotion::idle();
    burn.register_key(KeyCode::Space, true);
    for _ in 0..24 {
        drive.advance(&mut ship, 1.0 / 60.0, &burn, world.as_ref());
    }

    let idle = KeyMotion::idle();
    for _ in 0..20 {
        drive.advance(&mut ship, 1.0 / 60.0, &idle, world.as_ref());
    }
    assert!(drive.toggle_retro_flip(&ship));

    let flip_cues = HullWaveTrace::new(2).capture(|| {
        for _ in 0..90 {
            drive.advance(&mut ship, 1.0 / 60.0, &idle, world.as_ref());
        }
    });
    assert_eq!(flip_cues.first(), Some(&PropulsionCue::AttitudeValve));
    assert_eq!(flip_cues.last(), Some(&PropulsionCue::AttitudeValve));
}
