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

/// Add or update notification recipient (deduplicated case-insensitively by email)
pub fn add_notify_recipient(input: NotifyRecipientInput) -> Result<NotifyRecipient, String> {
    let conn = connect_vault_db()?;
    let clean_email = input.email.trim().to_string();
    let group = input.group_name.unwrap_or_else(|| "default".to_string());
    let is_active = input.is_active.unwrap_or(true);
    let act_int = if is_active { 1 } else { 0 };
    let now = Utc::now().timestamp();

    let existing: Option<(String, i64)> = conn
        .query_row(
            "SELECT id, created_at FROM notify_recipients WHERE LOWER(TRIM(email)) = LOWER(TRIM(?1)) ORDER BY created_at ASC LIMIT 1",
            params![&clean_email],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| format!("Failed to check existing notify recipient: {}", e))?;

    if let Some((existing_id, created_at)) = existing {
        conn.execute(
            "UPDATE notify_recipients SET email = ?1, group_name = ?2, is_active = ?3 WHERE id = ?4",
            params![&clean_email, &group, act_int, &existing_id],
        )
        .map_err(|e| format!("Failed to update notify recipient: {}", e))?;

        // Clean up any historical duplicate rows for the same normalized email
        // Justification: best-effort deduplication cleanup
        crate::error::record_ignored(
            conn.execute(
                "DELETE FROM notify_recipients WHERE LOWER(TRIM(email)) = LOWER(TRIM(?1)) AND id != ?2",
                params![&clean_email, &existing_id],
            ),
            "dedupe notify_recipients",
        );

        return Ok(NotifyRecipient {
            id: existing_id,
            email: clean_email,
            group_name: group,
            is_active,
            created_at,
        });
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO notify_recipients (id, email, group_name, is_active, created_at) VALUES (?, ?, ?, ?, ?)",
        params![&id, &clean_email, &group, act_int, now],
    )
    .map_err(|e| format!("Failed to add notify recipient: {}", e))?;

    Ok(NotifyRecipient {
        id,
        email: clean_email,
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
