//! Supabase Synchronization & Node Registry Module
//! Manages local node identity, heartbeat pings, and instance profile state sync.
#![allow(dead_code)]

pub mod config_a;
pub mod config_b;
pub mod fleet;
pub mod fleet_types;
pub mod migration;
pub mod node;
pub mod query;
pub mod sync;
pub mod tests;
pub mod types;

pub use config_a::*;
pub use config_b::*;
pub use fleet::*;
pub use fleet_types::*;
pub use migration::*;
pub use node::*;
pub use query::*;
pub use sync::*;
pub use tests::*;
pub use types::*;
