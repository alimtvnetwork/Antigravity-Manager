//! UI interaction audit recorder (Task-01 of `auditing-and-cli-ui-integration`).
//!
//! Captures frontend / CLI / Telegram UI interactions into SQLite and exposes
//! start / stop / clear / export operations plus a recording-indicator query as
//! Tauri commands. This module is separate from
//! [`crate::modules::audit_action`] (the spec-104 instance audit trail).

mod commands;
mod recorder;
mod storage;
mod types;

pub use commands::*;
pub use recorder::*;
pub use storage::*;
pub use types::*;
