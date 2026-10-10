//! JSON/state.vscdb merge utilities for instance copies.
use super::*;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Recursively merge source JSON into dest JSON object
pub fn deep_merge_json(dest: &mut serde_json::Value, source: &serde_json::Value) {
    match (dest, source) {
        (serde_json::Value::Object(dest_map), serde_json::Value::Object(source_map)) => {
            for (key, val) in source_map {
                if let Some(dest_val) = dest_map.get_mut(key) {
                    if dest_val.is_object() && val.is_object() {
                        deep_merge_json(dest_val, val);
                    } else {
                        *dest_val = val.clone();
                    }
                } else {
                    dest_map.insert(key.clone(), val.clone());
                }
            }
        }
        (dest_val, source_val) => {
            *dest_val = source_val.clone();
        }
    }
}

/// Synchronize opened paths list and workspace history in storage.json
pub fn merge_storage_json_recent_paths(from_inst: &InstanceConfig, to_inst: &InstanceConfig) {
    let mut src_storage_candidates = vec![PathBuf::from(&from_inst.data_dir)
        .join("User")
        .join("globalStorage")
        .join("storage.json")];
    #[cfg(target_os = "windows")]
    {
        src_storage_candidates.push(
            PathBuf::from(&from_inst.data_dir)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("storage.json"),
        );
        if let Ok(home) = get_instance_home_dir(&from_inst.id) {
            src_storage_candidates.push(
                home.join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage")
                    .join("storage.json"),
            );
        }
        if from_inst.is_default || from_inst.id == "default" {
            if let Ok(appdata) = std::env::var("APPDATA") {
                src_storage_candidates.push(
                    PathBuf::from(appdata)
                        .join("Antigravity")
                        .join("User")
                        .join("globalStorage")
                        .join("storage.json"),
                );
            }
        }
    }

    let src_storage_path = src_storage_candidates.into_iter().find(|p| p.is_file());
    let src_storage_val: Option<serde_json::Value> = src_storage_path.and_then(|p| {
        fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    });

    if let Some(src_val) = src_storage_val {
        let opened_paths = src_val.get("openedPathsList").cloned();
        let backup_workspaces = src_val.get("backupWorkspaces").cloned();

        let has_recent = opened_paths.is_some() || backup_workspaces.is_some();
        if has_recent {
            let mut dst_storage_targets = vec![PathBuf::from(&to_inst.data_dir)
                .join("User")
                .join("globalStorage")
                .join("storage.json")];
            #[cfg(target_os = "windows")]
            {
                dst_storage_targets.push(
                    PathBuf::from(&to_inst.data_dir)
                        .join("AppData")
                        .join("Roaming")
                        .join("Antigravity")
                        .join("User")
                        .join("globalStorage")
                        .join("storage.json"),
                );
                if let Ok(home) = get_instance_home_dir(&to_inst.id) {
                    dst_storage_targets.push(
                        home.join("AppData")
                            .join("Roaming")
                            .join("Antigravity")
                            .join("User")
                            .join("globalStorage")
                            .join("storage.json"),
                    );
                }
            }

            for dst_storage in dst_storage_targets {
                if let Some(parent) = dst_storage.parent() {
                    // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                    crate::error::record_ignored(fs::create_dir_all(parent), "create directory");
                }
                let mut dst_map: serde_json::Map<String, serde_json::Value> =
                    if dst_storage.exists() {
                        fs::read_to_string(&dst_storage)
                            .ok()
                            .and_then(|s| serde_json::from_str(&s).ok())
                            .and_then(|v: serde_json::Value| {
                                if let serde_json::Value::Object(m) = v {
                                    Some(m)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default()
                    } else {
                        serde_json::Map::new()
                    };

                if let Some(ref op) = opened_paths {
                    dst_map.insert("openedPathsList".to_string(), op.clone());
                }
                if let Some(ref bw) = backup_workspaces {
                    dst_map.insert("backupWorkspaces".to_string(), bw.clone());
                }

                if let Ok(pretty) =
                    serde_json::to_string_pretty(&serde_json::Value::Object(dst_map))
                {
                    // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                    crate::error::record_ignored(fs::write(dst_storage, pretty), "write file");
                }
            }
        }
    }
}

/// Synchronize recently opened paths and workspace history in state.vscdb SQLite tables
pub fn merge_state_vscdb_recent_paths(from_inst: &InstanceConfig, to_inst: &InstanceConfig) {
    let mut src_vscdb_candidates = vec![PathBuf::from(&from_inst.data_dir)
        .join("User")
        .join("globalStorage")
        .join("state.vscdb")];
    #[cfg(target_os = "windows")]
    {
        src_vscdb_candidates.push(
            PathBuf::from(&from_inst.data_dir)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb"),
        );
        if let Ok(home) = get_instance_home_dir(&from_inst.id) {
            src_vscdb_candidates.push(
                home.join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage")
                    .join("state.vscdb"),
            );
        }
        if from_inst.is_default || from_inst.id == "default" {
            if let Ok(appdata) = std::env::var("APPDATA") {
                src_vscdb_candidates.push(
                    PathBuf::from(appdata)
                        .join("Antigravity")
                        .join("User")
                        .join("globalStorage")
                        .join("state.vscdb"),
                );
            }
        }
    }

    let src_db_path = src_vscdb_candidates.into_iter().find(|p| p.is_file());
    if let Some(src_db) = src_db_path {
        let clean_src = src_db.to_string_lossy().replace('\\', "/");
        let uri_primary = format!(
            "file:///{}?mode=ro&immutable=1",
            clean_src.trim_start_matches('/')
        );
        let uri_alt = format!("file:{}?mode=ro&immutable=1", clean_src);

        let conn_src_res = rusqlite::Connection::open_with_flags(
            &uri_primary,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .or_else(|_| {
            rusqlite::Connection::open_with_flags(
                &uri_alt,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        })
        .or_else(|_| {
            rusqlite::Connection::open_with_flags(
                &src_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        });

        if let Ok(conn_src) = conn_src_res {
            // Justification: pragma is performance/concurrency tuning; the connection stays usable without it
            crate::error::record_ignored(
                conn_src.pragma_update(None, "busy_timeout", 2500),
                "set sqlite pragma",
            );
            let mut stmt = match conn_src.prepare(
                "SELECT key, value FROM ItemTable \
                 WHERE key LIKE '%recentlyOpened%' \
                    OR key LIKE '%history.%' \
                    OR key LIKE '%openedPaths%' \
                    OR key LIKE 'profileAssociations.%' \
                    OR key = 'workbench.colorTheme'",
            ) {
                Ok(s) => s,
                Err(_) => return,
            };

            let rows: Vec<(String, Vec<u8>)> = stmt
                .query_map([], |row| {
                    let k: String = row.get(0)?;
                    let v: Vec<u8> = row.get(1)?;
                    Ok((k, v))
                })
                .ok()
                .map(|mapped| mapped.flatten().collect())
                .unwrap_or_default();

            if rows.is_empty() {
                return;
            }

            let mut dst_vscdb_targets = vec![PathBuf::from(&to_inst.data_dir)
                .join("User")
                .join("globalStorage")
                .join("state.vscdb")];
            #[cfg(target_os = "windows")]
            {
                dst_vscdb_targets.push(
                    PathBuf::from(&to_inst.data_dir)
                        .join("AppData")
                        .join("Roaming")
                        .join("Antigravity")
                        .join("User")
                        .join("globalStorage")
                        .join("state.vscdb"),
                );
                if let Ok(home) = get_instance_home_dir(&to_inst.id) {
                    dst_vscdb_targets.push(
                        home.join("AppData")
                            .join("Roaming")
                            .join("Antigravity")
                            .join("User")
                            .join("globalStorage")
                            .join("state.vscdb"),
                    );
                }
            }

            for dst_db in dst_vscdb_targets {
                if let Some(parent) = dst_db.parent() {
                    // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                    crate::error::record_ignored(fs::create_dir_all(parent), "create directory");
                }
                if let Ok(conn_dst) = rusqlite::Connection::open(&dst_db) {
                    // Justification: pragma is performance/concurrency tuning; the connection stays usable without it
                    crate::error::record_ignored(
                        conn_dst.pragma_update(None, "busy_timeout", 2500),
                        "set sqlite pragma",
                    );
                    // Justification: non-critical sqlite bookkeeping write; read paths tolerate stale data
                    crate::error::record_ignored(
                        conn_dst.execute(
                            "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT UNIQUE ON CONFLICT REPLACE, value BLOB)",
                            [],
                        ),
                        "run sqlite statement",
                    );
                    for (k, v) in &rows {
                        // Justification: non-critical sqlite bookkeeping write; read paths tolerate stale data
                        crate::error::record_ignored(
                            conn_dst.execute(
                                "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
                                rusqlite::params![k, v],
                            ),
                            "run sqlite statement",
                        );
                    }
                }
            }
        }
    }
}

/// Purge opened paths list and recent project history from an instance's state stores
pub fn purge_recent_project_paths(inst: &InstanceConfig) {
    let mut storage_targets = vec![PathBuf::from(&inst.data_dir)
        .join("User")
        .join("globalStorage")
        .join("storage.json")];
    let mut vscdb_targets = vec![PathBuf::from(&inst.data_dir)
        .join("User")
        .join("globalStorage")
        .join("state.vscdb")];
    #[cfg(target_os = "windows")]
    {
        storage_targets.push(
            PathBuf::from(&inst.data_dir)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("storage.json"),
        );
        vscdb_targets.push(
            PathBuf::from(&inst.data_dir)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb"),
        );
        if let Ok(home) = get_instance_home_dir(&inst.id) {
            storage_targets.push(
                home.join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage")
                    .join("storage.json"),
            );
            vscdb_targets.push(
                home.join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage")
                    .join("state.vscdb"),
            );
        }
    }

    for st in storage_targets {
        if st.exists() {
            if let Ok(content) = fs::read_to_string(&st) {
                if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let serde_json::Value::Object(ref mut map) = val {
                        map.remove("openedPathsList");
                        map.remove("backupWorkspaces");
                        if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                            // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                            crate::error::record_ignored(fs::write(&st, pretty), "write file");
                        }
                    }
                }
            }
        }
    }

    for vt in vscdb_targets {
        if vt.exists() {
            if let Ok(conn) = rusqlite::Connection::open(&vt) {
                // Justification: non-critical sqlite bookkeeping write; read paths tolerate stale data
                crate::error::record_ignored(
                    conn.execute(
                        "DELETE FROM ItemTable WHERE key LIKE '%recentlyOpened%' OR key LIKE '%history.%' OR key LIKE '%openedPaths%'",
                        [],
                    ),
                    "run sqlite statement",
                );
            }
        }
    }
}
