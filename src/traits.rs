use pyo3::prelude::*;
use pyo3::types::PyDict;

pub trait Copyable {
    #![allow(unused)]
    fn copy(&self) -> Self;

    fn __copy__(&self) -> Self;

    fn __deepcopy__(&self, _memo: Bound<PyDict>) -> Self;
}
