use pyo3::prelude::*;

#[pymodule]
fn ippdme(_py: Python<'_>, _m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
