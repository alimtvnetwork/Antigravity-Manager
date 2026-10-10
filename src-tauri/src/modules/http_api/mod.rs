//! HTTP API Module
//! Provides local HTTP interfaces for external programs (e.g., VS Code extension) to call.

pub mod handlers;
pub mod server;
pub mod settings;
pub mod types;

pub use handlers::*;
pub use server::*;
pub use settings::*;
pub use types::*;
