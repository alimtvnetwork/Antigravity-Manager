//! Foreign Key Navigation Resolver for Split-SQLite databases.
//! Identifies outbound and inbound foreign key dependencies across relational tables.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    pub id: i64,
    pub seq: i64,
    pub target_table: String,
    pub from_column: String,
    pub to_column: String,
    pub on_update: String,
    pub on_delete: String,
    pub match_type: String,
}

/// Resolves outbound foreign keys defined on a specific table using PRAGMA foreign_key_list.
pub fn resolve_table_foreign_keys(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<ForeignKeyRef>> {
    let pragma_sql = format!(
        "PRAGMA foreign_key_list('{}')",
        table_name.replace('\'', "''")
    );
    let mut stmt = conn.prepare(&pragma_sql)?;

    let rows = stmt.query_map([], |row| {
        Ok(ForeignKeyRef {
            id: row.get(0)?,
            seq: row.get(1)?,
            target_table: row.get(2)?,
            from_column: row.get(3)?,
            to_column: row.get(4)?,
            on_update: row.get(5)?,
            on_delete: row.get(6)?,
            match_type: row.get(7)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// Identifies all inbound foreign keys from other tables pointing to `target_table`.
pub fn find_incoming_foreign_keys(
    conn: &Connection,
    target_table: &str,
) -> Result<Vec<(String, ForeignKeyRef)>> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master 
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
         ORDER BY name ASC",
    )?;

    let all_tables: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .filter_map(Result::ok)
        .collect();

    let mut incoming = Vec::new();
    let target_lower = target_table.to_ascii_lowercase();

    for tbl in all_tables {
        if tbl.eq_ignore_ascii_case(target_table) {
            continue;
        }

        let fks = resolve_table_foreign_keys(conn, &tbl)?;
        for fk in fks {
            if fk.target_table.to_ascii_lowercase() == target_lower {
                incoming.push((tbl.clone(), fk));
            }
        }
    }

    Ok(incoming)
}
