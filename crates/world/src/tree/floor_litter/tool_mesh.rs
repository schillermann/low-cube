use glam::Vec3;
use crate::MeshTarget;
use super::super::timber_yield::SurfaceFind;
use super::FloorLitter;

/// Sculpted 3D resting figure for handheld tools cast onto the planetary surface.
pub(super) struct GroundToolFigure<'a> {
    litter: &'a FloorLitter,
    yaw: f32,
    find: SurfaceFind,
}

impl<'a> GroundToolFigure<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(litter: &'a FloorLitter, yaw: f32, find: SurfaceFind) -> Self {
        Self { litter, yaw, find }
    }

    fn rot(&self, along: f32, across: f32, h: f32) -> (f32, f32, f32) {
        let (s, c) = self.yaw.sin_cos();
        (along * c - across * s, along * s + across * c, h)
    }

    fn bar(
        &self,
        a: (f32, f32, f32),
        b: (f32, f32, f32),
        r0: f32,
        r1: f32,
        col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let p0 = self.rot(a.0, a.1, a.2);
        let p1 = self.rot(b.0, b.1, b.2);
        self.litter.emit_bough(p0, p1, r0, r1, col, col * 0.92, target);
    }

    fn emit_spade(&self, haft: Vec3, collar: Vec3, blade: Vec3, target: &mut dyn MeshTarget) {
        self.bar((-0.22, -0.038, 0.022), (-0.22, 0.038, 0.022), 0.013, 0.013, haft, target);
        self.bar((-0.22, 0.0, 0.022), (0.09, 0.0, 0.022), 0.014, 0.016, haft, target);
        self.bar((0.08, 0.0, 0.023), (0.13, 0.0, 0.023), 0.019, 0.022, collar, target);
        self.bar((0.12, -0.022, 0.020), (0.24, -0.014, 0.018), 0.024, 0.012, blade, target);
        self.bar((0.12, 0.022, 0.020), (0.24, 0.014, 0.018), 0.024, 0.012, blade, target);
    }

    fn emit_pickaxe(&self, haft: Vec3, collar: Vec3, arm: Vec3, tip: Vec3, target: &mut dyn MeshTarget) {
        self.bar((-0.22, 0.0, 0.020), (0.17, 0.0, 0.024), 0.014, 0.016, haft, target);
        self.bar((0.11, -0.026, 0.028), (0.16, 0.026, 0.028), 0.022, 0.022, collar, target);
        self.bar((0.13, -0.120, 0.024), (0.13, 0.120, 0.024), 0.018, 0.018, arm, target);
        self.bar((0.09, -0.170, 0.018), (0.13, -0.110, 0.024), 0.005, 0.015, tip, target);
        self.bar((0.13, 0.110, 0.024), (0.09, 0.170, 0.018), 0.015, 0.005, tip, target);
    }

    pub(super) fn emit(&self, target: &mut dyn MeshTarget) {
        match self.find {
            SurfaceFind::TimberTorch => {
                let (wood, ember) = (Vec3::new(0.45, 0.28, 0.14), Vec3::new(0.96, 0.52, 0.12));
                self.bar((-0.18, 0.0, 0.022), (0.12, 0.0, 0.028), 0.017, 0.021, wood, target);
                self.bar((0.12, 0.0, 0.028), (0.21, 0.0, 0.044), 0.026, 0.010, ember, target);
            }
            SurfaceFind::WoodenMallet => {
                let (haft, head) = (Vec3::new(0.48, 0.31, 0.16), Vec3::new(0.36, 0.24, 0.13));
                self.bar((-0.20, 0.0, 0.024), (0.13, 0.0, 0.032), 0.016, 0.018, haft, target);
                self.bar((0.10, -0.075, 0.042), (0.10, 0.075, 0.042), 0.040, 0.040, head, target);
            }
            SurfaceFind::WoodmanAxe => {
                let (ash, iron, steel, edge) = (
                    Vec3::new(0.64, 0.45, 0.25),
                    Vec3::new(0.42, 0.46, 0.52),
                    Vec3::new(0.68, 0.73, 0.79),
                    Vec3::new(0.92, 0.95, 0.99),
                );
                self.bar((-0.22, 0.0, 0.020), (0.18, 0.0, 0.024), 0.015, 0.017, ash, target);
                self.bar((0.11, -0.01, 0.026), (0.17, 0.03, 0.026), 0.022, 0.024, iron, target);
                self.bar((0.08, 0.03, 0.024), (0.18, 0.085, 0.024), 0.018, 0.022, steel, target);
                self.bar((0.07, 0.085, 0.022), (0.19, 0.110, 0.022), 0.010, 0.010, edge, target);
            }
            SurfaceFind::KnightlySword => {
                let (leather, iron, steel) = (
                    Vec3::new(0.26, 0.15, 0.09),
                    Vec3::new(0.32, 0.35, 0.40),
                    Vec3::new(0.84, 0.88, 0.94),
                );
                self.bar((-0.26, 0.0, 0.022), (-0.23, 0.0, 0.022), 0.020, 0.020, iron, target);
                self.bar((-0.23, 0.0, 0.020), (-0.12, 0.0, 0.022), 0.015, 0.016, leather, target);
                self.bar((-0.12, -0.07, 0.024), (-0.12, 0.07, 0.024), 0.013, 0.013, iron, target);
                self.bar((-0.11, 0.0, 0.022), (0.28, 0.0, 0.018), 0.018, 0.004, steel, target);
            }
            SurfaceFind::WoodenSpade => self.emit_spade(
                Vec3::new(0.56, 0.38, 0.21),
                Vec3::new(0.44, 0.29, 0.16),
                Vec3::new(0.68, 0.49, 0.28),
                target,
            ),
            SurfaceFind::CopperSpade => self.emit_spade(
                Vec3::new(0.54, 0.37, 0.20),
                Vec3::new(0.52, 0.28, 0.15),
                Vec3::new(0.84, 0.46, 0.24),
                target,
            ),
            SurfaceFind::IronSpade => self.emit_spade(
                Vec3::new(0.54, 0.37, 0.20),
                Vec3::new(0.30, 0.33, 0.38),
                Vec3::new(0.66, 0.70, 0.76),
                target,
            ),
            SurfaceFind::TitaniumSpade => self.emit_spade(
                Vec3::new(0.24, 0.27, 0.32),
                Vec3::new(0.42, 0.52, 0.64),
                Vec3::new(0.72, 0.80, 0.90),
                target,
            ),
            SurfaceFind::BoarMeat => {
                let (bone, meat, fat_rind) = (
                    Vec3::new(0.92, 0.89, 0.80),
                    Vec3::new(0.68, 0.19, 0.16),
                    Vec3::new(0.90, 0.78, 0.68),
                );
                self.bar((-0.18, 0.0, 0.036), (-0.06, 0.0, 0.046), 0.020, 0.018, bone, target);
                self.bar((-0.07, 0.0, 0.056), (0.12, 0.0, 0.062), 0.058, 0.072, meat, target);
                self.bar((0.10, 0.0, 0.062), (0.17, 0.0, 0.052), 0.066, 0.036, fat_rind, target);
            }
            SurfaceFind::FlintPickaxe => self.emit_pickaxe(
                Vec3::new(0.46, 0.29, 0.16),
                Vec3::new(0.70, 0.56, 0.34),
                Vec3::new(0.24, 0.27, 0.34),
                Vec3::new(0.76, 0.82, 0.90),
                target,
            ),
            SurfaceFind::CopperPickaxe => self.emit_pickaxe(
                Vec3::new(0.58, 0.40, 0.22),
                Vec3::new(0.52, 0.28, 0.15),
                Vec3::new(0.84, 0.46, 0.23),
                Vec3::new(0.98, 0.70, 0.45),
                target,
            ),
            SurfaceFind::IronPickaxe => self.emit_pickaxe(
                Vec3::new(0.58, 0.40, 0.22),
                Vec3::new(0.32, 0.36, 0.42),
                Vec3::new(0.64, 0.69, 0.76),
                Vec3::new(0.90, 0.94, 0.99),
                target,
            ),
            SurfaceFind::TitaniumPickaxe => self.emit_pickaxe(
                Vec3::new(0.24, 0.27, 0.32),
                Vec3::new(0.40, 0.48, 0.58),
                Vec3::new(0.68, 0.76, 0.86),
                Vec3::new(0.94, 0.98, 1.00),
                target,
            ),
            _ => {}
        }
    }
}
