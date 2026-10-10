//! Telegram Inbound Watcher and Remote Command Daemon
//! Polls Telegram Bot API for remote commands, cluster snapshot queries,
//! Chat ID auto-discovery, and dispatches instructions into local execution or Supabase.
#![allow(dead_code)]

pub mod cluster;
pub mod commands;
pub mod config;
pub mod daemon;
pub mod detection;
pub mod formatting;
pub mod injection;
pub mod media;
pub mod processing;
pub mod reports_a;
pub mod reports_b;
pub mod reports_cluster_a;
pub mod reports_cluster_b;
pub mod tests;
pub mod types;

pub use cluster::*;
pub use commands::*;
pub use config::*;
pub use daemon::*;
pub use detection::*;
pub use formatting::*;
pub use injection::*;
pub use media::*;
pub use processing::*;
pub use reports_a::*;
pub use reports_b::*;
pub use reports_cluster_a::*;
pub use reports_cluster_b::*;
pub use tests::*;
pub use types::*;
