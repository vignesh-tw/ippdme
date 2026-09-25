// pyo3's #[pymethods]/#[pyfunction] macros expand into wrapper code that
// triggers this lint spuriously; see https://github.com/PyO3/pyo3/issues/4568.
#![allow(clippy::useless_conversion)]

mod client;
mod error;
mod mock;
mod response;
mod runtime;

use pyo3::prelude::*;

use client::PyIppClient;
use mock::PyIppMockServer;
use response::PyResponse;

#[pymodule]
fn _ippdme(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIppClient>()?;
    m.add_class::<PyIppMockServer>()?;
    m.add_class::<PyResponse>()?;
    Ok(())
}
