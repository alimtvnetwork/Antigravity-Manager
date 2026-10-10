use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::*;

/// Locate the best CLI source binary (`agm.exe` / `agm`) and copy it to both:
/// 1. Persistent dedicated Update CLI: `%LOCALAPPDATA%\agm-cli\agm-update-cli.exe`
/// 2. Isolated per-run temp Update CLI: `%TEMP%\agm-updater\agm-update-cli-<pid>.exe`
pub fn prepare_isolated_update_cli(caller_pid: u32) -> Result<PathBuf, String> {
    let current_exe =
        env::current_exe().map_err(|e| format!("Cannot locate current executable: {}", e))?;
    let current_dir = current_exe
        .parent()
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let bin_name = if cfg!(target_os = "windows") {
        "agm.exe"
    } else {
        "agm"
    };

    let mut candidates: Vec<PathBuf> = Vec::new();
    // 1. Direct sibling CLI in current executable directory
    candidates.push(current_dir.join(bin_name));

    // 2. Installed CLI in standard user directories
    #[cfg(target_os = "windows")]
    if let Ok(local) = env::var("LOCALAPPDATA") {
        let local_path = PathBuf::from(&local);
        candidates.push(local_path.join("agm-cli").join("agm-update-cli.exe"));
        candidates.push(local_path.join("agm-cli").join("agm.exe"));
        candidates.push(local_path.join("Programs").join("agm-alim").join("agm.exe"));
    }

    #[cfg(not(target_os = "windows"))]
    if let Ok(home) = env::var("HOME") {
        let home_path = PathBuf::from(&home);
        candidates.push(home_path.join(".local").join("bin").join("agm-update-cli"));
        candidates.push(home_path.join(".local").join("bin").join("agm"));
    }

    // 3. Search PATH for agm
    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            let p = dir.join(bin_name);
            if p.exists() && !candidates.contains(&p) {
                candidates.push(p);
            }
        }
    }

    // Pick first existing CLI candidate; if none exists, fallback to current_exe (which handles update commands directly)
    let updater_src = candidates
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| current_exe.clone());

    // Maintain persistent dedicated `agm-update-cli` (and `agm.exe` if source is `agm.exe`) in agm-cli folder
    #[cfg(target_os = "windows")]
    if let Ok(local) = env::var("LOCALAPPDATA") {
        let cli_dir = PathBuf::from(local).join("agm-cli");
        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
        crate::error::record_ignored(fs::create_dir_all(&cli_dir), "create_dir_all");
        let dedicated_cli = cli_dir.join("agm-update-cli.exe");
        if updater_src != dedicated_cli && updater_src.exists() {
            // Justification: best-effort file copy; logged for diagnosis
            crate::error::record_ignored(fs::copy(&updater_src, &dedicated_cli), "fs::copy");
        }
        let sibling_agm = current_dir.join("agm.exe");
        let global_agm = cli_dir.join("agm.exe");
        if sibling_agm.exists() && sibling_agm != global_agm {
            // Justification: best-effort file copy; logged for diagnosis
            crate::error::record_ignored(fs::copy(&sibling_agm, &global_agm), "fs::copy");
        }
    }

    let temp_dir = env::temp_dir().join("agm-updater");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp updater dir {:?}: {}", temp_dir, e))?;

    let temp_name = if cfg!(target_os = "windows") {
        format!("agm-update-cli-{}.exe", caller_pid)
    } else {
        format!("agm-update-cli-{}", caller_pid)
    };
    let mut temp_cli_path = temp_dir.join(&temp_name);

    if temp_cli_path.exists() {
        if let Err(_) = fs::remove_file(&temp_cli_path) {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let alt_name = if cfg!(target_os = "windows") {
                format!("agm-update-cli-{}-{}.exe", caller_pid, ts)
            } else {
                format!("agm-update-cli-{}-{}", caller_pid, ts)
            };
            temp_cli_path = temp_dir.join(alt_name);
        }
    }

    fs::copy(&updater_src, &temp_cli_path).map_err(|e| {
        format!(
            "Failed to copy updater {:?} -> {:?}: {}",
            updater_src, temp_cli_path, e
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Justification: best-effort permission hardening; logged
        crate::error::record_ignored(
            fs::set_permissions(&temp_cli_path, fs::Permissions::from_mode(0o755)),
            "set_permissions",
        );
    }

    Ok(temp_cli_path)
}

/// Spawn the isolated `agm-update-cli` process in its own console / session without `cmd.exe /c start` quote mangling.
pub fn spawn_delegated_update_cli(
    temp_cli_path: &Path,
    wait_pid: u32,
    install_dir: &Path,
    target_exe: &Path,
    is_relaunch: bool,
    target_version: Option<&str>,
) -> Result<u32, String> {
    let mut cmd = Command::new(temp_cli_path);
    cmd.arg("delegate-update")
        .arg("--delegated-worker")
        .arg("--wait-pid")
        .arg(wait_pid.to_string())
        .arg("--install-dir")
        .arg(install_dir.to_string_lossy().as_ref())
        .arg("--target-exe")
        .arg(target_exe.to_string_lossy().as_ref());

    if is_relaunch {
        cmd.arg("--relaunch");
    } else {
        cmd.arg("--no-relaunch");
    }

    if let Some(ver) = target_version {
        if !ver.trim().is_empty() {
            cmd.arg("--version").arg(ver.trim());
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x00000010;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(CREATE_NEW_CONSOLE | CREATE_NEW_PROCESS_GROUP);
    }

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn delegated update CLI: {}", e))?;

    Ok(child.id())
}
