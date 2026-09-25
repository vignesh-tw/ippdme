use std::future::Future;
use std::sync::Arc;

use ippdme_core::CoordSystem;
use ippdme_net::{IppClient, Result as NetResult};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::error::to_py_err;
use crate::response::PyResponse;

/// A connected I++ DME client. Every method blocks the calling Python thread
/// (with the GIL released) until a correlated response arrives or the
/// request times out.
#[pyclass(name = "IppClient")]
pub struct PyIppClient {
    inner: Arc<IppClient>,
}

impl PyIppClient {
    fn block_on<F, Fut>(&self, py: Python<'_>, f: F) -> PyResult<PyResponse>
    where
        F: FnOnce(Arc<IppClient>) -> Fut + Send,
        Fut: Future<Output = NetResult<ippdme_core::Message>>,
    {
        let client = self.inner.clone();
        py.allow_threads(|| crate::runtime::runtime().block_on(f(client)))
            .map(PyResponse::from)
            .map_err(to_py_err)
    }
}

fn parse_coord_system(name: &str) -> PyResult<CoordSystem> {
    match name.to_ascii_uppercase().as_str() {
        "MCS" => Ok(CoordSystem::Mcs),
        "PCS" => Ok(CoordSystem::Pcs),
        other => Err(PyValueError::new_err(format!(
            "unknown coordinate system {other:?}, expected \"MCS\" or \"PCS\""
        ))),
    }
}

#[pymethods]
impl PyIppClient {
    /// Connect to `addr` (e.g. `"127.0.0.1:1294"`).
    #[staticmethod]
    fn connect(py: Python<'_>, addr: String) -> PyResult<Self> {
        py.allow_threads(|| {
            crate::runtime::runtime().block_on(async move { IppClient::connect(addr).await })
        })
        .map(|inner| PyIppClient {
            inner: Arc::new(inner),
        })
        .map_err(to_py_err)
    }

    fn start_session(&self, py: Python<'_>) -> PyResult<PyResponse> {
        self.block_on(py, |c| async move { c.start_session().await })
    }

    fn end_session(&self, py: Python<'_>) -> PyResult<PyResponse> {
        self.block_on(py, |c| async move { c.end_session().await })
    }

    fn get_dme_version(&self, py: Python<'_>) -> PyResult<PyResponse> {
        self.block_on(py, |c| async move { c.get_dme_version().await })
    }

    fn home(&self, py: Python<'_>) -> PyResult<PyResponse> {
        self.block_on(py, |c| async move { c.home().await })
    }

    fn go_to(&self, py: Python<'_>, x: f64, y: f64, z: f64) -> PyResult<PyResponse> {
        self.block_on(py, move |c| async move { c.go_to(x, y, z).await })
    }

    fn pt_meas(&self, py: Python<'_>) -> PyResult<PyResponse> {
        self.block_on(py, |c| async move { c.pt_meas().await })
    }

    fn set_coord_system(&self, py: Python<'_>, coord_system: &str) -> PyResult<PyResponse> {
        let cs = parse_coord_system(coord_system)?;
        self.block_on(py, move |c| async move { c.set_coord_system(cs).await })
    }

    /// Send a raw I++ DME term, e.g. `"OnMoveArc(...)"`, for commands not yet
    /// covered by a dedicated method.
    fn send_raw(&self, py: Python<'_>, term: &str) -> PyResult<PyResponse> {
        let term =
            ippdme_core::parse_term_str(term).map_err(|e| PyValueError::new_err(e.to_string()))?;
        self.block_on(py, move |c| async move { c.send(term).await })
    }
}
