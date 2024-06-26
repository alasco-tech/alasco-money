use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use crate::money::get_decimal;

#[pyclass]
#[derive(Debug, Clone)]
pub struct MoneyWithVATRatio {
    pub net_ratio: Decimal,
    pub gross_ratio: Decimal,
}

#[pymethods]
impl MoneyWithVATRatio {
    #[new]
    fn new(net_ratio: Bound<PyAny>, gross_ratio: Bound<PyAny>) -> PyResult<Self> {
        let net_ratio_decimal = match get_decimal(net_ratio) {
            Ok(decimal) => decimal,
            Err(_) => return Err(PyValueError::new_err("Invalid decimal")),
        };
        let gross_ratio_decimal = match get_decimal(gross_ratio) {
            Ok(decimal) => decimal,
            Err(_) => return Err(PyValueError::new_err("Invalid decimal")),
        };

        Ok(Self {
            net_ratio: net_ratio_decimal,
            gross_ratio: gross_ratio_decimal,
        })
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
        format!(
            "MoneyWithVATRatio(net_ratio='{}', gross_ratio='{}')",
            self.net_ratio, self.gross_ratio
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "MoneyWithVATRatio(net_ratio='{}', gross_ratio='{}')",
            self.net_ratio, self.gross_ratio
        )
    }

    fn __neg__(&self) -> Self {
        Self {
            net_ratio: Decimal::new(0, 0) - self.net_ratio,
            gross_ratio: Decimal::new(0, 0) - self.gross_ratio,
        }
    }

    fn __add__(&self, other: Self) -> Self {
        Self {
            net_ratio: self.net_ratio + other.net_ratio,
            gross_ratio: self.gross_ratio + other.gross_ratio,
        }
    }

    fn __sub__(&self, other: Self) -> Self {
        Self {
            net_ratio: self.net_ratio - other.net_ratio,
            gross_ratio: self.gross_ratio - other.gross_ratio,
        }
    }

    fn __mul__(&self, other: f64) -> Self {
        let other_decimal = Decimal::from_f64(other).unwrap();

        Self {
            net_ratio: self.net_ratio * other_decimal,
            gross_ratio: self.gross_ratio * other_decimal,
        }
    }

    fn __truediv__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(other_decimal) = get_decimal(other) {
            if other_decimal == Decimal::new(0, 0) {
                Err(pyo3::exceptions::PyZeroDivisionError::new_err(
                    "Division by zero",
                ))
            } else {
                Ok(Self {
                    net_ratio: self.net_ratio / other_decimal,
                    gross_ratio: self.gross_ratio / other_decimal,
                })
            }
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Unsupported operand",
            ))
        }
    }

    fn __eq__(&self, other: Self) -> bool {
        self.get_net_ratio() == other.get_net_ratio()
            && self.get_gross_ratio() == other.get_gross_ratio()
    }

    fn for_json(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let dict = PyDict::new_bound(py);
            dict.set_item("net_ratio", self.net_ratio.to_string())?;
            dict.set_item("gross_ratio", self.gross_ratio.to_string())?;
            Ok(dict.into())
        })
    }

    #[staticmethod]
    fn zero() -> Self {
        Self {
            net_ratio: Decimal::new(0, 0),
            gross_ratio: Decimal::new(0, 0),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (value, _info=None))]
    fn validate(value: Bound<PyAny>, _info: Option<Bound<PyAny>>) -> PyResult<Self> {
        if let Ok(money_with_vat_ratio) = value.extract::<Self>() {
            return Ok(money_with_vat_ratio);
        } else if let Ok(dict) = value.extract::<Bound<PyDict>>() {
            if let Ok(Some(net_ratio)) = dict.get_item("net_ratio") {
                if let Ok(Some(gross_ratio)) = dict.get_item("gross_ratio") {
                    if let Ok(true_net_ratio) = net_ratio.extract::<Decimal>() {
                        if let Ok(true_gross_ratio) = gross_ratio.extract::<Decimal>() {
                            return Ok(Self {
                                net_ratio: true_net_ratio,
                                gross_ratio: true_gross_ratio,
                            });
                        }
                    }
                }
            }
        }

        Err(PyValueError::new_err("Validation error"))
    }

    #[staticmethod]
    fn __get_pydantic_json_schema__(
        _core_schema: Bound<PyAny>,
        _handler: Bound<PyAny>,
        py: Python,
    ) -> PyResult<PyObject> {
        let net_ratio = PyDict::new_bound(py);
        net_ratio.set_item("title", "Net ratio")?;
        net_ratio.set_item("type", "string")?;
        net_ratio.set_item("example", "0.23")?;

        let gross_ratio = PyDict::new_bound(py);
        gross_ratio.set_item("title", "Gross ratio")?;
        gross_ratio.set_item("type", "string")?;
        gross_ratio.set_item("example", "0.23")?;

        let properties = PyDict::new_bound(py);
        properties.set_item("net_ratio", net_ratio)?;
        properties.set_item("gross_ratio", gross_ratio)?;

        let dict = PyDict::new_bound(py);
        dict.set_item("properties", properties)?;
        dict.set_item("type", "object")?;

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
            |args: &Bound<PyTuple>, _kwargs: Option<&Bound<PyDict>>| -> PyResult<Self> {
                Self::validate(args.get_item(0).unwrap(), None)
            },
        )?;

        // Define serialization function
        let serialize_fn = PyCFunction::new_closure_bound(
            py,
            None,
            None,
            |args: &Bound<PyTuple>, _: Option<&Bound<PyDict>>| -> PyResult<PyObject> {
                if let Ok(money_with_vat_ratio) = args.get_item(0)?.extract::<Self>() {
                    return money_with_vat_ratio.for_json();
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
