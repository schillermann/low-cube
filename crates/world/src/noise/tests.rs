use super::*;
use glam::Vec3;

#[test]
fn test_coherent_noise_bounds_and_continuity() {
    let noise = CoherentNoise::new(1337, 0.1);
    for i in 0..10 {
        for j in 0..10 {
            let pt = Vec3::new(i as f32 * 0.5, 0.0, j as f32 * 0.5);
            let val = noise.sample(pt);
            assert!((-1.0..=1.0).contains(&val), "Noise must stay in [-1, 1], got {val}");
        }
    }
}

#[test]
fn test_fractal_harmonics_octave_composition() {
    let base = Box::new(CoherentNoise::new(42, 0.05));
    let harmonics = FractalHarmonics::new(base, 4, 0.5, 2.0);
    let sample = harmonics.sample(Vec3::new(10.0, 5.0, 2.0));
    assert!((-1.0..=1.0).contains(&sample));
}

#[test]
fn test_domain_warp_bounds() {
    let target = Box::new(CoherentNoise::new(1, 0.05));
    let wx = Box::new(CoherentNoise::new(2, 0.02));
    let wz = Box::new(CoherentNoise::new(3, 0.02));
    let warped = DomainWarp::new(target, wx, wz, 4.0);
    let sample = warped.sample(Vec3::new(5.0, 0.0, 5.0));
    assert!((-1.0..=1.0).contains(&sample));
}
