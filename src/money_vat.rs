use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyIterator, PyTuple};
use rust_decimal::Decimal;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crate::money::Money;
use crate::money::MONEY_PRECISION;
use crate::money::{decimal_add, get_decimal};
use crate::money_vat_ratio::MoneyWithVATRatio;

const GERMAN_VAT_RATES: [i16; 5] = [0, 5, 7, 16, 19];
const KNOWN_VAT_RATES: [i16; 9] = [0, 5, 7, 10, 13, 16, 19, 20, 25];

#[pyclass(subclass)]
#[derive(Debug, Clone)]
pub struct MoneyWithVAT {
    pub net: Money,
    pub tax: Money,
}

#[pymethods]
impl MoneyWithVAT {
    #[new]
    #[pyo3(signature = (net=None, tax=None))]
    fn new(net: Option<Bound<PyAny>>, tax: Option<Bound<PyAny>>) -> PyResult<Self> {
        let net_result = Money::new(net);
        let tax_result = Money::new(tax);

        match (net_result, tax_result) {
            (Ok(net_money), Ok(tax_money)) => Ok(MoneyWithVAT {
                net: net_money,
                tax: tax_money,
            }),
            (Err(err), _) => Err(err),
            (_, Err(err)) => Err(err),
        }
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
            amount: decimal_add(self.net.amount, self.tax.amount),
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

        if Self::known_vat_rates().contains(&tax_rate) {
            return tax_rate;
        }

        for rate in Self::known_vat_rates() {
            let vat = rate * self.net.amount;
            let vat_diff = (decimal_add(vat, -self.tax.amount)).abs();
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
                amount: decimal_add(self.get_gross().round(Some(2)).amount, -rounded_net),
            },
        };
    }

    fn rounded_to_money_field_precision(&self) -> Self {
        Self {
            net: self.net.round(MONEY_PRECISION),
            tax: self.tax.round(MONEY_PRECISION),
        }
    }

    fn __str__(&self) -> String {
        format!(
            "MoneyWithVAT(net='{}', tax='{}')",
            self.net.amount, self.tax.amount
        )
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

    fn __neg__(&self) -> Self {
        Self {
            net: self.net.__neg__(),
            tax: self.tax.__neg__(),
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

    fn __add__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(other_money_with_vat) = other.extract::<Self>() {
            Ok(Self {
                net: Money {
                    amount: decimal_add(self.net.amount, other_money_with_vat.net.amount),
                },
                tax: Money {
                    amount: decimal_add(self.tax.amount, other_money_with_vat.tax.amount),
                },
            })
        } else if let Ok(other_decimal) = get_decimal(other) {
            Ok(Self {
                net: Money {
                    amount: decimal_add(self.net.amount, other_decimal),
                },
                tax: self.tax.clone(),
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
        if let Ok(other_money_with_vat) = other.extract::<Self>() {
            Ok(Self {
                net: Money {
                    amount: decimal_add(self.net.amount, -other_money_with_vat.net.amount),
                },
                tax: Money {
                    amount: decimal_add(self.tax.amount, -other_money_with_vat.tax.amount),
                },
            })
        } else if let Ok(other_decimal) = get_decimal(other) {
            Ok(Self {
                net: Money {
                    amount: decimal_add(self.net.amount, -other_decimal),
                },
                tax: self.tax.clone(),
            })
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Unsupported operand",
            ))
        }
    }

    fn __mul__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(other_ratio) = other.extract::<MoneyWithVATRatio>() {
            let net_value = other_ratio.net_ratio * self.net.amount;
            Ok(Self {
                net: Money { amount: net_value },
                tax: Money {
                    amount: decimal_add(
                        other_ratio.gross_ratio * self.get_gross().amount,
                        -net_value,
                    ),
                },
            })
        } else if let Ok(other_decimal) = get_decimal(other) {
            Ok(Self {
                net: Money {
                    amount: self.net.amount * other_decimal,
                },
                tax: Money {
                    amount: self.tax.amount * other_decimal,
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

    fn __truediv__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        let other_decimal = match get_decimal(other) {
            Ok(decimal) => decimal,
            Err(_) => return Err(pyo3::exceptions::PyTypeError::new_err("Invalid decimal")),
        };

        if other_decimal == Decimal::new(0, 0) {
            Err(pyo3::exceptions::PyZeroDivisionError::new_err(
                "Division by zero",
            ))
        } else {
            Ok(Self {
                net: Money {
                    amount: self.net.amount / other_decimal,
                },
                tax: Money {
                    amount: self.tax.amount / other_decimal,
                },
            })
        }
    }

    fn __rtruediv__(&self, other: Bound<PyAny>) -> PyResult<Self> {
        let other_decimal = match get_decimal(other) {
            Ok(decimal) => decimal,
            Err(_) => return Err(pyo3::exceptions::PyTypeError::new_err("Invalid decimal")),
        };

        if self.net.amount == Decimal::new(0, 0) || self.tax.amount == Decimal::new(0, 0) {
            Err(pyo3::exceptions::PyZeroDivisionError::new_err(
                "Division by zero",
            ))
        } else {
            Ok(Self {
                net: Money {
                    amount: other_decimal / self.net.amount,
                },
                tax: Money {
                    amount: other_decimal / self.tax.amount,
                },
            })
        }
    }

    fn __bool__(&self) -> bool {
        !self.net.amount.is_zero() || !self.tax.amount.is_zero()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.get_gross().amount.cmp(&other.get_gross().amount))
    }

    #[staticmethod]
    #[pyo3(signature = (*args))]
    fn max(args: &Bound<PyTuple>) -> PyResult<Self> {
        let items = if args.len() == 1 {
            PyIterator::from_bound_object(&args.get_item(0).unwrap()).unwrap()
        } else {
            PyIterator::from_bound_object(&args).unwrap()
        };

        let mut max_net: Option<Decimal> = None;
        let mut max_gross: Option<Decimal> = None;

        for item in items {
            if let Ok(raw_value) = item {
                if let Ok(value) = raw_value.extract::<MoneyWithVAT>() {
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
        }

        if let Some(true_max_net) = max_net {
            if let Some(true_max_gross) = max_gross {
                Ok(MoneyWithVAT {
                    net: Money {
                        amount: true_max_net,
                    },
                    tax: Money {
                        amount: decimal_add(true_max_gross, -true_max_net),
                    },
                })
            } else {
                Err(pyo3::exceptions::PyValueError::new_err(
                    "Insufficient arguments",
                ))
            }
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(
                "Insufficient arguments",
            ))
        }
    }

    #[staticmethod]
    fn ratio(dividend: Self, divisor: Self) -> PyResult<MoneyWithVATRatio> {
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
    #[pyo3(signature = (dividend=None, divisor=None))]
    fn safe_ratio(dividend: Option<Self>, divisor: Option<Self>) -> Option<MoneyWithVATRatio> {
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
    #[pyo3(signature = (dividend=None, divisor=None))]
    fn safe_ratio_decimal(
        dividend: Option<Self>,
        divisor: Option<Decimal>,
    ) -> Option<MoneyWithVAT> {
        if let Some(true_dividend) = dividend {
            if let Some(true_divisor) = divisor {
                if true_divisor == Decimal::new(0, 0) {
                    None
                } else {
                    Some(MoneyWithVAT {
                        net: Money {
                            amount: true_dividend.net.amount / true_divisor,
                        },
                        tax: Money {
                            amount: true_dividend.tax.amount / true_divisor,
                        },
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
    fn fast_sum(iterable: Bound<PyAny>) -> PyResult<Self> {
        let iterator = PyIterator::from_bound_object(&iterable)?;

        let mut net_sum = Decimal::new(0, 0);
        let mut tax_sum = Decimal::new(0, 0);

        for elem in iterator {
            if let Ok(item) = elem {
                if let Ok(value) = item.extract::<Self>() {
                    net_sum = decimal_add(net_sum, value.net.amount);
                    tax_sum = decimal_add(tax_sum, value.tax.amount);
                }
            }
        }

        Ok(MoneyWithVAT {
            net: Money { amount: net_sum },
            tax: Money { amount: tax_sum },
        })
    }

    #[staticmethod]
    fn fast_sum_with_none(iterable: Bound<PyAny>) -> PyResult<Option<Self>> {
        let iterator = PyIterator::from_bound_object(&iterable)?;

        let mut net_sum: Decimal = Decimal::new(0, 0);
        let mut tax_sum: Decimal = Decimal::new(0, 0);
        let mut any_value: bool = false;

        for elem in iterator {
            if let Ok(item) = elem {
                if let Ok(Some(value)) = item.extract::<Option<Self>>() {
                    net_sum = decimal_add(net_sum, value.net.amount);
                    tax_sum = decimal_add(tax_sum, value.tax.amount);
                    any_value = true;
                }
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
    #[pyo3(signature = (value, _info=None))]
    fn validate(value: Bound<PyAny>, _info: Option<Bound<PyAny>>) -> PyResult<Self> {
        if let Ok(money_with_vat) = value.extract::<Self>() {
            return Ok(money_with_vat);
        } else if let Ok(dict) = value.extract::<Bound<PyDict>>() {
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
    }

    #[staticmethod]
    fn __get_pydantic_json_schema__(
        _core_schema: Bound<PyAny>,
        _handler: Bound<PyAny>,
        py: Python,
    ) -> PyResult<PyObject> {
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
                if let Ok(money_with_vat) = args.get_item(0)?.extract::<Self>() {
                    return money_with_vat.for_json();
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

    #[staticmethod]
    #[pyo3(signature = (dict=None))]
    fn from_json(dict: Option<Bound<PyAny>>) -> PyResult<Self> {
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

    #[staticmethod]
    fn german_vat_rates() -> [Decimal; 5] {
        GERMAN_VAT_RATES.map(|n| Decimal::new(n as i64, 2))
    }

    #[staticmethod]
    fn known_vat_rates() -> [Decimal; 9] {
        KNOWN_VAT_RATES.map(|n| Decimal::new(n as i64, 2))
    }
}

fn json_to_money_vat(raw: Option<Bound<PyAny>>) -> PyResult<MoneyWithVAT> {
    let dig = |any: &Bound<PyAny>, key: &str| {
        if let Ok(dict) = any.extract::<Bound<PyDict>>() {
            if let Ok(Some(amount_with_vat)) = dict.get_item("amount_with_vat") {
                if let Ok(true_amount_with_vat) = amount_with_vat.extract::<Bound<PyDict>>() {
                    if let Ok(Some(target)) = true_amount_with_vat.get_item(key) {
                        if let Ok(true_target) = target.extract::<Bound<PyDict>>() {
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

    if let Some(true_dict) = raw {
        let raw_net = dig(&true_dict, "net");
        let raw_gross = dig(&true_dict, "gross");

        return match (raw_net, raw_gross) {
            (Ok(net), Ok(gross)) => Ok(MoneyWithVAT {
                net: Money { amount: net },
                tax: Money {
                    amount: decimal_add(gross, -net),
                },
            }),
            _ => Err(PyValueError::new_err("Invalid dict")),
        };
    }

    Err(PyValueError::new_err("Invalid dict"))
}
