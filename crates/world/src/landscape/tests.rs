use super::*;
use crate::topology::ClimateZone;
use crate::{Face, MeshTarget, PlanarFace, Quad};
use glam::Vec3;

struct QuadCounter {
    count: usize,
}

impl MeshTarget for QuadCounter {
    fn push_quad(&mut self, _quad: Quad) {
        self.count += 1;
    }
}

#[test]
fn test_volcanic_caldera_elevations_and_rim() {
    let half = 8.0;
    let base = Box::new(PlanarFace::in_zone(ClimateZone::Volcanic, Vec3::ZERO, half, 16));
    let caldera = VolcanicCaldera::in_zone(base, ClimateZone::Volcanic, half);
    let frame = ClimateZone::Volcanic.surface_frame(half);

    let test_pt = frame.point_at(1.75, 0.75, 0.0);
    let constrained = caldera.constrain(test_pt, 1.0);
    assert!(frame.height_of(constrained) > half + 1.0);

    let mut counter = QuadCounter { count: 0 };
    caldera.emit_surface(&mut counter);
    assert!(counter.count > 256);
}

#[test]
fn test_desert_dunes_slopes_and_valley() {
    let half = 8.0;
    let base = Box::new(PlanarFace::in_zone(ClimateZone::Desert, Vec3::ZERO, half, 16));
    let dunes = DesertDunes::in_zone(base, ClimateZone::Desert, half);
    let frame = ClimateZone::Desert.surface_frame(half);

    let peak_pt = frame.point_at(1.75, 2.25, 0.0);
    let constrained = dunes.constrain(peak_pt, 1.0);
    assert!(frame.height_of(constrained) >= half + 1.5);

    let mut counter = QuadCounter { count: 0 };
    dunes.emit_surface(&mut counter);
    assert!(counter.count > 256);
}

#[test]
fn test_glacial_massif_horns_and_cirque() {
    let half = 8.0;
    let base = Box::new(PlanarFace::in_zone(ClimateZone::Arctic, Vec3::ZERO, half, 16));
    let glacial = GlacialMassif::in_zone(base, ClimateZone::Arctic, half);
    let frame = ClimateZone::Arctic.surface_frame(half);

    let horn_pt = frame.point_at(1.25, 2.25, 0.0);
    let constrained = glacial.constrain(horn_pt, 1.0);
    assert!(frame.height_of(constrained) >= half + 1.5);

    let mut counter = QuadCounter { count: 0 };
    glacial.emit_surface(&mut counter);
    assert!(counter.count > 256);
}

#[test]
fn test_jungle_karst_towers_and_river_valley() {
    let half = 8.0;
    let base = Box::new(PlanarFace::in_zone(ClimateZone::Jungle, Vec3::ZERO, half, 16));
    let karst = JungleKarst::in_zone(base, ClimateZone::Jungle, half);
    let frame = ClimateZone::Jungle.surface_frame(half);

    let pinnacle_pt = frame.point_at(1.25, 2.25, 0.0);
    let constrained = karst.constrain(pinnacle_pt, 1.0);
    assert!(frame.height_of(constrained) >= half + 1.5);

    let mut counter = QuadCounter { count: 0 };
    karst.emit_surface(&mut counter);
    assert!(counter.count > 256);
}

#[test]
fn test_steppe_terrace_mesa_and_coulee() {
    let half = 8.0;
    let base = Box::new(PlanarFace::in_zone(ClimateZone::Steppe, Vec3::ZERO, half, 16));
    let terrace = SteppeTerrace::in_zone(base, ClimateZone::Steppe, half);
    let frame = ClimateZone::Steppe.surface_frame(half);

    let mesa_pt = frame.point_at(2.25, 2.25, 0.0);
    let constrained = terrace.constrain(mesa_pt, 1.0);
    assert!(frame.height_of(constrained) > half + 1.0);

    let mut counter = QuadCounter { count: 0 };
    terrace.emit_surface(&mut counter);
    assert!(counter.count > 256);
}
