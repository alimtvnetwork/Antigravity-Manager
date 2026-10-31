//! SQLite JSON Column Extractor Helper.
//! Extracts structured nested values from JSON text columns using SQLite's native json_extract.

use rusqlite::{Connection, Result};

/// Queries a specific JSON path across rows of a JSON-containing SQLite column.
pub fn query_json_property(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    json_path: &str,
    limit: usize,
) -> Result<Vec<Option<String>>> {
    let clean_table = table_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "");
    let clean_col = column_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "");
    let bound_limit = if limit == 0 { 50 } else { limit.min(1000) };

    let formatted_path = if json_path.starts_with('$') {
        json_path.to_string()
    } else {
        format!("$.{}", json_path)
    };

    let query_sql = format!(
        "SELECT json_extract(\"{}\", ?1) FROM \"{}\" WHERE \"{}\" IS NOT NULL LIMIT ?2",
        clean_col, clean_table, clean_col
    );

    let mut stmt = conn.prepare(&query_sql)?;
    let rows = stmt.query_map([formatted_path, bound_limit.to_string()], |row| {
        row.get::<_, Option<String>>(0)
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }

    Ok(results)
}
