//! Helpers for cloning IDE trees, settings, and summaries.
use super::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Sanitize cloned conversation summaries in the target instance home and data directories.
/// Resets in-flight session flags (not_fully_idle = 0, status = 'IDLE') so a newly cloned
/// instance profile never inherits active running conversation states from the source.
pub fn sanitize_cloned_instance_summaries(target_id: &str) -> Result<(), String> {
    let mut candidates = Vec::new();

    if let Ok(home) = get_instance_home_dir(target_id) {
        candidates.push(
            home.join(".gemini")
                .join("antigravity")
                .join("conversation_summaries.db"),
        );
        candidates.push(
            home.join(".gemini")
                .join("antigravity-ide")
                .join("conversation_summaries.db"),
        );
        candidates.push(
            home.join(".gemini")
                .join("antigravity-cli")
                .join("conversation_summaries.db"),
        );
    }

    if let Ok(registry) = load_registry() {
        if let Some(inst) = registry.instances.iter().find(|i| i.id == target_id) {
            let data_dir = PathBuf::from(&inst.data_dir);
            candidates.push(
                data_dir
                    .join(".gemini")
                    .join("antigravity")
                    .join("conversation_summaries.db"),
            );
            candidates.push(
                data_dir
                    .join(".gemini")
                    .join("antigravity-ide")
                    .join("conversation_summaries.db"),
            );
            candidates.push(data_dir.join("conversation_summaries.db"));
        }
    }

    if let Ok(instances_dir) = get_instances_dir() {
        let instance_home_db = instances_dir
            .join(target_id)
            .join("home")
            .join(".gemini")
            .join("antigravity")
            .join("conversation_summaries.db");
        if !candidates.contains(&instance_home_db) {
            candidates.push(instance_home_db);
        }
    }

    for db_path in candidates {
        if db_path.exists() {
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                // Justification: pragma is performance/concurrency tuning; the connection stays usable without it
                crate::error::record_ignored(
                    conn.pragma_update(None, "busy_timeout", 3000),
                    "set sqlite pragma",
                );
                let has_table = conn
                    .query_row(
                        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='conversation_summaries'",
                        [],
                        |_| Ok(()),
                    )
                    .is_ok();
                if has_table {
                    let update_res = conn.execute(
                        "UPDATE conversation_summaries 
                         SET not_fully_idle = 0, status = 'IDLE' 
                         WHERE not_fully_idle != 0 OR status LIKE '%RUNNING%'",
                        [],
                    );
                    match update_res {
                        Ok(count) => {
                            if count > 0 {
                                crate::modules::logger::log_info(&format!(
                                    "[Instance] Sanitized {} active conversation summaries in {}",
                                    count,
                                    db_path.display()
                                ));
                            }
                        }
                        Err(e) => {
                            crate::modules::logger::log_warn(&format!(
                                "[Instance] Failed to sanitize conversation summaries in {}: {}",
                                db_path.display(),
                                e
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Rename an existing instance profile
pub fn rename_instance(instance_id: &str, new_name: String) -> Result<InstanceConfig, String> {
    let mut registry = load_registry()?;
    let trimmed = new_name.trim();
    let is_empty = trimmed.is_empty();
    if is_empty {
        return Err("Profile name cannot be empty".to_string());
    }

    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;

    registry.instances[pos].name = trimmed.to_string();
    let updated = registry.instances[pos].clone();
    save_registry(&registry)?;

    // Justification: settings injection is best-effort; the instance still launches with defaults
    crate::error::record_ignored(
        inject_instance_settings(&updated),
        "inject instance settings",
    );

    Ok(updated)
}

/// Settings and workspace databases that must survive a clone even if a later
/// file in the full tree is locked.
pub const REQUIRED_IDE_REL_PATHS: &[&str] = &[
    "User/settings.json",
    "User/keybindings.json",
    "User/security_presets.json",
    "User/antigravity_policies.json",
    "User/snippets",
    "User/globalStorage",
    "User/workspaceStorage",
];

/// Directories under `~/.gemini` that hold IDE settings and repo databases.
pub const GEMINI_CLONE_DIRS: &[&str] = &[
    "antigravity",
    "antigravity-ide",
    "antigravity-cli",
    "policies",
    "config",
];

pub(crate) fn copy_required_ide_files(src: &Path, dst: &Path) -> Result<(), String> {
    for rel in REQUIRED_IDE_REL_PATHS {
        let from = src.join(rel);
        if !from.exists() {
            continue;
        }
        let to = dst.join(rel);
        if from.is_dir() {
            copy_dir_recursive(&from, &to).map_err(|e| format!("Failed to copy {}: {}", rel, e))?;
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent for {}: {}", rel, e))?;
            }
            if rel.ends_with(".vscdb") || rel.ends_with(".db") {
                safe_clone_sqlite_db(&from, &to)?;
            } else {
                fs::copy(&from, &to).map_err(|e| format!("Failed to copy {}: {}", rel, e))?;
            }
        }
    }
    Ok(())
}

pub(crate) fn source_profile_home(source: &InstanceConfig) -> Option<PathBuf> {
    if source.is_default || source.id == "default" {
        std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()
            .map(PathBuf::from)
    } else {
        get_instance_home_dir(&source.id).ok()
    }
}

/// Copy the source IDE's `.gemini` settings and repo trees into the new home.
/// The default instance lives in the real user profile; named instances live
/// under `instances/<id>/home`. Locks and caches stay behind via `copy_dir_recursive`.
pub fn copy_source_ide_trees(source: &InstanceConfig, dest: &InstanceConfig) -> Result<(), String> {
    if let Some(src_home) = source_profile_home(source) {
        if let Ok(dst_home) = get_instance_home_dir(&dest.id) {
            // Justification: best-effort config seeding; missing files fall back to defaults
            crate::error::record_ignored(
                copy_gemini_trees(&src_home, &dst_home),
                "copy gemini config trees",
            );
        }
    } else if let Ok(dst_home) = get_instance_home_dir(&dest.id) {
        if let Some(sys_home) = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()
            .map(PathBuf::from)
        {
            // Justification: best-effort config seeding; missing files fall back to defaults
            crate::error::record_ignored(
                copy_gemini_trees(&sys_home, &dst_home),
                "copy gemini config trees",
            );
        }
    }

    let system_home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(PathBuf::from);
    let dst_data_gemini = PathBuf::from(&dest.data_dir).join(".gemini");
    for name in &["policies", "config"] {
        let mut src_opt = source_profile_home(source).map(|h| h.join(".gemini").join(name));
        if src_opt.as_ref().map(|p| !p.exists()).unwrap_or(true) {
            if let Some(ref sys) = system_home {
                let sys_candidate = sys.join(".gemini").join(name);
                if sys_candidate.exists() {
                    src_opt = Some(sys_candidate);
                }
            }
        }
        if let Some(src_p) = src_opt.filter(|p| p.exists()) {
            // Justification: best-effort tree copy; parity sync completes it on launch
            crate::error::record_ignored(
                copy_dir_recursive(&src_p, &dst_data_gemini.join(name)),
                "copy directory tree",
            );
        }
    }

    copy_source_user_settings(source, dest)?;
    Ok(())
}

/// Deep copies user settings, themes, keybindings, and snippets from source instance to target instance.
/// On Windows, copies into both `instance_data_dir/User` AND `instance_home/AppData/Roaming/Antigravity/User`.
pub fn copy_source_user_settings(
    source: &InstanceConfig,
    dest: &InstanceConfig,
) -> Result<(), String> {
    let dst_data_dir = PathBuf::from(&dest.data_dir);
    let dst_user_dir = dst_data_dir.join("User");
    // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
    crate::error::record_ignored(fs::create_dir_all(&dst_user_dir), "create directory");

    let mut dst_dirs = vec![dst_user_dir];

    #[cfg(target_os = "windows")]
    {
        if let Ok(dst_home) = get_instance_home_dir(&dest.id) {
            let dst_appdata_user = dst_home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User");
            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
            crate::error::record_ignored(fs::create_dir_all(&dst_appdata_user), "create directory");
            dst_dirs.push(dst_appdata_user);
        }
    }

    let mut src_user_dirs: Vec<PathBuf> = Vec::new();

    if source.is_default || source.id == "default" {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let p = PathBuf::from(appdata).join("Antigravity").join("User");
                if p.exists() && !src_user_dirs.contains(&p) {
                    src_user_dirs.push(p);
                }
            }
        }
        let default_user = get_default_antigravity_data_dir().join("User");
        if default_user.exists() && !src_user_dirs.contains(&default_user) {
            src_user_dirs.push(default_user);
        }
        let source_data_user = PathBuf::from(&source.data_dir).join("User");
        if source_data_user.exists() && !src_user_dirs.contains(&source_data_user) {
            src_user_dirs.push(source_data_user);
        }
    } else {
        let source_data_user = PathBuf::from(&source.data_dir).join("User");
        if source_data_user.exists() && !src_user_dirs.contains(&source_data_user) {
            src_user_dirs.push(source_data_user);
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(source_home) = get_instance_home_dir(&source.id) {
                let source_appdata_user = source_home
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User");
                if source_appdata_user.exists() && !src_user_dirs.contains(&source_appdata_user) {
                    src_user_dirs.push(source_appdata_user);
                }
            }
        }
    }

    // Copy all files and snippet directories from src_user_dirs into all dst_dirs
    for src_user in &src_user_dirs {
        if let Ok(entries) = fs::read_dir(src_user) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let name_str = file_name.to_string_lossy().to_lowercase();

                if name_str.ends_with(".lock")
                    || name_str.contains("cache")
                    || name_str == "lockfile"
                {
                    continue;
                }

                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        for dst in &dst_dirs {
                            // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
                            crate::error::record_ignored(
                                fs::copy(entry.path(), dst.join(&file_name)),
                                "copy file",
                            );
                        }
                    } else if file_type.is_dir() && name_str == "snippets" {
                        for dst in &dst_dirs {
                            // Justification: best-effort tree copy; parity sync completes it on launch
                            crate::error::record_ignored(
                                copy_dir_recursive(&entry.path(), &dst.join(&file_name)),
                                "copy directory tree",
                            );
                        }
                    }
                }
            }
        }
    }

    // Specifically ensure settings.json (which contains workbench.colorTheme, read folders,
    // browser settings, etc.) is copied into BOTH places so that regardless of whether the IDE
    // reads from data_dir or APPDATA, the color theme and settings are identical to the source!
    let mut candidate_settings: Vec<PathBuf> = Vec::new();
    for src_user in &src_user_dirs {
        let s = src_user.join("settings.json");
        if s.is_file() {
            candidate_settings.push(s);
        }
    }

    if let Some(best_settings) = pick_best_settings_path(&candidate_settings) {
        for dst in &dst_dirs {
            let target_settings = dst.join("settings.json");
            if let Some(parent) = target_settings.parent() {
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(parent), "create directory");
            }
            // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
            crate::error::record_ignored(fs::copy(best_settings, target_settings), "copy file");
        }
        crate::modules::logger::log_info(&format!(
            "[Instance] Cloned best source settings from {} into {} user targets",
            best_settings.display(),
            dst_dirs.len()
        ));
    }

    // Also explicitly ensure security_presets.json and antigravity_policies.json are copied into all dst_dirs
    for file_name in &["security_presets.json", "antigravity_policies.json"] {
        for src_user in &src_user_dirs {
            let p = src_user.join(file_name);
            if p.is_file() {
                for dst in &dst_dirs {
                    // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
                    crate::error::record_ignored(fs::copy(&p, dst.join(file_name)), "copy file");
                }
                break;
            }
        }
    }

    Ok(())
}

pub(crate) fn pick_best_settings_path(candidates: &[PathBuf]) -> Option<&PathBuf> {
    if candidates.is_empty() {
        return None;
    }
    if candidates.len() == 1 {
        return Some(&candidates[0]);
    }

    let score_file = |path: &PathBuf| -> (bool, u64, u64) {
        let has_theme = fs::read_to_string(path)
            .map(|content| content.contains("workbench.colorTheme"))
            .unwrap_or(false);
        let (mtime, len) = fs::metadata(path)
            .map(|m| {
                let mt = m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                (mt, m.len())
            })
            .unwrap_or((0, 0));
        (has_theme, mtime, len)
    };

    candidates.iter().max_by_key(|p| score_file(p))
}

pub(crate) fn copy_gemini_trees(src_home: &Path, dst_home: &Path) -> Result<(), String> {
    fs::create_dir_all(dst_home.join(".gemini"))
        .map_err(|e| format!("Failed to create dest .gemini: {}", e))?;

    let system_home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(PathBuf::from);

    for name in GEMINI_CLONE_DIRS {
        let mut src = src_home.join(".gemini").join(name);
        if !src.exists() {
            if let Some(ref sys) = system_home {
                let sys_candidate = sys.join(".gemini").join(name);
                if sys_candidate.exists() {
                    src = sys_candidate;
                } else {
                    continue;
                }
            } else {
                continue;
            }
        }
        let dst = dst_home.join(".gemini").join(name);
        copy_dir_recursive(&src, &dst).map_err(|e| {
            format!(
                "Failed to copy {} from {} to {}: {}",
                name,
                src.display(),
                dst.display(),
                e
            )
        })?;
        crate::modules::logger::log_info(&format!(
            "[Instance] Cloned IDE tree {} into {}",
            src.display(),
            dst.display()
        ));
    }
    Ok(())
}
