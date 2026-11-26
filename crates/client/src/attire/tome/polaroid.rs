use glam::Vec3;
use world::{BlockMold, MeshTarget, Quad};
use crate::menu::PixelStamp;
use super::super::burden::SlotBurden;
use super::super::compartment::SlotPlaque;
use super::super::limb::{FacetedSolid, LimbFrame, TaperedPrism};
use super::super::pouch::BeltSatchel;
use super::super::relic::{HeldRelic, RelicFigure};
use super::super::YouthAttire;
use super::domain::FolioTab;
use super::entry::CraftingEntry;

/// Decorator over any `SlotPlaque` painting a bottom-right emerald readiness pip and multi-yield tag.
#[derive(Clone, Copy)]
pub struct ReadinessPipCompartment<P> {
    origin: P,
    ready: bool,
    batch_yield: u32,
}

impl<P> ReadinessPipCompartment<P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: P, ready: bool, batch_yield: u32) -> Self {
        Self {
            origin,
            ready,
            batch_yield,
        }
    }
}

impl<P: SlotPlaque> SlotPlaque for ReadinessPipCompartment<P> {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        self.origin.emit_slot(frame, target);
        if self.ready {
            let (xr, yb, span, z_pip) = (0.043, -0.043, 0.023, 0.0195);
            let normal = (frame.point(0.0, 1.0, -1.0) - frame.point(0.0, 0.0, 0.0)).normalize_or_zero();
            target.push_quad(Quad::new(
                frame.point(xr - span, yb, z_pip),
                frame.point(xr, yb, z_pip),
                frame.point(xr, yb + span, z_pip),
                frame.point(xr, yb + span, z_pip),
                normal,
                Vec3::new(0.24, 1.48, 0.56),
            ));
        }
        if self.batch_yield > 1 {
            let tag = frame.shifted(-0.024, -0.028, -0.095);
            PixelStamp::new(0.0020, 0.0020, Vec3::new(1.58, 1.54, 1.32))
                .emit_on_frame(&format!("X{}", self.batch_yield), 0.0, 0.0, 0.0, &tag, target);
        }
    }
}

/// Flat paper rectangle inside Polaroid card stock implementing `SlotPlaque` (`3` private fields).
#[derive(Clone, Copy)]
struct PaperSheet {
    bounds: (f32, f32, f32, f32),
    depth: f32,
    pigment: Vec3,
}

impl PaperSheet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(bounds: (f32, f32, f32, f32), depth: f32, pigment: Vec3) -> Self {
        Self { bounds, depth, pigment }
    }
}

impl SlotPlaque for PaperSheet {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (x0, y0, x1, y1) = self.bounds;
        let normal = (frame.point(0.0, 0.25, -1.0) - frame.point(0.0, 0.0, 0.0)).normalize_or_zero();
        target.push_quad(Quad::new(
            frame.point(x0, y0, self.depth),
            frame.point(x1, y0, self.depth),
            frame.point(x1, y1, self.depth),
            frame.point(x0, y1, self.depth),
            normal,
            self.pigment,
        ));
    }
}

/// Tactile Polaroid print card with drop shadow, warm cream paper stock, dark aperture, and bottom chin caption.
#[derive(Clone, Copy)]
pub struct PolaroidCard {
    title: &'static str,
    batch_yield: u32,
}

impl PolaroidCard {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(title: &'static str, batch_yield: u32) -> Self {
        Self { title, batch_yield }
    }
}

impl SlotPlaque for PolaroidCard {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        PaperSheet::new((-0.088, -0.094, 0.096, 0.086), 0.0195, Vec3::new(0.04, 0.05, 0.08)).emit_slot(frame, target);
        PaperSheet::new((-0.092, -0.090, 0.092, 0.090), 0.0185, Vec3::new(0.76, 0.74, 0.68)).emit_slot(frame, target);
        PaperSheet::new((-0.089, -0.087, 0.089, 0.087), 0.0178, Vec3::new(0.98, 0.96, 0.91)).emit_slot(frame, target);
        PaperSheet::new((-0.076, -0.036, 0.076, 0.074), 0.0170, Vec3::new(0.11, 0.14, 0.21)).emit_slot(frame, target);

        let caption = if self.batch_yield > 1 {
            format!("{} X{}", self.title, self.batch_yield)
        } else {
            self.title.to_string()
        };
        PixelStamp::new(0.0017, 0.0020, Vec3::new(0.10, 0.13, 0.20))
            .emit_on_frame(&caption, 0.0, -0.062, 0.0160, frame, target);
    }
}

/// Decorator over `PolaroidCard` showcasing the hero 3D `RelicFigure` and sculpting an amber thumbtack pin.
#[derive(Clone, Copy)]
pub struct PinnedPolaroid<P> {
    origin: P,
    figure: RelicFigure,
}

impl<P> PinnedPolaroid<P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: P, figure: RelicFigure) -> Self {
        Self { origin, figure }
    }
}

impl<P: SlotPlaque> SlotPlaque for PinnedPolaroid<P> {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        self.origin.emit_slot(frame, target);
        let stage = frame.shifted(0.0, 0.020, -0.006).scaled(1.12);
        self.figure.emit_showcase(&stage, target);

        let tack_axis = frame.shifted(0.0, 0.081, 0.0175).pitched(-std::f32::consts::FRAC_PI_2);
        TaperedPrism::hexagonal(
            Vec3::ZERO,
            Vec3::new(0.0, 0.008, 0.0),
            0.003,
            0.008,
            Vec3::new(0.78, 0.42, 0.16),
        )
        .emit(&tack_axis, true, target);
        TaperedPrism::hexagonal(
            Vec3::new(0.0, 0.008, 0.0),
            Vec3::new(0.0, 0.013, 0.0),
            0.009,
            0.007,
            Vec3::new(0.95, 0.56, 0.20),
        )
        .emit(&tack_axis, true, target);
    }
}

/// Right-page visual formula (`[Ing 1] + [Ing 2] -> [Product]`) paired with weight grade and `CRAFT` stamp.
#[derive(Clone, Copy)]
pub struct IngredientFormula<'a> {
    entry: CraftingEntry,
    satchel: &'a BeltSatchel,
    attire: YouthAttire,
}

impl<'a> IngredientFormula<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(entry: CraftingEntry, satchel: &'a BeltSatchel, attire: YouthAttire) -> Self {
        Self {
            entry,
            satchel,
            attire,
        }
    }

    fn emit_mini_ingredient(&self, relic: HeldRelic, needed: u32, x: f32, row: &LimbFrame, target: &mut dyn MeshTarget) {
        let owned = self.satchel.resource_tally(relic, self.attire);
        let enough = owned >= needed;
        let rim = if enough { Vec3::new(0.28, 0.72, 0.42) } else { Vec3::new(0.78, 0.34, 0.24) };
        let cell = row.shifted(x, 0.008, 0.0).scaled(0.46);
        relic.figure(BlockMold::cube()).emit_in_slot(enough, rim, &cell, target);

        let qty_col = if enough { Vec3::new(0.84, 1.56, 0.92) } else { Vec3::new(1.58, 0.84, 0.68) };
        PixelStamp::new(0.0017, 0.0018, qty_col)
            .emit_on_frame(&format!("{owned}/{needed}"), x, -0.022, 0.0155, row, target);
        PixelStamp::new(0.0015, 0.0016, Vec3::new(1.36, 1.40, 1.48))
            .emit_on_frame(relic.short_title(), x, -0.036, 0.0155, row, target);
    }
}

impl<'a> SlotPlaque for IngredientFormula<'a> {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let ings = self.entry.ingredients();
        let ready = self.entry.craftable(self.satchel, self.attire);
        let eq = frame.shifted(0.0, -0.068, -0.004);
        let op_stamp = PixelStamp::new(0.0022, 0.0022, Vec3::new(1.48, 1.46, 1.38));

        if ings[1].1 > 0 {
            self.emit_mini_ingredient(ings[0].0, ings[0].1, -0.088, &eq, target);
            op_stamp.emit_on_frame("+", -0.044, 0.008, 0.0155, &eq, target);
            self.emit_mini_ingredient(ings[1].0, ings[1].1, 0.0, &eq, target);
            op_stamp.emit_on_frame("->", 0.046, 0.008, 0.0155, &eq, target);
        } else {
            self.emit_mini_ingredient(ings[0].0, ings[0].1, -0.048, &eq, target);
            op_stamp.emit_on_frame("->", 0.018, 0.008, 0.0155, &eq, target);
        }
        let out_x = if ings[1].1 > 0 { 0.090 } else { 0.072 };
        let out_rim = if ready { Vec3::new(0.95, 0.78, 0.24) } else { Vec3::new(0.42, 0.46, 0.56) };
        let out_cell = eq.shifted(out_x, 0.008, 0.0).scaled(0.46);
        self.entry.product().figure(BlockMold::cube()).emit_in_slot(ready, out_rim, &out_cell, target);
        PixelStamp::new(0.0017, 0.0018, Vec3::new(1.56, 1.46, 0.88))
            .emit_on_frame(&format!("X{}", self.entry.batch_yield()), out_x, -0.022, 0.0155, &eq, target);

        let bar = frame.shifted(0.0, -0.150, -0.006);
        SlotBurden::new(self.entry.product(), self.entry.batch_yield())
            .emit_grade_badge(&bar.shifted(-0.060, 0.0, 0.0), target);

        let btn = bar.shifted(0.062, 0.0, 0.0);
        let (btn_bg, btn_rim, btn_txt, label) = if ready {
            (
                Vec3::new(0.86, 0.62, 0.16),
                Vec3::new(1.00, 0.84, 0.34),
                Vec3::new(0.09, 0.11, 0.16),
                "CRAFT",
            )
        } else {
            (
                Vec3::new(0.14, 0.17, 0.24),
                Vec3::new(0.36, 0.32, 0.28),
                Vec3::new(0.96, 0.82, 0.72),
                "LOCKED",
            )
        };
        btn.emit_framed_plaque(0.054, 0.015, 0.0020, btn_bg, btn_rim, target);
        PixelStamp::new(0.0021, 0.0022, btn_txt).emit_on_frame(label, 0.0, 0.0, 0.0155, &btn, target);
    }
}

/// Null-Object plaque rendered cleanly on the right page when `[X] READY TO CRAFT` yields zero items in a tab.
#[derive(Clone, Copy)]
pub struct VacantScopePlaque {
    tab: FolioTab,
}

impl VacantScopePlaque {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tab: FolioTab) -> Self {
        Self { tab }
    }
}

impl SlotPlaque for VacantScopePlaque {
    fn emit_slot(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let card = frame.shifted(0.0, 0.018, -0.006);
        card.emit_framed_plaque(0.118, 0.068, 0.0022, Vec3::new(0.10, 0.13, 0.20), self.tab.pigment(), target);
        PixelStamp::new(0.0020, 0.0021, Vec3::new(1.52, 1.34, 0.64))
            .emit_on_frame("NO READY ITEMS", 0.0, 0.022, 0.016, &card, target);
        PixelStamp::new(0.0017, 0.0018, Vec3::new(1.28, 1.36, 1.48))
            .emit_on_frame(&format!("SECTION: {}", self.tab.label()), 0.0, 0.0, 0.016, &card, target);
        PixelStamp::new(0.0016, 0.0017, Vec3::new(0.92, 1.42, 1.08))
            .emit_on_frame("PRESS F FOR FULL LIST", 0.0, -0.024, 0.016, &card, target);
    }
}
