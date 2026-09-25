//! `ippdme-core`: pure protocol parser, AST, and serializer for the I++ DME
//! ASCII protocol used to command Coordinate Measuring Machines (CMMs).
//!
//! This crate has no I/O; see `ippdme-net` for the async TCP client and mock
//! server built on top of it.

pub mod ast;
pub mod commands;
pub mod error;
pub mod parser;
pub mod response;

pub use ast::{Marker, Message, Tag, Term};
pub use commands::{Command, CoordSystem, CsyTransform, CsyTransformKind, Point};
pub use error::{IppError, Result};
pub use parser::{parse_message, parse_term_str};
