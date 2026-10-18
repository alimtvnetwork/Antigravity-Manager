//! Core traits for the type-safe Rust ORM framework.

use crate::error::AppError;
use rusqlite::types::{ToSql, ToSqlOutput};
use rusqlite::Row;

/// Value parameter for parameterized queries
#[derive(Debug, Clone, PartialEq)]
pub enum ValueParam {
    Text(String),
    Int(i64),
    Float(f64),
    Null,
}

impl ToSql for ValueParam {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            ValueParam::Text(s) => Ok(ToSqlOutput::from(s.as_str())),
            ValueParam::Int(i) => Ok(ToSqlOutput::from(*i)),
            ValueParam::Float(f) => Ok(ToSqlOutput::from(*f)),
            ValueParam::Null => Ok(ToSqlOutput::from(rusqlite::types::Null)),
        }
    }
}

impl From<String> for ValueParam {
    fn from(s: String) -> Self {
        ValueParam::Text(s)
    }
}

impl From<&str> for ValueParam {
    fn from(s: &str) -> Self {
        ValueParam::Text(s.to_string())
    }
}

impl From<i64> for ValueParam {
    fn from(i: i64) -> Self {
        ValueParam::Int(i)
    }
}

impl From<i32> for ValueParam {
    fn from(i: i32) -> Self {
        ValueParam::Int(i as i64)
    }
}

impl From<bool> for ValueParam {
    fn from(b: bool) -> Self {
        ValueParam::Int(if b { 1 } else { 0 })
    }
}

impl From<f64> for ValueParam {
    fn from(f: f64) -> Self {
        ValueParam::Float(f)
    }
}

impl<T: Into<ValueParam>> From<Option<T>> for ValueParam {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => ValueParam::Null,
        }
    }
}

/// Core trait representing an ORM table entity
pub trait Entity: Sized + 'static {
    const TABLE_NAME: &'static str;
    const PRIMARY_KEY: &'static str;
    type IdType: std::fmt::Display + ToSql;

    fn id(&self) -> &Self::IdType;
}

/// Trait for deserializing an entity from a SQLite row
pub trait FromRow: Sized {
    fn from_row(row: &Row) -> Result<Self, AppError>;
}

/// Trait for extracting column names and values from an entity for insert/update operations
pub trait ToValues {
    fn to_columns() -> &'static [&'static str];
    fn to_values(&self) -> Vec<(&'static str, ValueParam)>;
}

/// Core trait representing a readonly database view
pub trait ViewEntity: Sized + 'static {
    const VIEW_NAME: &'static str;
    const DB_FILE: &'static str;
    const DDL: &'static str;
}
