use ippdme_core::IppError;
use ippdme_net::NetError;
use pyo3::exceptions::{PyConnectionError, PyTimeoutError, PyValueError};
use pyo3::PyErr;

pub fn to_py_err(e: NetError) -> PyErr {
    match e {
        NetError::Timeout(_) => PyTimeoutError::new_err(e.to_string()),
        NetError::ConnectionClosed | NetError::Io(_) | NetError::Shutdown => {
            PyConnectionError::new_err(e.to_string())
        }
        NetError::Protocol(IppError::InvalidArgument { .. }) => {
            PyValueError::new_err(e.to_string())
        }
        NetError::Tls(_) | NetError::Protocol(_) => PyConnectionError::new_err(e.to_string()),
    }
}
