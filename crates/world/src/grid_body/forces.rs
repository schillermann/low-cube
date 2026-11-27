use glam::Vec3;

/// Focused contract evaluating fictitious acceleration (`m/s^2`) at `(local_pos, local_vel)` inside a `GridBody`.
pub trait InertialForce {
    fn acceleration_at(&self, local_pos: Vec3, local_vel: Vec3) -> Vec3;
}

/// Null-Object `InertialForce` representing inertial deep-space rest (`0.0 m/s^2`).
#[derive(Clone, Copy, Debug)]
pub struct ZeroGravityField;

impl ZeroGravityField {
    /// Primary constructor initializing the zero-state inertial baseline.
    pub const fn new() -> Self {
        Self
    }
}

impl InertialForce for ZeroGravityField {
    fn acceleration_at(&self, _local_pos: Vec3, _local_vel: Vec3) -> Vec3 {
        Vec3::ZERO
    }
}

/// `InertialForce` decorator injecting Einstein equivalence linear thrust gravity $-\vec{a}_{\text{local}}$ (`2` fields).
#[derive(Clone, Copy, Debug)]
pub struct ThrustGravity<F> {
    inner: F,
    local_thrust_accel: Vec3,
}

impl<F> ThrustGravity<F> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: F, local_thrust_accel: Vec3) -> Self {
        Self { inner, local_thrust_accel }
    }
}

impl<F: InertialForce> InertialForce for ThrustGravity<F> {
    fn acceleration_at(&self, local_pos: Vec3, local_vel: Vec3) -> Vec3 {
        self.inner.acceleration_at(local_pos, local_vel) - self.local_thrust_accel
    }
}

/// `InertialForce` decorator injecting outward centrifugal spin gravity $-\vec{\omega} \times (\vec{\omega} \times (\vec{r} - \vec{c}))$ (`3` fields).
#[derive(Clone, Copy, Debug)]
pub struct CentrifugalRing<F> {
    inner: F,
    local_omega: Vec3,
    com_local: Vec3,
}

impl<F> CentrifugalRing<F> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: F, local_omega: Vec3, com_local: Vec3) -> Self {
        Self { inner, local_omega, com_local }
    }
}

impl<F: InertialForce> InertialForce for CentrifugalRing<F> {
    fn acceleration_at(&self, local_pos: Vec3, local_vel: Vec3) -> Vec3 {
        let arm = local_pos - self.com_local;
        let centrifugal = -self.local_omega.cross(self.local_omega.cross(arm));
        self.inner.acceleration_at(local_pos, local_vel) + centrifugal
    }
}

/// `InertialForce` decorator injecting Coriolis deflection $-2\,\vec{\omega} \times \vec{v}_{\text{local}}$ (`2` fields).
#[derive(Clone, Copy, Debug)]
pub struct CoriolisDeflection<F> {
    inner: F,
    local_omega: Vec3,
}

impl<F> CoriolisDeflection<F> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: F, local_omega: Vec3) -> Self {
        Self { inner, local_omega }
    }
}

impl<F: InertialForce> InertialForce for CoriolisDeflection<F> {
    fn acceleration_at(&self, local_pos: Vec3, local_vel: Vec3) -> Vec3 {
        let coriolis = -2.0 * self.local_omega.cross(local_vel);
        self.inner.acceleration_at(local_pos, local_vel) + coriolis
    }
}

/// Polymorphic `0 g` magnetic boot latch holding astronauts onto a `GridBody` hull until they kick off (`Space`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MagBootGrip {
    Disengaged,
    Latched { hull_up: Vec3 },
}

impl MagBootGrip {
    pub const fn evaluate(
        on_hull_deck: bool,
        hull_up: Vec3,
        jump_kick: bool,
        fictitious_accel_sq: f32,
    ) -> Self {
        if on_hull_deck && !jump_kick && fictitious_accel_sq < 0.09 {
            Self::Latched { hull_up }
        } else {
            Self::Disengaged
        }
    }

    pub const fn latched(self) -> bool {
        matches!(self, Self::Latched { .. })
    }

    pub const fn up_or(self, fallback: Vec3) -> Vec3 {
        match self {
            Self::Latched { hull_up } => hull_up,
            Self::Disengaged => fallback,
        }
    }
}

/// Ship-local non-inertial dynamics snapshot carried by `DeckFrame` (`3` strictly private fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeckDynamics {
    local_proper_accel: Vec3,
    local_omega: Vec3,
    com_local: Vec3,
}

impl DeckDynamics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(local_proper_accel: Vec3, local_omega: Vec3, com_local: Vec3) -> Self {
        Self { local_proper_accel, local_omega, com_local }
    }

    pub const fn thrust_only(thrust_g: f32) -> Self {
        Self::new(Vec3::new(0.0, thrust_g, 0.0), Vec3::ZERO, Vec3::ZERO)
    }

    pub const fn local_proper_accel(self) -> Vec3 {
        self.local_proper_accel
    }

    pub const fn local_omega(self) -> Vec3 {
        self.local_omega
    }

    /// Evaluates the complete `CoriolisDeflection(CentrifugalRing(ThrustGravity(ZeroGravityField)))` chain.
    pub fn fictitious_acceleration(self, local_pos: Vec3, local_vel: Vec3) -> Vec3 {
        let chain = CoriolisDeflection::new(
            CentrifugalRing::new(
                ThrustGravity::new(ZeroGravityField::new(), self.local_proper_accel),
                self.local_omega,
                self.com_local,
            ),
            self.local_omega,
        );
        chain.acceleration_at(local_pos, local_vel)
    }
}
