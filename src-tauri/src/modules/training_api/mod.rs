//! Machine Training REST API Engine
//!
//! Provides external endpoints for seeking into node telemetry, ingesting training/learning signals,
//! and remotely modifying machine and model routing states. Controlled via settings toggle.

pub mod db;
pub mod learning;
pub mod machines;
pub mod telemetry;

pub use db::*;
pub use learning::*;
pub use machines::*;
pub use telemetry::*;
