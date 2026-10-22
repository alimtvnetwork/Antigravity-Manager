//! Generic SQLite table row pagination engine with offset and cursor mechanics.
//! Computes total page counts, navigation state, and extracts generic JSON row dictionaries.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    pub page: usize,
    pub page_size: usize,
    pub sort_column: Option<String>,
    pub sort_direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse<T> {
    pub data: Vec<T>,
    pub total_rows: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Executes a paginated query against any SQLite table, returning JSON row representations.
pub fn query_paginated_table(
    conn: &Connection,
    table_name: &str,
    req: PageRequest,
) -> Result<PageResponse<Value>> {
    let sanitized_table = table_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "");
    
    // 1. Calculate total row count
    let count_sql = format!("SELECT COUNT(*) FROM \"{}\"", sanitized_table);
    let total_rows: usize = conn.query_row(&count_sql, [], |r| r.get(0))?;

    let page = req.page.max(1);
    let page_size = req.page_size.clamp(1, 500);
    let total_pages = if total_rows == 0 {
        1
    } else {
        (total_rows + page_size - 1) / page_size
    };

    let offset = (page - 1) * page_size;

    // 2. Build ordering clause
    let order_clause = if let Some(ref col) = req.sort_column {
        let clean_col = col.replace(|c: char| !c.is_alphanumeric() && c != '_', "");
        let dir = req
            .sort_direction
            .as_deref()
            .unwrap_or("ASC")
            .to_ascii_uppercase();
        let valid_dir = if dir == "DESC" { "DESC" } else { "ASC" };
        format!("ORDER BY \"{}\" {}", clean_col, valid_dir)
    } else {
        "ORDER BY 1 ASC".to_string()
    };

    // 3. Query paginated rows
    let query_sql = format!(
        "SELECT * FROM \"{}\" {} LIMIT {} OFFSET {}",
        sanitized_table, order_clause, page_size, offset
    );

    let mut stmt = conn.prepare(&query_sql)?;
    let col_names: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows_data = Vec::new();
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let mut obj = Map::new();
        for (i, col_name) in col_names.iter().enumerate() {
            let val = match row.get_ref(i)? {
                rusqlite::types::ValueRef::Null => Value::Null,
                rusqlite::types::ValueRef::Integer(v) => Value::from(v),
                rusqlite::types::ValueRef::Real(v) => Value::from(v),
                rusqlite::types::ValueRef::Text(v) => {
                    Value::from(String::from_utf8_lossy(v).to_string())
                }
                rusqlite::types::ValueRef::Blob(v) => {
                    Value::from(format!("<blob {} bytes>", v.len()))
                }
            };
            obj.insert(col_name.clone(), val);
        }
        rows_data.push(Value::Object(obj));
    }

    Ok(PageResponse {
        data: rows_data,
        total_rows,
        page,
        page_size,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    })
}
