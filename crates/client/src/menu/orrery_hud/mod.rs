mod burn_magnet;
mod compass_ring;
mod course_alignment;
mod hud_pen;
mod hull_channels;
mod lateral_slip;
mod overlays;
mod phosphor;
mod prograde_drift;
mod scale_decorators;
mod sight_stage;
mod spine;
mod standoff;
mod touchdown_ghost;
mod wire_ship;
mod wire_stage;

pub use self::hull_channels::ShipOpticHud;
pub use self::overlays::{ChannelStripOverlay, OpticOverlay};
#[cfg(test)]
pub use self::burn_magnet::BurnMagnetOverlay;
#[cfg(test)]
pub use self::compass_ring::CompassRingOverlay;
#[cfg(test)]
pub use self::course_alignment::CourseAlignmentOverlay;
#[cfg(test)]
pub use self::hull_channels::{BellyFlightHud, NoseFlightHud};
#[cfg(test)]
pub use self::lateral_slip::LateralSlipOverlay;
#[cfg(test)]
pub use self::overlays::{
    RangeTapeOverlay as TestRangeTapeOverlay, SilentOverlay,
    TargetLockOverlay as TestTargetLockOverlay,
};
#[cfg(test)]
pub use self::phosphor::PhosphorTone;
#[cfg(test)]
pub use self::prograde_drift::ProgradeDriftOverlay;
#[cfg(test)]
pub use self::scale_decorators::{
    ApproachZoneScale as TestApproachZoneScale, ChevronReadoutScale as TestChevronReadoutScale,
    GlassPlateScale,
};
#[cfg(test)]
pub use self::sight_stage::{COMPASS_RADIUS, DirectionSight, HudSightStage};
#[cfg(test)]
pub use self::spine::{GraduatedSpine as TestGraduatedSpine, RangeScale, ScaleDomain};
#[cfg(test)]
pub use self::standoff::{BoresightTape, ReticleStandoffScale};
#[cfg(test)]
pub use self::touchdown_ghost::TouchdownGhostOverlay;
#[cfg(test)]
pub use self::wire_ship::WireframeShipOverlay;
