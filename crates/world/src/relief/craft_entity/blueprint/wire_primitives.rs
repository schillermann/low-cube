use glam::Vec3;

pub const MAX_WIRE_EDGES: usize = 128;
pub const MAX_WIRE_CELLS: usize = 24;
pub const MAX_EXHAUST_JETS: usize = 16;

/// Propulsion station role inside `HullWireframe`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellRole {
    MainBell,
    ActiveMainBell,
    FusionBell,
    ActiveFusionBell,
    AttitudePod,
    ActivePod,
}

/// Chromatic exhaust family distinguishing RCS vernier gas, chemical bells, and fusion plasma.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JetTone {
    VernierCyan,
    BellAmber,
    FusionBlue,
}

/// Quantized 3D outer-hull crease segment in normalized ship-local space (`0.001` precision).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireframeEdge {
    start: [i16; 3],
    end: [i16; 3],
}

impl WireframeEdge {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(start: [i16; 3], end: [i16; 3]) -> Self {
        Self { start, end }
    }

    pub const fn vacant() -> Self {
        Self::new([0; 3], [0; 3])
    }

    pub fn segment(a_local: Vec3, b_local: Vec3, scale: f32) -> Self {
        let q = |v: f32| ((v * scale).clamp(-1.24, 1.24) * 1000.0).round().clamp(-1250.0, 1250.0) as i16;
        Self::new(
            [q(a_local.x), q(a_local.y), q(a_local.z)],
            [q(b_local.x), q(b_local.y), q(b_local.z)],
        )
    }

    pub(super) fn unpack(self) -> (Vec3, Vec3) {
        let pt = |c: [i16; 3]| {
            Vec3::new(
                f32::from(c[0]) * 0.001,
                f32::from(c[1]) * 0.001,
                f32::from(c[2]) * 0.001,
            )
        };
        (pt(self.start), pt(self.end))
    }
}

/// Quantized 3D ship-local thruster station center, half-extents, and `CellRole`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireframeCell {
    locus: [i8; 3],
    span: [u8; 3],
    role: CellRole,
}

impl WireframeCell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(locus: [i8; 3], span: [u8; 3], role: CellRole) -> Self {
        Self { locus, span, role }
    }

    pub const fn vacant() -> Self {
        Self::new([0; 3], [16; 3], CellRole::MainBell)
    }

    pub fn prism(local_center: Vec3, local_half: Vec3, scale: f32, role: CellRole) -> Self {
        let ql = |v: f32| ((v * scale).clamp(-1.15, 1.15) * 100.0).round().clamp(-120.0, 120.0) as i8;
        let qs = |v: f32| ((v * scale).clamp(0.06, 1.00) * 100.0).round().clamp(6.0, 100.0) as u8;
        Self::new(
            [ql(local_center.x), ql(local_center.y), ql(local_center.z)],
            [qs(local_half.x), qs(local_half.y), qs(local_half.z)],
            role,
        )
    }

    pub(super) fn unpack(self) -> (CellRole, Vec3, Vec3) {
        let pos = Vec3::new(
            f32::from(self.locus[0]) * 0.01,
            f32::from(self.locus[1]) * 0.01,
            f32::from(self.locus[2]) * 0.01,
        );
        let half = Vec3::new(
            f32::from(self.span[0]) * 0.01,
            f32::from(self.span[1]) * 0.01,
            f32::from(self.span[2]) * 0.01,
        );
        (self.role, pos, half)
    }
}

/// Quantized 3D nozzle mouth, ship-local exhaust unit vector, and firing throttle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExhaustJet {
    mouth: [i8; 3],
    dir: [i8; 3],
    intensity: (JetTone, u8),
}

impl ExhaustJet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(mouth: [i8; 3], dir: [i8; 3], intensity: (JetTone, u8)) -> Self {
        Self { mouth, dir, intensity }
    }

    pub const fn vacant() -> Self {
        Self::new([0; 3], [0, -100, 0], (JetTone::VernierCyan, 0))
    }

    pub fn active(local_mouth: Vec3, scale: f32, local_dir: Vec3, tone: JetTone, throttle: f32) -> Self {
        let qm = |v: f32| ((v * scale).clamp(-1.24, 1.24) * 100.0).round().clamp(-125.0, 125.0) as i8;
        let u = local_dir.normalize_or_zero();
        let qd = |v: f32| (v.clamp(-1.0, 1.0) * 100.0).round() as i8;
        let thr = (throttle.clamp(0.0, 1.0) * 255.0).round() as u8;
        Self::new(
            [qm(local_mouth.x), qm(local_mouth.y), qm(local_mouth.z)],
            [qd(u.x), qd(u.y), qd(u.z)],
            (tone, thr),
        )
    }

    pub(super) fn unpack(self) -> (JetTone, Vec3, Vec3, f32) {
        let (tone, thr) = self.intensity;
        let m = Vec3::new(
            f32::from(self.mouth[0]) * 0.01,
            f32::from(self.mouth[1]) * 0.01,
            f32::from(self.mouth[2]) * 0.01,
        );
        let d = Vec3::new(
            f32::from(self.dir[0]) * 0.01,
            f32::from(self.dir[1]) * 0.01,
            f32::from(self.dir[2]) * 0.01,
        )
        .normalize_or_zero();
        (tone, m, d, f32::from(thr) / 255.0)
    }
}
