use super::super::deck::SoundDeck;
use super::super::{AcousticWave, MonoSource};
use super::{
    CanopyWash, MuffledCanopy, PatterDrops, PrecipitationCue, RainWaveTrace, ShowerDemand,
    WindSwell, LOOP_PERIOD_SEC,
};

#[test]
fn test_rain_decorator_chain_wraps_periodically_without_seam_click_and_muffles_snow() {
    let wash = CanopyWash::velvet(0x517C_C1B7);
    let rain = MuffledCanopy::woodland_rain(WindSwell::periodic(PatterDrops::foliage(wash)));
    let snow = MuffledCanopy::polar_snow(WindSwell::periodic(wash));

    let seam_err = (rain.amplitude_at(0.0) - rain.amplitude_at(LOOP_PERIOD_SEC)).abs();
    assert!(
        seam_err < 1e-5,
        "3.0s rain decorator chain must wrap seamlessly across loop boundary, got diff={seam_err}"
    );

    let (mut rain_hf, mut snow_hf) = (0.0_f32, 0.0_f32);
    let dt = 1.0 / 24_000.0;
    for step in 1..4_800 {
        let t = (step as f32) * dt;
        rain_hf += (rain.dry_sample(t) - rain.dry_sample(t - dt)).abs();
        snow_hf += (snow.dry_sample(t) - snow.dry_sample(t - dt)).abs();
    }
    assert!(
        rain_hf > snow_hf * 1.45,
        "Woodland rain patter ({rain_hf}) must carry crisper droplet transients than muffled polar snow ({snow_hf})"
    );
}

#[test]
fn test_sound_deck_streams_woodland_cabin_and_polar_showers_and_fades_to_silence() {
    let cues = RainWaveTrace::new(3).capture(|| {
        ShowerDemand::calm().publish();
        ShowerDemand::new(0.65, false, false).publish();
        ShowerDemand::new(0.65, false, true).publish();
        ShowerDemand::new(0.55, true, false).publish();
    });
    assert_eq!(
        cues,
        vec![
            PrecipitationCue::WoodlandRain,
            PrecipitationCue::CabinRain,
            PrecipitationCue::PolarSnowfall,
        ]
    );

    let mut deck = SoundDeck::studio();
    let mut open_chunk = [0i16; 960];
    let mut cabin_chunk = [0i16; 960];
    for _ in 0..8 {
        deck.update_shower(ShowerDemand::new(0.85, false, false));
        deck.mix_stereo_into(&mut open_chunk);
    }
    let open_energy: f64 = open_chunk.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
    assert!(open_energy > 1.0e6, "Active rain shower must produce rich stereo audio energy");

    let mut cabin_deck = SoundDeck::studio();
    for _ in 0..8 {
        cabin_deck.update_shower(ShowerDemand::new(0.85, false, true));
        cabin_deck.mix_stereo_into(&mut cabin_chunk);
    }
    let cabin_energy: f64 = cabin_chunk.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
    assert!(
        cabin_energy < open_energy * 0.60,
        "Cabin-sheltered rain ({cabin_energy}) must be noticeably softer than open woodland rain ({open_energy})"
    );

    for _ in 0..90 {
        deck.mix_stereo_into(&mut open_chunk);
    }
    let tail_peak = open_chunk.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
    assert_eq!(tail_peak, 0, "Expired ShowerDemand lease must slew rain loop smoothly to zero");
}
