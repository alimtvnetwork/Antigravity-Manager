//! Type-safe Rust ORM framework and Split-DB generator abstractions.

pub mod builder;
pub mod connection;
pub mod generated;
pub mod traits;

#[cfg(test)]
pub mod tests;

pub use builder::{Order, QueryBuilder, ViewQueryBuilder};
pub use connection::{
    init_split_db_schema, open_diesel_split_db, open_split_db, open_split_db_readonly,
    verify_split_db_integrity,
};
pub use generated::*;
pub use traits::{Entity, FromRow, ToValues, ValueParam, ViewEntity};
