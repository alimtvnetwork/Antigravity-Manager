use base64::prelude::*;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use super::*;

// ---------------------------------------------------------------------------
// Notification Settings CRUD
// ---------------------------------------------------------------------------

pub(crate) fn map_settings_row(r: &rusqlite::Row) -> rusqlite::Result<EmailNotificationSettings> {
    let is_en: i32 = r.get(1)?;
    let p_int: u32 = r.get(2)?;
    let in_int: u32 = r.get(3)?;
    let base_p: u32 = r.get(4).unwrap_or(4);
    let act_s: u32 = r.get(5).unwrap_or(10);
    let n_quota: i32 = r.get(6)?;
    let q_drop: u32 = r.get(7)?;
    let n_ws: i32 = r.get(8)?;
    let n_idle: i32 = r.get(9)?;
    let a_prompt: i32 = r.get(10)?;
    let a_cli: i32 = r.get(11)?;
    let a_inst: i32 = r.get(12)?;
    let n_update: i32 = r.get(16).unwrap_or(1);
    Ok(EmailNotificationSettings {
        id: r.get(0)?,
        is_enabled: is_en > 0,
        polling_interval_minutes: p_int,
        inbox_check_interval_minutes: in_int,
        baseline_polling_interval_minutes: base_p,
        active_awaiting_interval_seconds: act_s,
        notify_on_quota_drop: n_quota > 0,
        quota_drop_threshold_percent: q_drop,
        notify_on_workspace_switch: n_ws > 0,
        notify_on_idle_workspace: n_idle > 0,
        notify_on_system_update: n_update > 0,
        allow_remote_prompt_execution: a_prompt > 0,
        allow_remote_cli_execution: a_cli > 0,
        allow_remote_instance_rotation: a_inst > 0,
        local_machine_name: r.get(13)?,
        local_machine_ip: r.get(14)?,
        updated_at: r.get(15)?,
    })
}

/// Load notification settings
pub fn get_notification_settings() -> Result<EmailNotificationSettings, String> {
    let conn = connect_vault_db()?;
    let sql = "SELECT id, is_enabled, polling_interval_minutes, inbox_check_interval_minutes,
                      baseline_polling_interval_minutes, active_awaiting_interval_seconds,
                      notify_on_quota_drop, quota_drop_threshold_percent, notify_on_workspace_switch,
                      notify_on_idle_workspace, allow_remote_prompt_execution, allow_remote_cli_execution,
                      allow_remote_instance_rotation, local_machine_name, local_machine_ip, updated_at,
                      notify_on_system_update
               FROM email_notification_settings WHERE id = 'global'";
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Query prepare failed: {}", e))?;
    let row = stmt
        .query_row([], map_settings_row)
        .optional()
        .map_err(|e| format!("Failed to query notification settings: {}", e))?;
    Ok(row.unwrap_or_default())
}

/// Save notification settings
pub fn save_notification_settings(settings: EmailNotificationSettings) -> Result<(), String> {
    let conn = connect_vault_db()?;
    let now = Utc::now().timestamp();

    let is_en = if settings.is_enabled { 1 } else { 0 };
    let n_quota = if settings.notify_on_quota_drop { 1 } else { 0 };
    let n_ws = if settings.notify_on_workspace_switch {
        1
    } else {
        0
    };
    let n_idle = if settings.notify_on_idle_workspace {
        1
    } else {
        0
    };
    let a_prompt = if settings.allow_remote_prompt_execution {
        1
    } else {
        0
    };
    let a_cli = if settings.allow_remote_cli_execution {
        1
    } else {
        0
    };
    let a_inst = if settings.allow_remote_instance_rotation {
        1
    } else {
        0
    };
    let n_update = if settings.notify_on_system_update {
        1
    } else {
        0
    };

    conn.execute(
        "INSERT INTO email_notification_settings
         (id, is_enabled, polling_interval_minutes, inbox_check_interval_minutes,
          baseline_polling_interval_minutes, active_awaiting_interval_seconds,
          notify_on_quota_drop, quota_drop_threshold_percent, notify_on_workspace_switch,
          notify_on_idle_workspace, allow_remote_prompt_execution, allow_remote_cli_execution,
          allow_remote_instance_rotation, local_machine_name, local_machine_ip, updated_at,
          notify_on_system_update)
         VALUES ('global', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            is_enabled = excluded.is_enabled,
            polling_interval_minutes = excluded.polling_interval_minutes,
            inbox_check_interval_minutes = excluded.inbox_check_interval_minutes,
            baseline_polling_interval_minutes = excluded.baseline_polling_interval_minutes,
            active_awaiting_interval_seconds = excluded.active_awaiting_interval_seconds,
            notify_on_quota_drop = excluded.notify_on_quota_drop,
            quota_drop_threshold_percent = excluded.quota_drop_threshold_percent,
            notify_on_workspace_switch = excluded.notify_on_workspace_switch,
            notify_on_idle_workspace = excluded.notify_on_idle_workspace,
            allow_remote_prompt_execution = excluded.allow_remote_prompt_execution,
            allow_remote_cli_execution = excluded.allow_remote_cli_execution,
            allow_remote_instance_rotation = excluded.allow_remote_instance_rotation,
            local_machine_name = excluded.local_machine_name,
            local_machine_ip = excluded.local_machine_ip,
            updated_at = excluded.updated_at,
            notify_on_system_update = excluded.notify_on_system_update",
        params![
            is_en,
            settings.polling_interval_minutes,
            settings.inbox_check_interval_minutes,
            settings.baseline_polling_interval_minutes,
            settings.active_awaiting_interval_seconds,
            n_quota,
            settings.quota_drop_threshold_percent,
            n_ws,
            n_idle,
            a_prompt,
            a_cli,
            a_inst,
            &settings.local_machine_name,
            &settings.local_machine_ip,
            now,
            n_update,
        ],
    )
    .map_err(|e| format!("Failed to save notification settings: {}", e))?;

    Ok(())
}
