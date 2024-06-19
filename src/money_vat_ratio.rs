use pyo3::prelude::*;
use rust_decimal::Decimal;

#[pyclass]
#[derive(Debug, Clone)]
pub struct MoneyWithVATRatio {
    pub net_ratio: Decimal,
    pub gross_ratio: Decimal,
}

#[pymethods]
impl MoneyWithVATRatio {
    #[new]
    fn new(net_ratio: Decimal, gross_ratio: Decimal, _py: Python) -> Self {
        Self {
            net_ratio,
            gross_ratio,
        }
    }

    #[getter(net_ratio)]
    fn get_net_ratio(&self) -> Decimal {
        self.net_ratio
    }

    #[getter(gross_ratio)]
    fn get_gross_ratio(&self) -> Decimal {
        self.gross_ratio
    }

    fn __str__(&self) -> String {
        format!("{} {}", self.net_ratio, self.gross_ratio)
    }

    fn __repr__(&self) -> String {
        format!(
            "MoneyWithVATRatio(net='{}', gross='{}')",
            self.net_ratio, self.gross_ratio
        )
    }

    fn __mul__(&self, other: Decimal) -> Self {
        Self {
            net_ratio: self.net_ratio * other,
            gross_ratio: self.gross_ratio * other,
        }
    }
}
