use base64::prelude::*;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::*;

// ---------------------------------------------------------------------------
// Notification Recipients CRUD
// ---------------------------------------------------------------------------

/// List notification recipients
pub fn list_notify_recipients() -> Result<Vec<NotifyRecipient>, String> {
    let conn = connect_vault_db()?;
    let mut stmt = conn
        .prepare("SELECT id, email, group_name, is_active, created_at FROM notify_recipients ORDER BY created_at ASC")
        .map_err(|e| format!("Failed to prepare list recipients: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            let act_int: i32 = row.get(3)?;
            Ok(NotifyRecipient {
                id: row.get(0)?,
                email: row.get(1)?,
                group_name: row.get(2)?,
                is_active: act_int > 0,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("Failed to query notify recipients: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}

/// Normalize an email address for duplicate comparison (trim, lowercase).
/// This is the single normalization point for notification recipients (plan 88).
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Outcome of an upsert-or-skip recipient add (plan 88, subtask 001).
#[derive(Debug, Clone)]
pub enum AddRecipientOutcome {
    /// A new row was inserted.
    Added(NotifyRecipient),
    /// A row with the same normalized email already existed and was returned
    /// unchanged (no group or active-flag overwrite).
    AlreadyExists(NotifyRecipient),
}

impl AddRecipientOutcome {
    /// The resulting recipient, whether newly added or pre-existing.
    pub fn recipient(&self) -> &NotifyRecipient {
        match self {
            AddRecipientOutcome::Added(r) => r,
            AddRecipientOutcome::AlreadyExists(r) => r,
        }
    }

    /// True when a new row was inserted.
    pub fn is_added(&self) -> bool {
        matches!(self, AddRecipientOutcome::Added(_))
    }
}

fn row_to_recipient(row: &rusqlite::Row) -> rusqlite::Result<NotifyRecipient> {
    let act_int: i32 = row.get(3)?;
    Ok(NotifyRecipient {
        id: row.get(0)?,
        email: row.get(1)?,
        group_name: row.get(2)?,
        is_active: act_int > 0,
        created_at: row.get(4)?,
    })
}

/// Find the stored recipient for a normalized email, if any (oldest row wins).
pub fn find_notify_recipient_by_normalized_email(
    conn: &Connection,
    normalized_email: &str,
) -> Result<Option<NotifyRecipient>, String> {
    conn.query_row(
        "SELECT id, email, group_name, is_active, created_at FROM notify_recipients \
         WHERE LOWER(TRIM(email)) = ?1 ORDER BY created_at ASC, rowid ASC LIMIT 1",
        params![normalized_email],
        row_to_recipient,
    )
    .optional()
    .map_err(|e| format!("Failed to check existing notify recipient: {}", e))
}

/// Upsert-or-skip core, on an explicit connection (plan 88, subtask 001).
/// If a recipient with the same normalized email exists, it is returned
/// unchanged as `AlreadyExists`; otherwise a new row is inserted.
pub fn add_notify_recipient_with_conn(
    conn: &Connection,
    input: NotifyRecipientInput,
) -> Result<AddRecipientOutcome, String> {
    let clean_email = input.email.trim().to_string();
    let normalized = normalize_email(&clean_email);

    if let Some(existing) = find_notify_recipient_by_normalized_email(conn, &normalized)? {
        return Ok(AddRecipientOutcome::AlreadyExists(existing));
    }

    let group = input.group_name.unwrap_or_else(|| "default".to_string());
    let is_active = input.is_active.unwrap_or(true);
    let act_int = if is_active { 1 } else { 0 };
    let now = Utc::now().timestamp();
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO notify_recipients (id, email, group_name, is_active, created_at) VALUES (?, ?, ?, ?, ?)",
        params![&id, &clean_email, &group, act_int, now],
    )
    .map_err(|e| format!("Failed to add notify recipient: {}", e))?;

    Ok(AddRecipientOutcome::Added(NotifyRecipient {
        id,
        email: clean_email,
        group_name: group,
        is_active,
        created_at: now,
    }))
}

/// Add a notification recipient, skipping duplicates by normalized email.
/// Returns the new row, or the existing row unchanged when the address is
/// already present. Use `add_notify_recipient_outcome` when the caller needs
/// to distinguish the two cases.
pub fn add_notify_recipient(input: NotifyRecipientInput) -> Result<NotifyRecipient, String> {
    Ok(add_notify_recipient_outcome(input)?.recipient().clone())
}

/// Add a notification recipient and report whether it was added or skipped
/// as a duplicate (plan 88, subtask 001).
pub fn add_notify_recipient_outcome(
    input: NotifyRecipientInput,
) -> Result<AddRecipientOutcome, String> {
    let conn = connect_vault_db()?;
    add_notify_recipient_with_conn(&conn, input)
}

/// A group of stored recipients that share one normalized email (plan 88, subtask 003).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateRecipientGroup {
    /// Normalized (trimmed, lowercased) email shared by the group.
    pub normalized_email: String,
    /// Number of stored rows in the group.
    pub row_count: usize,
    /// Id of the row a merge would keep (oldest row).
    pub keeper_id: String,
}

/// Find duplicate recipient groups: normalized emails with more than one row.
pub fn find_duplicate_recipients() -> Result<Vec<DuplicateRecipientGroup>, String> {
    let conn = connect_vault_db()?;
    find_duplicate_recipients_with_conn(&conn)
}

/// Find duplicate recipient groups on an explicit connection.
pub fn find_duplicate_recipients_with_conn(
    conn: &Connection,
) -> Result<Vec<DuplicateRecipientGroup>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT LOWER(TRIM(email)) AS norm, COUNT(*) AS cnt, \
                    (SELECT id FROM notify_recipients r2 WHERE LOWER(TRIM(r2.email)) = LOWER(TRIM(r.email)) \
                     ORDER BY r2.created_at ASC, r2.rowid ASC LIMIT 1) AS keeper \
             FROM notify_recipients r GROUP BY norm HAVING cnt > 1 ORDER BY norm ASC",
        )
        .map_err(|e| format!("Failed to prepare duplicate recipient scan: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(DuplicateRecipientGroup {
                normalized_email: row.get(0)?,
                row_count: row.get::<_, i64>(1)? as usize,
                keeper_id: row.get(2)?,
            })
        })
        .map_err(|e| format!("Failed to scan duplicate recipients: {}", e))?;
    let mut groups = Vec::new();
    for row in rows {
        groups.push(
            row.map_err(|e| format!("Failed to read a duplicate recipient group: {}", e))?,
        );
    }
    Ok(groups)
}

/// Merge duplicate recipients (plan 88, subtask 003): for each normalized
/// email keep the oldest row, set it active if any duplicate was active,
/// delete the rest in one transaction, then ensure the unique index exists.
/// Returns the number of rows deleted.
pub fn merge_duplicate_recipients() -> Result<usize, String> {
    let mut conn = connect_vault_db()?;
    merge_duplicate_recipients_with_conn(&mut conn)
}

/// Merge duplicate recipients on an explicit connection.
pub fn merge_duplicate_recipients_with_conn(conn: &mut Connection) -> Result<usize, String> {
    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start dedupe transaction: {}", e))?;

    // A row is a "keeper" when no older row (by created_at, then rowid)
    // shares its normalized email. Prefer is_active = 1 on the keeper when
    // any duplicate in its group is active.
    tx.execute(
        "UPDATE notify_recipients SET is_active = 1 \
         WHERE is_active = 0 \
           AND NOT EXISTS (SELECT 1 FROM notify_recipients older \
                           WHERE LOWER(TRIM(older.email)) = LOWER(TRIM(notify_recipients.email)) \
                             AND (older.created_at < notify_recipients.created_at \
                                  OR (older.created_at = notify_recipients.created_at \
                                      AND older.rowid < notify_recipients.rowid))) \
           AND EXISTS (SELECT 1 FROM notify_recipients dup \
                       WHERE LOWER(TRIM(dup.email)) = LOWER(TRIM(notify_recipients.email)) \
                         AND dup.is_active = 1)",
        [],
    )
    .map_err(|e| format!("Failed to promote active flag on duplicate keepers: {}", e))?;

    // Delete every non-keeper row (any row that has an older sibling).
    let deleted = tx
        .execute(
            "DELETE FROM notify_recipients \
             WHERE EXISTS (SELECT 1 FROM notify_recipients older \
                           WHERE LOWER(TRIM(older.email)) = LOWER(TRIM(notify_recipients.email)) \
                             AND (older.created_at < notify_recipients.created_at \
                                  OR (older.created_at = notify_recipients.created_at \
                                      AND older.rowid < notify_recipients.rowid)))",
            [],
        )
        .map_err(|e| format!("Failed to delete duplicate recipients: {}", e))?;

    tx.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_notify_recipients_email_lower ON notify_recipients (LOWER(TRIM(email)))",
        [],
    )
    .map_err(|e| format!("Failed to create unique recipient index: {}", e))?;

    tx.commit()
        .map_err(|e| format!("Failed to commit dedupe transaction: {}", e))?;
    Ok(deleted)
}

/// Delete notification recipient
pub fn delete_notify_recipient(id: &str) -> Result<(), String> {
    let conn = connect_vault_db()?;
    conn.execute("DELETE FROM notify_recipients WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete notify recipient: {}", e))?;
    Ok(())
}
