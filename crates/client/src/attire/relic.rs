use glam::Vec3;
use world::{BlockMold, MeshTarget, Quad};
use crate::menu::PixelStamp;
use super::limb::{FacetedSolid, LimbFrame, TaperedPrism};

struct LitSlotTarget<'a> {
    light_dir: Vec3,
    zenith: Vec3,
    inner: &'a mut dyn MeshTarget,
}

impl<'a> MeshTarget for LitSlotTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let lit = quad.transfer(|v0, v1, v2, v3, n, col| {
            let shade = 1.30 + 0.45 * n.dot(self.light_dir).max(0.0);
            Quad::new(v0, v1, v2, v3, self.zenith, col * shade + Vec3::splat(0.07))
        });
        self.inner.push_quad(lit);
    }
}

/// Polymorphic martial action style bound to the held relic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MartialStyle { Unarmed, PalmCradle, LumberjackCleave, SwordplaySlash, SpadeDig }

/// Object held in the avatar's right or left hand visible in first-person and multiplayer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldRelic {
    EmptyPalm, QuarriedStone, TimberTorch, WoodenMallet, WoodmanAxe, KnightlySword, IronSpade, ExcavatedEarth,
}

impl HeldRelic {
    #[cfg(test)]
    pub fn next(self) -> Self {
        match self {
            Self::EmptyPalm => Self::QuarriedStone, Self::QuarriedStone => Self::TimberTorch,
            Self::TimberTorch => Self::WoodenMallet, Self::WoodenMallet => Self::WoodmanAxe,
            Self::WoodmanAxe => Self::KnightlySword, Self::KnightlySword => Self::IronSpade,
            Self::IronSpade => Self::ExcavatedEarth, Self::ExcavatedEarth => Self::EmptyPalm,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 8 {
            0 => Self::EmptyPalm, 1 => Self::QuarriedStone, 2 => Self::TimberTorch, 3 => Self::WoodenMallet,
            4 => Self::WoodmanAxe, 5 => Self::KnightlySword, 6 => Self::IronSpade, _ => Self::ExcavatedEarth,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::EmptyPalm => 0, Self::QuarriedStone => 1, Self::TimberTorch => 2, Self::WoodenMallet => 3,
            Self::WoodmanAxe => 4, Self::KnightlySword => 5, Self::IronSpade => 6, Self::ExcavatedEarth => 7,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::EmptyPalm => "EMPTY SATCHEL", Self::QuarriedStone => "STONE BLOCK",
            Self::TimberTorch => "TIMBER TORCH", Self::WoodenMallet => "WOODEN MALLET",
            Self::WoodmanAxe => "WOODMAN AXE", Self::KnightlySword => "KNIGHT SWORD",
            Self::IronSpade => "IRON SPADE", Self::ExcavatedEarth => "EARTH BLOCK",
        }
    }

    pub fn gripped(self) -> bool { !matches!(self, Self::EmptyPalm) }
    pub fn free_palm(self) -> bool { matches!(self, Self::EmptyPalm) }
    pub fn spade_blade(self) -> bool { matches!(self, Self::IronSpade) }
    pub fn placeable_block(self) -> bool { matches!(self, Self::ExcavatedEarth | Self::QuarriedStone) }

    pub fn martial_style(self) -> MartialStyle {
        match self {
            Self::EmptyPalm => MartialStyle::Unarmed,
            Self::ExcavatedEarth | Self::QuarriedStone => MartialStyle::PalmCradle,
            Self::IronSpade => MartialStyle::SpadeDig,
            Self::WoodenMallet | Self::WoodmanAxe => MartialStyle::LumberjackCleave,
            Self::TimberTorch | Self::KnightlySword => MartialStyle::SwordplaySlash,
        }
    }

    /// Emits the relic in the palm, sculpting placeable blocks resting inside the upturned hollow palm.
    pub fn emit_molded_in_palm(self, side: f32, mold: BlockMold, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (gx, gz) = (-side * 0.025, 0.054);
        match self {
            Self::EmptyPalm => {}
            Self::QuarriedStone => {
                frame.emit_cradled_block(side, Vec3::new(-side * 0.015, 0.001, 0.048), mold, 0.096, Vec3::new(0.50, 0.51, 0.53), Vec3::new(0.62, 0.64, 0.66), target);
            }
            Self::TimberTorch => {
                TaperedPrism::hexagonal(Vec3::new(gx, -0.10, gz), Vec3::new(gx, 0.18, gz + 0.01), 0.013, 0.017, Vec3::new(0.45, 0.28, 0.14))
                    .emit(frame, false, target);
                TaperedPrism::hexagonal(Vec3::new(gx, 0.18, gz + 0.01), Vec3::new(gx, 0.25, gz + 0.015), 0.022, 0.008, Vec3::new(0.96, 0.52, 0.12))
                    .emit(frame, true, target);
            }
            Self::WoodenMallet => {
                TaperedPrism::hexagonal(Vec3::new(gx, -0.10, gz), Vec3::new(gx, 0.18, gz), 0.013, 0.014, Vec3::new(0.48, 0.31, 0.16))
                    .emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(gx, 0.14, gz - 0.04), Vec3::new(gx, 0.20, gz + 0.05), 0.032, 0.052, 0.032, 0.052, Vec3::new(0.36, 0.24, 0.13))
                    .emit(frame, true, target);
            }
            Self::WoodmanAxe => {
                let (ash_wood, worn_ash, forged_iron, blade_steel, bright_edge) = (
                    Vec3::new(0.64, 0.45, 0.25), Vec3::new(0.52, 0.35, 0.19), Vec3::new(0.42, 0.46, 0.52),
                    Vec3::new(0.68, 0.73, 0.79), Vec3::new(0.92, 0.95, 0.99),
                );
                TaperedPrism::hexagonal(Vec3::new(gx, -0.155, gz - 0.008), Vec3::new(gx, -0.115, gz), 0.016, 0.012, worn_ash).emit(frame, true, target);
                TaperedPrism::hexagonal(Vec3::new(gx, -0.115, gz), Vec3::new(gx, 0.045, gz), 0.012, 0.013, worn_ash).emit(frame, false, target);
                TaperedPrism::hexagonal(Vec3::new(gx, 0.045, gz), Vec3::new(gx, 0.252, gz + 0.004), 0.013, 0.015, ash_wood).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(gx, 0.184, gz - 0.004), Vec3::new(gx, 0.244, gz - 0.002), 0.017, 0.025, 0.018, 0.026, forged_iron).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.194, gz + 0.032), Vec3::new(gx, 0.242, gz + 0.034), 0.013, 0.024, 0.014, 0.026, forged_iron).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.128, gz + 0.064), Vec3::new(gx, 0.194, gz + 0.062), 0.008, 0.020, 0.011, 0.026, blade_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.194, gz + 0.062), Vec3::new(gx, 0.254, gz + 0.064), 0.011, 0.026, 0.010, 0.024, blade_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.124, gz + 0.084), Vec3::new(gx, 0.172, gz + 0.094), 0.003, 0.010, 0.004, 0.014, bright_edge).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.172, gz + 0.094), Vec3::new(gx, 0.218, gz + 0.096), 0.004, 0.014, 0.004, 0.014, bright_edge).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.218, gz + 0.096), Vec3::new(gx, 0.258, gz + 0.086), 0.004, 0.014, 0.003, 0.010, bright_edge).emit(frame, true, target);
            }
            Self::KnightlySword => {
                let (grip_leather, guard_iron, fuller_steel, honed_edge) = (
                    Vec3::new(0.26, 0.15, 0.09), Vec3::new(0.32, 0.35, 0.40), Vec3::new(0.56, 0.60, 0.66), Vec3::new(0.84, 0.88, 0.94),
                );
                // Wheel pommel (Scheibenknauf) and leather-bound grip seated inside the closed palm
                TaperedPrism::new(8, Vec3::new(gx, -0.106, gz), Vec3::new(gx, -0.068, gz), 0.015, 0.024, 0.015, 0.024, guard_iron).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(gx, -0.068, gz), Vec3::new(gx, 0.048, gz), 0.011, 0.015, 0.012, 0.016, grip_leather).emit(frame, false, target);
                // Cruciform crossguard (Parierstange) extending across the plane of the double-edged blade
                TaperedPrism::new(6, Vec3::new(gx, 0.046, gz), Vec3::new(gx, 0.066, gz), 0.013, 0.082, 0.012, 0.076, guard_iron).emit(frame, true, target);
                // Double-edged medieval blade with central fuller (Hohlkehle) tapering to a sharp point at y = 0.68
                TaperedPrism::new(4, Vec3::new(gx, 0.066, gz), Vec3::new(gx, 0.420, gz), 0.005, 0.025, 0.004, 0.020, honed_edge).emit(frame, false, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.068, gz), Vec3::new(gx, 0.390, gz), 0.006, 0.011, 0.005, 0.008, fuller_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.420, gz), Vec3::new(gx, 0.680, gz), 0.004, 0.020, 0.001, 0.002, honed_edge).emit(frame, true, target);
            }
            Self::IronSpade => {
                let (ash_wood, worn_ash, socket_iron, spade_steel, honed_edge) = (
                    Vec3::new(0.54, 0.37, 0.20), Vec3::new(0.42, 0.28, 0.15),
                    Vec3::new(0.30, 0.33, 0.38), Vec3::new(0.54, 0.58, 0.64), Vec3::new(0.84, 0.88, 0.93),
                );
                TaperedPrism::new(6, Vec3::new(gx, -0.164, gz), Vec3::new(gx, -0.144, gz), 0.012, 0.034, 0.012, 0.034, worn_ash).emit(frame, true, target);
                TaperedPrism::hexagonal(Vec3::new(gx, -0.150, gz), Vec3::new(gx, 0.190, gz), 0.012, 0.014, ash_wood).emit(frame, false, target);
                TaperedPrism::hexagonal(Vec3::new(gx, 0.185, gz), Vec3::new(gx, 0.235, gz + 0.004), 0.016, 0.020, socket_iron).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.230, gz + 0.008), Vec3::new(gx, 0.345, gz + 0.016), 0.007, 0.052, 0.006, 0.046, spade_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.345, gz + 0.016), Vec3::new(gx, 0.395, gz + 0.022), 0.006, 0.046, 0.002, 0.018, honed_edge).emit(frame, true, target);
            }
            Self::ExcavatedEarth => {
                frame.emit_cradled_block(side, Vec3::new(-side * 0.015, 0.001, 0.048), mold, 0.096, Vec3::new(0.44, 0.31, 0.19), Vec3::new(0.52, 0.38, 0.24), target);
            }
        }
    }

    /// Emits a background-filled 4-bar slot frame with the 3D molded block or tool showcased in front.
    pub fn emit_in_satchel_slot(self, mold: BlockMold, rim: Vec3, slot_frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let bg = if rim.x > 0.7 { Vec3::new(0.28, 0.33, 0.44) } else { Vec3::new(0.16, 0.19, 0.27) };
        slot_frame.emit_framed_plaque(0.044, 0.044, 0.0032, bg, rim * 1.25, target);
        let light_dir = (slot_frame.point(0.4, 0.6, -0.7) - slot_frame.point(0.0, 0.0, 0.0)).normalize_or_zero();
        let zenith = (slot_frame.point(0.0, 1.0, -1.0) - slot_frame.point(0.0, 0.0, 0.0)).normalize_or_zero();
        let mut lit = LitSlotTarget { light_dir, zenith, inner: target };
        let showcase = slot_frame.shifted(0.0, 0.0, -0.048).pitched(-0.46).yawed(0.64);
        match self {
            Self::EmptyPalm => {}
            Self::ExcavatedEarth => {
                showcase.emit_upright_block(Vec3::new(0.0, -0.025, 0.0), mold, 0.082, Vec3::new(0.44, 0.31, 0.19), Vec3::new(0.56, 0.41, 0.25), &mut lit);
            }
            Self::QuarriedStone => {
                showcase.emit_upright_block(Vec3::new(0.0, -0.025, 0.0), mold, 0.082, Vec3::new(0.50, 0.51, 0.53), Vec3::new(0.66, 0.68, 0.70), &mut lit);
            }
            _ => {
                let (scale, yaw, mid_y, mid_z) = match self {
                    Self::KnightlySword => (0.090, 0.45, 0.285, 0.054),
                    Self::IronSpade => (0.125, 0.45, 0.115, 0.062),
                    Self::WoodmanAxe => (0.165, 1.32, 0.052, 0.092),
                    Self::TimberTorch => (0.185, 0.45, 0.075, 0.058),
                    _ => (0.205, 0.85, 0.050, 0.054),
                };
                let tool_frame = slot_frame.shifted(0.0, 0.0, -0.035).pitched(-0.22).yawed(yaw).scaled(scale).shifted(0.025, -mid_y, -mid_z);
                self.emit_molded_in_palm(1.0, mold, &tool_frame, &mut lit);
            }
        }
    }

    /// Emits the framed inventory legend plaque underneath the satchel slots.
    pub fn emit_satchel_legend(self, legend_frame: &LimbFrame, target: &mut dyn MeshTarget) {
        legend_frame.emit_framed_plaque(0.168, 0.032, 0.0024, Vec3::new(0.08, 0.10, 0.15), Vec3::new(0.42, 0.27, 0.16), target);
        PixelStamp::new(0.0024, 0.0024, Vec3::new(1.55, 1.30, 0.48)).emit_on_frame(self.title(), 0.0, 0.017, 0.016, legend_frame, target);
        let hint = PixelStamp::new(0.0022, 0.0022, Vec3::new(1.42, 1.46, 1.54));
        hint.emit_on_frame("WASD: NAVIGATION  ESC: CLOSE", 0.0, 0.0, 0.016, legend_frame, target);
        hint.emit_on_frame("Q: LEFT HAND  E: RIGHT HAND", 0.0, -0.017, 0.016, legend_frame, target);
    }
}
