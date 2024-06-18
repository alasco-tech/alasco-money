mod money;
mod money_vat;

use env_logger;
use pyo3::prelude::*;

use crate::money::sum_;
use crate::money::Money;
use crate::money_vat::MoneyWithVAT;

#[pymodule]
fn alasco_money(_py: Python, m: &PyModule) -> PyResult<()> {
    env_logger::init(); // Initialize logging
    m.add_class::<Money>()?;
    m.add_class::<MoneyWithVAT>()?;
    m.add_function(wrap_pyfunction!(sum_, m)?).unwrap();
    Ok(())
}
