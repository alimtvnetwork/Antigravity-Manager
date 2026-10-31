//! Database schema introspection service for Split-SQLite databases.
//! Queries table catalogs, column types, constraints, and indices via SQLite pragmas.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumnInfo {
    pub cid: i64,
    pub name: String,
    pub data_type: String,
    pub is_not_null: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableIndexInfo {
    pub seq: i64,
    pub name: String,
    pub is_unique: bool,
    pub origin: String,
    pub is_partial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchemaInfo {
    pub table_name: String,
    pub sql: Option<String>,
    pub columns: Vec<TableColumnInfo>,
    pub indices: Vec<TableIndexInfo>,
}

/// Lists all user tables present in the connected SQLite database.
pub fn list_user_tables(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master 
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
         ORDER BY name ASC",
    )?;

    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut tables = Vec::new();
    for r in rows {
        tables.push(r?);
    }
    Ok(tables)
}

/// Inspects the full schema for a given table including column definitions and indexes.
pub fn inspect_table_schema(conn: &Connection, table_name: &str) -> Result<TableSchemaInfo> {
    // 1. Get CREATE TABLE sql
    let sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table_name],
            |row| row.get(0),
        )
        .ok();

    // 2. Query PRAGMA table_info(table_name)
    let pragma_table_info = format!("PRAGMA table_info('{}')", table_name.replace('\'', "''"));
    let mut stmt_cols = conn.prepare(&pragma_table_info)?;
    let col_rows = stmt_cols.query_map([], |row| {
        Ok(TableColumnInfo {
            cid: row.get(0)?,
            name: row.get(1)?,
            data_type: row.get(2)?,
            is_not_null: row.get::<_, i64>(3)? != 0,
            default_value: row.get(4)?,
            is_primary_key: row.get::<_, i64>(5)? != 0,
        })
    })?;

    let mut columns = Vec::new();
    for c in col_rows {
        columns.push(c?);
    }

    // 3. Query PRAGMA index_list(table_name)
    let pragma_index_list = format!("PRAGMA index_list('{}')", table_name.replace('\'', "''"));
    let mut indices = Vec::new();
    if let Ok(mut stmt_idx) = conn.prepare(&pragma_index_list) {
        if let Ok(idx_rows) = stmt_idx.query_map([], |row| {
            Ok(TableIndexInfo {
                seq: row.get(0)?,
                name: row.get(1)?,
                is_unique: row.get::<_, i64>(2)? != 0,
                origin: row.get(3)?,
                is_partial: row.get::<_, i64>(4)? != 0,
            })
        }) {
            for idx in idx_rows.flatten() {
                indices.push(idx);
            }
        }
    }

    Ok(TableSchemaInfo {
        table_name: table_name.to_string(),
        sql,
        columns,
        indices,
    })
}

/// Inspects all tables and schemas across the entire database.
pub fn inspect_all_schemas(conn: &Connection) -> Result<Vec<TableSchemaInfo>> {
    let tables = list_user_tables(conn)?;
    let mut schemas = Vec::with_capacity(tables.len());
    for tbl in tables {
        schemas.push(inspect_table_schema(conn, &tbl)?);
    }
    Ok(schemas)
}
