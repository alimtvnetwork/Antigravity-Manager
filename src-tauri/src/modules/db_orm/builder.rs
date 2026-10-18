//! Type-safe fluent query builder for SQLite ORM operations.

use crate::error::AppError;
use crate::modules::db_orm::traits::{Entity, FromRow, ToValues, ValueParam, ViewEntity};
use rusqlite::Connection;
use std::marker::PhantomData;

/// Sort order for query builder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Order::Asc => "ASC",
            Order::Desc => "DESC",
        }
    }
}

/// Where condition representation
#[derive(Debug, Clone)]
enum WhereClause {
    Eq(&'static str, ValueParam),
    In(&'static str, Vec<ValueParam>),
    Raw(String),
}

/// Fluent type-safe query builder for table entities
pub struct QueryBuilder<T: Entity + FromRow> {
    conditions: Vec<WhereClause>,
    order_bys: Vec<(&'static str, Order)>,
    limit_val: Option<usize>,
    offset_val: Option<usize>,
    _phantom: PhantomData<T>,
}

impl<T: Entity + FromRow> Default for QueryBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Entity + FromRow> QueryBuilder<T> {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            order_bys: Vec::new(),
            limit_val: None,
            offset_val: None,
            _phantom: PhantomData,
        }
    }

    pub fn select() -> Self {
        Self::new()
    }

    pub fn where_eq(mut self, column: &'static str, value: impl Into<ValueParam>) -> Self {
        self.conditions.push(WhereClause::Eq(column, value.into()));
        self
    }

    pub fn where_in<V: Into<ValueParam>>(mut self, column: &'static str, values: Vec<V>) -> Self {
        if !values.is_empty() {
            let params: Vec<ValueParam> = values.into_iter().map(Into::into).collect();
            self.conditions.push(WhereClause::In(column, params));
        }
        self
    }

    pub fn where_raw(mut self, sql: impl Into<String>) -> Self {
        self.conditions.push(WhereClause::Raw(sql.into()));
        self
    }

    pub fn order_by(mut self, column: &'static str, order: Order) -> Self {
        self.order_bys.push((column, order));
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit_val = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.offset_val = Some(n);
        self
    }

    fn build_sql(&self) -> (String, Vec<ValueParam>) {
        let mut sql = format!("SELECT * FROM {}", T::TABLE_NAME);
        let mut params = Vec::new();

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            let mut cond_strs = Vec::new();
            for c in &self.conditions {
                match c {
                    WhereClause::Eq(col, val) => {
                        cond_strs.push(format!("{} = ?", col));
                        params.push(val.clone());
                    }
                    WhereClause::In(col, vals) => {
                        let placeholders = vec!["?"; vals.len()].join(", ");
                        cond_strs.push(format!("{} IN ({})", col, placeholders));
                        params.extend(vals.clone());
                    }
                    WhereClause::Raw(raw) => {
                        cond_strs.push(raw.clone());
                    }
                }
            }
            sql.push_str(&cond_strs.join(" AND "));
        }

        if !self.order_bys.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_strs: Vec<String> = self
                .order_bys
                .iter()
                .map(|(col, ord)| format!("{} {}", col, ord.as_sql()))
                .collect();
            sql.push_str(&order_strs.join(", "));
        }

        if let Some(l) = self.limit_val {
            sql.push_str(&format!(" LIMIT {}", l));
        }

        if let Some(o) = self.offset_val {
            sql.push_str(&format!(" OFFSET {}", o));
        }

        (sql, params)
    }

    pub fn fetch_all(&self, conn: &Connection) -> Result<Vec<T>, AppError> {
        let (sql, params) = self.build_sql();
        let mut stmt = conn.prepare(&sql).map_err(AppError::Database)?;

        let rusqlite_params: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let rows = stmt
            .query_map(rusqlite_params.as_slice(), |row| {
                T::from_row(row).map_err(|e| match e {
                    AppError::Database(rusqlite_err) => rusqlite_err,
                    other => rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            other.to_string(),
                        )),
                    ),
                })
            })
            .map_err(AppError::Database)?;

        let mut results = Vec::new();
        for r in rows {
            let entity = r.map_err(AppError::Database)?;
            results.push(entity);
        }
        Ok(results)
    }

    pub fn fetch_one(&self, conn: &Connection) -> Result<Option<T>, AppError> {
        let limited = Self {
            conditions: self.conditions.clone(),
            order_bys: self.order_bys.clone(),
            limit_val: Some(1),
            offset_val: self.offset_val,
            _phantom: PhantomData,
        };
        let items = limited.fetch_all(conn)?;
        Ok(items.into_iter().next())
    }

    /// Find single entity by primary key
    pub fn find_by_id(conn: &Connection, id: &T::IdType) -> Result<Option<T>, AppError> {
        let sql = format!(
            "SELECT * FROM {} WHERE {} = ? LIMIT 1",
            T::TABLE_NAME,
            T::PRIMARY_KEY
        );
        let mut stmt = conn.prepare(&sql).map_err(AppError::Database)?;

        let mut rows = stmt
            .query_map([id], |row| {
                T::from_row(row).map_err(|e| match e {
                    AppError::Database(rusqlite_err) => rusqlite_err,
                    other => rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            other.to_string(),
                        )),
                    ),
                })
            })
            .map_err(AppError::Database)?;

        match rows.next() {
            Some(res) => Ok(Some(res.map_err(AppError::Database)?)),
            None => Ok(None),
        }
    }

    /// Count all records in the table
    pub fn count(conn: &Connection) -> Result<i64, AppError> {
        let sql = format!("SELECT COUNT(*) FROM {}", T::TABLE_NAME);
        let count: i64 = conn
            .query_row(&sql, [], |row| row.get(0))
            .map_err(AppError::Database)?;
        Ok(count)
    }

    /// Delete record by primary key
    pub fn delete_by_id(conn: &Connection, id: &T::IdType) -> Result<bool, AppError> {
        let sql = format!("DELETE FROM {} WHERE {} = ?", T::TABLE_NAME, T::PRIMARY_KEY);
        let affected = conn.execute(&sql, [id]).map_err(AppError::Database)?;
        Ok(affected > 0)
    }

    /// Insert entity into database
    pub fn insert(conn: &Connection, entity: &T) -> Result<(), AppError>
    where
        T: ToValues,
    {
        let pairs = entity.to_values();
        let cols: Vec<&str> = pairs.iter().map(|(c, _)| *c).collect();
        let placeholders = vec!["?"; cols.len()].join(", ");
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            T::TABLE_NAME,
            cols.join(", "),
            placeholders
        );

        let params: Vec<&dyn rusqlite::ToSql> = pairs
            .iter()
            .map(|(_, val)| val as &dyn rusqlite::ToSql)
            .collect();

        conn.execute(&sql, params.as_slice())
            .map_err(AppError::Database)?;
        Ok(())
    }

    /// Update entity in database by primary key
    pub fn update(conn: &Connection, entity: &T) -> Result<(), AppError>
    where
        T: ToValues,
    {
        let pairs = entity.to_values();
        // Exclude primary key from SET clause
        let set_pairs: Vec<(&&str, &ValueParam)> = pairs
            .iter()
            .filter(|(col, _)| *col != T::PRIMARY_KEY)
            .map(|(c, v)| (c, v))
            .collect();

        let set_clause = set_pairs
            .iter()
            .map(|(col, _)| format!("{} = ?", col))
            .collect::<Vec<String>>()
            .join(", ");

        let sql = format!(
            "UPDATE {} SET {} WHERE {} = ?",
            T::TABLE_NAME,
            set_clause,
            T::PRIMARY_KEY
        );

        let mut params: Vec<&dyn rusqlite::ToSql> = set_pairs
            .iter()
            .map(|(_, val)| *val as &dyn rusqlite::ToSql)
            .collect();

        let pk_val = entity.id();
        params.push(pk_val as &dyn rusqlite::ToSql);

        conn.execute(&sql, params.as_slice())
            .map_err(AppError::Database)?;
        Ok(())
    }
}

/// Fluent type-safe query builder for readonly views
pub struct ViewQueryBuilder<V: ViewEntity + FromRow> {
    conditions: Vec<WhereClause>,
    order_bys: Vec<(&'static str, Order)>,
    limit_val: Option<usize>,
    offset_val: Option<usize>,
    _phantom: PhantomData<V>,
}

impl<V: ViewEntity + FromRow> Default for ViewQueryBuilder<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: ViewEntity + FromRow> ViewQueryBuilder<V> {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            order_bys: Vec::new(),
            limit_val: None,
            offset_val: None,
            _phantom: PhantomData,
        }
    }

    pub fn select() -> Self {
        Self::new()
    }

    pub fn where_eq(mut self, column: &'static str, value: impl Into<ValueParam>) -> Self {
        self.conditions.push(WhereClause::Eq(column, value.into()));
        self
    }

    pub fn where_in<VVal: Into<ValueParam>>(
        mut self,
        column: &'static str,
        values: Vec<VVal>,
    ) -> Self {
        if !values.is_empty() {
            let params: Vec<ValueParam> = values.into_iter().map(Into::into).collect();
            self.conditions.push(WhereClause::In(column, params));
        }
        self
    }

    pub fn where_raw(mut self, sql: impl Into<String>) -> Self {
        self.conditions.push(WhereClause::Raw(sql.into()));
        self
    }

    pub fn order_by(mut self, column: &'static str, order: Order) -> Self {
        self.order_bys.push((column, order));
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit_val = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.offset_val = Some(n);
        self
    }

    fn build_sql(&self) -> (String, Vec<ValueParam>) {
        let mut sql = format!("SELECT * FROM {}", V::VIEW_NAME);
        let mut params = Vec::new();

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            let mut cond_strs = Vec::new();
            for c in &self.conditions {
                match c {
                    WhereClause::Eq(col, val) => {
                        cond_strs.push(format!("{} = ?", col));
                        params.push(val.clone());
                    }
                    WhereClause::In(col, vals) => {
                        let placeholders = vec!["?"; vals.len()].join(", ");
                        cond_strs.push(format!("{} IN ({})", col, placeholders));
                        params.extend(vals.clone());
                    }
                    WhereClause::Raw(raw) => {
                        cond_strs.push(raw.clone());
                    }
                }
            }
            sql.push_str(&cond_strs.join(" AND "));
        }

        if !self.order_bys.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_strs: Vec<String> = self
                .order_bys
                .iter()
                .map(|(col, ord)| format!("{} {}", col, ord.as_sql()))
                .collect();
            sql.push_str(&order_strs.join(", "));
        }

        if let Some(l) = self.limit_val {
            sql.push_str(&format!(" LIMIT {}", l));
        }

        if let Some(o) = self.offset_val {
            sql.push_str(&format!(" OFFSET {}", o));
        }

        (sql, params)
    }

    pub fn fetch_all(&self, conn: &Connection) -> Result<Vec<V>, AppError> {
        let (sql, params) = self.build_sql();
        let mut stmt = conn.prepare(&sql).map_err(AppError::Database)?;

        let rusqlite_params: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let rows = stmt
            .query_map(rusqlite_params.as_slice(), |row| {
                V::from_row(row).map_err(|e| match e {
                    AppError::Database(rusqlite_err) => rusqlite_err,
                    other => rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            other.to_string(),
                        )),
                    ),
                })
            })
            .map_err(AppError::Database)?;

        let mut results = Vec::new();
        for r in rows {
            let item = r.map_err(AppError::Database)?;
            results.push(item);
        }
        Ok(results)
    }

    pub fn fetch_one(&self, conn: &Connection) -> Result<Option<V>, AppError> {
        let limited = Self {
            conditions: self.conditions.clone(),
            order_bys: self.order_bys.clone(),
            limit_val: Some(1),
            offset_val: self.offset_val,
            _phantom: PhantomData,
        };
        let items = limited.fetch_all(conn)?;
        Ok(items.into_iter().next())
    }
}
