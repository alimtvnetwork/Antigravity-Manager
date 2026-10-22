//! PRAGMA Compile-Options Introspection Scanner.
//! Queries runtime SQLite compile-time feature flags (`ENABLE_FTS5`, `ENABLE_JSON1`, `THREADSAFE`).

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

/// Report of runtime SQLite engine compilation flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteEngineCapabilities {
    pub compile_options: Vec<String>,
    pub has_json1: bool,
    pub has_fts5: bool,
    pub is_threadsafe: bool,
}

/// Queries `PRAGMA compile_options` to discover runtime SQLite engine capabilities.
pub fn inspect_sqlite_capabilities(conn: &Connection) -> Result<SqliteEngineCapabilities> {
    let mut stmt = conn.prepare("PRAGMA compile_options")?;
    let options: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    let has_json1 = options.iter().any(|o| o.contains("ENABLE_JSON1"));
    let has_fts5 = options.iter().any(|o| o.contains("ENABLE_FTS5"));
    let is_threadsafe = options
        .iter()
        .any(|o| o.starts_with("THREADSAFE=") && !o.ends_with("=0"));

    Ok(SqliteEngineCapabilities {
        compile_options: options,
        has_json1,
        has_fts5,
        is_threadsafe,
    })
}
