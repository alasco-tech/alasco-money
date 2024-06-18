use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

    #[getter]
    fn is_positive(&self) -> bool {
        self.gross().amount > Decimal::new(0, 0)
    }

    #[getter]
    fn is_negative(&self) -> bool {
        self.gross().amount < Decimal::new(0, 0)
    }

    fn is_equal_up_to_cents(&self, other: Self) -> bool {
        self.gross().round(2).amount == other.gross().round(2).amount
    }

    fn is_lower_up_to_cents(&self, other: Self) -> bool {
        self.gross().round(2).amount < other.gross().round(2).amount
    }

    fn is_lower_or_equal_up_to_cents(&self, other: Self) -> bool {
        self.is_equal_up_to_cents(other.clone()) || self.is_lower_up_to_cents(other.clone())
    }

    fn rounded_to_cents(&self) -> Self {
        let rounded_net = self.net.round(2).amount;
        return Self {
            net: Money {
                amount: rounded_net,
            },
            tax: Money {
                amount: self.gross().round(2).amount - rounded_net,
            },
        };
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

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.gross().amount.hash(&mut hasher);
        hasher.finish()
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

    fn __abs__(&self) -> Self {
        Self {
            net: Money {
                amount: self.net.amount.abs(),
            },
            tax: Money {
                amount: self.tax.amount.abs(),
            },
        }
    }

    fn __bool__(&self) -> bool {
        !self.net.amount.is_zero() || !self.tax.amount.is_zero()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.gross().amount == other.gross().amount
    }

    fn __ne__(&self, other: &Self) -> bool {
        self.gross().amount != other.gross().amount
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

    #[staticmethod]
    fn fast_sum(operands: Vec<Self>, _py: Python) -> PyResult<Self> {
        let mut net_sum = Decimal::new(0, 0);
        let mut tax_sum = Decimal::new(0, 0);

        for item in operands {
            net_sum += item.net.amount;
            tax_sum += item.tax.amount;
        }

        Ok(MoneyWithVAT {
            net: Money { amount: net_sum },
            tax: Money { amount: tax_sum },
        })
    }

    #[staticmethod]
    fn fast_sum_with_none(operands: Vec<Option<Self>>, _py: Python) -> PyResult<Option<Self>> {
        // Convert Python Iterable to Rust Iterator

        let mut net_sum: Decimal = Decimal::new(0, 0);
        let mut tax_sum: Decimal = Decimal::new(0, 0);
        let mut any_value: bool = false;

        for item in operands {
            if let Some(value) = item {
                net_sum += value.net.amount;
                tax_sum += value.tax.amount;
                any_value = true;
            }
        }

        if !any_value {
            Ok(None)
        } else {
            Ok(Some(MoneyWithVAT {
                net: Money { amount: net_sum },
                tax: Money { amount: tax_sum },
            }))
        }
    }
}
