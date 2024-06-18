use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::money::Money;

#[pyclass]
#[derive(Debug, Clone)]
pub struct MoneyWithVAT {
    pub net: Money,
    pub tax: Money,
}

#[pymethods]
impl MoneyWithVAT {
    #[new]
    fn new(net: Option<PyObject>, tax: Option<PyObject>, py: Python) -> PyResult<Self> {
        let net_money = if let Some(net_obj) = net {
            if let Ok(money) = net_obj.extract::<PyRef<Money>>(py) {
                money.clone()
            } else if let Ok(s) = net_obj.extract::<&str>(py) {
                let amount = Decimal::from_str(s)
                    .map_err(|_| PyValueError::new_err("Invalid decimal string"))?;
                Money { amount }
            } else if let Ok(f) = net_obj.extract::<f64>(py) {
                let amount =
                    Decimal::from_f64(f).ok_or_else(|| PyValueError::new_err("Invalid float"))?;
                Money { amount }
            } else {
                Money {
                    amount: Decimal::new(0, 0),
                }
            }
        } else {
            Money {
                amount: Decimal::new(0, 0),
            }
        };

        let tax_money = if let Some(tax_obj) = tax {
            if let Ok(money) = tax_obj.extract::<PyRef<Money>>(py) {
                money.clone()
            } else if let Ok(s) = tax_obj.extract::<&str>(py) {
                let amount = Decimal::from_str(s)
                    .map_err(|_| PyValueError::new_err("Invalid decimal string"))?;
                Money { amount }
            } else if let Ok(f) = tax_obj.extract::<f64>(py) {
                let amount =
                    Decimal::from_f64(f).ok_or_else(|| PyValueError::new_err("Invalid float"))?;
                Money { amount }
            } else {
                Money {
                    amount: Decimal::new(0, 0),
                }
            }
        } else {
            Money {
                amount: Decimal::new(0, 0),
            }
        };

        Ok(MoneyWithVAT {
            net: net_money,
            tax: tax_money,
        })
    }

    #[getter]
    fn net(&self) -> Money {
        self.net.clone()
    }

    #[getter]
    fn tax(&self) -> Money {
        self.tax.clone()
    }

    #[getter]
    fn gross(&self) -> Money {
        Money {
            amount: self.net.amount + self.tax.amount,
        }
    }

    fn __add__(&self, other: &Self) -> Self {
        Self {
            net: Money {
                amount: self.net.amount + other.net.amount,
            },
            tax: Money {
                amount: self.tax.amount + other.tax.amount,
            },
        }
    }

    fn __sub__(&self, other: &Self) -> Self {
        Self {
            net: Money {
                amount: self.net.amount - other.net.amount,
            },
            tax: Money {
                amount: self.tax.amount - other.tax.amount,
            },
        }
    }

    fn __mul__(&self, other: f64) -> Self {
        Self {
            net: Money {
                amount: self.net.amount * Decimal::from_f64(other).unwrap(),
            },
            tax: Money {
                amount: self.tax.amount * Decimal::from_f64(other).unwrap(),
            },
        }
    }

    fn __truediv__(&self, other: f64) -> Self {
        Self {
            net: Money {
                amount: self.net.amount / Decimal::from_f64(other).unwrap(),
            },
            tax: Money {
                amount: self.tax.amount / Decimal::from_f64(other).unwrap(),
            },
        }
    }

    fn __neg__(&self) -> Self {
        Self {
            net: Money {
                amount: -self.net.amount,
            },
            tax: Money {
                amount: -self.tax.amount,
            },
        }
    }

    fn __str__(&self) -> String {
        format!("{} {}", self.net.amount, self.tax.amount)
    }

    fn __repr__(&self) -> String {
        format!(
            "MoneyWithVAT(net='{}', tax='{}')",
            self.net.amount, self.tax.amount
        )
    }

    fn __bool__(&self) -> bool {
        !self.net.amount.is_zero() || !self.tax.amount.is_zero()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.gross().amount == other.gross().amount
    }

    fn __lt__(&self, other: &Self) -> bool {
        self.gross().amount < other.gross().amount
    }

    fn __le__(&self, other: &Self) -> bool {
        self.gross().amount <= other.gross().amount
    }

    fn __gt__(&self, other: &Self) -> bool {
        self.gross().amount > other.gross().amount
    }

    fn __ge__(&self, other: &Self) -> bool {
        self.gross().amount >= other.gross().amount
    }
}
