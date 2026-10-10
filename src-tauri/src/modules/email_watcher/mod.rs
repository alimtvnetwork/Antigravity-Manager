//! Background Email Watcher Daemon Module
//! Captures machine telemetry, monitors low quota, detects idle running projects,
//! and polls inbound mailbox for remote execution commands.
#![allow(dead_code)]

pub mod control;
pub mod detection;
pub mod heartbeat;
pub mod sensors;
pub mod state;
pub mod tests;

pub use control::*;
pub use detection::*;
pub use heartbeat::*;
pub use sensors::*;
pub use state::*;
pub use tests::*;
