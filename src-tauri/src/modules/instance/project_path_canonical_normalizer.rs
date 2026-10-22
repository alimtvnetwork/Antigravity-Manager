//! Project path canonical normalizer.
//! Cleanses and standardizes project folder paths across Windows, macOS, and Linux
//! by resolving 8.3 short paths, normalizing directory slashes, and removing trailing separators.

use std::path::{Path, PathBuf};

pub struct ProjectPathCanonicalNormalizer;

impl ProjectPathCanonicalNormalizer {
    /// Normalizes raw filesystem paths into consistent forward-slash canonical representations.
    pub fn normalize(raw_path: &str) -> String {
        let trimmed = raw_path.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // Try canonicalizing if path exists on disk
        let path_buf = PathBuf::from(trimmed);
        let resolved = if let Ok(canonical) = path_buf.canonicalize() {
            canonical
        } else {
            path_buf
        };

        let mut formatted = resolved.to_string_lossy().to_string();

        // Strip Windows verbatim prefix (\\?\)
        if let Some(stripped) = formatted.strip_prefix(r"\\?\") {
            formatted = stripped.to_string();
        }

        // Standardize all backslashes to forward slashes
        let standardized = formatted.replace('\\', "/");

        // Strip trailing slash unless root directory (e.g. "C:/" or "/")
        if standardized.len() > 3 && standardized.ends_with('/') {
            standardized[..standardized.len() - 1].to_string()
        } else {
            standardized
        }
    }

    /// Checks if two divergent path representations resolve to the same underlying project folder.
    pub fn is_equivalent(path_a: &str, path_b: &str) -> bool {
        let norm_a = Self::normalize(path_a);
        let norm_b = Self::normalize(path_b);
        norm_a.eq_ignore_ascii_case(&norm_b)
    }
}
