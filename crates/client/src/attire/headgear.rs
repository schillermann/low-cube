use glam::Vec3;
use world::{MeshTarget, Quad};
use super::crown::HeadSculpture;
use super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};
use super::pose::BodyPose;

/// Polymorphic state of the integrated EVA helmet floodlight (active from second 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LampBeam {
    Illuminated,
    Extinguished,
}

impl LampBeam {
    pub const fn illuminated(self) -> bool {
        matches!(self, Self::Illuminated)
    }

    pub const fn toggled(self) -> Self {
        match self {
            Self::Illuminated => Self::Extinguished,
            Self::Extinguished => Self::Illuminated,
        }
    }

    pub const fn wire_bit(self) -> u8 {
        match self {
            Self::Illuminated => 0x00,
            Self::Extinguished => 0x80,
        }
    }

    pub const fn from_wire_byte(byte: u8) -> Self {
        if (byte & 0x80) == 0 {
            Self::Illuminated
        } else {
            Self::Extinguished
        }
    }
}

/// Sculpted EVA pressure-helmet visor dome variant applied by `PressureHelmHead`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelmVisor {
    ClearBubble,
    SolarGold,
    SurveyCyan,
    TacticalSlit,
    OpticArray,
    ArmoredCowl,
}

impl HelmVisor {
    fn emit(self, pose: &BodyPose, frame: &LimbFrame, (shell, trim): (Vec3, Vec3), target: &mut dyn MeshTarget) {
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let titanium = Vec3::new(0.68, 0.73, 0.80);
        let dark_alloy = Vec3::new(0.24, 0.28, 0.34);
        let gold_foil = Vec3::new(0.94, 0.72, 0.22);
        let cyan_glass = Vec3::new(0.28, 0.82, 0.96);

        // Rear & crown pressurized shell enclosing the occiput and temples
        TaperedPrism::new(8, Vec3::new(0.0, hs(0.010), hw(-0.022)), Vec3::new(0.0, hs(0.168), hw(-0.016)), hw(0.116), hw(0.120), hw(0.118), hw(0.122), shell)
            .emit(frame, false, target);
        TaperedPrism::new(8, Vec3::new(0.0, hs(0.164), hw(-0.014)), Vec3::new(0.0, hs(0.248), hw(-0.008)), hw(0.116), hw(0.120), hw(0.082), hw(0.088), shell)
            .emit(frame, true, target);
        for side in [-1.0_f32, 1.0_f32] {
            AxialLimb::new(
                Vec3::new(side * hw(0.098), hs(0.010), hw(0.018)),
                Vec3::new(side * hw(0.104), hs(0.164), hw(0.012)),
                hw(0.020),
                hw(0.024),
                trim,
            )
            .emit(frame, true, target);
        }
        // Chin pressure seal bar
        TaperedPrism::new(6, Vec3::new(0.0, hs(-0.012), hw(0.026)), Vec3::new(0.0, hs(0.026), hw(0.038)), hw(0.094), hw(0.082), hw(0.098), hw(0.086), titanium)
            .emit(frame, true, target);

        match self {
            Self::ClearBubble => {
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.150), hw(0.044)), Vec3::new(0.0, hs(0.184), hw(0.056)), hw(0.096), hw(0.058), hw(0.088), hw(0.052), trim)
                    .emit(frame, true, target);
            }
            Self::SolarGold => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.030), hw(0.034)), Vec3::new(0.0, hs(0.166), hw(0.038)), hw(0.096), hw(0.074), hw(0.098), hw(0.076), gold_foil)
                    .emit(frame, true, target);
            }
            Self::SurveyCyan => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.034), hw(0.034)), Vec3::new(0.0, hs(0.164), hw(0.038)), hw(0.094), hw(0.072), hw(0.096), hw(0.074), cyan_glass)
                    .emit(frame, true, target);
            }
            Self::TacticalSlit => {
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.024), hw(0.036)), Vec3::new(0.0, hs(0.092), hw(0.046)), hw(0.094), hw(0.068), hw(0.096), hw(0.072), dark_alloy)
                    .emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.092), hw(0.048)), Vec3::new(0.0, hs(0.148), hw(0.050)), hw(0.092), hw(0.066), hw(0.090), hw(0.064), gold_foil)
                    .emit(frame, true, target);
            }
            Self::OpticArray => {
                for side in [-1.0_f32, 1.0_f32] {
                    TaperedPrism::new(8, Vec3::new(side * hw(0.042), hs(0.104), hw(0.090)), Vec3::new(side * hw(0.042), hs(0.142), hw(0.090)), hw(0.030), hw(0.018), hw(0.030), hw(0.018), dark_alloy)
                        .emit(frame, true, target);
                    TaperedPrism::new(6, Vec3::new(side * hw(0.042), hs(0.108), hw(0.098)), Vec3::new(side * hw(0.042), hs(0.138), hw(0.098)), hw(0.022), hw(0.012), hw(0.022), hw(0.012), cyan_glass)
                        .emit(frame, true, target);
                }
            }
            Self::ArmoredCowl => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.146), hw(-0.004)), Vec3::new(0.0, hs(0.186), hw(0.010)), hw(0.132), hw(0.138), hw(0.114), hw(0.118), titanium)
                    .emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.078), hw(0.042)), Vec3::new(0.0, hs(0.148), hw(0.046)), hw(0.094), hw(0.068), hw(0.092), hw(0.066), cyan_glass)
                    .emit(frame, true, target);
            }
        }
    }
}

/// `HeadSculpture` decorator wrapping an inner head with anEVA pressure-helmet shell and visor (`3` fields).
#[derive(Clone, Copy, Debug)]
pub struct PressureHelmHead<S> {
    origin: S,
    visor: HelmVisor,
    palette: (Vec3, Vec3),
}

impl<S> PressureHelmHead<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, visor: HelmVisor, palette: (Vec3, Vec3)) -> Self {
        Self { origin, visor, palette }
    }
}

impl<S: HeadSculpture> HeadSculpture for PressureHelmHead<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let frame = pose.crown_frame();
        self.visor.emit(pose, &frame, self.palette, target);
    }
}

/// `HeadSculpture` decorator mounting a UHF suit transceiver pod and whip antenna (`2` fields).
#[derive(Clone, Copy, Debug)]
pub struct RadioMastHead<S> {
    origin: S,
    accent: Vec3,
}

impl<S> RadioMastHead<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, accent: Vec3) -> Self {
        Self { origin, accent }
    }
}

impl<S: HeadSculpture> HeadSculpture for RadioMastHead<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let frame = pose.crown_frame();
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let chassis = Vec3::new(0.22, 0.26, 0.32);
        TaperedPrism::hexagonal(
            Vec3::new(hw(-0.112), hs(0.092), hw(-0.012)),
            Vec3::new(hw(-0.128), hs(0.146), hw(-0.012)),
            hw(0.020),
            hw(0.018),
            chassis,
        )
        .emit(&frame, true, target);
        AxialLimb::new(
            Vec3::new(hw(-0.122), hs(0.144), hw(-0.014)),
            Vec3::new(hw(-0.132), hs(0.328), hw(-0.042)),
            hw(0.006),
            hw(0.003),
            self.accent,
        )
        .emit(&frame, true, target);
    }
}

/// `HeadSculpture` decorator mounting the integrated forehead LED floodlight module (`2` fields).
#[derive(Clone, Copy, Debug)]
pub struct HelmetLampHead<S> {
    origin: S,
    beam: LampBeam,
}

impl<S> HelmetLampHead<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, beam: LampBeam) -> Self {
        Self { origin, beam }
    }
}

impl<S: HeadSculpture> HeadSculpture for HelmetLampHead<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let frame = pose.crown_frame();
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let bezel = Vec3::new(0.24, 0.28, 0.35);
        TaperedPrism::new(
            6,
            Vec3::new(0.0, hs(0.184), hw(0.068)),
            Vec3::new(0.0, hs(0.222), hw(0.096)),
            hw(0.040),
            hw(0.022),
            hw(0.036),
            hw(0.020),
            bezel,
        )
        .emit(&frame, true, target);

        let z_lens = hw(0.109);
        let p0 = frame.point(hw(-0.028), hs(0.190), z_lens);
        let p1 = frame.point(hw(0.028), hs(0.190), z_lens);
        let p2 = frame.point(hw(0.028), hs(0.216), z_lens);
        let p3 = frame.point(hw(-0.028), hs(0.216), z_lens);
        let normal = (p1 - p0).cross(p3 - p0).normalize_or_zero();
        if self.beam.illuminated() {
            target.push_quad(Quad::glowing(
                p0,
                p1,
                p2,
                p3,
                normal,
                Vec3::new(0.92, 0.98, 1.00),
                1.85,
            ));
        } else {
            target.push_quad(Quad::new(p0, p1, p2, p3, normal, Vec3::new(0.16, 0.20, 0.26)));
        }
    }
}
