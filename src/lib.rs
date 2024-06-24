mod money;
mod money_vat;
mod money_vat_ratio;
mod traits;

use env_logger;
use pyo3::prelude::*;

use crate::money::sum_;
use crate::money::Money;
use crate::money_vat::MoneyWithVAT;
use crate::money_vat_ratio::MoneyWithVATRatio;

#[pymodule]
fn alasco_money(m: &Bound<'_, PyModule>) -> PyResult<()> {
    env_logger::init(); // Initialize logging
    m.add_class::<Money>()?;
    m.add_class::<MoneyWithVAT>()?;
    m.add_class::<MoneyWithVATRatio>()?;
    m.add_function(wrap_pyfunction!(sum_, m)?)?;
    Ok(())
}
