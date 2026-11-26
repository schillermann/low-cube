use crate::menu::widget::ScreenBounds;

/// Normalized screen bounds for the `MY WORLDS` roster and `CREATE WORLD` configurator panels.
#[derive(Clone, Copy, Debug)]
pub struct WorldLayout;

impl WorldLayout {
    pub const PAGE_SIZE: usize = 4;

    /// Primary constructor initializing the zero-state layout bounds.
    pub const fn new() -> Self {
        Self
    }

    pub const fn card(&self) -> ScreenBounds {
        ScreenBounds::new(-0.68, -0.74, 0.68, 0.64)
    }

    pub fn slot_row(&self, slot: usize) -> ScreenBounds {
        let top = 0.40 - (slot as f32) * 0.15;
        ScreenBounds::new(-0.60, top - 0.12, 0.60, top)
    }

    pub fn prev_page_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.60, -0.31, -0.32, -0.21)
    }

    pub fn next_page_button(&self) -> ScreenBounds {
        ScreenBounds::new(0.32, -0.31, 0.60, -0.21)
    }

    pub fn play_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.60, -0.48, -0.02, -0.35)
    }

    pub fn new_world_button(&self) -> ScreenBounds {
        ScreenBounds::new(0.02, -0.48, 0.60, -0.35)
    }

    pub fn delete_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.60, -0.64, -0.02, -0.52)
    }

    pub fn roster_back_button(&self) -> ScreenBounds {
        ScreenBounds::new(0.02, -0.64, 0.60, -0.52)
    }

    pub fn name_field(&self) -> ScreenBounds {
        ScreenBounds::new(-0.60, 0.25, 0.34, 0.37)
    }

    pub fn clear_name_button(&self) -> ScreenBounds {
        ScreenBounds::new(0.38, 0.25, 0.60, 0.37)
    }

    pub fn size_prev_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.60, -0.01, -0.42, 0.11)
    }

    pub fn size_readout(&self) -> ScreenBounds {
        ScreenBounds::new(-0.38, -0.01, 0.38, 0.11)
    }

    pub fn size_next_button(&self) -> ScreenBounds {
        ScreenBounds::new(0.42, -0.01, 0.60, 0.11)
    }

    pub fn seed_field(&self) -> ScreenBounds {
        ScreenBounds::new(-0.60, -0.27, 0.26, -0.15)
    }

    pub fn random_seed_button(&self) -> ScreenBounds {
        ScreenBounds::new(0.30, -0.27, 0.60, -0.15)
    }

    pub fn create_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.60, -0.47, 0.60, -0.33)
    }

    pub fn create_back_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.34, -0.64, 0.34, -0.52)
    }
}
