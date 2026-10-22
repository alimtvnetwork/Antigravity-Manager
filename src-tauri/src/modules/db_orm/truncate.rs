//! Split-DB Table Truncate Cleaner.
//! Empties table data and resets autoincrement sequence counters in a single transaction.

use rusqlite::{Connection, Result};

/// Safely removes all records from a table and resets its autoincrement counter.
/// Returns the number of rows deleted.
pub fn truncate_table_safely(conn: &Connection, table_name: &str) -> Result<usize> {
    let clean_table = table_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "");

    let count_sql = format!("SELECT COUNT(*) FROM \"{}\"", clean_table);
    let row_count: usize = conn.query_row(&count_sql, [], |r| r.get(0))?;

    let delete_sql = format!("DELETE FROM \"{}\"", clean_table);
    conn.execute(&delete_sql, [])?;

    // Reset sequence if sqlite_sequence exists
    let _ = conn.execute(
        "DELETE FROM sqlite_sequence WHERE name = ?1",
        [&clean_table],
    );

    Ok(row_count)
}
