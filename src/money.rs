use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyIterator, PyTuple};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

pub const MONEY_PRECISION: Option<i32> = Some(12);

#[pyclass(subclass)]
#[derive(Debug, Clone)]
pub struct Money {
    pub amount: Decimal,
}

#[pymethods]
impl Money {
    #[new]
    fn new(amount: Option<Bound<PyAny>>) -> PyResult<Self> {
        if let Some(obj) = amount {
            if let Ok(money) = obj.extract::<Self>() {
                Ok(Self {
                    amount: money.amount.clone(),
                })
            } else if let Ok(s) = obj.extract::<&str>() {
                match Decimal::from_str(s) {
                    Ok(decimal) => Ok(Self { amount: decimal }),
                    Err(_) => Err(PyValueError::new_err("Invalid decimal")),
                }
            } else if let Ok(decimal) = get_decimal(obj) {
                Ok(Self { amount: decimal })
            } else {
                Err(PyValueError::new_err("Invalid type"))
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
                false,
            ),
        }
    }

    pub fn round_up(&self, n: Option<i32>) -> Self {
        Self {
            amount: round_with_negative_scale(
                self.amount,
                if let Some(true_n) = n { true_n } else { 0 },
                true,
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

    fn __neg__(&self) -> Self {
        Self {
            amount: if self.amount != Decimal::new(0, 0) {
                -self.amount
            } else {
                self.amount
            },
        }
    }

    fn __abs__(&self) -> Self {
        Self {
            amount: self.amount.abs(),
        }
    }

    fn __add__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(other_money) = other.extract::<Self>() {
            Ok(Self {
                amount: self.amount + other_money.amount,
            })
        } else if let Ok(other_decimal) = get_decimal(other) {
            Ok(Self {
                amount: self.amount + other_decimal,
            })
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
        if let Ok(other_money) = other.extract::<Self>() {
            Ok(Self {
                amount: self.amount - other_money.amount,
            })
        } else if let Ok(other_decimal) = get_decimal(other) {
            Ok(Self {
                amount: self.amount - other_decimal,
            })
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Unsupported operand",
            ))
        }
    }

    fn __rsub__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        self.__neg__().__add__(other)
    }

    fn __mul__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(other_decimal) = get_decimal(other) {
            Ok(Self {
                amount: self.amount * other_decimal,
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
            if let Ok(other_money) = other.extract::<Self>() {
                if other_money.amount == Decimal::new(0, 0) {
                    Err(pyo3::exceptions::PyZeroDivisionError::new_err(
                        "Division by zero",
                    ))
                } else {
                    Ok((self.amount / other_money.amount).into_py(py))
                }
            } else if let Ok(other_decimal) = get_decimal(other) {
                if other_decimal == Decimal::new(0, 0) {
                    Err(pyo3::exceptions::PyZeroDivisionError::new_err(
                        "Division by zero",
                    ))
                } else {
                    Ok(Self {
                        amount: self.amount / other_decimal,
                    }
                    .into_py(py))
                }
            } else {
                Err(pyo3::exceptions::PyTypeError::new_err(
                    "Unsupported operand",
                ))
            }
        })
    }

    fn __rtruediv__(&self, other: Bound<PyAny>) -> PyResult<PyObject> {
        if self.amount == Decimal::new(0, 0) {
            return Err(pyo3::exceptions::PyZeroDivisionError::new_err(
                "Division by zero",
            ));
        }

        Python::with_gil(|py| {
            if let Ok(other_money) = other.extract::<Self>() {
                Ok((other_money.amount / self.amount).into_py(py))
            } else if let Ok(other_decimal) = get_decimal(other) {
                Ok(Self {
                    amount: other_decimal / self.amount,
                }
                .into_py(py))
            } else {
                Err(pyo3::exceptions::PyTypeError::new_err(
                    "Unsupported operand",
                ))
            }
        })
    }

    fn __bool__(&self) -> bool {
        !self.amount.is_zero()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.amount.cmp(&other.amount))
    }

    pub fn for_json(&self) -> String {
        self.round(MONEY_PRECISION).amount.to_string()
    }

    #[staticmethod]
    fn validate(value: Bound<PyAny>, _info: Option<Bound<PyAny>>) -> PyResult<Self> {
        if let Ok(money) = value.extract::<Self>() {
            return Ok(money);
        } else if let Ok(decimal) = value.extract::<Decimal>() {
            return Ok(Money { amount: decimal });
        }

        Err(PyValueError::new_err("Validation error"))
    }

    #[staticmethod]
    fn __get_pydantic_json_schema__(
        _core_schema: Bound<PyAny>,
        _handler: Bound<PyAny>,
        py: Python,
    ) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        dict.set_item("example", "123.123456789012")?;
        dict.set_item("type", "string")?;

        Ok(dict.into())
    }

    #[staticmethod]
    fn __get_pydantic_core_schema__(
        _source: Bound<PyAny>,
        _handler: Bound<PyAny>,
        py: Python,
    ) -> PyResult<PyObject> {
        // Define validation function
        let validate_fn = PyCFunction::new_closure_bound(
            py,
            None,
            None,
            |args: &Bound<PyTuple>, _: Option<&Bound<PyDict>>| -> PyResult<Self> {
                Self::validate(args.get_item(0).unwrap(), None)
            },
        )?;

        // Define serialization function
        let serialize_fn = PyCFunction::new_closure_bound(
            py,
            None,
            None,
            |args: &Bound<PyTuple>, _: Option<&Bound<PyDict>>| -> PyResult<String> {
                if let Ok(money) = args.get_item(0)?.extract::<Self>() {
                    return Ok(money.for_json());
                }

                Err(PyValueError::new_err("Validation error"))
            },
        )?;

        let function = PyDict::new_bound(py);
        function.set_item("type", "with-info")?;
        function.set_item("function", validate_fn)?;

        let serialization = PyDict::new_bound(py);
        serialization.set_item("type", "function-plain")?;
        serialization.set_item("when_used", "json")?;
        serialization.set_item("function", serialize_fn)?;

        let schema = PyDict::new_bound(py);
        schema.set_item("type", "function-plain")?;
        schema.set_item("function", function)?;
        schema.set_item("serialization", serialization)?;

        Ok(schema.into())
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn __copy__(&self) -> Self {
        self.clone()
    }

    pub fn __deepcopy__(&self, _memo: Bound<PyDict>) -> Self {
        self.clone()
    }
}

#[pyfunction]
pub fn sum_(elems: Bound<PyAny>) -> PyResult<Money> {
    let iterator = PyIterator::from_bound_object(&elems)?;
    let mut amount: Decimal = Decimal::new(0, 0);

    for elem in iterator {
        if let Ok(item) = elem {
            if let Ok(Some(value)) = item.extract::<Option<Money>>() {
                amount += value.amount;
            }
        }
    }

    Ok(Money { amount })
}

fn round_with_negative_scale(value: Decimal, scale: i32, round_up: bool) -> Decimal {
    let strategy = if round_up {
        RoundingStrategy::MidpointAwayFromZero
    } else {
        RoundingStrategy::MidpointNearestEven
    };

    if scale >= 0 {
        return value.round_dp_with_strategy(scale as u32, strategy);
    }

    let factor = Decimal::new(10_i64.pow((-scale) as u32), 0);
    (value / factor).round() * factor
}

pub fn get_decimal(obj: Bound<PyAny>) -> PyResult<Decimal> {
    if let Ok(_) = obj.extract::<Money>() {
        Err(PyValueError::new_err("Invalid decimal"))
    } else if let Ok(amount) = obj.extract::<Decimal>() {
        Ok(amount)
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(Decimal::from_f64(f).unwrap())
    } else {
        Err(PyValueError::new_err("Invalid decimal"))
    }
}
