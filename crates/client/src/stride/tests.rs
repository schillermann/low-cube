use glam::Vec3;
use super::decorator::{DeflectedStride, RestingStride, StrideResolution, StrideTrial};

struct MockTrial {
    passable: bool,
    destination: Vec3,
}

impl StrideTrial for MockTrial {
    fn passable(&self) -> bool {
        self.passable
    }

    fn destination(&self) -> Vec3 {
        self.destination
    }
}

#[test]
fn test_deflected_stride_returns_primary_when_passable() {
    let fallback = Box::new(RestingStride::new(Vec3::ZERO));
    let primary = Box::new(MockTrial {
        passable: true,
        destination: Vec3::new(1.0, 0.0, 0.0),
    });
    let resolution = DeflectedStride::new(primary, fallback);
    assert_eq!(resolution.resolve(), Vec3::new(1.0, 0.0, 0.0));
}

#[test]
fn test_deflected_stride_delegates_to_fallback_when_obstructed() {
    let fallback = Box::new(RestingStride::new(Vec3::new(0.0, 5.0, 0.0)));
    let primary = Box::new(MockTrial {
        passable: false,
        destination: Vec3::new(1.0, 0.0, 0.0),
    });
    let resolution = DeflectedStride::new(primary, fallback);
    assert_eq!(resolution.resolve(), Vec3::new(0.0, 5.0, 0.0));
}

#[test]
fn test_deflected_stride_cascades_multiple_decorators() {
    let resting = Box::new(RestingStride::new(Vec3::new(0.0, 1.0, 0.0)));
    let lateral = Box::new(DeflectedStride::new(
        Box::new(MockTrial {
            passable: true,
            destination: Vec3::new(0.0, 1.0, 2.0),
        }),
        resting,
    ));
    let blocked_primary = Box::new(MockTrial {
        passable: false,
        destination: Vec3::new(2.0, 1.0, 0.0),
    });
    let chain = DeflectedStride::new(blocked_primary, lateral);
    assert_eq!(chain.resolve(), Vec3::new(0.0, 1.0, 2.0));
}
