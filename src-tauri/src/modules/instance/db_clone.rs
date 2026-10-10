//! Safe SQLite cloning and recursive directory copy.
use super::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Clones an SQLite database safely even when active processes hold open locks or WAL files.
/// First attempts a read-only rusqlite connection in immutable URI mode (?mode=ro&immutable=1) with SQLite Online Backup API.
/// Executes incremental backup loop (step(100)) with exponential backoff (up to 2.5s) to handle transient busy/locked writes.
/// If the backup API fails or is unavailable, falls back to direct shared file copying including `-wal` and `-shm` sidecars.
pub fn safe_clone_sqlite_db(src_db: &Path, dst_db: &Path) -> Result<(), String> {
    if !src_db.exists() {
        return Ok(());
    }

    if let Some(parent) = dst_db.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create destination directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    // Attempt 1: Online SQLite Backup API via read-only connection in immutable URI mode
    let try_backup = || -> Result<(), String> {
        let clean_src = src_db.to_string_lossy().replace('\\', "/");
        let uri_primary_ro = format!("file:///{}?mode=ro", clean_src.trim_start_matches('/'));
        let uri_alt_ro = format!("file:{}?mode=ro", clean_src);
        let uri_primary_imm = format!(
            "file:///{}?mode=ro&immutable=1",
            clean_src.trim_start_matches('/')
        );
        let uri_alt_imm = format!("file:{}?mode=ro&immutable=1", clean_src);

        let src_conn = rusqlite::Connection::open_with_flags(
            &uri_primary_ro,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .or_else(|_| {
            rusqlite::Connection::open_with_flags(
                &uri_alt_ro,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        })
        .or_else(|_| {
            rusqlite::Connection::open_with_flags(
                src_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
        })
        .or_else(|_| {
            rusqlite::Connection::open_with_flags(
                &uri_primary_imm,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        })
        .or_else(|_| {
            rusqlite::Connection::open_with_flags(
                &uri_alt_imm,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        })
        .map_err(|e| format!("Failed to open source SQLite db in read-only mode: {}", e))?;

        // Justification: pragma is performance/concurrency tuning; the connection stays usable without it
        crate::error::record_ignored(
            src_conn.pragma_update(None, "busy_timeout", 2500),
            "set sqlite pragma",
        );
        // Justification: pragma is performance/concurrency tuning; the connection stays usable without it
        crate::error::record_ignored(
            src_conn.pragma_update(None, "journal_mode", "WAL"),
            "set sqlite pragma",
        );

        if dst_db.exists() {
            // Justification: cleanup of an optional file; absence is the normal case
            crate::error::record_ignored(fs::remove_file(dst_db), "remove file");
        }
        let mut sidecar_wal = dst_db.as_os_str().to_os_string();
        sidecar_wal.push("-wal");
        // Justification: cleanup of an optional file; absence is the normal case
        crate::error::record_ignored(fs::remove_file(PathBuf::from(sidecar_wal)), "remove file");
        let mut sidecar_shm = dst_db.as_os_str().to_os_string();
        sidecar_shm.push("-shm");
        // Justification: cleanup of an optional file; absence is the normal case
        crate::error::record_ignored(fs::remove_file(PathBuf::from(sidecar_shm)), "remove file");

        let mut dst_conn = rusqlite::Connection::open(dst_db)
            .map_err(|e| format!("Failed to open destination SQLite db: {}", e))?;

        // Justification: pragma is performance/concurrency tuning; the connection stays usable without it
        crate::error::record_ignored(
            dst_conn.pragma_update(None, "busy_timeout", 2500),
            "set sqlite pragma",
        );
        // Justification: pragma is performance/concurrency tuning; the connection stays usable without it
        crate::error::record_ignored(
            dst_conn.pragma_update(None, "journal_mode", "WAL"),
            "set sqlite pragma",
        );

        let backup = rusqlite::backup::Backup::new(&src_conn, &mut dst_conn)
            .map_err(|e| format!("Failed to initialize SQLite backup: {}", e))?;

        let start_time = std::time::Instant::now();
        let max_wait = std::time::Duration::from_millis(2500);
        let mut backoff = std::time::Duration::from_millis(10);

        loop {
            match backup.step(100) {
                Ok(rusqlite::backup::StepResult::Done) => break,
                Ok(rusqlite::backup::StepResult::More) => {
                    backoff = std::time::Duration::from_millis(10);
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Ok(rusqlite::backup::StepResult::Busy)
                | Ok(rusqlite::backup::StepResult::Locked) => {
                    if start_time.elapsed() >= max_wait {
                        return Err(
                            "SQLite backup timed out due to transient lock/busy state".to_string()
                        );
                    }
                    std::thread::sleep(backoff);
                    backoff = (backoff * 2).min(std::time::Duration::from_millis(250));
                }
                Ok(_) => break,
                Err(e) => {
                    return Err(format!("SQLite backup step error: {}", e));
                }
            }
        }

        Ok(())
    };

    if let Err(err) = try_backup() {
        crate::modules::logger::log_info(&format!(
            "[Instance] Online SQLite backup for {} failed ({}); falling back to shared file copy",
            src_db.display(),
            err
        ));

        // Attempt 2: Fallback direct file copy with -wal and -shm sidecars
        let copy_shared = |from: &Path, to: &Path| -> std::io::Result<u64> {
            match fs::copy(from, to) {
                Ok(bytes) => Ok(bytes),
                Err(_) => {
                    let mut reader = fs::File::open(from)?;
                    let mut writer = fs::File::create(to)?;
                    std::io::copy(&mut reader, &mut writer)
                }
            }
        };

        copy_shared(src_db, dst_db).map_err(|e| {
            format!(
                "Failed to copy SQLite database file {} to {}: {}",
                src_db.display(),
                dst_db.display(),
                e
            )
        })?;

        for suffix in &["-wal", "-shm"] {
            let mut src_sidecar = src_db.as_os_str().to_os_string();
            src_sidecar.push(suffix);
            let src_sidecar_path = PathBuf::from(src_sidecar);

            let mut dst_sidecar = dst_db.as_os_str().to_os_string();
            dst_sidecar.push(suffix);
            let dst_sidecar_path = PathBuf::from(dst_sidecar);

            if src_sidecar_path.exists() {
                // Justification: best-effort sidecar copy; the destination db re-creates its WAL/SHM sidecars on open
                crate::error::record_ignored(
                    copy_shared(&src_sidecar_path, &dst_sidecar_path),
                    "copy db sidecar file",
                );
            }
        }
    }

    Ok(())
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        if let Err(e) = fs::create_dir_all(dst) {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Failed to create directory {}: {}",
                dst.display(),
                e
            ));
            return Ok(());
        }
    }

    let entries = match fs::read_dir(src) {
        Ok(entries) => entries,
        Err(e) => {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Failed to read directory {}: {}",
                src.display(),
                e
            ));
            return Ok(());
        }
    };

    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy().to_lowercase();

        // Skip volatile lock files, active socket handles, crashpad, and volatile caches
        let is_lock = name_str == "lockfile"
            || name_str.ends_with(".lock")
            || name_str.starts_with("singleton");
        let is_volatile_cache = name_str == "code cache"
            || name_str == "gpucache"
            || name_str == "dawngraphitecache"
            || name_str == "blob_storage"
            || name_str == "service worker"
            || name_str == "crashpad"
            || name_str.starts_with(".org.chromium");
        if is_lock || is_volatile_cache {
            continue;
        }

        let dest_child = dst.join(&file_name);
        if file_type.is_dir() {
            // Justification: best-effort tree copy; parity sync completes it on launch
            crate::error::record_ignored(
                copy_dir_recursive(&entry.path(), &dest_child),
                "copy directory tree",
            );
        } else if name_str.ends_with(".vscdb") || name_str.ends_with(".db") {
            if let Err(e) = safe_clone_sqlite_db(&entry.path(), &dest_child) {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Failed to safe clone SQLite db {}: {}",
                    entry.path().display(),
                    e
                ));
            }
        } else if name_str.ends_with(".vscdb-wal")
            || name_str.ends_with(".vscdb-shm")
            || name_str.ends_with(".db-wal")
            || name_str.ends_with(".db-shm")
        {
            // Handled alongside parent database in safe_clone_sqlite_db
            continue;
        } else {
            match fs::copy(entry.path(), &dest_child) {
                Ok(_) => {}
                Err(err) => {
                    let copy_stream = || -> std::io::Result<u64> {
                        let mut reader = fs::File::open(entry.path())?;
                        let mut writer = fs::File::create(&dest_child)?;
                        std::io::copy(&mut reader, &mut writer)
                    };
                    if let Err(e) = copy_stream() {
                        crate::modules::logger::log_warn(&format!(
                            "[Instance] Could not copy file {}: primary: {}, fallback: {}",
                            entry.path().display(),
                            err,
                            e
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
