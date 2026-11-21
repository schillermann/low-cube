use glam::Vec3;
use crate::MeshTarget;
use super::SlatePosture;

/// Recessed cobalt-sapphire slate display showing glowing cyan rover and rocket schematics.
pub(super) struct GuildSchematic {
    slate: SlatePosture,
}

impl GuildSchematic {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(slate: SlatePosture) -> Self {
        Self { slate }
    }

    pub(super) fn emit(&self, target: &mut dyn MeshTarget) {
        let cobalt_glass = Vec3::new(0.07, 0.24, 0.56);
        let grid_cyan = Vec3::new(0.20, 0.58, 0.90);
        let bright_cyan = Vec3::new(0.74, 0.97, 1.00);

        self.slate.emit_glow_rect([-0.184, 0.184, 0.056, 0.332], 0.0325, cobalt_glass, 0.36, target);
        self.emit_blueprint_grid(grid_cyan, target);
        self.emit_rover_elevation(bright_cyan, target);
        self.emit_rocket_profile(bright_cyan, target);
        self.emit_guild_cartouche(bright_cyan, target);
    }

    fn emit_line(&self, rect: [f32; 4], col: Vec3, glow: f32, target: &mut dyn MeshTarget) {
        self.slate.emit_glow_rect(rect, 0.0385, col, glow, target);
    }

    fn emit_hollow_box(&self, rect: [f32; 4], stroke: f32, col: Vec3, glow: f32, target: &mut dyn MeshTarget) {
        let [x0, x1, s0, s1] = rect;
        self.emit_line([x0, x1, s0, s0 + stroke], col, glow, target);
        self.emit_line([x0, x1, s1 - stroke, s1], col, glow, target);
        self.emit_line([x0, x0 + stroke, s0, s1], col, glow, target);
        self.emit_line([x1 - stroke, x1, s0, s1], col, glow, target);
    }

    fn emit_blueprint_grid(&self, grid_col: Vec3, target: &mut dyn MeshTarget) {
        let w_grid = 0.0355;
        for border in [
            [-0.174, 0.174, 0.066, 0.072],
            [-0.174, 0.174, 0.316, 0.322],
            [-0.174, -0.168, 0.066, 0.322],
            [0.168, 0.174, 0.066, 0.322],
        ] {
            self.slate.emit_glow_rect(border, w_grid, grid_col, 0.54, target);
        }
        for gx in [-0.088f32, 0.0f32, 0.088f32] {
            self.slate.emit_glow_rect([gx - 0.0025, gx + 0.0025, 0.072, 0.316], w_grid, grid_col, 0.44, target);
        }
        for gs in [0.148f32, 0.232f32] {
            self.slate.emit_glow_rect([-0.168, 0.168, gs - 0.0025, gs + 0.0025], w_grid, grid_col, 0.44, target);
        }
    }

    fn emit_rover_elevation(&self, cyan: Vec3, target: &mut dyn MeshTarget) {
        let st = 0.0065;
        self.emit_hollow_box([-0.146, 0.038, 0.144, 0.194], st, cyan, 0.90, target);
        self.emit_hollow_box([-0.114, 0.002, 0.190, 0.248], st, cyan, 0.90, target);
        self.emit_line([-0.056, -0.050, 0.194, 0.244], cyan, 0.88, target);

        self.emit_line([-0.026, -0.020, 0.248, 0.288], cyan, 0.90, target);
        self.emit_line([-0.040, -0.006, 0.286, 0.294], cyan, 0.92, target);

        for wx in [-0.104f32, -0.004f32] {
            self.emit_hollow_box([wx - 0.026, wx + 0.026, 0.108, 0.158], st, cyan, 0.92, target);
            self.emit_line([wx - 0.008, wx + 0.008, 0.126, 0.140], cyan, 0.95, target);
        }
    }

    fn emit_rocket_profile(&self, cyan: Vec3, target: &mut dyn MeshTarget) {
        let st = 0.0060;
        self.emit_hollow_box([0.094, 0.134, 0.164, 0.258], st, cyan, 0.90, target);
        self.emit_line([0.094, 0.134, 0.208, 0.214], cyan, 0.86, target);

        self.emit_hollow_box([0.102, 0.126, 0.256, 0.284], st, cyan, 0.90, target);
        self.emit_line([0.110, 0.118, 0.284, 0.304], cyan, 0.94, target);

        self.emit_line([0.078, 0.096, 0.152, 0.184], cyan, 0.88, target);
        self.emit_line([0.132, 0.150, 0.152, 0.184], cyan, 0.88, target);
        self.emit_line([0.102, 0.126, 0.144, 0.164], cyan, 0.92, target);
    }

    fn emit_guild_cartouche(&self, cyan: Vec3, target: &mut dyn MeshTarget) {
        self.emit_hollow_box([0.062, 0.158, 0.080, 0.116], 0.005, cyan, 0.84, target);
        self.emit_line([0.074, 0.146, 0.095, 0.101], cyan, 0.82, target);

        self.emit_line([0.032, 0.038, 0.082, 0.114], cyan, 0.88, target);
        self.emit_line([0.019, 0.051, 0.095, 0.101], cyan, 0.88, target);

        for tx in [-0.154f32, -0.126f32, -0.098f32, -0.070f32] {
            self.emit_line([tx, tx + 0.014, 0.082, 0.094], cyan, 0.82, target);
        }
    }
}
