use super::timber_yield::TimberYield;

/// Behavioral destination receiving harvested forestry yield.
pub trait TimberDepository {
    fn deposit_timber(&mut self, yield_kind: TimberYield, quantity: u32);
}
