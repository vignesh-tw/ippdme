//! `ippdme-imposter`: a Mountebank-style, stub-driven test double for the
//! I++ DME protocol, built on top of `ippdme-core` and `ippdme-net`.
//!
//! Bind an [`Imposter`] to a port, register [`Stub`]s (a [`Predicate`]
//! matching an incoming call, paired with a sequence of responses), and
//! `serve` it. Stubs can be built programmatically in Rust or loaded from a
//! YAML file — both go through the same [`Stub`]/[`Predicate`]/
//! [`ResponseSpec`] types, so the two configuration surfaces can never drift
//! apart. See `examples/imposter.yaml` for the YAML shape.

pub mod config;
pub mod error;
pub mod predicate;
pub mod response;
pub mod server;
pub mod stub;

pub use config::ImposterConfig;
pub use error::{ImposterError, Result};
pub use predicate::Predicate;
pub use response::{ResponseSpec, TimedResponse};
pub use server::{spawn_ephemeral, Imposter, ImposterBuilder, ImposterHandle};
pub use stub::{Stub, StubBuilder};
