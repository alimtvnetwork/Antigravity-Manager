//! Transaction Rollback Guard Utility.
//! RAII guard ensuring automatic ROLLBACK if a multi-statement transaction is dropped before explicit commit.

use rusqlite::{Connection, Result};

/// RAII transaction guard that automatically issues ROLLBACK unless committed.
pub struct RollbackGuard<'a> {
    conn: &'a Connection,
    is_committed: bool,
}

impl<'a> RollbackGuard<'a> {
    /// Begins an immediate transaction and wraps it in a rollback guard.
    pub fn begin(conn: &'a Connection) -> Result<Self> {
        conn.execute_batch("BEGIN IMMEDIATE;")?;
        Ok(Self {
            conn,
            is_committed: false,
        })
    }

    /// Commits the active transaction, disarming the rollback guard.
    pub fn commit(mut self) -> Result<()> {
        self.conn.execute_batch("COMMIT;")?;
        self.is_committed = true;
        Ok(())
    }

    /// Explicitly aborts the transaction.
    pub fn rollback(mut self) -> Result<()> {
        self.conn.execute_batch("ROLLBACK;")?;
        self.is_committed = true;
        Ok(())
    }
}

impl<'a> Drop for RollbackGuard<'a> {
    fn drop(&mut self) {
        if !self.is_committed {
            let _ = self.conn.execute_batch("ROLLBACK;");
        }
    }
}
