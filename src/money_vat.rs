use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crate::money::Money;
use crate::money_vat_ratio::MoneyWithVATRatio;

const KNOWN_VAT_RATES: [i16; 9] = [0, 5, 7, 10, 13, 16, 19, 20, 25];

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

    #[getter(net)]
    fn get_net(&self) -> Money {
        self.net.clone()
    }

    #[getter(tax)]
    fn get_tax(&self) -> Money {
        self.tax.clone()
    }

    #[getter(gross)]
    fn get_gross(&self) -> Money {
        Money {
            amount: self.net.amount + self.tax.amount,
        }
    }

    #[getter(tax_rate)]
    fn get_tax_rate(&self) -> Decimal {
        if self.net.amount == Decimal::new(0, 0) {
            Decimal::new(0, 0)
        } else {
            self.tax.amount / self.net.amount
        }
    }

    #[getter(tax_rate_for_display)]
    fn get_tax_rate_for_display(&self) -> Decimal {
        let boundary = Decimal::from_str("0.05").unwrap();
        let tax_rate = self.get_tax_rate();
        let vat_rates = KNOWN_VAT_RATES.map(|n| Decimal::new(n as i64, 2));

        if vat_rates.contains(&tax_rate) {
            return tax_rate;
        }

        for rate in vat_rates {
            let vat = rate * self.net.amount;
            let vat_diff = (vat - self.tax.amount).abs();
            if vat_diff < boundary {
                return rate;
            }
        }

        return tax_rate;
    }

    #[getter(is_positive)]
    fn get_is_positive(&self) -> bool {
        self.get_gross().amount > Decimal::new(0, 0)
    }

    #[getter(is_negative)]
    fn get_is_negative(&self) -> bool {
        self.get_gross().amount < Decimal::new(0, 0)
    }

    fn is_equal_up_to_cents(&self, other: Self) -> bool {
        self.get_gross().round(Some(2)).amount == other.get_gross().round(Some(2)).amount
    }

    fn is_lower_up_to_cents(&self, other: Self) -> bool {
        self.get_gross().round(Some(2)).amount < other.get_gross().round(Some(2)).amount
    }

    fn is_lower_or_equal_up_to_cents(&self, other: Self) -> bool {
        self.is_equal_up_to_cents(other.clone()) || self.is_lower_up_to_cents(other.clone())
    }

    fn rounded_to_cents(&self) -> Self {
        let rounded_net = self.net.round(Some(2)).amount;
        return Self {
            net: Money {
                amount: rounded_net,
            },
            tax: Money {
                amount: self.get_gross().round(Some(2)).amount - rounded_net,
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
        self.get_gross().amount.hash(&mut hasher);
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

    fn __mul__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(other_ratio) = other.extract::<PyRef<MoneyWithVATRatio>>() {
            let net_value = other_ratio.net_ratio * self.net.amount;
            Ok(Self {
                net: Money { amount: net_value },
                tax: Money {
                    amount: other_ratio.gross_ratio * self.get_gross().amount - net_value,
                },
            })
        } else if let Ok(i) = other.extract::<i32>() {
            let other_value = Decimal::from_i32(i).unwrap();
            Ok(Self {
                net: Money {
                    amount: self.net.amount * other_value,
                },
                tax: Money {
                    amount: self.tax.amount * other_value,
                },
            })
        } else if let Ok(i) = other.extract::<f64>() {
            let other_value = Decimal::from_f64(i).unwrap();
            Ok(Self {
                net: Money {
                    amount: self.net.amount * other_value,
                },
                tax: Money {
                    amount: self.tax.amount * other_value,
                },
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

    fn __truediv__(&self, other: f64) -> PyResult<Self> {
        let other_value = Decimal::from_f64(other).unwrap();

        if other_value == Decimal::new(0, 0) {
            Err(pyo3::exceptions::PyZeroDivisionError::new_err(
                "Division by zero",
            ))
        } else {
            Ok(Self {
                net: Money {
                    amount: self.net.amount / other_value,
                },
                tax: Money {
                    amount: self.tax.amount / other_value,
                },
            })
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
        self.get_gross().amount == other.get_gross().amount
    }

    fn __lt__(&self, other: &Self) -> bool {
        self.get_gross().amount < other.get_gross().amount
    }

    fn __le__(&self, other: &Self) -> bool {
        self.get_gross().amount <= other.get_gross().amount
    }

    fn __gt__(&self, other: &Self) -> bool {
        self.get_gross().amount > other.get_gross().amount
    }

    fn __ge__(&self, other: &Self) -> bool {
        self.get_gross().amount >= other.get_gross().amount
    }

    #[staticmethod]
    fn max(items: Vec<Option<Self>>) -> Option<Self> {
        let mut max_net: Option<Decimal> = None;
        let mut max_gross: Option<Decimal> = None;

        for item in items {
            if let Some(value) = item {
                max_net = Some(if let Some(true_max_net) = max_net {
                    true_max_net.max(value.get_net().amount)
                } else {
                    value.get_net().amount
                });
                max_gross = Some(if let Some(true_max_gross) = max_gross {
                    true_max_gross.max(value.get_gross().amount)
                } else {
                    value.get_gross().amount
                });
            }
        }

        if let Some(true_max_net) = max_net {
            if let Some(true_max_gross) = max_gross {
                Some(Self {
                    net: Money {
                        amount: true_max_net,
                    },
                    tax: Money {
                        amount: true_max_gross - true_max_net,
                    },
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    #[staticmethod]
    fn ratio(dividend: &Self, divisor: &Self) -> PyResult<MoneyWithVATRatio> {
        if divisor.net.amount == Decimal::new(0, 0)
            || divisor.get_gross().amount == Decimal::new(0, 0)
        {
            Err(pyo3::exceptions::PyZeroDivisionError::new_err(
                "Division by zero",
            ))
        } else {
            Ok(MoneyWithVATRatio {
                net_ratio: dividend.net.amount / divisor.net.amount,
                gross_ratio: dividend.get_gross().amount / divisor.get_gross().amount,
            })
        }
    }

    #[staticmethod]
    fn safe_ratio(dividend: Option<&Self>, divisor: Option<&Self>) -> Option<MoneyWithVATRatio> {
        let fixed_dividend = if let Some(true_dividend) = dividend {
            true_dividend.rounded_to_cents()
        } else {
            MoneyWithVAT {
                net: Money {
                    amount: Decimal::new(0, 0),
                },
                tax: Money {
                    amount: Decimal::new(0, 0),
                },
            }
        };
        let fixed_divisor = if let Some(true_divisor) = divisor {
            true_divisor.rounded_to_cents()
        } else {
            MoneyWithVAT {
                net: Money {
                    amount: Decimal::new(0, 0),
                },
                tax: Money {
                    amount: Decimal::new(0, 0),
                },
            }
        };

        if fixed_divisor.net.amount == Decimal::new(0, 0)
            || fixed_divisor.get_gross().amount == Decimal::new(0, 0)
        {
            None
        } else {
            Some(MoneyWithVATRatio {
                net_ratio: fixed_dividend.net.amount / fixed_divisor.net.amount,
                gross_ratio: fixed_dividend.get_gross().amount / fixed_divisor.get_gross().amount,
            })
        }
    }

    #[staticmethod]
    fn safe_ratio_decimal(
        dividend: Option<&Self>,
        divisor: Option<Decimal>,
    ) -> Option<MoneyWithVATRatio> {
        if let Some(true_dividend) = dividend {
            if let Some(true_divisor) = divisor {
                if true_divisor == Decimal::new(0, 0) {
                    None
                } else {
                    Some(MoneyWithVATRatio {
                        net_ratio: true_dividend.net.amount / true_divisor,
                        gross_ratio: true_dividend.get_gross().amount / true_divisor,
                    })
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    #[staticmethod]
    fn fast_sum(operands: Vec<Option<Self>>, _py: Python) -> PyResult<Self> {
        let mut net_sum = Decimal::new(0, 0);
        let mut tax_sum = Decimal::new(0, 0);

        for item in operands {
            if let Some(value) = item {
                net_sum += value.net.amount;
                tax_sum += value.tax.amount;
            }
        }

        Ok(MoneyWithVAT {
            net: Money { amount: net_sum },
            tax: Money { amount: tax_sum },
        })
    }

    #[staticmethod]
    fn fast_sum_with_none(operands: Vec<Option<Self>>, _py: Python) -> PyResult<Option<Self>> {
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

    fn for_json(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let dict = PyDict::new_bound(py);
            dict.set_item("net", self.net.for_json())?;
            dict.set_item("tax", self.tax.for_json())?;
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
        //         "net": {
        //             "example": "123.123456789012",
        //             "title": "Net amount",
        //             "type": "string",
        //         },
        //         "tax": {
        //             "example": "123.123456789012",
        //             "title": "Tax amount",
        //             "type": "string",
        //         },
        //     },
        //     "type": "object",
        // }

        let net = PyDict::new_bound(py);
        net.set_item("title", "Net amount")?;
        net.set_item("type", "string")?;
        net.set_item("example", "123.123456789012")?;

        let tax = PyDict::new_bound(py);
        tax.set_item("title", "Tax amount")?;
        tax.set_item("type", "string")?;
        tax.set_item("example", "123.123456789012")?;

        let properties = PyDict::new_bound(py);
        properties.set_item("net", net)?;
        properties.set_item("tax", tax)?;

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
                if let Ok(money_with_vat) = args.get_item(0)?.extract::<Self>() {
                    return Ok(money_with_vat);
                } else if let Ok(dict) = args.get_item(0)?.extract::<&PyDict>() {
                    if let Ok(Some(net)) = dict.get_item("net") {
                        if let Ok(Some(tax)) = dict.get_item("tax") {
                            if let Ok(true_net) = net.extract::<Decimal>() {
                                if let Ok(true_tax) = tax.extract::<Decimal>() {
                                    return Ok(Self {
                                        net: Money { amount: true_net },
                                        tax: Money { amount: true_tax },
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

    #[staticmethod]
    fn from_json(dict: Option<Bound<PyAny>>, _py: Python) -> PyResult<Self> {
        match json_to_money_vat(dict) {
            Ok(value) => Ok(value),
            Err(err) => Err(err),
        }
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

fn json_to_money_vat(raw: Option<Bound<PyAny>>) -> PyResult<MoneyWithVAT> {
    let dig = |any: &PyAny, key: &str| {
        if let Ok(dict) = any.extract::<&PyDict>() {
            if let Ok(Some(amount_with_vat)) = dict.get_item("amount_with_vat") {
                if let Ok(true_amount_with_vat) = amount_with_vat.extract::<&PyDict>() {
                    if let Ok(Some(target)) = true_amount_with_vat.get_item(key) {
                        if let Ok(true_target) = target.extract::<&PyDict>() {
                            if let Ok(Some(amount)) = true_target.get_item("amount") {
                                if let Ok(true_amount) = amount.extract::<Decimal>() {
                                    return Ok(true_amount);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(PyValueError::new_err("Invalid dict"))
    };

    if let Some(true_raw) = raw {
        if let Ok(true_dict) = true_raw.extract::<&PyAny>() {
            let raw_net = dig(true_dict, "net");
            let raw_gross = dig(true_dict, "gross");

            return match (raw_net, raw_gross) {
                (Ok(net), Ok(gross)) => Ok(MoneyWithVAT {
                    net: Money { amount: net },
                    tax: Money {
                        amount: gross - net,
                    },
                }),
                _ => Err(PyValueError::new_err("Invalid dict")),
            };
        }
    }

    Err(PyValueError::new_err("Invalid dict"))
}
