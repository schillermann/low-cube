use glam::Vec3;
use world::MeshTarget;
use crate::menu::PixelStamp;
use super::burden::SlotBurden;
use super::limb::LimbFrame;
use super::relic::RelicFigure;

/// Focused interface for composable satchel compartment and plaque layers.
pub trait SlotPlaque {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget);
}

/// Base satchel compartment backplate rendering the recessed or illuminated background and 4-bar border.
#[derive(Clone, Copy)]
pub struct CompartmentBackplate {
    selected: bool,
    occupied: bool,
    rim: Vec3,
}

impl CompartmentBackplate {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(selected: bool, occupied: bool, rim: Vec3) -> Self {
        Self { selected, occupied, rim }
    }
}

impl SlotPlaque for CompartmentBackplate {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let bg = match (self.occupied, self.selected) {
            (false, true) => Vec3::new(0.22, 0.27, 0.37),
            (false, false) => Vec3::new(0.09, 0.11, 0.16),
            (true, true) => Vec3::new(0.28, 0.33, 0.44),
            (true, false) => Vec3::new(0.16, 0.19, 0.27),
        };
        frame.emit_framed_plaque(0.044, 0.044, 0.0032, bg, self.rim * 1.25, target);
    }
}

/// Decorator painting the unframed weight-grade corner triangle onto the compartment backplate behind the 3D item.
#[derive(Clone, Copy)]
pub struct WeightedCompartment<P> {
    origin: P,
    burden: SlotBurden,
}

impl<P> WeightedCompartment<P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: P, burden: SlotBurden) -> Self {
        Self { origin, burden }
    }
}

impl<P: SlotPlaque> SlotPlaque for WeightedCompartment<P> {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        self.origin.emit_slot(frame, target);
        self.burden.emit_corner_triangle(frame, target);
    }
}

/// Decorator rendering a 3D `RelicFigure` showcase in front of an underlying `SlotPlaque`.
#[derive(Clone, Copy)]
pub struct ShowcasedCompartment<P> {
    origin: P,
    figure: RelicFigure,
}

impl<P> ShowcasedCompartment<P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: P, figure: RelicFigure) -> Self {
        Self { origin, figure }
    }
}

impl<P: SlotPlaque> SlotPlaque for ShowcasedCompartment<P> {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        self.origin.emit_slot(frame, target);
        self.figure.emit_showcase(frame, target);
    }
}

/// Decorator stamping a foreground eye-projected white stack tally badge over an occupied compartment.
#[derive(Clone, Copy)]
pub struct TalliedCompartment<P> {
    origin: P,
    eye: Vec3,
    rim: Vec3,
    count: u32,
}

impl<P> TalliedCompartment<P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: P, eye: Vec3, rim: Vec3, count: u32) -> Self {
        Self { origin, eye, rim, count }
    }
}

impl<P: SlotPlaque> SlotPlaque for TalliedCompartment<P> {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        self.origin.emit_slot(frame, target);
        if self.count > 0 {
            let badge = frame.projected_toward(self.eye, 0.62).shifted(0.031, -0.035, 0.0);
            badge.emit_framed_plaque(0.013, 0.0085, 0.0016, Vec3::new(0.07, 0.09, 0.14), self.rim * 1.25, target);
            PixelStamp::new(0.0021, 0.0021, Vec3::splat(1.68))
                .emit_on_frame(&format!("{}", self.count), 0.0, 0.0, 0.016, &badge, target);
        }
    }
}

/// Satchel legend plaque displaying the selected slot's item title, English weight grade, and controls.
#[derive(Clone, Copy)]
pub struct LegendCompartment {
    burden: SlotBurden,
}

impl LegendCompartment {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(burden: SlotBurden) -> Self {
        Self { burden }
    }
}

impl SlotPlaque for LegendCompartment {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        frame.emit_framed_plaque(0.182, 0.032, 0.0024, Vec3::new(0.08, 0.10, 0.15), Vec3::new(0.42, 0.27, 0.16), target);
        let heading = format!("{}", self.burden);
        PixelStamp::new(0.0021, 0.0023, Vec3::new(1.55, 1.30, 0.48))
            .emit_on_frame(&heading, 0.0, 0.017, 0.016, frame, target);
        let hint = PixelStamp::new(0.0021, 0.0021, Vec3::new(1.42, 1.46, 1.54));
        hint.emit_on_frame("WASD: NAVIGATION  ESC: CLOSE", 0.0, 0.0, 0.016, frame, target);
        hint.emit_on_frame("Q: LEFT HAND  E: RIGHT HAND", 0.0, -0.017, 0.016, frame, target);
    }
}
