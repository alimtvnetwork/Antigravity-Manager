// Serializable Read-Uncommitted Guard (Imp-527)
// Enforces PRAGMA read_uncommitted = 0 on SQLite connections to guarantee
// that dirty reads cannot occur across concurrent worker transactions.

use rusqlite::Connection;
use tracing::debug;

pub struct ReadUncommittedGuard;

impl ReadUncommittedGuard {
    /// Enforces read_uncommitted = 0 (strict serializable isolation) on the connection.
    pub fn enforce_strict_isolation(conn: &Connection) -> Result<bool, rusqlite::Error> {
        conn.execute("PRAGMA read_uncommitted = 0;", [])?;

        let mut stmt = conn.prepare("PRAGMA read_uncommitted;")?;
        let is_uncommitted: i32 = stmt.query_row([], |row| row.get(0))?;

        let is_strict = is_uncommitted == 0;
        debug!(is_strict, "Verified SQLite read_uncommitted = 0 pragma");
        Ok(is_strict)
    }
}
