use glam::Vec3;
use world::MeshTarget;
use crate::menu::PixelStamp;
use super::super::limb::{AxialLimb, FacetedSolid, LimbFrame};
use super::super::pouch::BeltSatchel;
use super::super::YouthAttire;
use super::domain::{FolioSection, FolioTab, StockScope};
use super::shelf::{CanonicalShelf, ReadyStockShelf, RecipeShelf, TabbedShelf};

/// Composable 3D tactile field-folio spread contract decorated by spine rings, index tabs, grid, and Polaroid layers.
pub trait FolioSpread {
    fn emit_spread(&self, frame: &LimbFrame, target: &mut dyn MeshTarget);
}

/// Dark-slate stitched folio cover and dual-page parchment-slate spread.
#[derive(Clone, Copy)]
pub struct SlateFolioCovers {
    accent: Vec3,
}

impl SlateFolioCovers {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(accent: Vec3) -> Self {
        Self { accent }
    }
}

impl FolioSpread for SlateFolioCovers {
    fn emit_spread(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let cover = frame.shifted(0.0, 0.0, 0.010);
        let rim = (self.accent * 0.55 + Vec3::new(0.20, 0.24, 0.32)).clamp(Vec3::ZERO, Vec3::ONE);
        cover.emit_framed_plaque(0.324, 0.218, 0.0036, Vec3::new(0.06, 0.08, 0.12), rim, target);

        let left_page = frame.shifted(-0.162, 0.006, 0.002);
        left_page.emit_framed_plaque(0.146, 0.194, 0.0020, Vec3::new(0.10, 0.13, 0.19), Vec3::new(0.20, 0.25, 0.34), target);

        let right_page = frame.shifted(0.162, 0.006, 0.002);
        right_page.emit_framed_plaque(0.146, 0.194, 0.0020, Vec3::new(0.11, 0.14, 0.21), Vec3::new(0.22, 0.27, 0.36), target);
    }
}

/// Decorator sculpting the recessed central spine groove and four metallic 3D ring-binder coils.
#[derive(Clone, Copy)]
pub struct RingSpineFolio<S> {
    origin: S,
}

impl<S> RingSpineFolio<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S) -> Self {
        Self { origin }
    }
}

impl<S: FolioSpread> FolioSpread for RingSpineFolio<S> {
    fn emit_spread(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        self.origin.emit_spread(frame, target);
        let spine = frame.shifted(0.0, 0.006, -0.001);
        spine.emit_framed_plaque(0.010, 0.196, 0.0016, Vec3::new(0.05, 0.06, 0.09), Vec3::new(0.16, 0.20, 0.28), target);

        let steel = Vec3::new(0.68, 0.72, 0.78);
        let gleam = Vec3::new(0.86, 0.90, 0.96);
        let eyelet = Vec3::new(0.04, 0.05, 0.08);
        for ry in [0.132, 0.048, -0.036, -0.120] {
            AxialLimb::new(Vec3::new(-0.025, ry, 0.017), Vec3::new(-0.021, ry, 0.013), 0.0042, 0.0042, eyelet).emit(frame, true, target);
            AxialLimb::new(Vec3::new(0.021, ry, 0.013), Vec3::new(0.025, ry, 0.017), 0.0042, 0.0042, eyelet).emit(frame, true, target);
            AxialLimb::new(Vec3::new(-0.023, ry, 0.015), Vec3::new(-0.009, ry + 0.003, 0.004), 0.0030, 0.0032, steel).emit(frame, true, target);
            AxialLimb::new(Vec3::new(-0.009, ry + 0.003, 0.004), Vec3::new(0.009, ry + 0.003, 0.004), 0.0032, 0.0032, gleam).emit(frame, true, target);
            AxialLimb::new(Vec3::new(0.009, ry + 0.003, 0.004), Vec3::new(0.023, ry, 0.015), 0.0032, 0.0030, steel).emit(frame, true, target);
        }
    }
}

/// Decorator rendering the four protruding color-coded side index tabs (`TOOLS`, `BLOCKS`, `ROVER`, `ORBITAL`).
#[derive(Clone, Copy)]
pub struct IndexTabbedFolio<'a, S> {
    origin: S,
    section: FolioSection,
    satchel: &'a BeltSatchel,
    attire: YouthAttire,
}

impl<'a, S> IndexTabbedFolio<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_section(
        origin: S,
        section: FolioSection,
        satchel: &'a BeltSatchel,
        attire: YouthAttire,
    ) -> Self {
        Self {
            origin,
            section,
            satchel,
            attire,
        }
    }

    pub const fn new(
        origin: S,
        focused_tab: FolioTab,
        satchel: &'a BeltSatchel,
        attire: YouthAttire,
    ) -> Self {
        Self::with_section(origin, FolioSection::field(focused_tab), satchel, attire)
    }

    fn ready_in_tab(&self, tab: FolioTab) -> bool {
        let shelf = ReadyStockShelf::at_venue(
            TabbedShelf::new(CanonicalShelf::catalog(), tab),
            (StockScope::ReadyStock, self.section.venue()),
            self.satchel,
            self.attire,
        );
        let mut ready = Vec::with_capacity(4);
        shelf.collect_recipes(&mut ready);
        !ready.is_empty()
    }
}

impl<'a, S: FolioSpread> FolioSpread for IndexTabbedFolio<'a, S> {
    fn emit_spread(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        self.origin.emit_spread(frame, target);
        for (idx, tab) in FolioTab::all().into_iter().enumerate() {
            let selected = tab == self.section.tab();
            let ready = self.ready_in_tab(tab);
            let ty = 0.132 - (idx as f32) * 0.084;
            let tx = if selected { 0.340 } else { 0.332 };
            let tz = if selected { -0.003 } else { 0.006 };
            let pigment = tab.pigment();
            let fill = if selected { pigment * 0.92 } else { pigment * 0.56 };
            let rim = if selected {
                Vec3::new(1.00, 0.88, 0.36)
            } else if ready {
                (pigment * 1.15).clamp(Vec3::ZERO, Vec3::splat(1.2))
            } else {
                pigment * 0.72
            };
            let tab_frame = frame.shifted(tx, ty, tz);
            tab_frame.emit_framed_plaque(0.032, 0.034, 0.0020, fill, rim, target);
            let txt_col = if selected { Vec3::new(0.07, 0.09, 0.13) } else { Vec3::new(1.52, 1.50, 1.44) };
            PixelStamp::new(0.0015, 0.0018, txt_col)
                .emit_on_frame(tab.badge_label(), 0.004, 0.0, 0.0155, &tab_frame, target);
        }
    }
}
