use std::future::Future;
use std::sync::Arc;

use ippdme_core::{Command, CoordSystem};
use ippdme_net::{IppClient, NetError, Result as NetResult, TlsClientConfig, TlsIdentity};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::error::to_py_err;
use crate::response::PyResponse;

/// A connected I++ DME client. Methods return the raw server [`PyResponse`]
/// (ack, data or error), so tests can assert on error replies too. Every
/// method blocks the calling Python thread
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

/// The host part of a `host:port` address, without IPv6 brackets.
fn host_of(addr: &str) -> &str {
    let host = addr.rsplit_once(':').map_or(addr, |(host, _)| host);
    host.trim_start_matches('[').trim_end_matches(']')
}

#[pymethods]
impl PyIppClient {
    /// Connect to `addr` (e.g. `"127.0.0.1:1294"`).
    ///
    /// Passing `ca_cert` (a PEM file path) switches the connection to TLS
    /// 1.3, verifying the server against that CA. `server_name` is the name
    /// the server certificate must match (default: the host part of
    /// `addr`). Supply both `client_cert` and `client_key` (PEM file paths)
    /// to present a client certificate for mutual TLS.
    #[staticmethod]
    #[pyo3(signature = (addr, *, ca_cert=None, server_name=None, client_cert=None, client_key=None))]
    fn connect(
        py: Python<'_>,
        addr: String,
        ca_cert: Option<String>,
        server_name: Option<String>,
        client_cert: Option<String>,
        client_key: Option<String>,
    ) -> PyResult<Self> {
        let tls = match ca_cert {
            Some(ca) => {
                let identity = match (client_cert, client_key) {
                    (Some(cert), Some(key)) => {
                        Some(TlsIdentity::from_files(cert, key).map_err(to_py_err)?)
                    }
                    (None, None) => None,
                    _ => {
                        return Err(PyValueError::new_err(
                            "client_cert and client_key must be given together",
                        ))
                    }
                };
                let name = server_name.unwrap_or_else(|| host_of(&addr).to_owned());
                let ca_pem = std::fs::read(&ca)
                    .map_err(NetError::from)
                    .map_err(to_py_err)?;
                Some(TlsClientConfig::new(&name, &ca_pem, identity.as_ref()).map_err(to_py_err)?)
            }
            None if server_name.is_some() || client_cert.is_some() || client_key.is_some() => {
                return Err(PyValueError::new_err(
                    "server_name, client_cert and client_key require ca_cert (TLS)",
                ))
            }
            None => None,
        };

        py.allow_threads(|| {
            crate::runtime::runtime().block_on(async move {
                match tls {
                    Some(tls) => IppClient::connect_tls(addr, &tls).await,
                    None => IppClient::connect(addr).await,
                }
            })
        })
        .map(|inner| PyIppClient {
            inner: Arc::new(inner),
        })
        .map_err(to_py_err)
    }

    fn start_session(&self, py: Python<'_>) -> PyResult<PyResponse> {
        self.block_on(py, |c| async move {
            c.send_command(Command::StartSession).await
        })
    }

    fn end_session(&self, py: Python<'_>) -> PyResult<PyResponse> {
        self.block_on(
            py,
            |c| async move { c.send_command(Command::EndSession).await },
        )
    }

    fn get_dme_version(&self, py: Python<'_>) -> PyResult<PyResponse> {
        self.block_on(py, |c| async move {
            c.send_command(Command::GetDmeVersion).await
        })
    }

    fn home(&self, py: Python<'_>) -> PyResult<PyResponse> {
        self.block_on(py, |c| async move { c.send_command(Command::Home).await })
    }

    fn go_to(&self, py: Python<'_>, x: f64, y: f64, z: f64) -> PyResult<PyResponse> {
        self.block_on(py, move |c| async move {
            c.send_command(Command::go_to(x, y, z)?).await
        })
    }

    fn pt_meas(&self, py: Python<'_>) -> PyResult<PyResponse> {
        self.block_on(
            py,
            |c| async move { c.send_command(Command::pt_meas()).await },
        )
    }

    fn set_coord_system(&self, py: Python<'_>, coord_system: &str) -> PyResult<PyResponse> {
        let cs = parse_coord_system(coord_system)?;
        self.block_on(py, move |c| async move {
            c.send_command(Command::SetCoordSystem(cs)).await
        })
    }

    /// Send a raw I++ DME term, e.g. `"OnMoveArc(...)"`, for commands not yet
    /// covered by a dedicated method.
    fn send_raw(&self, py: Python<'_>, term: &str) -> PyResult<PyResponse> {
        let term =
            ippdme_core::parse_term_str(term).map_err(|e| PyValueError::new_err(e.to_string()))?;
        self.block_on(py, move |c| async move { c.send(term).await })
    }
}
