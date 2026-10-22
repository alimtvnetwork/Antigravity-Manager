//! Profile Sandboxed Directory Validator.
//! Verifies that profile storage directories exist, have valid filesystem write permissions,
//! and reside strictly within allowed app data roots to prevent path traversal outside sandbox boundaries.

use std::path::{Path, PathBuf};

/// Validation error outcomes for instance directories.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ProfileDirValidationError {
    #[error("Directory path does not exist: {0}")]
    NotFound(String),
    #[error("Path is a file, not a directory: {0}")]
    NotADirectory(String),
    #[error("Directory path escapes the allowed base root {base:?}: {path:?}")]
    PathTraversalDetected { path: PathBuf, base: PathBuf },
    #[error("Directory lacks write permissions: {0}")]
    WritePermissionDenied(String),
}

/// Validates that a user data directory is contained within the authorized base root.
pub fn validate_profile_directory(
    path: &Path,
    allowed_base_root: &Path,
) -> Result<PathBuf, ProfileDirValidationError> {
    if !path.exists() {
        return Err(ProfileDirValidationError::NotFound(path.display().to_string()));
    }

    if !path.is_dir() {
        return Err(ProfileDirValidationError::NotADirectory(path.display().to_string()));
    }

    // Canonicalize to resolve symlinks and directory dots
    let canonical_path = path
        .canonicalize()
        .map_err(|_| ProfileDirValidationError::NotFound(path.display().to_string()))?;

    let canonical_base = allowed_base_root
        .canonicalize()
        .map_err(|_| ProfileDirValidationError::NotFound(allowed_base_root.display().to_string()))?;

    if !canonical_path.starts_with(&canonical_base) {
        return Err(ProfileDirValidationError::PathTraversalDetected {
            path: canonical_path,
            base: canonical_base,
        });
    }

    // Check writeability by probing directory metadata
    if let Ok(meta) = canonical_path.metadata() {
        if meta.permissions().readonly() {
            return Err(ProfileDirValidationError::WritePermissionDenied(
                canonical_path.display().to_string(),
            ));
        }
    }

    Ok(canonical_path)
}
