//! clean_versions_cmds — Remove stale binary versions and update artifacts.
//!
//! Scans known paths (%LOCALAPPDATA%\Programs\antigravity-tools, C:\Program Files,
//! %LOCALAPPDATA%\agm-cli, %LOCALAPPDATA%\Programs\agm-alim, ~/.cargo/bin, %TEMP%)
//! and removes stale versions and old backup files.
//!
//! CRITICAL INVARIANT:
//! Strictly preserves SQLite databases (*.db, *.vscdb), configuration files
//! (*.json, *.toml), user accounts, and the currently running executable.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Check if a path is a protected user database, config, or resume state file.
pub fn is_protected_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if matches!(
        ext.as_str(),
        "db" | "sqlite" | "sqlite3" | "vscdb" | "json" | "toml" | "yaml" | "yml" | "env"
    ) {
        return true;
    }

    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    file_name.contains("state.vscdb")
        || file_name.contains("accounts.db")
        || file_name.contains("task_index.db")
        || file_name.contains("gui_config.json")
        || file_name.starts_with(".antigravity")
        || file_name.contains("storage.json")
}

/// Check if a file is a candidate for stale cleanup.
pub fn is_stale_artifact_candidate(path: &Path, current_exe: Option<&Path>) -> bool {
    if is_protected_file(path) {
        return false;
    }

    if let Some(curr) = current_exe {
        if let (Ok(can_a), Ok(can_b)) = (path.canonicalize(), curr.canonicalize()) {
            if can_a == can_b {
                return false;
            }
        }
        if path == curr {
            return false;
        }
    }

    let name = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    // Old backup files from updates
    if name.ends_with(".old")
        || name.ends_with(".bak")
        || name.contains(".exe.old")
        || name.contains(".old.")
        || name.ends_with(".delete-pending")
    {
        return true;
    }

    // Temporary updater directories/files
    if name.starts_with("agm-update-") || name.starts_with("tauri-updater-") {
        return true;
    }

    false
}

/// Discover candidate scan roots on the local machine.
pub fn get_candidate_scan_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // 1. Local AppData Programs
    if let Ok(local) = env::var("LOCALAPPDATA") {
        let p = PathBuf::from(&local);
        dirs.push(p.join("Programs").join("agm-alim"));
        dirs.push(p.join("Programs").join("antigravity-tools"));
        dirs.push(p.join("Programs").join("agm"));
        dirs.push(p.join("agm-cli"));
    }

    // 2. Cargo bin
    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        dirs.push(PathBuf::from(cargo_home).join("bin"));
    } else if let Ok(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) {
        dirs.push(PathBuf::from(home).join(".cargo").join("bin"));
    }

    // 3. Program Files
    if let Ok(pf) = env::var("ProgramFiles") {
        let p = PathBuf::from(&pf);
        dirs.push(p.join("antigravity-tools"));
        dirs.push(p.join("agm-alim"));
        dirs.push(p.join("agm"));
    }
    if let Ok(pf86) = env::var("ProgramFiles(x86)") {
        let p = PathBuf::from(&pf86);
        dirs.push(p.join("antigravity-tools"));
        dirs.push(p.join("agm-alim"));
    }

    // 4. Temp directories
    let tmp = env::temp_dir();
    dirs.push(tmp);

    dirs.retain(|d| d.exists() && d.is_dir());
    dirs
}

/// Extract version string from a binary by running `<bin> version` or `<bin> --version`.
fn query_bin_version(bin_path: &Path) -> Option<String> {
    let output = Command::new(bin_path).arg("version").output().ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("agm v") || line.contains("v") {
                return Some(line.trim().to_string());
            }
        }
    }
    None
}

pub fn cmd_clean_other_versions(args: &[String]) {
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    let verbose = args.iter().any(|a| a == "--verbose" || a == "-v");
    let force = args.iter().any(|a| a == "--force" || a == "-f");

    println!("🧹 [AGM:CLEAN-VERSIONS] Scanning for stale versions and update artifacts...");

    let current_exe = env::current_exe().ok();
    if let Some(ref curr) = current_exe {
        println!("  • Current running binary: {}", curr.display());
    }

    let scan_dirs = get_candidate_scan_dirs();
    let mut files_to_delete: Vec<PathBuf> = Vec::new();
    let mut dirs_to_delete: Vec<PathBuf> = Vec::new();
    let mut total_bytes: u64 = 0;

    for dir in &scan_dirs {
        if verbose {
            println!("  Scanning directory: {}", dir.display());
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if is_protected_file(&path) {
                continue;
            }

            if path.is_dir() {
                let dname = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if dname.starts_with("agm-update-") || dname.starts_with("tauri-updater-") {
                    dirs_to_delete.push(path);
                }
                continue;
            }

            // Check if it's an old backup artifact (*.old, *.bak)
            if is_stale_artifact_candidate(&path, current_exe.as_deref()) {
                if let Ok(meta) = fs::metadata(&path) {
                    total_bytes += meta.len();
                }
                files_to_delete.push(path);
                continue;
            }

            // Check for duplicate/stale agm.exe in non-current locations
            let fname = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            if (fname == "agm.exe" || fname == "agm") && current_exe.as_deref() != Some(&path) {
                // If it's in .cargo/bin and current_exe is not in .cargo/bin, it's a stale duplicate cargo install
                let in_cargo_bin = path.to_string_lossy().contains(".cargo");
                let in_programs_alim = path.to_string_lossy().contains("agm-alim");

                if in_cargo_bin || in_programs_alim {
                    if let Some(curr) = current_exe.as_ref() {
                        let is_same = match (path.canonicalize(), curr.canonicalize()) {
                            (Ok(a), Ok(b)) => a == b,
                            _ => path == *curr,
                        };
                        if !is_same {
                            let ver_info = query_bin_version(&path)
                                .unwrap_or_else(|| "stale duplicate".to_string());
                            println!(
                                "  [DETECTED] Stale binary in other path: {} ({})",
                                path.display(),
                                ver_info
                            );
                            if let Ok(meta) = fs::metadata(&path) {
                                total_bytes += meta.len();
                            }
                            files_to_delete.push(path);
                        }
                    }
                }
            }
        }
    }

    if files_to_delete.is_empty() && dirs_to_delete.is_empty() {
        println!("\n✨ No stale versions, update artifacts, or .old files found.");
        println!("  Your system is clean!");
        return;
    }

    let mb = (total_bytes as f64) / (1024.0 * 1024.0);
    println!(
        "\nFound {} stale file(s) and {} temporary folder(s) (~{:.2} MB):",
        files_to_delete.len(),
        dirs_to_delete.len(),
        mb
    );

    for f in &files_to_delete {
        println!("  - [FILE] {}", f.display());
    }
    for d in &dirs_to_delete {
        println!("  - [DIR]  {}", d.display());
    }

    if dry_run {
        println!("\n🔍 [DRY-RUN] No files were deleted. Re-run without --dry-run to clean.");
        return;
    }

    let mut deleted_count = 0;
    let mut failed_count = 0;

    for f in &files_to_delete {
        match fs::remove_file(f) {
            Ok(_) => {
                deleted_count += 1;
                if verbose {
                    println!("    ✓ Removed: {}", f.display());
                }
            }
            Err(e) => {
                failed_count += 1;
                eprintln!("    ✗ Failed to remove {}: {}", f.display(), e);
            }
        }
    }

    for d in &dirs_to_delete {
        match fs::remove_dir_all(d) {
            Ok(_) => {
                deleted_count += 1;
                if verbose {
                    println!("    ✓ Removed dir: {}", d.display());
                }
            }
            Err(e) => {
                failed_count += 1;
                eprintln!("    ✗ Failed to remove dir {}: {}", d.display(), e);
            }
        }
    }

    println!(
        "\n✅ Cleanup complete: {} items removed ({:.2} MB reclaimed).",
        deleted_count, mb
    );
    if failed_count > 0 {
        println!(
            "⚠️  {} items could not be removed (files may be in use by active processes).",
            failed_count
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_protected_file() {
        assert!(is_protected_file(Path::new("accounts.db")));
        assert!(is_protected_file(Path::new("state.vscdb")));
        assert!(is_protected_file(Path::new("task_index.db")));
        assert!(is_protected_file(Path::new("gui_config.json")));
        assert!(is_protected_file(Path::new(
            ".antigravity_resume_task.json"
        )));
        assert!(is_protected_file(Path::new("C:/data/config.toml")));
        assert!(is_protected_file(Path::new("/var/data/custom.sqlite3")));

        assert!(!is_protected_file(Path::new("agm.exe")));
        assert!(!is_protected_file(Path::new("agm.exe.old")));
        assert!(!is_protected_file(Path::new("agm-alim.exe.old")));
    }

    #[test]
    fn test_is_stale_artifact_candidate() {
        let curr = Path::new("C:/bin/agm.exe");
        assert!(is_stale_artifact_candidate(
            Path::new("C:/bin/agm.exe.old"),
            Some(curr)
        ));
        assert!(is_stale_artifact_candidate(
            Path::new("C:/bin/agm.bak"),
            Some(curr)
        ));
        assert!(is_stale_artifact_candidate(
            Path::new("C:/temp/agm-update-123.exe"),
            Some(curr)
        ));

        // Current binary is NEVER candidate
        assert!(!is_stale_artifact_candidate(curr, Some(curr)));

        // Protected files are NEVER candidate
        assert!(!is_stale_artifact_candidate(
            Path::new("C:/bin/accounts.db"),
            Some(curr)
        ));
    }
}
