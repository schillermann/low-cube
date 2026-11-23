use glam::{Vec2, Vec3};
use world::{
    BlockMedium, BlockMold, MeshTarget, Quad, SurfaceFrame, VehicleAssembly, VehicleAxes,
    VehiclePart, WedgeHeading,
};
use crate::controls::{KeyboardControls, LocomotionTarget};

pub trait CraftFixture {
    fn assembly(&self) -> VehicleAssembly;
}

pub struct QuadRcsCraft;

impl QuadRcsCraft {
    pub const fn new() -> Self {
        Self
    }
}

impl CraftFixture for QuadRcsCraft {
    fn assembly(&self) -> VehicleAssembly {
        let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let att = |heading: WedgeHeading| {
            BlockMold::shape_from_stamp([1, heading.code(), BlockMedium::RocketNozzle.code()])
        };
        let parts = vec![
            VehiclePart::new((0, 0), 0.5, BlockMold::cube().in_medium(BlockMedium::CabinSeat).shape_for(Vec3::NEG_Z, &frame)),
            VehiclePart::new((0, -1), 0.5, BlockMold::cube().in_medium(BlockMedium::FlightConsole).shape_for(Vec3::Z, &frame)),
            VehiclePart::new((0, 0), 0.0, BlockMold::cube().in_medium(BlockMedium::RocketNozzle).shape_for(Vec3::NEG_Z, &frame)),
            VehiclePart::new((-1, 0), 0.5, att(WedgeHeading::East)),
            VehiclePart::new((1, 0), 0.5, att(WedgeHeading::West)),
            VehiclePart::new((0, -2), 0.5, att(WedgeHeading::North)),
            VehiclePart::new((0, 1), 0.5, att(WedgeHeading::South)),
            VehiclePart::new((0, 0), 1.0, BlockMold::cube().in_medium(BlockMedium::QuarriedStone).shape_for(Vec3::Z, &frame)),
            VehiclePart::new((0, -1), 1.0, BlockMold::cube().in_medium(BlockMedium::QuarriedStone).shape_for(Vec3::Z, &frame)),
            VehiclePart::new((0, 0), 1.5, BlockMold::cube().in_medium(BlockMedium::QuarriedStone).shape_for(Vec3::Z, &frame)),
            VehiclePart::new((0, -1), 1.5, BlockMold::cube().in_medium(BlockMedium::QuarriedStone).shape_for(Vec3::Z, &frame)),
        ];
        VehicleAssembly::new(
            Vec3::new(0.25, 10.0, 0.25),
            VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y),
            Vec3::NEG_Z,
            parts,
        )
    }
}

mod card_metrics {
    use glam::{Vec2, Vec3};

    pub trait QuadTally: Copy {
        fn recorded(&self, center: Vec2, glow: f32, col: Vec3) -> Self;
        fn active(&self, min_exhaust: usize) -> bool;
        fn blank(&self) -> bool;
        fn mean_steer(&self) -> Vec2;
    }

    #[derive(Clone, Copy)]
    pub struct CardQuadTally {
        card_quads: usize,
        steer_quads: usize,
        exhaust_cores: usize,
        steer_sum: Vec2,
    }

    impl CardQuadTally {
        pub const fn new(
            card_quads: usize,
            steer_quads: usize,
            exhaust_cores: usize,
            steer_sum: Vec2,
        ) -> Self {
            Self { card_quads, steer_quads, exhaust_cores, steer_sum }
        }
    }

    impl QuadTally for CardQuadTally {
        fn recorded(&self, center: Vec2, glow: f32, col: Vec3) -> Self {
            if center.x >= -0.34 || center.y >= -0.28 {
                return *self;
            }
            let steer_hit = glow > 0.97 && (col - Vec3::new(0.44, 1.00, 0.52)).length() < 0.02;
            let exhaust_hit = glow > 0.98 && col.x > 0.80 && col.y > 0.90;
            Self::new(
                self.card_quads + 1,
                self.steer_quads + usize::from(steer_hit),
                self.exhaust_cores + usize::from(exhaust_hit),
                if steer_hit { self.steer_sum + center } else { self.steer_sum },
            )
        }

        fn active(&self, min_exhaust: usize) -> bool {
            self.card_quads > 30 && self.exhaust_cores >= min_exhaust && self.steer_quads >= 3
        }

        fn blank(&self) -> bool {
            self.card_quads == 0
        }

        fn mean_steer(&self) -> Vec2 {
            assert!(self.steer_quads > 0, "STEER quads absent in HUD probe");
            self.steer_sum / (self.steer_quads as f32)
        }
    }

    pub trait QuadSpan: Copy {
        fn enclosed(&self, corners: [Vec3; 4]) -> Self;
        fn within_card(&self) -> bool;
    }

    #[derive(Clone, Copy)]
    pub struct CardBounds {
        min_pt: Vec3,
        max_pt: Vec3,
    }

    impl CardBounds {
        pub const fn new(min_pt: Vec3, max_pt: Vec3) -> Self {
            Self { min_pt, max_pt }
        }
    }

    impl QuadSpan for CardBounds {
        fn enclosed(&self, corners: [Vec3; 4]) -> Self {
            let (mut lo, mut hi) = (self.min_pt, self.max_pt);
            for v in corners {
                lo = lo.min(v);
                hi = hi.max(v);
            }
            Self::new(lo, hi)
        }

        fn within_card(&self) -> bool {
            self.min_pt.x >= -0.955
                && self.max_pt.x <= -0.340
                && self.min_pt.y >= -0.935
                && self.max_pt.y <= -0.290
        }
    }
}

use self::card_metrics::{CardBounds, CardQuadTally, QuadSpan, QuadTally};

pub struct WireHudProbe {
    tally: CardQuadTally,
    bounds: CardBounds,
}

impl WireHudProbe {
    pub const fn new(tally: CardQuadTally, bounds: CardBounds) -> Self {
        Self { tally, bounds }
    }

    pub const fn empty() -> Self {
        Self::new(
            CardQuadTally::new(0, 0, 0, Vec2::ZERO),
            CardBounds::new(Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        )
    }

    pub fn active_telemetry(&self, min_exhaust: usize) -> bool {
        self.tally.active(min_exhaust)
    }

    pub fn blank_in_cockpit(&self) -> bool {
        self.tally.blank()
    }

    pub fn confined_within_card(&self) -> bool {
        self.bounds.within_card()
    }

    pub fn mirrors_steer_x_against(&self, aft: &Self) -> bool {
        self.tally.mean_steer().x > -0.60 && aft.tally.mean_steer().x < -0.70
    }

    pub fn inverts_steer_y_against(&self, belly: &Self) -> bool {
        belly.tally.mean_steer().y > self.tally.mean_steer().y + 0.08
    }
}

impl MeshTarget for WireHudProbe {
    fn push_quad(&mut self, quad: Quad) {
        let glow = quad.radiate(|g| g);
        quad.transfer(|v0, v1, v2, v3, _, col| {
            let c = Vec2::new((v0.x + v2.x) * 0.5, (v0.y + v2.y) * 0.5);
            self.tally = self.tally.recorded(c, glow, col);
            self.bounds = self.bounds.enclosed([v0, v1, v2, v3]);
        });
    }
}

pub struct LiftAndRollControls;

impl LiftAndRollControls {
    pub const fn new() -> Self {
        Self
    }
}

impl KeyboardControls for LiftAndRollControls {
    fn drive_locomotion(&self, _fwd: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.jump();
        target.roll(1.0);
    }
}
