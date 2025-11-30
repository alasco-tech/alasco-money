use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use regex::Regex;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};

use crate::money::Money;

pub fn decimal_extract(obj: Bound<PyAny>) -> PyResult<Decimal> {
    if obj.extract::<Money>().is_ok() {
        Err(PyValueError::new_err("Invalid decimal"))
    } else if let Ok(mut amount) = obj.extract::<Decimal>() {
        if obj.to_string().trim_start().starts_with("-") {
            // Hack for minus zero
            amount.set_sign_negative(true);
        };
        Ok(amount)
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(Decimal::from_f64(f).unwrap())
    } else if let Ok(s) = obj.extract::<&str>() {
        let re = Regex::new(r"^0(\.0+)?[eE][+-]\d+$").unwrap();
        if re.is_match(s) {
            Ok(Decimal::new(0, 0))
        } else {
            Err(PyValueError::new_err("Invalid decimal"))
        }
    } else {
        Err(PyValueError::new_err("Invalid decimal"))
    }
}

// Negates decimals the way of Python
pub fn decimal_neg(right: Decimal) -> Decimal {
    if right == Decimal::new(-0, 0) {
        Decimal::new(0, 0)
    } else {
        -right
    }
}

// Adds decimals the way of Python
pub fn decimal_add(left: Decimal, right: Decimal) -> Decimal {
    let zero = Decimal::new(0, 0);

    if left.abs() == zero && right.abs() == zero {
        if left.is_sign_negative() && right.is_sign_negative() {
            -zero
        } else {
            zero
        }
    } else {
        left + right
    }
}

// Multiplies decimals the way of Python
pub fn decimal_mult(left: Decimal, right: Decimal) -> Decimal {
    let zero = Decimal::new(0, 0);

    if left.abs() == zero || right.abs() == zero {
        if left.is_sign_negative() == right.is_sign_negative() {
            zero
        } else {
            -zero
        }
    } else {
        left * right
    }
}

// Divides decimals the way of Python
pub fn decimal_div(left: Decimal, right: Decimal) -> Decimal {
    let zero = Decimal::new(0, 0);

    if left.abs() == zero && right.abs() != zero {
        if left.is_sign_negative() == right.is_sign_negative() {
            zero
        } else {
            -zero
        }
    } else {
        left / right
    }
}

// Rounds decimals the way of Python
pub fn decimal_round(value: Decimal, scale: i32) -> Decimal {
    if scale >= 0 {
        return value.round_dp_with_strategy(scale as u32, RoundingStrategy::MidpointNearestEven);
    }

    let factor = Decimal::new(10_i64.pow((-scale) as u32), 0);
    decimal_mult(decimal_div(value, factor).round(), factor)
}
