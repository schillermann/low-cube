use glam::Vec3;
use crate::{ClimateZone, MeshTarget, Quad, SightCone};
use super::{
    CloudStratum, CloudVolume, ConvergenceCell, MoistureField, PlanetaryWeather, StratumBand,
    TroposphereDeck, TroposphereField, WeatherSample,
};

struct ClassifiedQuadSink {
    cloud_quads: usize,
    rain_quads: usize,
    optics: (f32, f32),
}

impl ClassifiedQuadSink {
    const fn empty() -> Self {
        Self {
            cloud_quads: 0,
            rain_quads: 0,
            optics: (1.0, 0.0),
        }
    }
}

impl MeshTarget for ClassifiedQuadSink {
    fn push_quad(&mut self, quad: Quad) {
        if quad.radiate(|g| g < -7.5) {
            return;
        }
        if quad.radiate(|g| g < -5.5) {
            self.rain_quads += 1;
        } else if quad.radiate(|g| g < -3.5) {
            self.cloud_quads += 1;
            quad.radiate(|g| {
                self.optics.1 = self.optics.1.max(-g - 4.0);
                true
            });
            quad.transfer(|_, _, _, _, n, col| {
                assert!(
                    n.length() > 1.02 && n.length() < 2.02,
                    "Cloud puff normal must encode constant organic lobe seed without grid seams"
                );
                let luma = (col.x + col.y + col.z) / 3.0;
                self.optics.0 = self.optics.0.min(luma);
            });
        }
    }
}

#[test]
fn test_three_cloud_strata_float_high_above_trees_with_clear_fly_through_corridors() {
    let half = 64.0;
    let (h0, t0) = CloudStratum::new(0, half).elevation_band();
    let (h1, t1) = CloudStratum::new(1, half).elevation_band();
    let (h2, _t2) = CloudStratum::new(2, half).elevation_band();

    assert!(h0 >= 32.0, "Lowest Cumulus base ({h0}m) must float high above trees to avoid oppressive skies");
    assert!(h0 + t0 < h1 - 4.0, "Clear fly-through air corridor must separate Layer 0 and Layer 1");
    assert!(h1 + t1 < h2 - 4.0, "Clear fly-through air corridor must separate Layer 1 and Layer 2");

    let sample = ConvergenceCell::new(0.65, 0.25, 0.0, false);
    let low_vol = CloudVolume::from_cell(0.0, 0.0, 24.0, CloudStratum::new(0, half), sample);
    let mid_vol = CloudVolume::from_cell(0.0, 0.0, 24.0, CloudStratum::new(1, half), sample);

    assert_eq!(low_vol.immersion_at(0.0, 0.0, 12.0).0, 0.0);
    assert!(low_vol.immersion_at(0.0, 0.0, h0 + 2.0).0 > 0.30);
    assert_eq!(low_vol.immersion_at(0.0, 0.0, (h0 + t0 + h1) * 0.5).0, 0.0);
    assert!(mid_vol.immersion_at(0.0, 0.0, h1 + 1.2).0 > 0.30);
}

#[test]
fn test_sky_stays_mostly_open_while_accumulated_showers_darken_before_raining() {
    let half = 64.0;
    let mut total_samples = 0usize;
    let mut clear_samples = 0usize;
    let mut light_darkness_max = 0.0_f32;
    let mut raining_darkness_min = 1.0_f32;
    let mut found_fair = false;
    let mut found_storm = false;

    for step_t in 0..32 {
        let field = MoistureField::new(ClimateZone::Temperate, half, (step_t as f32) * 24.0);
        for iu in -4..=4 {
            for iv in -4..=4 {
                total_samples += 1;
                let cell = field.sample_layer(0, (iu as f32) * 14.0, (iv as f32) * 14.0);
                if !cell.present() {
                    clear_samples += 1;
                }
                cell.transfer(|mass, dark, rain, frozen| {
                    assert!(!frozen);
                    if mass > 0.05 && mass < 0.22 {
                        found_fair = true;
                        light_darkness_max = light_darkness_max.max(dark);
                        assert_eq!(rain, 0.0, "Sparse fair-weather clouds must never rain");
                    }
                    if rain > 0.15 {
                        found_storm = true;
                        raining_darkness_min = raining_darkness_min.min(dark);
                    }
                });
            }
        }
    }

    let clear_ratio = (clear_samples as f32) / (total_samples as f32);
    assert!(
        clear_ratio > 0.65,
        "Temperate sky must stay predominantly open and sunny (>65% clear), got {:.1}%",
        clear_ratio * 100.0
    );
    assert!(found_fair && found_storm, "Temperate sky must host both fair cumulus and rare showers");
    assert!(
        raining_darkness_min > light_darkness_max + 0.25 && raining_darkness_min > 0.38,
        "Accumulated rain clouds must darken visibly vs fair cumulus: storm_min={raining_darkness_min}, fair_max={light_darkness_max}"
    );
}

#[test]
fn test_biome_hydrology_contrasts_jungle_showers_arctic_snow_and_arid_desert() {
    let half = 64.0;
    let mut jungle_rain = 0usize;
    let mut arctic_snow = 0usize;
    let mut desert_rain = 0usize;

    for step_t in 0..32 {
        let t = (step_t as f32) * 22.0;
        let j_field = MoistureField::new(ClimateZone::Jungle, half, t);
        let a_field = MoistureField::new(ClimateZone::Arctic, half, t);
        let d_field = MoistureField::new(ClimateZone::Desert, half, t);
        for iu in -4..=4 {
            for iv in -4..=4 {
                let (u, v) = ((iu as f32) * 13.0, (iv as f32) * 13.0);
                let jc = j_field.sample_layer(0, u, v);
                if jc.raining() && !jc.frozen() {
                    jungle_rain += 1;
                }
                let ac = a_field.sample_layer(0, u, v);
                if ac.raining() && ac.frozen() {
                    arctic_snow += 1;
                }
                if d_field.sample_layer(0, u, v).raining() {
                    desert_rain += 1;
                }
            }
        }
    }

    assert!(jungle_rain > 0, "Jungle must accumulate localized liquid-rain showers");
    assert!(arctic_snow > 0, "Arctic must accumulate frozen snow showers");
    assert_eq!(desert_rain, 0, "Arid Desert heat must evaporate low moisture before rain saturation");
}

#[test]
fn test_troposphere_deck_omits_hanging_curtains_and_bounds_neighbor_faces() {
    let mut dry_viewer_sink = ClassifiedQuadSink::empty();
    for step_t in 0..24 {
        let t = (step_t as f32) * 25.0;
        let deck = TroposphereDeck::new(64.0, t);
        let eye = Vec3::new(0.0, 66.0, 0.0);
        if deck.precipitation_at(eye).0 <= 0.04 {
            deck.emit_around(SightCone::omnidirectional(eye), &mut dry_viewer_sink);
        }
    }
    assert!(dry_viewer_sink.cloud_quads > 8, "High clouds must emit soft multi-puff billows");
    assert_eq!(
        dry_viewer_sink.rain_quads, 0,
        "Distant storms must never hang rectangular Virga curtains down from the sky when dry at viewer"
    );

    let mut shower_sink = ClassifiedQuadSink::empty();
    let mut peak_overcast = 0.0_f32;
    let mut above_storm_overcast = 1.0_f32;
    for step_t in 0..64 {
        let t = (step_t as f32) * 28.0;
        let deck = TroposphereDeck::new(64.0, t);
        for iu in -2..=2 {
            let probe = Vec3::new((iu as f32) * 18.0, 4.0, 66.0);
            if deck.precipitation_at(probe).0 > 0.16 {
                peak_overcast = peak_overcast.max(deck.overcast_at(probe));
                above_storm_overcast =
                    above_storm_overcast.min(deck.overcast_at(Vec3::new(probe.x, probe.y, 118.0)));
                deck.emit_around(SightCone::omnidirectional(probe), &mut shower_sink);
            }
        }
    }
    assert!(shower_sink.rain_quads > 0, "Active overhead shower must emit fine falling raindrops");
    assert!(
        peak_overcast > 0.55 && above_storm_overcast == 0.0,
        "Rainstorms must strongly darken sky overcast ({peak_overcast}) below deck while clearing above clouds ({above_storm_overcast})"
    );
    assert!(
        shower_sink.optics.0 < 0.32 && shower_sink.optics.1 > 0.78,
        "Raining storm canopy must darken (luma={}) and thicken (alpha={}) to occlude sunny blue sky",
        shower_sink.optics.0,
        shower_sink.optics.1
    );

    let mut continental_sink = ClassifiedQuadSink::empty();
    TroposphereDeck::new(2_048.0, 42.0).emit_around(
        SightCone::omnidirectional(Vec3::new(0.0, 2_050.0, 0.0)),
        &mut continental_sink,
    );
    assert!(
        continental_sink.cloud_quads < 400,
        "4096^3 continental world must bound cloud puffs to O(1) local window, got {}",
        continental_sink.cloud_quads
    );
}

#[test]
fn test_walking_toward_cloud_rotates_billboard_smoothly_without_axis_flip() {
    struct FirstCornerSink(Option<Vec3>);
    impl MeshTarget for FirstCornerSink {
        fn push_quad(&mut self, quad: Quad) {
            if self.0.is_none() {
                quad.transfer(|v0, _, _, _, _, _| self.0 = Some(v0));
            }
        }
    }

    let frame = ClimateZone::Temperate.surface_frame(64.0);
    let vol = CloudVolume::from_cell(
        0.0,
        0.0,
        24.0,
        CloudStratum::new(0, 64.0),
        ConvergenceCell::new(0.55, 0.10, 0.0, false),
    );
    let mut prev_v0: Option<Vec3> = None;
    for step in 0..=40 {
        let walk_x = -85.0 + (step as f32) * 2.0;
        let viewer = frame.point_at(walk_x, 0.0, 1.7);
        let mut sink = FirstCornerSink(None);
        vol.emit(viewer, 1.0, &frame, &mut sink);
        let cur_v0 = sink.0.expect("cloud puff must emit corner");
        if let Some(prev) = prev_v0 {
            let jump = (cur_v0 - prev).length();
            assert!(
                jump < 0.85,
                "Cloud puff billboard snapped discontinuously ({jump:.2}m) while walking at x={walk_x}"
            );
        }
        prev_v0 = Some(cur_v0);
    }
}
