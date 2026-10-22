use super::timber_yield::TimberYield;

/// Behavioral destination receiving harvested forestry yield.
pub trait TimberDepository {
    fn deposit_timber(&mut self, yield_kind: TimberYield, quantity: u32);
}

/// Forest floor depository absorbing timber harvests during surface felling.
pub struct WoodlandDepository;

impl WoodlandDepository {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

impl TimberDepository for WoodlandDepository {
    fn deposit_timber(&mut self, _yield_kind: TimberYield, _quantity: u32) {}
}
