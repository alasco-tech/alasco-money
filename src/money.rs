use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::str::FromStr;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Money {
    pub amount: Decimal,
}

#[pymethods]
impl Money {
    #[new]
    fn new(amount: Option<PyObject>, py: Python) -> PyResult<Self> {
        if let Some(obj) = amount {
            if let Ok(money) = obj.extract::<PyRef<Self>>(py) {
                Ok(Self {
                    amount: money.amount.clone(),
                })
            } else if let Ok(s) = obj.extract::<&str>(py) {
                match Decimal::from_str(s) {
                    Ok(decimal) => Ok(Self { amount: decimal }),
                    Err(_) => Err(PyValueError::new_err("Invalid decimal string")),
                }
            } else if let Ok(f) = obj.extract::<f64>(py) {
                Ok(Self {
                    amount: Decimal::from_f64(f).unwrap(),
                })
            } else {
                Err(PyValueError::new_err("Invalid type for amount"))
            }
        } else {
            Ok(Self {
                amount: Decimal::new(0, 0),
            })
        }
    }

    // #[getter]
    // fn amount(&self) -> String {
    //     self.amount.to_string()
    // }

    fn round(&self, n: u32) -> Self {
        Self {
            amount: self.amount.round_dp(n),
        }
    }

    fn __add__(&self, other: &Self) -> Self {
        Self {
            amount: self.amount + other.amount,
        }
    }

    fn __sub__(&self, other: &Self) -> Self {
        Self {
            amount: self.amount - other.amount,
        }
    }

    fn __mul__(&self, other: f64) -> Self {
        Self {
            amount: self.amount * Decimal::from_f64(other).unwrap(),
        }
    }

    fn __truediv__(&self, other: f64) -> Self {
        Self {
            amount: self.amount / Decimal::from_f64(other).unwrap(),
        }
    }

    fn __neg__(&self) -> Self {
        Self {
            amount: -self.amount,
        }
    }

    fn __str__(&self) -> String {
        self.amount.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Money('{}')", self.amount)
    }

    fn __bool__(&self) -> bool {
        !self.amount.is_zero()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.amount == other.amount
    }

    fn __lt__(&self, other: &Self) -> bool {
        self.amount < other.amount
    }

    fn __le__(&self, other: &Self) -> bool {
        self.amount <= other.amount
    }

    fn __gt__(&self, other: &Self) -> bool {
        self.amount > other.amount
    }

    fn __ge__(&self, other: &Self) -> bool {
        self.amount >= other.amount
    }
}
