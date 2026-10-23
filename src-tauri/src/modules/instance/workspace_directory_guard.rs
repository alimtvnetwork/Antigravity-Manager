// Instance Workspace Subdirectory Guard (Imp-504)
// Validates and pre-creates required instance directory hierarchies
// (`User`, `logs`, `workspaceStorage`) prior to process initialization.

use std::fs;
use std::path::Path;
use tracing::debug;

pub const REQUIRED_INSTANCE_SUBDIRS: &[&str] = &["User", "logs", "workspaceStorage", "Backups"];

pub struct WorkspaceDirectoryGuard;

impl WorkspaceDirectoryGuard {
    /// Ensures that all standard subdirectories exist within the instance data folder.
    /// Returns the count of directories newly created.
    pub fn ensure_subdirectories<P: AsRef<Path>>(data_dir: P) -> std::io::Result<usize> {
        let base = data_dir.as_ref();
        let mut created_count = 0;

        for &subdir in REQUIRED_INSTANCE_SUBDIRS {
            let target_path = base.join(subdir);
            if !target_path.exists() {
                fs::create_dir_all(&target_path)?;
                debug!(path = %target_path.display(), "Created missing instance subdirectory");
                created_count += 1;
            }
        }

        Ok(created_count)
    }

    /// Verifies if all required subdirectories already exist.
    pub fn verify_subdirectories_exist<P: AsRef<Path>>(data_dir: P) -> bool {
        let base = data_dir.as_ref();
        REQUIRED_INSTANCE_SUBDIRS
            .iter()
            .all(|&subdir| base.join(subdir).exists())
    }
}
