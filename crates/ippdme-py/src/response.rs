use ippdme_core::Message;
use pyo3::prelude::*;

/// A parsed response from a CMM/gateway: an `Ack`, `Error`, or `Data` message
/// correlated to the request that produced it.
#[pyclass(name = "Response")]
#[derive(Clone)]
pub struct PyResponse {
    inner: Message,
}

impl From<Message> for PyResponse {
    fn from(inner: Message) -> Self {
        PyResponse { inner }
    }
}

#[pymethods]
impl PyResponse {
    fn is_ack(&self) -> bool {
        self.inner.is_ack()
    }

    fn is_error(&self) -> bool {
        self.inner.is_error()
    }

    fn is_data(&self) -> bool {
        self.inner.is_data()
    }

    #[getter]
    fn tag(&self) -> u32 {
        self.inner.tag().0
    }

    /// The top-level function name of the response term, e.g. `"Ack"`,
    /// `"Error"`, or `"PtMeas"`.
    #[getter]
    fn name(&self) -> Option<String> {
        self.inner.term().name().map(str::to_string)
    }

    /// Read a numeric parameter, e.g. `response.get("X")` for
    /// `PtMeas(X(10.002), ...)`.
    fn get(&self, param: &str) -> Option<f64> {
        self.inner.term().get_num_param(param)
    }

    /// Read a nested identifier-valued parameter shaped like `Name(Value)`,
    /// e.g. a coordinate system tag inside a compound response.
    fn get_ident(&self, param: &str) -> Option<String> {
        self.inner
            .term()
            .get_ident_param(param)
            .map(str::to_string)
    }

    /// Read the first bare positional argument as an identifier, e.g. the
    /// `UnknownCommand` reason in `Error(UnknownCommand)`.
    #[getter]
    fn reason(&self) -> Option<String> {
        self.inner.term().args().first().and_then(|t| match t {
            ippdme_core::Term::Ident(s) => Some(s.clone()),
            _ => None,
        })
    }

    fn __repr__(&self) -> String {
        format!("Response({})", self.inner)
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }
}
