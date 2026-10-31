//! Full Antigravity Clean Delete & Reinstall Engine (`Imp-Purge`).
//!
//! Conforms to `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration options for the safe 4-stage purge protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeOptions {
    pub keep_accounts: bool,
    pub purge_instances: bool,
    pub uninstall_binaries: bool,
    pub force: bool,
}

impl Default for PurgeOptions {
    fn default() -> Self {
        Self {
            keep_accounts: true,
            purge_instances: true,
            uninstall_binaries: false,
            force: false,
        }
    }
}

/// Verification metrics returned upon successful execution of the purge protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeSummary {
    pub processes_terminated: usize,
    pub directories_purged: Vec<String>,
    pub bytes_reclaimed: u64,
    pub accounts_safeguarded: bool,
}

/// Executes the safe 4-stage purge protocol across Windows, macOS, and Linux.
pub fn execute_purge_protocol(options: PurgeOptions) -> Result<PurgeSummary, AppError> {
    // -------------------------------------------------------------------------
    // Stage 1: Caller IDE Self-Preservation & Process Sweep
    // -------------------------------------------------------------------------
    let caller_inst = crate::modules::instance::close::detect_caller_instance_id();
    let ancestry_report =
        crate::modules::instance::caller_process_ancestry_tree_scanner::scan_process_ancestry(0);

    let is_caller_active = caller_inst.is_some() || ancestry_report.is_caller_ide_detected;
    if is_caller_active && !options.force {
        eprintln!(
            "[PROTECTION] Operation targets running caller IDE session. Aborting to prevent session destruction."
        );
        eprintln!("[PROTECTION] Rerun with '--force' if you genuinely intended to terminate the active workspace.");
        std::process::exit(
            crate::modules::instance::caller_ide_exit_code_enforcer::CALLER_PROTECTION_EXIT_CODE,
        );
    }

    let processes_terminated =
        sweep_antigravity_processes(ancestry_report.process_chain, options.force)?;

    // -------------------------------------------------------------------------
    // Stage 2: Credential Safeguard Snapshot
    // -------------------------------------------------------------------------
    let staging_dir = std::env::temp_dir().join("agm-purge-safeguard");
    let mut accounts_safeguarded = false;
    if options.keep_accounts {
        accounts_safeguarded = snapshot_credentials(&staging_dir)?;
    }

    // -------------------------------------------------------------------------
    // Stage 3: Deep Filesystem Wipe
    // -------------------------------------------------------------------------
    let (directories_purged, bytes_reclaimed) = wipe_target_directories(&options)?;

    // -------------------------------------------------------------------------
    // Stage 4: Automated Clean Re-provisioning
    // -------------------------------------------------------------------------
    reprovision_clean_scaffolding(&staging_dir, accounts_safeguarded)?;

    Ok(PurgeSummary {
        processes_terminated,
        directories_purged,
        bytes_reclaimed,
        accounts_safeguarded,
    })
}

/// Sweeps and terminates running Antigravity IDE processes.
fn sweep_antigravity_processes(
    caller_chain: Vec<crate::modules::instance::caller_process_ancestry_tree_scanner::ProcessNode>,
    force: bool,
) -> Result<usize, AppError> {
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let caller_pids: std::collections::HashSet<u32> =
        caller_chain.into_iter().map(|n| n.pid).collect();
    let current_pid = std::process::id();

    let mut terminated_count = 0usize;

    for (pid, proc) in system.processes() {
        let p = pid.as_u32();
        if p == current_pid {
            continue;
        }

        if !force && caller_pids.contains(&p) {
            continue;
        }

        let name = proc.name().to_string_lossy().to_lowercase();
        let exe = proc
            .exe()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let is_target = name.contains("antigravity")
            || exe.contains("antigravity")
            || name == "antigravity.exe"
            || name == "antigravity";

        if is_target {
            #[cfg(target_os = "windows")]
            {
                // Justification: Forceful termination of targeted IDE process during purge
                let _ = crate::modules::instance::close::force_terminate_pid_win32(p);
            }
            #[cfg(not(target_os = "windows"))]
            {
                proc.kill();
            }
            terminated_count += 1;
        }
    }

    Ok(terminated_count)
}

/// Backs up authentication credentials to a temporary staging directory.
fn snapshot_credentials(staging_dir: &Path) -> Result<bool, AppError> {
    if staging_dir.exists() {
        // Justification: Cleaning old staging vault prior to fresh snapshot
        crate::error::record_ignored(
            fs::remove_dir_all(staging_dir),
            "clean old credential snapshot directory",
        );
    }
    fs::create_dir_all(staging_dir).map_err(AppError::Io)?;

    let mut saved_any = false;

    // 1. Check AGM account data dir
    if let Ok(data_dir) = crate::modules::account::get_data_dir() {
        let targets = ["accounts.json", "credentials.json", "secure_tokens.db"];
        for target in targets {
            let src = data_dir.join(target);
            if src.is_file() {
                let dest = staging_dir.join(target);
                if fs::copy(&src, &dest).is_ok() {
                    saved_any = true;
                }
            }
        }
    }

    // 2. Check ~/.gemini/antigravity accounts
    if let Some(home) = dirs::home_dir() {
        let gemini_dir = home.join(".gemini").join("antigravity");
        let gemini_acc = gemini_dir.join("accounts.json");
        if gemini_acc.is_file() {
            let dest = staging_dir.join("gemini_accounts.json");
            if fs::copy(&gemini_acc, &dest).is_ok() {
                saved_any = true;
            }
        }
    }

    Ok(saved_any)
}

/// Recursively computes directory size in bytes.
fn get_dir_size(path: &Path) -> u64 {
    let mut total_size = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            } else if p.is_dir() {
                total_size += get_dir_size(&p);
            }
        }
    }
    total_size
}

/// Collects cross-platform purge directories and purges them safely.
fn wipe_target_directories(options: &PurgeOptions) -> Result<(Vec<String>, u64), AppError> {
    let mut target_dirs: Vec<PathBuf> = Vec::new();
    let home = dirs::home_dir();

    // All OS: Gemini Core & Alt
    if let Some(ref h) = home {
        target_dirs.push(h.join(".gemini").join("antigravity"));
        target_dirs.push(h.join(".gemini").join("antigravity-ide"));
        target_dirs.push(h.join(".gemini").join("antigravity-cli"));

        if options.purge_instances {
            target_dirs.push(h.join(".antigravity_tools").join("instances"));
        }
    }

    // Platform-specific directories
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            target_dirs.push(PathBuf::from(&appdata).join("Antigravity"));
        }
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            target_dirs.push(PathBuf::from(&localappdata).join("Antigravity-Updater"));
            if options.uninstall_binaries {
                target_dirs.push(
                    PathBuf::from(&localappdata)
                        .join("Programs")
                        .join("Antigravity"),
                );
            }
        }
        let temp_dir = std::env::temp_dir();
        if let Ok(entries) = fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.starts_with("antigravity") {
                    target_dirs.push(entry.path());
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(ref h) = home {
            target_dirs.push(
                h.join("Library")
                    .join("Application Support")
                    .join("Antigravity"),
            );
            target_dirs.push(
                h.join("Library")
                    .join("Caches")
                    .join("com.google.antigravity"),
            );
            target_dirs.push(h.join("Library").join("Logs").join("Antigravity"));
            if options.uninstall_binaries {
                target_dirs.push(PathBuf::from("/Applications/Antigravity.app"));
                target_dirs.push(h.join("Applications").join("Antigravity.app"));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(ref h) = home {
            target_dirs.push(h.join(".config").join("Antigravity"));
            target_dirs.push(h.join(".config").join("antigravity"));
            target_dirs.push(h.join(".cache").join("Antigravity"));
            target_dirs.push(h.join(".cache").join("antigravity"));
            if options.uninstall_binaries {
                target_dirs.push(PathBuf::from("/opt/antigravity"));
                target_dirs.push(h.join(".local").join("share").join("antigravity"));
            }
        }
    }

    let mut purged_dirs = Vec::new();
    let mut bytes_reclaimed = 0u64;

    for dir in target_dirs {
        if dir.exists() {
            let size = if dir.is_dir() {
                get_dir_size(&dir)
            } else {
                fs::metadata(&dir).map(|m| m.len()).unwrap_or(0)
            };

            let remove_res = if dir.is_dir() {
                fs::remove_dir_all(&dir)
            } else {
                fs::remove_file(&dir)
            };

            if remove_res.is_ok() {
                bytes_reclaimed += size;
                purged_dirs.push(dir.to_string_lossy().to_string());
            } else {
                // Justification: Best-effort wipe of locked temporary directory during purge
                crate::error::record_ignored(remove_res, "remove target purge directory");
            }
        }
    }

    Ok((purged_dirs, bytes_reclaimed))
}

/// Restores clean directory scaffolding and preserved credentials.
fn reprovision_clean_scaffolding(
    staging_dir: &Path,
    accounts_safeguarded: bool,
) -> Result<(), AppError> {
    if let Some(home) = dirs::home_dir() {
        let base_gemini = home.join(".gemini").join("antigravity");
        // Re-create baseline folders
        fs::create_dir_all(base_gemini.join("brain")).map_err(AppError::Io)?;
        fs::create_dir_all(base_gemini.join("conversations")).map_err(AppError::Io)?;
        fs::create_dir_all(base_gemini.join("logs")).map_err(AppError::Io)?;

        // Restore accounts if safeguarded
        if accounts_safeguarded && staging_dir.is_dir() {
            if let Ok(data_dir) = crate::modules::account::get_data_dir() {
                let _ = fs::create_dir_all(&data_dir);
                let targets = ["accounts.json", "credentials.json", "secure_tokens.db"];
                for target in targets {
                    let src = staging_dir.join(target);
                    if src.is_file() {
                        let dest = data_dir.join(target);
                        // Justification: Best-effort credential restoration from temporary snapshot
                        crate::error::record_ignored(
                            fs::copy(&src, &dest),
                            "restore credential file from snapshot",
                        );
                    }
                }
            }

            let gemini_acc_src = staging_dir.join("gemini_accounts.json");
            if gemini_acc_src.is_file() {
                let gemini_acc_dest = base_gemini.join("accounts.json");
                // Justification: Best-effort Gemini accounts restoration
                crate::error::record_ignored(
                    fs::copy(&gemini_acc_src, &gemini_acc_dest),
                    "restore gemini accounts.json",
                );
            }

            // Cleanup staging dir
            // Justification: Cleanup of temporary staging vault after restoration
            crate::error::record_ignored(
                fs::remove_dir_all(staging_dir),
                "remove staging snapshot directory",
            );
        }
    }

    // Default configuration injection
    // Justification: Best-effort default tuning injection
    crate::error::record_ignored(
        crate::modules::instance::tuning::set_instance_turbo_mode(None, true),
        "set default turbo mode after purge",
    );
    crate::error::record_ignored(
        crate::modules::instance::tuning::set_instance_plan_review(None, false),
        "set default plan review after purge",
    );

    Ok(())
}
