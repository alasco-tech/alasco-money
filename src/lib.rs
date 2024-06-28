use pyo3::prelude::*;

mod decimals;
mod money;
mod money_vat;
mod money_vat_ratio;

#[pymodule]
mod alasco_money {
    use super::*;
    use env_logger;

    #[pymodule_export]
    use crate::money::Money;

    #[pymodule_export]
    use crate::money_vat::MoneyWithVAT;

    #[pymodule_export]
    use crate::money_vat_ratio::MoneyWithVATRatio;

    #[pymodule_export]
    use crate::money::sum_;

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        env_logger::init(); // Initialize logging
        Ok(())
    }
}
