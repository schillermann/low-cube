use glam::Vec3;
use crate::{ClimateZone, MeshTarget, Quad, SightCone};
use super::{
    CloudStratum, CloudVolume, ConvergenceCell, MoistureField, PlanetaryWeather, StratumBand,
    TroposphereDeck, TroposphereField, WeatherSample,
};

struct ClassifiedQuadSink {
    cloud_quads: usize,
    rain_quads: usize,
    darkest_luma: f32,
    brightest_luma: f32,
}

impl ClassifiedQuadSink {
    const fn empty() -> Self {
        Self {
            cloud_quads: 0,
            rain_quads: 0,
            darkest_luma: 1.0,
            brightest_luma: 0.0,
        }
    }
}

impl MeshTarget for ClassifiedQuadSink {
    fn push_quad(&mut self, quad: Quad) {
        if quad.radiate(|g| g < -5.5) {
            self.rain_quads += 1;
        } else if quad.radiate(|g| g < -3.5) {
            self.cloud_quads += 1;
            quad.transfer(|_, _, _, _, _, col| {
                let luma = (col.x + col.y + col.z) / 3.0;
                self.darkest_luma = self.darkest_luma.min(luma);
                self.brightest_luma = self.brightest_luma.max(luma);
            });
        }
    }
}

#[test]
fn test_three_cloud_strata_occupy_separated_fly_through_altitudes_below_aurora() {
    let half = 64.0;
    let (h0, t0) = CloudStratum::new(0, half).elevation_band();
    let (h1, t1) = CloudStratum::new(1, half).elevation_band();
    let (h2, t2) = CloudStratum::new(2, half).elevation_band();

    assert!(h0 + t0 < h1 - 2.0, "Clear fly-through air corridor must separate Layer 0 and Layer 1");
    assert!(h1 + t1 < h2 - 2.0, "Clear fly-through air corridor must separate Layer 1 and Layer 2");
    assert!(h2 + t2 < half * 0.44 + 0.5, "Highest Cirrus deck must sit below the 3D Ionosphere Aurora base");

    let sample = ConvergenceCell::new(0.65, 0.30, 0.0, false);
    let low_vol = CloudVolume::from_cell(0.0, 0.0, 16.0, CloudStratum::new(0, half), sample);
    let mid_vol = CloudVolume::from_cell(0.0, 0.0, 16.0, CloudStratum::new(1, half), sample);

    assert_eq!(low_vol.immersion_at(0.0, 0.0, 2.0).0, 0.0);
    assert!(low_vol.immersion_at(0.0, 0.0, h0 + 1.5).0 > 0.35);
    assert_eq!(low_vol.immersion_at(0.0, 0.0, (h0 + t0 + h1) * 0.5).0, 0.0);
    assert!(mid_vol.immersion_at(0.0, 0.0, h1 + 1.0).0 > 0.35);
}

#[test]
fn test_cloud_accumulation_darkens_storm_bellies_before_triggering_rain_and_draining() {
    let half = 64.0;
    let mut light_darkness_max = 0.0_f32;
    let mut raining_darkness_min = 1.0_f32;
    let mut found_fair = false;
    let mut found_storm = false;

    for step_t in 0..24 {
        let field = MoistureField::new(ClimateZone::Temperate, half, (step_t as f32) * 18.0);
        for iu in -4..=4 {
            for iv in -4..=4 {
                let cell = field.sample_layer(0, (iu as f32) * 11.0, (iv as f32) * 11.0);
                cell.transfer(|mass, dark, rain, frozen| {
                    assert!(!frozen);
                    if mass > 0.05 && mass < 0.25 {
                        found_fair = true;
                        light_darkness_max = light_darkness_max.max(dark);
                        assert_eq!(rain, 0.0, "Sparse fair-weather clouds must never rain");
                    }
                    if rain > 0.20 {
                        found_storm = true;
                        raining_darkness_min = raining_darkness_min.min(dark);
                    }
                });
            }
        }
    }

    assert!(found_fair && found_storm, "Temperate sky must host both fair cumulus and converged rainstorms");
    assert!(
        raining_darkness_min > light_darkness_max + 0.25 && raining_darkness_min > 0.42,
        "Accumulated rain clouds must be visibly darker than fair cumulus: storm_min={raining_darkness_min}, fair_max={light_darkness_max}"
    );
}

#[test]
fn test_biome_hydrology_contrasts_jungle_downpours_arctic_snow_and_arid_desert() {
    let half = 64.0;
    let mut jungle_rain = 0usize;
    let mut arctic_snow = 0usize;
    let mut desert_rain = 0usize;

    for step_t in 0..20 {
        let t = (step_t as f32) * 15.0;
        let j_field = MoistureField::new(ClimateZone::Jungle, half, t);
        let a_field = MoistureField::new(ClimateZone::Arctic, half, t);
        let d_field = MoistureField::new(ClimateZone::Desert, half, t);
        for iu in -3..=3 {
            for iv in -3..=3 {
                let (u, v) = ((iu as f32) * 12.0, (iv as f32) * 12.0);
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

    assert!(jungle_rain > 0, "Jungle must accumulate dark liquid-rain downpours");
    assert!(arctic_snow > 0, "Arctic must accumulate frozen snow-squall clouds");
    assert_eq!(desert_rain, 0, "Arid Desert heat must evaporate low moisture before rain saturation");
}

#[test]
fn test_troposphere_deck_emits_clouds_rain_virga_cockpit_veil_and_bounds_continental_worlds() {
    let mut sink = ClassifiedQuadSink::empty();
    for step_t in 0..16 {
        let deck = TroposphereDeck::new(64.0, (step_t as f32) * 20.0);
        deck.emit_around(SightCone::omnidirectional(Vec3::new(0.0, 66.0, 0.0)), &mut sink);
    }
    assert!(sink.cloud_quads > 40, "Multi-layer clouds must emit 3D translucent cluster quads");
    assert!(sink.rain_quads > 0, "Accumulated storm fronts must emit Virga rain shafts / streaks");
    assert!(
        sink.brightest_luma > 0.92 && sink.darkest_luma < 0.48,
        "Cloud palette must span bright white crowns ({}) to dark storm bellies ({})",
        sink.brightest_luma,
        sink.darkest_luma
    );

    let mut continental_sink = ClassifiedQuadSink::empty();
    let giant_deck = TroposphereDeck::new(2_048.0, 42.0);
    giant_deck.emit_around(
        SightCone::omnidirectional(Vec3::new(0.0, 2_050.0, 0.0)),
        &mut continental_sink,
    );
    assert!(
        continental_sink.cloud_quads > 0 && continental_sink.cloud_quads < 1_800,
        "4096^3 continental world must bound cloud emission to O(1) local window, got {}",
        continental_sink.cloud_quads
    );
}
