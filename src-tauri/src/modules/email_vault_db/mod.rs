//! Split Email Vault Database Module
//! Dedicated SQLite Database for email accounts, separate credentials vault,
//! notification recipients, watcher settings, and inbound audit logs.
#![allow(dead_code)]

pub mod accounts;
pub mod audit;
pub mod crypto;
pub mod db;
pub mod recipients;
pub mod settings;
pub mod tests;
pub mod types;

pub use accounts::*;
pub use audit::*;
pub use crypto::*;
pub use db::*;
pub use recipients::*;
pub use settings::*;
pub use tests::*;
pub use types::*;
