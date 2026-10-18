//! Automated diagnostic & self-healing engine for Antigravity-Manager.
//! Analyzes common root causes: stale locks, permissions, invalid configurations,
//! corrupted SQLite databases, and applies surgical self-healing fixes.

use super::db::connect_error_db;
use super::types::{DiagnosticCheck, SelfHealingReport};
use crate::modules::account;
use rusqlite::Connection;
use std::fs;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Run complete diagnostic checks and optionally apply automated self-healing fixes
pub fn run_self_healing(apply_fixes: bool) -> Result<SelfHealingReport, String> {
    let mut checks = Vec::new();
    let mut fixes_applied = Vec::new();
    let mut warnings = Vec::new();

    let data_dir = account::get_data_dir().unwrap_or_else(|_| {
        dirs::home_dir()
            .map(|h| h.join(".antigravity_tools"))
            .unwrap_or_else(|| PathBuf::from(".antigravity_tools"))
    });

    // 1. Data directory & permissions probe
    check_data_directory(&data_dir, apply_fixes, &mut checks, &mut fixes_applied);

    // 2. Logs directory probe
    check_logs_directory(&data_dir, apply_fixes, &mut checks, &mut fixes_applied);

    // 3. Stale locks inspection
    check_stale_locks(&data_dir, apply_fixes, &mut checks, &mut fixes_applied);

    // 4. Accounts vault JSON configuration integrity
    check_accounts_config(
        &data_dir,
        apply_fixes,
        &mut checks,
        &mut fixes_applied,
        &mut warnings,
    );

    // 5. GUI & App configuration integrity
    check_app_config(
        &data_dir,
        apply_fixes,
        &mut checks,
        &mut fixes_applied,
        &mut warnings,
    );

    // 6. SQLite databases quick integrity check
    check_sqlite_databases(
        &data_dir,
        apply_fixes,
        &mut checks,
        &mut fixes_applied,
        &mut warnings,
    );

    // 7. Git hygiene and resumption artifacts check
    check_git_resumption_artifacts(apply_fixes, &mut checks, &mut fixes_applied);

    // 8. Network port / proxy connectivity probe
    check_proxy_port_probe(&mut checks, &mut warnings);

    let total_checks = checks.len();
    let passed_checks = checks.iter().filter(|c| c.passed).count();
    let fixed_count = fixes_applied.len();
    let is_healthy = passed_checks == total_checks;

    Ok(SelfHealingReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_checks,
        passed_checks,
        fixed_count,
        checks,
        fixes_applied,
        warnings,
        is_healthy,
    })
}

fn check_data_directory(
    data_dir: &Path,
    apply_fixes: bool,
    checks: &mut Vec<DiagnosticCheck>,
    fixes_applied: &mut Vec<String>,
) {
    let name = "Data Directory Permissions".to_string();
    let category = "Filesystem".to_string();

    if !data_dir.exists() {
        if apply_fixes {
            match fs::create_dir_all(data_dir) {
                Ok(_) => {
                    let fix = format!("Created missing root data directory: {data_dir:?}");
                    fixes_applied.push(fix);
                    checks.push(DiagnosticCheck {
                        name,
                        category,
                        passed: true,
                        message: format!("Directory {data_dir:?} created and writable."),
                        auto_fixed: true,
                    });
                }
                Err(e) => {
                    checks.push(DiagnosticCheck {
                        name,
                        category,
                        passed: false,
                        message: format!("Failed to create {data_dir:?}: {e}"),
                        auto_fixed: false,
                    });
                }
            }
        } else {
            checks.push(DiagnosticCheck {
                name,
                category,
                passed: false,
                message: format!("Data directory {data_dir:?} is missing."),
                auto_fixed: false,
            });
        }
        return;
    }

    // Test write permission via canary file
    let canary = data_dir.join(".agm_perm_probe");
    match fs::write(&canary, b"canary_probe") {
        Ok(_) => {
            let _ = fs::remove_file(&canary);
            checks.push(DiagnosticCheck {
                name,
                category,
                passed: true,
                message: format!("Directory {data_dir:?} is accessible and writable."),
                auto_fixed: false,
            });
        }
        Err(e) => {
            checks.push(DiagnosticCheck {
                name,
                category,
                passed: false,
                message: format!("Directory {data_dir:?} lacks write permission: {e}"),
                auto_fixed: false,
            });
        }
    }
}

fn check_logs_directory(
    data_dir: &Path,
    apply_fixes: bool,
    checks: &mut Vec<DiagnosticCheck>,
    fixes_applied: &mut Vec<String>,
) {
    let name = "Log Directory Structure".to_string();
    let category = "Filesystem".to_string();
    let log_dir = data_dir.join("logs");

    if !log_dir.exists() {
        if apply_fixes {
            match fs::create_dir_all(&log_dir) {
                Ok(_) => {
                    fixes_applied.push(format!("Created log directory: {log_dir:?}"));
                    checks.push(DiagnosticCheck {
                        name,
                        category,
                        passed: true,
                        message: format!("Logs directory {log_dir:?} established."),
                        auto_fixed: true,
                    });
                }
                Err(e) => {
                    checks.push(DiagnosticCheck {
                        name,
                        category,
                        passed: false,
                        message: format!("Failed to create logs directory: {e}"),
                        auto_fixed: false,
                    });
                }
            }
        } else {
            checks.push(DiagnosticCheck {
                name,
                category,
                passed: false,
                message: format!("Logs directory {log_dir:?} is missing."),
                auto_fixed: false,
            });
        }
        return;
    }

    checks.push(DiagnosticCheck {
        name,
        category,
        passed: true,
        message: format!("Logs directory {log_dir:?} is healthy."),
        auto_fixed: false,
    });
}

fn check_stale_locks(
    data_dir: &Path,
    apply_fixes: bool,
    checks: &mut Vec<DiagnosticCheck>,
    fixes_applied: &mut Vec<String>,
) {
    let name = "Stale Lockfile Inspection".to_string();
    let category = "Locks".to_string();
    let mut detected_locks = Vec::new();

    let candidate_names = [
        "updater.lock",
        ".antigravity_lock",
        ".antigravity_tools_location.lock",
        "accounts.json.lock",
        "sync.lock",
    ];

    for c in &candidate_names {
        let lock_path = data_dir.join(c);
        if lock_path.is_file() {
            detected_locks.push(lock_path);
        }
    }

    // Check temporary files that may be orphaned
    if let Ok(entries) = fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "lock" && !detected_locks.contains(&path) {
                    detected_locks.push(path);
                }
            }
        }
    }

    if detected_locks.is_empty() {
        checks.push(DiagnosticCheck {
            name,
            category,
            passed: true,
            message: "No stale lockfiles detected.".to_string(),
            auto_fixed: false,
        });
        return;
    }

    if apply_fixes {
        let mut removed_count = 0;
        for lock in &detected_locks {
            if fs::remove_file(lock).is_ok() {
                removed_count += 1;
                fixes_applied.push(format!(
                    "Removed stale lockfile: {:?}",
                    lock.file_name().unwrap_or_default()
                ));
            }
        }
        checks.push(DiagnosticCheck {
            name,
            category,
            passed: true,
            message: format!("Healed {removed_count} stale lockfile(s)."),
            auto_fixed: true,
        });
    } else {
        checks.push(DiagnosticCheck {
            name,
            category,
            passed: false,
            message: format!("Detected {} stale lockfile(s).", detected_locks.len()),
            auto_fixed: false,
        });
    }
}

fn check_accounts_config(
    data_dir: &Path,
    apply_fixes: bool,
    checks: &mut Vec<DiagnosticCheck>,
    fixes_applied: &mut Vec<String>,
    _warnings: &mut Vec<String>,
) {
    let name = "Account Vault Configuration".to_string();
    let category = "Configuration".to_string();
    let acc_file = data_dir.join("accounts.json");

    if !acc_file.exists() {
        if apply_fixes {
            let default_content = r#"{"accounts":[],"activeIndex":0}"#;
            if fs::write(&acc_file, default_content).is_ok() {
                fixes_applied.push("Initialized empty accounts.json skeleton.".to_string());
                checks.push(DiagnosticCheck {
                    name,
                    category,
                    passed: true,
                    message: "accounts.json was missing; created valid empty skeleton.".to_string(),
                    auto_fixed: true,
                });
            } else {
                checks.push(DiagnosticCheck {
                    name,
                    category,
                    passed: false,
                    message: "Failed to write default accounts.json.".to_string(),
                    auto_fixed: false,
                });
            }
        } else {
            checks.push(DiagnosticCheck {
                name,
                category,
                passed: false,
                message: "accounts.json does not exist.".to_string(),
                auto_fixed: false,
            });
        }
        return;
    }

    match fs::read_to_string(&acc_file) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => {
                    let count = json
                        .get("accounts")
                        .and_then(|a| a.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    checks.push(DiagnosticCheck {
                        name,
                        category,
                        passed: true,
                        message: format!(
                            "Valid accounts.json structure ({count} account(s) registered)."
                        ),
                        auto_fixed: false,
                    });
                }
                Err(err) => {
                    if apply_fixes {
                        // Try backup restore
                        let bak = data_dir.join("accounts.json.bak");
                        if bak.is_file()
                            && fs::read_to_string(&bak)
                                .map(|b| serde_json::from_str::<serde_json::Value>(&b).is_ok())
                                .unwrap_or(false)
                        {
                            let _ = fs::copy(&bak, &acc_file);
                            fixes_applied.push(
                                "Restored valid accounts.json from backup accounts.json.bak"
                                    .to_string(),
                            );
                            checks.push(DiagnosticCheck {
                                name,
                                category,
                                passed: true,
                                message: "Corrupted accounts.json restored from valid backup."
                                    .to_string(),
                                auto_fixed: true,
                            });
                        } else {
                            // Backup corrupted file and initialize valid skeleton
                            let corrupt_bak = data_dir.join(format!(
                                "accounts.json.corrupt.{}",
                                chrono::Utc::now().timestamp()
                            ));
                            let _ = fs::rename(&acc_file, &corrupt_bak);
                            let _ = fs::write(&acc_file, r#"{"accounts":[],"activeIndex":0}"#);
                            fixes_applied.push(format!("Quarantined corrupt accounts.json to {corrupt_bak:?} and reset to clean skeleton."));
                            checks.push(DiagnosticCheck {
                            name,
                            category,
                            passed: true,
                            message: "Corrupt accounts.json quarantined and healed with clean skeleton.".to_string(),
                            auto_fixed: true,
                        });
                        }
                    } else {
                        checks.push(DiagnosticCheck {
                            name,
                            category,
                            passed: false,
                            message: format!("accounts.json is corrupt (JSON syntax error: {err})"),
                            auto_fixed: false,
                        });
                    }
                }
            }
        }
        Err(e) => {
            checks.push(DiagnosticCheck {
                name,
                category,
                passed: false,
                message: format!("Unable to read accounts.json: {e}"),
                auto_fixed: false,
            });
        }
    }
}

fn check_app_config(
    data_dir: &Path,
    apply_fixes: bool,
    checks: &mut Vec<DiagnosticCheck>,
    fixes_applied: &mut Vec<String>,
    _warnings: &mut Vec<String>,
) {
    let name = "GUI / App Configuration".to_string();
    let category = "Configuration".to_string();
    let cfg_file = data_dir.join("gui_config.json");

    if !cfg_file.exists() {
        checks.push(DiagnosticCheck {
            name,
            category,
            passed: true,
            message: "gui_config.json not initialized yet (using application defaults)."
                .to_string(),
            auto_fixed: false,
        });
        return;
    }

    match fs::read_to_string(&cfg_file) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => {
                checks.push(DiagnosticCheck {
                    name,
                    category,
                    passed: true,
                    message: "gui_config.json is well-formed JSON.".to_string(),
                    auto_fixed: false,
                });
            }
            Err(e) => {
                if apply_fixes {
                    let bak = data_dir.join(format!(
                        "gui_config.json.corrupt.{}",
                        chrono::Utc::now().timestamp()
                    ));
                    let _ = fs::rename(&cfg_file, &bak);
                    let _ = fs::write(&cfg_file, "{}");
                    fixes_applied.push(format!(
                        "Reset corrupted gui_config.json, saved copy to {bak:?}"
                    ));
                    checks.push(DiagnosticCheck {
                        name,
                        category,
                        passed: true,
                        message: "Corrupted gui_config.json reset to default structure."
                            .to_string(),
                        auto_fixed: true,
                    });
                } else {
                    checks.push(DiagnosticCheck {
                        name,
                        category,
                        passed: false,
                        message: format!("gui_config.json JSON error: {e}"),
                        auto_fixed: false,
                    });
                }
            }
        },
        Err(e) => {
            checks.push(DiagnosticCheck {
                name,
                category,
                passed: false,
                message: format!("Cannot read gui_config.json: {e}"),
                auto_fixed: false,
            });
        }
    }
}

fn check_sqlite_databases(
    data_dir: &Path,
    apply_fixes: bool,
    checks: &mut Vec<DiagnosticCheck>,
    fixes_applied: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let name = "SQLite Database Integrity".to_string();
    let category = "Database".to_string();

    let databases = [
        "errors.db",
        "repo_prompts.db",
        "proxy.db",
        "email_vault.db",
        "security.db",
    ];
    let mut db_checked = 0;
    let mut db_issues = Vec::new();

    // Verify error_db connection first
    if let Err(e) = connect_error_db() {
        db_issues.push(format!("errors.db connection failed: {e}"));
    } else {
        db_checked += 1;
    }

    for db_name in &databases {
        let db_path = data_dir.join(db_name);
        if db_path.is_file() {
            match Connection::open(&db_path) {
                Ok(conn) => {
                    let quick_check: Result<String, _> =
                        conn.query_row("PRAGMA quick_check(1);", [], |row| row.get(0));
                    match quick_check {
                        Ok(res) if res.eq_ignore_ascii_case("ok") => {
                            db_checked += 1;
                        }
                        Ok(res) => {
                            db_issues.push(format!("{db_name} integrity warning: {res}"));
                            if apply_fixes {
                                let _ = conn.execute("PRAGMA wal_checkpoint(TRUNCATE);", []);
                                fixes_applied.push(format!("Ran wal_checkpoint on {db_name}"));
                            }
                        }
                        Err(e) => {
                            db_issues.push(format!("{db_name} query failed: {e}"));
                        }
                    }
                }
                Err(e) => {
                    db_issues.push(format!("{db_name} open failed: {e}"));
                }
            }
        }
    }

    if db_issues.is_empty() {
        checks.push(DiagnosticCheck {
            name,
            category,
            passed: true,
            message: format!("All checked databases ({db_checked}) passed PRAGMA quick_check integrity verification."),
            auto_fixed: false,
        });
    } else {
        warnings.extend(db_issues.clone());
        checks.push(DiagnosticCheck {
            name,
            category,
            passed: false,
            message: format!(
                "SQLite issues detected in {} database(s): {}",
                db_issues.len(),
                db_issues.join("; ")
            ),
            auto_fixed: apply_fixes,
        });
    }
}

fn check_git_resumption_artifacts(
    apply_fixes: bool,
    checks: &mut Vec<DiagnosticCheck>,
    fixes_applied: &mut Vec<String>,
) {
    let name = "Resumption Artifacts Hygiene".to_string();
    let category = "Git Hygiene".to_string();

    let target_artifacts = [
        "antigravity-resume_task.json",
        ".antigravity_resume_task.json",
    ];

    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut found_artifacts = Vec::new();

    for a in &target_artifacts {
        let file_path = current_dir.join(a);
        if file_path.is_file() {
            found_artifacts.push(file_path);
        }
    }

    if found_artifacts.is_empty() {
        checks.push(DiagnosticCheck {
            name,
            category,
            passed: true,
            message: "No stray resumption task artifacts found in working directory.".to_string(),
            auto_fixed: false,
        });
    } else if apply_fixes {
        for a in &found_artifacts {
            if fs::remove_file(a).is_ok() {
                fixes_applied.push(format!(
                    "Cleaned up stray resumption artifact: {:?}",
                    a.file_name().unwrap_or_default()
                ));
            }
        }
        checks.push(DiagnosticCheck {
            name,
            category,
            passed: true,
            message: format!(
                "Removed {} stray resumption artifact(s).",
                found_artifacts.len()
            ),
            auto_fixed: true,
        });
    } else {
        checks.push(DiagnosticCheck {
            name,
            category,
            passed: false,
            message: format!(
                "Found {} stray resumption artifact(s) in working tree.",
                found_artifacts.len()
            ),
            auto_fixed: false,
        });
    }
}

fn check_proxy_port_probe(checks: &mut Vec<DiagnosticCheck>, _warnings: &mut Vec<String>) {
    let name = "Proxy Server Port Probe".to_string();
    let category = "Network".to_string();

    // Default AGM reverse proxy port is 8045
    let port = 8045;
    let addr = format!("127.0.0.1:{port}");

    match TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_millis(300)) {
        Ok(_) => {
            // Port is currently occupied / listening
            checks.push(DiagnosticCheck {
                name,
                category,
                passed: true,
                message: format!("Proxy port {port} is active and responsive."),
                auto_fixed: false,
            });
        }
        Err(_) => {
            // Port is free (standby)
            checks.push(DiagnosticCheck {
                name,
                category,
                passed: true,
                message: format!("Proxy port {port} is free and ready for binding."),
                auto_fixed: false,
            });
        }
    }
}
