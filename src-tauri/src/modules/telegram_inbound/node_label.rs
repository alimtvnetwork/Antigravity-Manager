//! Display names for Telegram machine buttons and headers.
//!
//! A generated `Node-<digits>` alias is resolved through `gitmap ssh nodes export-json`
//! and cached in SQLite until `TelegramConfig.node_name_refresh_hours` elapses.
//! Any other alias is already a proper name and is cached without spawning GitMap.

use crate::modules::ssh_manager::SshNodeExportItem;
use serde_json::Value;

/// True when `alias` is an auto-generated `Node-` plus digits label.
pub fn is_generated_node_alias(alias: &str) -> bool {
    let alias = alias.trim();
    let bytes = alias.as_bytes();
    if bytes.len() < 6 {
        return false;
    }
    if !bytes[..5].eq_ignore_ascii_case(b"Node-") {
        return false;
    }
    bytes[5..].iter().all(|b| b.is_ascii_digit())
}

/// True when `alias` addresses the machine running this bot.
pub fn is_current_machine(alias: &str) -> bool {
    let alias = alias.trim();
    if alias.eq_ignore_ascii_case("local") || alias.eq_ignore_ascii_case("localhost") {
        return true;
    }
    let fleet = super::open_shot::local_machine_label();
    if alias.eq_ignore_ascii_case(fleet.trim()) {
        return true;
    }
    let detected = crate::modules::email_watcher::detect_machine_name();
    alias.eq_ignore_ascii_case(detected.trim())
}

/// Button label: `This machine` on this host, otherwise the cached proper name.
pub fn machine_button_text(alias: &str) -> String {
    if is_current_machine(alias) {
        "This machine".to_string()
    } else {
        display_name_for(alias)
    }
}

/// HTML header. The current machine uses the green circle; others use the desktop mark.
pub fn machine_header_html(alias: &str) -> String {
    if is_current_machine(alias) {
        "\u{1F7E2} <b>This machine</b>".to_string()
    } else {
        let name = escape_html(&display_name_for(alias));
        format!("\u{1F5A5}\u{FE0F} <b>{name}</b>")
    }
}

/// Inline-keyboard button. `style` is included only when the caller supplies one
/// (`success`, `primary`, or `danger`).
pub fn button_json(text: &str, callback_data: &str, style: Option<&str>) -> Value {
    let mut obj = serde_json::json!({
        "text": text,
        "callback_data": callback_data,
    });
    if let Some(style) = style {
        obj["style"] = Value::String(style.to_string());
    }
    obj
}

/// Proper name for `alias`. Non-generated aliases skip GitMap. Generated aliases
/// reuse SQLite until the refresh window elapses, then query GitMap once.
pub fn display_name_for(alias: &str) -> String {
    let alias = alias.trim();
    if alias.is_empty() {
        return String::new();
    }
    if !is_generated_node_alias(alias) {
        remember(alias, alias);
        return alias.to_string();
    }
    if let Some(cached) = fresh_cached_name(alias) {
        return cached;
    }
    let resolved = match export_gitmap_nodes() {
        Ok(items) => pick_better_alias(alias, &items).unwrap_or_else(|| fallback_name(alias)),
        Err(e) => {
            // Justification: a failed GitMap lookup must not block the Telegram label; the OS name or original alias is still shown.
            crate::error::record_ignored(
                Err::<(), String>(e),
                "gitmap ssh nodes export-json for node display name",
            );
            fallback_name(alias)
        }
    };
    remember(alias, &resolved);
    resolved
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn refresh_window_secs() -> i64 {
    let hours = match super::config::load_config() {
        Ok(cfg) => cfg.node_name_refresh_hours,
        Err(e) => {
            // Justification: the display-name TTL falls back to 24 hours when the Telegram config cannot be read.
            crate::error::record_ignored(
                Err::<(), _>(e),
                "load telegram config for node name refresh hours",
            );
            super::types::default_node_name_refresh_hours()
        }
    };
    i64::try_from(hours.saturating_mul(3600)).unwrap_or(i64::MAX)
}

fn fallback_name(alias: &str) -> String {
    let detected = crate::modules::email_watcher::detect_machine_name();
    if !detected.trim().is_empty() && !is_generated_node_alias(&detected) {
        detected
    } else {
        alias.to_string()
    }
}

fn pick_better_alias(requested: &str, items: &[SshNodeExportItem]) -> Option<String> {
    let exact = items
        .iter()
        .find(|item| item.alias.eq_ignore_ascii_case(requested));
    if let Some(item) = exact {
        if !is_generated_node_alias(&item.alias) && !item.alias.trim().is_empty() {
            return Some(item.alias.clone());
        }
    }
    if !is_generated_node_alias(requested) {
        return None;
    }
    let ip = exact
        .map(|item| item.ip_address.trim())
        .filter(|ip| !ip.is_empty());
    let Some(ip) = ip else {
        return None;
    };
    items.iter().find_map(|item| {
        if item.alias.trim().is_empty() || is_generated_node_alias(&item.alias) {
            return None;
        }
        if item.ip_address.eq_ignore_ascii_case(ip) {
            Some(item.alias.clone())
        } else {
            None
        }
    })
}

fn export_gitmap_nodes() -> Result<Vec<SshNodeExportItem>, String> {
    let mut cmd = std::process::Command::new("gitmap");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let out = cmd
        .args(["ssh", "nodes", "export-json"])
        .output()
        .map_err(|e| format!("run gitmap ssh nodes export-json: {e}"))?;
    if !out.status.success() {
        let code = out
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "terminated".to_string());
        return Err(format!("gitmap ssh nodes export-json exited {code}"));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    parse_export_items(&stdout)
}

fn parse_export_items(stdout: &str) -> Result<Vec<SshNodeExportItem>, String> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err("empty gitmap ssh nodes export".to_string());
    }
    let parsed: Value =
        serde_json::from_str(trimmed).map_err(|e| format!("parse gitmap ssh nodes json: {e}"))?;
    let (attrs_opt, unwrapped) = crate::modules::json_envelope::unpack_envelope(parsed.clone());
    let effective = if attrs_opt.is_some() {
        unwrapped
    } else {
        parsed
    };

    let mut items = Vec::new();
    if let Some(main) = effective.get("mainMachine") {
        if let Ok(item) = serde_json::from_value::<SshNodeExportItem>(main.clone()) {
            items.push(item);
        }
    }
    if let Some(nodes) = effective.get("nodes").and_then(|n| n.as_array()) {
        for node in nodes {
            if let Ok(item) = serde_json::from_value::<SshNodeExportItem>(node.clone()) {
                items.push(item);
            }
        }
    }
    if !items.is_empty() {
        return Ok(items);
    }
    serde_json::from_value::<Vec<SshNodeExportItem>>(effective)
        .map_err(|e| format!("unrecognized gitmap ssh nodes json: {e}"))
}

fn cache_path() -> Result<std::path::PathBuf, String> {
    let data_dir = crate::modules::account::get_data_dir()?;
    Ok(data_dir.join("node_display_cache.db"))
}

fn open_cache() -> Result<rusqlite::Connection, String> {
    let path = cache_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create node cache dir: {e}"))?;
    }
    let conn =
        rusqlite::Connection::open(&path).map_err(|e| format!("open node display cache: {e}"))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS node_display_cache (
            node_key TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );",
    )
    .map_err(|e| format!("create node_display_cache: {e}"))?;
    Ok(conn)
}

fn fresh_cached_name(alias: &str) -> Option<String> {
    let window = refresh_window_secs();
    match read_cache(alias) {
        Ok(Some((name, updated_at))) => {
            let now = chrono::Utc::now().timestamp();
            let age = now.saturating_sub(updated_at);
            if age <= window {
                Some(name)
            } else {
                None
            }
        }
        Ok(None) => None,
        Err(e) => {
            // Justification: a missing or unreadable cache row falls through to a live GitMap resolve.
            crate::error::record_ignored(Err::<(), String>(e), "read node display cache");
            None
        }
    }
}

fn read_cache(alias: &str) -> Result<Option<(String, i64)>, String> {
    let conn = open_cache()?;
    let mut stmt = conn
        .prepare(
            "SELECT display_name, updated_at FROM node_display_cache
             WHERE node_key = ?1 COLLATE NOCASE",
        )
        .map_err(|e| format!("prepare node display cache read: {e}"))?;
    let mut rows = stmt
        .query(rusqlite::params![alias])
        .map_err(|e| format!("query node display cache: {e}"))?;
    if let Some(row) = rows
        .next()
        .map_err(|e| format!("read node display cache row: {e}"))?
    {
        let name: String = row
            .get(0)
            .map_err(|e| format!("read cached display name: {e}"))?;
        let updated_at: i64 = row
            .get(1)
            .map_err(|e| format!("read cached updated_at: {e}"))?;
        return Ok(Some((name, updated_at)));
    }
    Ok(None)
}

fn remember(key: &str, display_name: &str) {
    // Justification: the display cache is opportunistic; a write failure still returns the resolved name.
    crate::error::record_ignored(
        upsert_display(key, display_name),
        "upsert node display cache",
    );
}

fn upsert_display(key: &str, display_name: &str) -> Result<(), String> {
    let conn = open_cache()?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO node_display_cache (node_key, display_name, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(node_key) DO UPDATE SET
            display_name = excluded.display_name,
            updated_at = excluded.updated_at",
        rusqlite::params![key, display_name, now],
    )
    .map_err(|e| format!("upsert node display cache: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_style_and_plain_alias_skip_gitmap() {
        let button = button_json("This machine", "send:m:0", Some("success"));
        assert_eq!(button["text"], "This machine");
        assert_eq!(button["callback_data"], "send:m:0");
        assert_eq!(button["style"], "success");
        assert!(button_json("Next", "send:next:0", None)
            .get("style")
            .is_none());

        assert!(!is_generated_node_alias("studio-pc"));
        assert!(is_generated_node_alias("Node-823632"));
        assert!(is_generated_node_alias("node-12"));
        assert!(!is_generated_node_alias("Node-"));
        assert!(!is_generated_node_alias("Node-12a"));

        assert_eq!(display_name_for("studio-pc"), "studio-pc");
    }
}
