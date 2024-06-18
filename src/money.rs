use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

    #[getter(amount)]
    fn get_amount(&self) -> Decimal {
        self.amount
    }

    pub fn round(&self, n: Option<i32>) -> Self {
        Self {
            amount: round_with_negative_scale(
                self.amount,
                if let Some(true_n) = n { true_n } else { 0 },
            ),
        }
    }

    fn __str__(&self) -> String {
        self.amount.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Money('{}')", self.amount)
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.amount.hash(&mut hasher);
        hasher.finish()
    }

    fn __add__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(other_money) = other.extract::<PyRef<Self>>() {
            Ok(Self {
                amount: self.amount + other_money.amount,
            })
        } else if let Ok(i) = other.extract::<i32>() {
            if i == 0 {
                Ok(self.clone())
            } else {
                Err(pyo3::exceptions::PyTypeError::new_err(
                    "Unsupported operand",
                ))
            }
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Unsupported operand",
            ))
        }
    }

    fn __radd__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        self.__add__(other)
    }

    fn __sub__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(other_money) = other.extract::<PyRef<Self>>() {
            Ok(Self {
                amount: self.amount - other_money.amount,
            })
        } else if let Ok(i) = other.extract::<i32>() {
            if i == 0 {
                Ok(self.clone())
            } else {
                Err(pyo3::exceptions::PyTypeError::new_err(
                    "Unsupported operand",
                ))
            }
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Unsupported operand",
            ))
        }
    }

    fn __rsub__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(other_money) = other.extract::<PyRef<Self>>() {
            Ok(Self {
                amount: other_money.amount - self.amount,
            })
        } else if let Ok(i) = other.extract::<i32>() {
            if i == 0 {
                Ok(Self {
                    amount: -self.amount,
                })
            } else {
                Err(pyo3::exceptions::PyTypeError::new_err(
                    "Unsupported operand",
                ))
            }
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Unsupported operand",
            ))
        }
    }

    fn __mul__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(i) = other.extract::<f64>() {
            Ok(Self {
                amount: self.amount * Decimal::from_f64(i).unwrap(),
            })
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Unsupported operand",
            ))
        }
    }

    fn __rmul__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        self.__mul__(other)
    }

    fn __truediv__(&self, other: Bound<PyAny>) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            if let Ok(other_money) = other.extract::<PyRef<Self>>() {
                Ok((self.amount / other_money.amount).into_py(py))
            } else if let Ok(i) = other.extract::<f64>() {
                Ok(Self {
                    amount: self.amount / Decimal::from_f64(i).unwrap(),
                }
                .into_py(py))
            } else if let Ok(i) = other.extract::<Decimal>() {
                Ok(Self {
                    amount: self.amount / i,
                }
                .into_py(py))
            } else {
                Err(pyo3::exceptions::PyTypeError::new_err(
                    "Unsupported operand",
                ))
            }
        })
    }

    fn __neg__(&self) -> Self {
        Self {
            amount: -self.amount,
        }
    }

    fn __abs__(&self) -> Self {
        Self {
            amount: self.amount.abs(),
        }
    }

    fn __bool__(&self) -> bool {
        !self.amount.is_zero()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.amount == other.amount
    }

    fn __ne__(&self, other: &Self) -> bool {
        self.amount != other.amount
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

#[pyfunction]
pub fn sum_(elems: Vec<Option<Money>>, _py: Python) -> PyResult<Money> {
    let mut amount: Decimal = Decimal::new(0, 0);

    for item in elems {
        if let Some(value) = item {
            amount += value.amount;
        }
    }

    Ok(Money { amount })
}

fn round_with_negative_scale(value: Decimal, scale: i32) -> Decimal {
    if scale >= 0 {
        return value.round_dp(scale as u32);
    }

    let factor = Decimal::new(10_i64.pow((-scale) as u32), 0);
    (value / factor).round() * factor
}
