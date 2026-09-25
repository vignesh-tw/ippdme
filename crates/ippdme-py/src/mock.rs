use ippdme_net::IppMockServer;
use pyo3::prelude::*;
use tokio::task::JoinHandle;

use crate::error::to_py_err;
use crate::runtime::runtime;

/// An embedded virtual CMM device. Bind it to a port (0 for an ephemeral
/// port) and run it in the background so a client in the same process, or on
/// the network, can connect to it.
#[pyclass(name = "IppMockServer")]
pub struct PyIppMockServer {
    port: u16,
    handle: Option<JoinHandle<ippdme_net::Result<()>>>,
}

#[pymethods]
impl PyIppMockServer {
    #[new]
    #[pyo3(signature = (port=1294))]
    fn new(port: u16) -> Self {
        PyIppMockServer { port, handle: None }
    }

    /// Bind and start accepting connections on a background Tokio task.
    /// Returns immediately; the server keeps running until `stop()` is
    /// called or the object is garbage-collected.
    fn start_in_background(&mut self, py: Python<'_>) -> PyResult<()> {
        if self.handle.is_some() {
            return Ok(());
        }
        let port = self.port;
        let server = py
            .allow_threads(|| runtime().block_on(IppMockServer::bind(("127.0.0.1", port))))
            .map_err(to_py_err)?;
        self.port = server.local_addr().map_err(to_py_err)?.port();
        self.handle = Some(runtime().spawn(server.serve()));
        Ok(())
    }

    /// The port actually bound (resolves ephemeral port 0 to its real
    /// value once `start_in_background()` has been called).
    #[getter]
    fn port(&self) -> u16 {
        self.port
    }

    fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

impl Drop for PyIppMockServer {
    fn drop(&mut self) {
        self.stop();
    }
}
