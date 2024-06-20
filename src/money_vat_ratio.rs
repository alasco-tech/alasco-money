use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};
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
            "MoneyWithVATRatio(net_ratio='{}', gross_ratio='{}')",
            self.net_ratio, self.gross_ratio
        )
    }

    fn __mul__(&self, other: Decimal) -> Self {
        Self {
            net_ratio: self.net_ratio * other,
            gross_ratio: self.gross_ratio * other,
        }
    }

    fn __eq__(&self, other: &Self) -> bool {
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
    fn __get_pydantic_json_schema__(
        _core_schema: Bound<PyAny>,
        _handler: Bound<PyAny>,
        py: Python,
    ) -> PyResult<PyObject> {
        // {
        //     "properties": {
        //         "net_ratio": {
        //             "title": "Net ratio",
        //             "type": "string",
        //             "example": "0.23",
        //         },
        //         "gross_ratio": {
        //             "title": "Gross ratio",
        //             "type": "string",
        //             "example": "0.23",
        //         },
        //     },
        //     "type": "object",
        // }

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
        // {
        //     "type": "function-plain",
        //     "function": {"type": "with-info", "function": lambda: None},
        //     "serialization": {
        //         "type": "function-plain",
        //         "function": lambda: None,
        //         "when_used": "json",
        //     },
        // }

        // Define validation function
        let validate_fn = PyCFunction::new_closure_bound(
            py,
            None,
            None,
            |args: &Bound<PyTuple>, _kwargs: Option<&Bound<PyDict>>| -> PyResult<Self> {
                if let Ok(money_with_vat_ratio) = args.get_item(0)?.extract::<Self>() {
                    return Ok(money_with_vat_ratio);
                } else if let Ok(dict) = args.get_item(0)?.extract::<&PyDict>() {
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
            },
        )?;

        let function = PyDict::new_bound(py);
        function.set_item("type", "with-info")?;
        function.set_item("function", validate_fn)?;

        let schema = PyDict::new_bound(py);

        schema.set_item("type", "function-plain")?;
        schema.set_item("function", function)?;

        // // Define serialization function
        // let serialize_fn = PyCFunction::new_closure(py, None,None  |args: &PyTuple| -> PyResult<PyObject> {
        //     let obj = args.get_item(0)?;
        //     let user: &User = obj.extract()?;
        //     let serialized = format!("User(name: '{}', age: {})", user.name, user.age);
        //     Ok(PyString::new(py, &serialized).into())
        // })?;
        // schema.set_item("serialize", serialize_fn)?;

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
