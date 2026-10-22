//! Workspace Storage Migrator & Re-anchor
//! Safely duplicates workspaceStorage session files between instance profiles,
//! preserving open editor tabs, split layout states, and project continuity.

use std::fs;
use std::path::Path;
use tracing::{debug, info};

pub struct WorkspaceStorageMigrator;

impl WorkspaceStorageMigrator {
    /// Copies workspaceStorage directory contents from source to destination,
    /// skipping volatile locks, socket files, and temporary caches.
    pub fn copy_workspace_storage(source_data_dir: &Path, dest_data_dir: &Path) -> std::io::Result<usize> {
        let src_storage = source_data_dir.join("User").join("workspaceStorage");
        let dst_storage = dest_data_dir.join("User").join("workspaceStorage");

        if !src_storage.exists() {
            return Ok(0);
        }

        if !dst_storage.exists() {
            fs::create_dir_all(&dst_storage)?;
        }

        let mut copied_workspaces = 0;
        let entries = match fs::read_dir(&src_storage) {
            Ok(iter) => iter,
            Err(_) => return Ok(0),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let folder_name = match path.file_name() {
                    Some(n) => n,
                    None => continue,
                };

                let target_folder = dst_storage.join(folder_name);
                if let Err(e) = Self::copy_dir_recursive(&path, &target_folder) {
                    debug!("Failed to copy workspace storage folder {:?}: {}", path, e);
                } else {
                    copied_workspaces += 1;
                }
            }
        }

        info!("Migrated {} workspaceStorage trees from {:?} to {:?}", copied_workspaces, src_storage, dst_storage);
        Ok(copied_workspaces)
    }

    fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }

        for entry in fs::read_dir(src)?.flatten() {
            let path = entry.path();
            let dest_path = dst.join(entry.file_name());

            if path.is_dir() {
                Self::copy_dir_recursive(&path, &dest_path)?;
            } else if path.is_file() {
                // Skip lock and socket files
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_ascii_lowercase();
                    if ext_lower == "lock" || ext_lower == "sock" {
                        continue;
                    }
                }
                let _ = fs::copy(&path, &dest_path);
            }
        }

        Ok(())
    }
}
