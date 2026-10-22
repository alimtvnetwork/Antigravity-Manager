// Instance Directory Permission Pre-Flight Guard (Imp-534)
// Validates filesystem read and write permissions in instance data and home
// directories prior to process launch to prevent cryptic Win32 / POSIX EACCES failures.

use std::fs;
use std::path::Path;

pub struct DirectoryPermissionGuard;

impl DirectoryPermissionGuard {
    /// Tests write permission by creating and removing a transient probe file.
    pub fn verify_write_permission<P: AsRef<Path>>(dir: P) -> Result<(), String> {
        let path = dir.as_ref();
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| format!("Failed to create directory {}: {}", path.display(), e))?;
        }

        let probe_file = path.join(".agm_write_probe.tmp");
        fs::write(&probe_file, b"probe")
            .map_err(|e| format!("Directory {} is not writable: {}", path.display(), e))?;

        let _ = fs::remove_file(&probe_file);
        Ok(())
    }

    /// Verifies both data and home paths for an instance.
    pub fn verify_instance_directories(data_dir: &Path, home_dir: &Path) -> Result<(), String> {
        Self::verify_write_permission(data_dir)?;
        Self::verify_write_permission(home_dir)?;
        Ok(())
    }
}
