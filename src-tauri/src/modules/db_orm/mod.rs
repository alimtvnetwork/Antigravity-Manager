//! Type-safe Rust ORM framework and Split-DB generator abstractions.

pub mod builder;
pub mod busy_handler;
pub mod busy_retry;
pub mod cache_size_tuner;
pub mod compile_options;
pub mod connection;
pub mod connection_pool_metrics;
pub mod dump;
pub mod fk_resolver;
pub mod foreign_keys;
pub mod fragmentation;
pub mod generated;
pub mod highwater;
pub mod index_health;
pub mod index_usage_inspector;
pub mod json_extract;
pub mod lock_checker;
pub mod lock_detector;
pub mod migration_checksum;
pub mod migration_gate;
pub mod pagination;
pub mod pool_limiter;
pub mod pragmas;
pub mod read_sandbox;
pub mod rollback_guard;
pub mod row_size;
pub mod safety_guard;
pub mod schema_inspector;
pub mod statistics;
pub mod table_profiler;
pub mod temp_store;
pub mod traits;
pub mod transaction_logger;
pub mod truncate;
pub mod vacuum_backup;
pub mod vacuum_optimizer;
pub mod vacuum_stepper;
pub mod version_drift;
pub mod wal_watchdog;
pub mod write_ahead_flush;

#[cfg(test)]
pub mod tests;

pub use builder::{Order, QueryBuilder, ViewQueryBuilder};
pub use busy_handler::*;
pub use busy_retry::*;
pub use cache_size_tuner::*;
pub use compile_options::*;
pub use connection::{
    init_split_db_schema, open_diesel_split_db, open_split_db, open_split_db_readonly,
    verify_split_db_integrity,
};
pub use connection_pool_metrics::*;
pub use dump::*;
pub use fk_resolver::*;
pub use foreign_keys::*;
pub use fragmentation::*;
pub use generated::*;
pub use highwater::*;
pub use index_health::*;
pub use index_usage_inspector::*;
pub use json_extract::*;
pub use lock_checker::*;
pub use lock_detector::*;
pub use migration_checksum::*;
pub use migration_gate::*;
pub use pagination::*;
pub use pool_limiter::*;
pub use pragmas::*;
pub use read_sandbox::*;
pub use rollback_guard::*;
pub use row_size::*;
pub use safety_guard::*;
pub use schema_inspector::*;
pub use statistics::*;
pub use table_profiler::*;
pub use temp_store::*;
pub use traits::{Entity, FromRow, ToValues, ValueParam, ViewEntity};
pub use transaction_logger::*;
pub use truncate::*;
pub use vacuum_backup::*;
pub use vacuum_optimizer::*;
pub use vacuum_stepper::*;
pub use version_drift::*;
pub use wal_watchdog::*;
pub use write_ahead_flush::*;
