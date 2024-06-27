use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};

use crate::money::Money;

pub fn get_decimal(obj: Bound<PyAny>) -> PyResult<Decimal> {
    if let Ok(_) = obj.extract::<Money>() {
        Err(PyValueError::new_err("Invalid decimal"))
    } else if let Ok(mut amount) = obj.extract::<Decimal>() {
        if obj.to_string().trim_start().starts_with("-") {
            // Hack for minus zero
            amount.set_sign_negative(true);
        };
        Ok(amount)
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(Decimal::from_f64(f).unwrap())
    } else {
        Err(PyValueError::new_err("Invalid decimal"))
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

// Rounds decimals the way of Python
pub fn round(value: Decimal, scale: i32, round_up: bool) -> Decimal {
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
