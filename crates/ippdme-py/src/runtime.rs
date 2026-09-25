use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// A single shared Tokio runtime backing all synchronous-looking Python
/// calls: each method blocks on it after releasing the GIL.
pub fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("failed to start Tokio runtime for ippdme"))
}
