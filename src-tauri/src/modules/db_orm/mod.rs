//! Type-safe Rust ORM framework and Split-DB generator abstractions.

pub mod builder;
pub mod busy_retry;
pub mod connection;
pub mod dump;
pub mod fk_resolver;
pub mod foreign_keys;
pub mod generated;
pub mod highwater;
pub mod index_health;
pub mod json_extract;
pub mod lock_checker;
pub mod migration_gate;
pub mod pagination;
pub mod pool_limiter;
pub mod pragmas;
pub mod read_sandbox;
pub mod row_size;
pub mod safety_guard;
pub mod schema_inspector;
pub mod statistics;
pub mod table_profiler;
pub mod temp_store;
pub mod traits;
pub mod truncate;
pub mod vacuum_backup;
pub mod vacuum_stepper;
pub mod version_drift;
pub mod wal_watchdog;

#[cfg(test)]
pub mod tests;

pub use builder::{Order, QueryBuilder, ViewQueryBuilder};
pub use busy_retry::*;
pub use connection::{
    init_split_db_schema, open_diesel_split_db, open_split_db, open_split_db_readonly,
    verify_split_db_integrity,
};
pub use dump::*;
pub use fk_resolver::*;
pub use foreign_keys::*;
pub use generated::*;
pub use highwater::*;
pub use index_health::*;
pub use json_extract::*;
pub use lock_checker::*;
pub use migration_gate::*;
pub use pagination::*;
pub use pool_limiter::*;
pub use pragmas::*;
pub use read_sandbox::*;
pub use row_size::*;
pub use safety_guard::*;
pub use schema_inspector::*;
pub use statistics::*;
pub use table_profiler::*;
pub use temp_store::*;
pub use traits::{Entity, FromRow, ToValues, ValueParam, ViewEntity};
pub use truncate::*;
pub use vacuum_backup::*;
pub use vacuum_stepper::*;
pub use version_drift::*;
pub use wal_watchdog::*;
