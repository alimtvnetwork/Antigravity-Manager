//! Request/response payload audit helpers.
//!
//! - Header serialization with API-key redaction
//! - Session / thinking markers are preserved for ops comparison
//! - Optional concise storage mode to keep SQLite small

use axum::http::HeaderMap;
use serde_json::{json, Map, Value};

mod fields;
mod headers;
mod payload;
mod simplify;
mod tests;

pub use fields::*;
pub use headers::*;
pub use payload::*;
pub use simplify::*;
pub use tests::*;
