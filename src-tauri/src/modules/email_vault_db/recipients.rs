use base64::prelude::*;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
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

/// Add notification recipient
pub fn add_notify_recipient(input: NotifyRecipientInput) -> Result<NotifyRecipient, String> {
    let conn = connect_vault_db()?;
    let id = Uuid::new_v4().to_string();
    let group = input.group_name.unwrap_or_else(|| "default".to_string());
    let is_active = input.is_active.unwrap_or(true);
    let act_int = if is_active { 1 } else { 0 };
    let now = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO notify_recipients (id, email, group_name, is_active, created_at) VALUES (?, ?, ?, ?, ?)",
        params![&id, &input.email, &group, act_int, now],
    )
    .map_err(|e| format!("Failed to add notify recipient: {}", e))?;

    Ok(NotifyRecipient {
        id,
        email: input.email,
        group_name: group,
        is_active,
        created_at: now,
    })
}

/// Delete notification recipient
pub fn delete_notify_recipient(id: &str) -> Result<(), String> {
    let conn = connect_vault_db()?;
    conn.execute("DELETE FROM notify_recipients WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete notify recipient: {}", e))?;
    Ok(())
}
