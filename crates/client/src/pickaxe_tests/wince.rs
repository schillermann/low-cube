use acoustics::{AcousticWave, DigWaveTrace, KnuckleWince};
use glam::Vec3;
use world::{
    BiomedQuadSphere, BlockMedium, BlockMold, PlanetaryWorld, WorldEarthworks as _,
};
use crate::attire::{EvaAttire, HeldRelic};
use crate::avatar::{Avatar as _, AvatarMotor as _, PlanetaryAvatar};
use crate::controls::KeyMotion;

#[test]
fn test_bare_hands_on_hard_stone_emit_aua_wince_while_earth_sand_tools_and_sky_do_not() {
    let vocal = KnuckleWince::vocal();
    let slap_peak = (1..28)
        .map(|i| vocal.amplitude_at((i as f32) * 0.001).abs())
        .fold(0.0_f32, f32::max);
    let au_peak = (80..180)
        .map(|i| vocal.amplitude_at((i as f32) * 0.001).abs())
        .fold(0.0_f32, f32::max);
    let dip_rms = (230..242)
        .map(|i| vocal.amplitude_at((i as f32) * 0.001).powi(2))
        .sum::<f32>()
        / 12.0;
    let ah_rms = (295..325)
        .map(|i| vocal.amplitude_at((i as f32) * 0.001).powi(2))
        .sum::<f32>()
        / 30.0;
    assert!(slap_peak > 0.12 && au_peak > 0.25 && ah_rms > dip_rms * 1.4);
    assert_eq!(vocal.amplitude_at(0.48), 0.0);
    let wince_wavs: Vec<Vec<u8>> = (0..4).map(|c| KnuckleWince::inflected(c).riff_wave()).collect();
    assert!(wince_wavs.windows(2).all(|w| w[0] != w[1]));

    let mut world = BiomedQuadSphere::barren(16);
    let stand = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let ground_y = world.constrain_to_surface(Vec3::new(0.25, 16.0, -0.75), 0.0).y;
    let aim_eye = Vec3::new(0.25, stand.y + 0.8, 0.25);
    let aim_dir = (Vec3::new(0.25, ground_y + 0.1, -0.75) - aim_eye).normalize();

    assert!(world.place_molded_surface_block(
        aim_eye,
        aim_dir,
        stand,
        BlockMold::cube().in_medium(BlockMedium::QuarriedStone),
    ));
    let mut miner = PlanetaryAvatar::at(stand);
    miner.rotate_degrees(0.0, 70.5);
    miner.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::SuitRadio, HeldRelic::EmptyPalm));

    let tool_trace = DigWaveTrace::new(0).record_variants(|| {
        assert!(!miner.use_right_hand(&mut world));
        let _ = miner.release_hand(&mut world);
    });
    assert!(tool_trace.is_empty(), "Transceiver rebound against stone must not cry Aua");

    let bare_trace = DigWaveTrace::new(2).record_variants(|| {
        assert!(!miner.use_left_hand(&mut world));
        for _ in 0..45 {
            miner.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
            assert!(!miner.step_digging(&mut world));
        }
        let _ = miner.release_hand(&mut world);
    });
    assert!(bare_trace.iter().all(|&tag| (200..204).contains(&tag)) && bare_trace[0] != bare_trace[1]);

    miner.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::EmptyPalm));
    let promoted_trace = DigWaveTrace::new(1).record_variants(|| {
        assert!(!miner.use_right_hand(&mut world));
        for _ in 0..35 {
            miner.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
            assert!(!miner.step_digging(&mut world));
        }
        let _ = miner.release_hand(&mut world);
    });
    assert!(promoted_trace.iter().all(|&tag| (200..204).contains(&tag)));

    let mut sky_gazer = PlanetaryAvatar::at(stand);
    sky_gazer.rotate_degrees(0.0, -65.0);
    let sky_trace = DigWaveTrace::new(0).record_variants(|| {
        assert!(!sky_gazer.use_left_hand(&mut world));
    });
    assert!(sky_trace.is_empty(), "Swinging bare hands at empty sky must stay silent");

    let _ = world.harvest_surface(aim_eye, aim_dir);
    assert!(world.place_molded_surface_block(
        aim_eye,
        aim_dir,
        stand,
        BlockMold::cube().in_medium(BlockMedium::Sand),
    ));
    let sand_trace = DigWaveTrace::new(1).record_variants(|| {
        let _ = miner.use_left_hand(&mut world);
        let _ = miner.release_hand(&mut world);
    });
    assert!(sand_trace.iter().all(|&tag| tag < 8), "Bare hands on sand must emit earthy dig sound, never Aua");
}
