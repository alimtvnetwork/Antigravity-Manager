//! Single-click database snapshot dumper for Split-SQLite databases.
//! Exports schema DDL and row data as standard portable SQL scripts.

use rusqlite::{Connection, Result};
use std::fmt::Write as FmtWrite;

/// Exports an entire SQLite database into a self-contained SQL dump script.
pub fn export_database_to_sql_dump(conn: &Connection) -> Result<String> {
    let mut dump = String::new();
    let _ = writeln!(dump, "-- Antigravity-Manager Split-SQLite Export Dump");
    let _ = writeln!(dump, "-- Timestamp: {}", chrono::Utc::now().to_rfc3339());
    let _ = writeln!(dump, "PRAGMA foreign_keys=OFF;");
    let _ = writeln!(dump, "BEGIN TRANSACTION;\n");

    // 1. Get all tables in dependency order
    let mut stmt = conn.prepare(
        "SELECT name, sql FROM sqlite_master 
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
         ORDER BY name ASC",
    )?;

    let tables: Vec<(String, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(Result::ok)
        .collect();

    for (table_name, ddl_opt) in tables {
        if let Some(ddl) = ddl_opt {
            let _ = writeln!(dump, "{};", ddl.trim_end_matches(';'));
        }

        // Export data
        let select_sql = format!("SELECT * FROM \"{}\"", table_name.replace('"', "\"\""));
        let mut row_stmt = conn.prepare(&select_sql)?;
        let col_count = row_stmt.column_count();

        let mut rows = row_stmt.query([])?;
        while let Some(row) = rows.next()? {
            let mut val_strs = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let s = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => "NULL".to_string(),
                    rusqlite::types::ValueRef::Integer(v) => v.to_string(),
                    rusqlite::types::ValueRef::Real(v) => v.to_string(),
                    rusqlite::types::ValueRef::Text(t) => {
                        let text_val = String::from_utf8_lossy(t);
                        format!("'{}'", text_val.replace('\'', "''"))
                    }
                    rusqlite::types::ValueRef::Blob(b) => {
                        let hex_str: String = b.iter().map(|b| format!("{:02X}", b)).collect();
                        format!("X'{}'", hex_str)
                    }
                };
                val_strs.push(s);
            }

            let _ = writeln!(
                dump,
                "INSERT INTO \"{}\" VALUES ({});",
                table_name,
                val_strs.join(", ")
            );
        }
        dump.push('\n');
    }

    // 2. Export indices
    let mut idx_stmt = conn.prepare(
        "SELECT sql FROM sqlite_master 
         WHERE type = 'index' AND sql IS NOT NULL
         ORDER BY name ASC",
    )?;

    let indices: Vec<String> = idx_stmt
        .query_map([], |r| r.get(0))?
        .filter_map(Result::ok)
        .collect();

    for idx_sql in indices {
        let _ = writeln!(dump, "{};", idx_sql.trim_end_matches(';'));
    }

    let _ = writeln!(dump, "\nCOMMIT;");
    Ok(dump)
}
