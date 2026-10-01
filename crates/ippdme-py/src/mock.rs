use std::time::Duration;

use ippdme_net::{IppMockServer, MockConfig, NetError, TlsIdentity, TlsServerConfig};
use pyo3::exceptions::PyValueError;
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
    tls: Option<TlsServerConfig>,
    config: MockConfig,
    handle: Option<JoinHandle<ippdme_net::Result<()>>>,
}

#[pymethods]
impl PyIppMockServer {
    /// Pass `cert` and `key` (PEM file paths) to serve TLS 1.3 instead of
    /// plain TCP. Also passing `client_ca` (a PEM file path) requires
    /// clients to present a certificate signed by that CA (mutual TLS).
    ///
    /// `latency_ms` is how long `Home` and `GoTo` take (default 500; use 0
    /// for fast tests). With `strict=True` the mock rejects commands issued
    /// out of order: `NoSession` before `StartSession`, `UserNotEnabled`
    /// before `EnableUser` for motion and measuring, and `NotHomed` before
    /// `Home` for `GoTo`/`PtMeas`.
    #[new]
    #[pyo3(signature = (port=1294, *, cert=None, key=None, client_ca=None, latency_ms=500, strict=false))]
    fn new(
        port: u16,
        cert: Option<String>,
        key: Option<String>,
        client_ca: Option<String>,
        latency_ms: u64,
        strict: bool,
    ) -> PyResult<Self> {
        let tls = match (cert, key) {
            (Some(cert), Some(key)) => {
                let identity = TlsIdentity::from_files(cert, key).map_err(to_py_err)?;
                let client_ca = client_ca
                    .map(std::fs::read)
                    .transpose()
                    .map_err(NetError::from)
                    .map_err(to_py_err)?;
                Some(TlsServerConfig::new(&identity, client_ca.as_deref()).map_err(to_py_err)?)
            }
            (None, None) if client_ca.is_none() => None,
            _ => {
                return Err(PyValueError::new_err(
                    "TLS needs cert and key together; client_ca requires them",
                ))
            }
        };
        let mut config = MockConfig::default().with_latency(Duration::from_millis(latency_ms));
        if strict {
            config = config.strict();
        }
        Ok(PyIppMockServer {
            port,
            tls,
            config,
            handle: None,
        })
    }

    /// Bind and start accepting connections on a background Tokio task.
    /// Returns immediately; the server keeps running until `stop()` is
    /// called or the object is garbage-collected.
    fn start_in_background(&mut self, py: Python<'_>) -> PyResult<()> {
        if self.handle.is_some() {
            return Ok(());
        }
        let port = self.port;
        let tls = self.tls.clone();
        let config = self.config;
        let server = py
            .allow_threads(|| {
                runtime().block_on(async move {
                    let mut builder = IppMockServer::builder().config(config);
                    if let Some(tls) = tls {
                        builder = builder.tls(tls);
                    }
                    builder.bind(("127.0.0.1", port)).await
                })
            })
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
