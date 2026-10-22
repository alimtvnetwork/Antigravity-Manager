//! Type-safe Rust ORM framework and Split-DB generator abstractions.

pub mod builder;
pub mod connection;
pub mod dump;
pub mod fk_resolver;
pub mod generated;
pub mod pagination;
pub mod safety_guard;
pub mod schema_inspector;
pub mod statistics;
pub mod traits;

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
pub use pagination::*;
pub use safety_guard::*;
pub use schema_inspector::*;
pub use statistics::*;
pub use traits::{Entity, FromRow, ToValues, ValueParam, ViewEntity};
