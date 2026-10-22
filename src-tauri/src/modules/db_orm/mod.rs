//! Type-safe Rust ORM framework and Split-DB generator abstractions.

pub mod builder;
pub mod connection;
pub mod dump;
pub mod fk_resolver;
pub mod generated;
pub mod highwater;
pub mod index_health;
pub mod json_extract;
pub mod lock_checker;
pub mod migration_gate;
pub mod pagination;
pub mod pool_limiter;
pub mod pragmas;
pub mod row_size;
pub mod safety_guard;
pub mod schema_inspector;
pub mod statistics;
pub mod temp_store;
pub mod traits;
pub mod truncate;
pub mod vacuum_backup;
pub mod wal_watchdog;

#[cfg(test)]
pub mod tests;

pub use builder::{Order, QueryBuilder, ViewQueryBuilder};
pub use connection::{
    init_split_db_schema, open_diesel_split_db, open_split_db, open_split_db_readonly,
    verify_split_db_integrity,
};
pub use dump::*;
pub use fk_resolver::*;
pub use generated::*;
pub use highwater::*;
pub use index_health::*;
pub use json_extract::*;
pub use lock_checker::*;
pub use migration_gate::*;
pub use pagination::*;
pub use pool_limiter::*;
pub use pragmas::*;
pub use row_size::*;
pub use safety_guard::*;
pub use schema_inspector::*;
pub use statistics::*;
pub use temp_store::*;
pub use traits::{Entity, FromRow, ToValues, ValueParam, ViewEntity};
pub use truncate::*;
pub use vacuum_backup::*;
pub use wal_watchdog::*;
