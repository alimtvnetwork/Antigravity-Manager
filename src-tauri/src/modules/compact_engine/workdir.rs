use crate::error::AppError;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Dynamic Work Directory Resolution Engine implementing the canonical 5-stage priority ladder.
pub struct WorkDirResolver;

impl WorkDirResolver {
    /// Resolves active workspace base directory using strict 5-stage priority hierarchy:
    /// P1: CLI parameter override (`--work-dir <path>`)
    /// P2: Environment variable `AGM_WORK_DIR`
    /// P3: GitMap master store query (`gitmap.db` -> `work_directories`)
    /// P4: Git ancestor root discovery (walk up from cwd for `.git`)
    /// P5: Platform default (`D:\work` or `~/work`)
    pub fn resolve_work_dir(cli_override: Option<&str>) -> Result<PathBuf, AppError> {
        // P1: CLI parameter override
        if let Some(raw) = cli_override {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                let p = PathBuf::from(trimmed);
                return Ok(normalize_canonical_path(&p));
            }
        }

        // P2: Environment variable AGM_WORK_DIR
        if let Ok(env_val) = std::env::var("AGM_WORK_DIR") {
            let trimmed = env_val.trim();
            if !trimmed.is_empty() {
                let p = PathBuf::from(trimmed);
                return Ok(normalize_canonical_path(&p));
            }
        }

        // P3: GitMap master store query (gitmap.db)
        if let Some(gitmap_dir) = Self::query_gitmap_default_workdir() {
            return Ok(gitmap_dir);
        }

        // P4: Git repository root ancestor of current working directory
        if let Some(git_ancestor_parent) = Self::detect_git_ancestor_parent() {
            return Ok(git_ancestor_parent);
        }

        // P5: Fallback platform default
        Ok(Self::platform_default_workdir())
    }

    /// Query GitMap default work directory from gitmap.db master store.
    pub fn query_gitmap_default_workdir() -> Option<PathBuf> {
        let candidates = get_gitmap_db_candidate_paths();
        for db_path in candidates {
            if !db_path.exists() {
                continue;
            }
            let conn = match Connection::open_with_flags(
                &db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Query work_directories table: preference for is_default = 1, then lowest id
            let query = "SELECT absolute_path FROM work_directories ORDER BY is_default DESC, id ASC LIMIT 1";
            if let Ok(mut stmt) = conn.prepare(query) {
                if let Ok(mut rows) = stmt.query([]) {
                    if let Ok(Some(row)) = rows.next() {
                        if let Ok(val) = row.get::<_, String>(0) {
                            let trimmed = val.trim();
                            if !trimmed.is_empty() {
                                let path = PathBuf::from(trimmed);
                                if path.exists() {
                                    return Some(normalize_canonical_path(&path));
                                }
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Detect parent folder of ancestor `.git` without spawning any git processes.
    pub fn detect_git_ancestor_parent() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        let mut curr: &Path = &cwd;
        while let Some(parent) = curr.parent() {
            if curr.join(".git").exists() {
                // curr is the repository root; its parent is the workspace directory containing it
                return Some(normalize_canonical_path(parent));
            }
            curr = parent;
        }
        None
    }

    /// Fallback default workspace path based on host operating system.
    pub fn platform_default_workdir() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            if Path::new(r"D:\work").exists() {
                PathBuf::from(r"D:\work")
            } else if Path::new(r"C:\work").exists() {
                PathBuf::from(r"C:\work")
            } else {
                dirs::home_dir()
                    .map(|h| h.join("work"))
                    .unwrap_or_else(|| PathBuf::from(r"D:\work"))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            dirs::home_dir()
                .map(|h| h.join("work"))
                .unwrap_or_else(|| PathBuf::from("/work"))
        }
    }
}

/// Discovers candidate filesystem paths for gitmap.db.
fn get_gitmap_db_candidate_paths() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(local_app_data) = dirs::data_local_dir() {
        candidates.push(
            local_app_data
                .join("gitmap-cli")
                .join("data")
                .join("gitmap.db"),
        );
    }
    #[cfg(target_os = "windows")]
    if let Ok(local_env) = std::env::var("LOCALAPPDATA") {
        candidates.push(
            PathBuf::from(local_env)
                .join("gitmap-cli")
                .join("data")
                .join("gitmap.db"),
        );
    }
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".gitmap").join("gitmap.db"));
        candidates.push(
            home.join(".local")
                .join("share")
                .join("gitmap-cli")
                .join("data")
                .join("gitmap.db"),
        );
    }
    candidates.push(PathBuf::from("cli").join("db").join("gitmap.db"));
    candidates.push(PathBuf::from("data").join("gitmap.db"));

    candidates
}

/// Normalizes a path, resolving symlinks/canonical representations where possible and stripping Windows `\\?\` prefix.
pub fn normalize_canonical_path(path: &Path) -> PathBuf {
    let raw = match std::fs::canonicalize(path) {
        Ok(c) => c,
        Err(_) => path.to_path_buf(),
    };
    let s = raw.to_string_lossy().to_string();
    let clean = clean_path_string(&s);
    PathBuf::from(clean)
}

/// Cleans a path string: strips Windows UNC prefix and normalizes backslashes to forward slashes.
pub fn clean_path_string(input: &str) -> String {
    let stripped = input.trim_start_matches(r"\\?\").trim_start_matches("//?/");
    let replaced = stripped.replace('\\', "/");
    let trimmed = replaced.trim_end_matches('/');
    if trimmed.is_empty() {
        if replaced.starts_with('/') {
            "/".to_string()
        } else {
            ".".to_string()
        }
    } else {
        trimmed.to_string()
    }
}

/// Normalizes any path to a forward-slash clean string.
pub fn clean_path_str(path: &Path) -> String {
    clean_path_string(&path.to_string_lossy())
}

/// Transforms an absolute or workspace path into a relative forward-slash path against `work_dir`.
pub fn to_relative_work_path(abs_path: &Path, work_dir: &Path) -> Result<String, AppError> {
    let target_clean = clean_path_str(abs_path);
    let work_clean = clean_path_str(work_dir);

    // If target is already relative, keep it normalized
    if !abs_path.is_absolute() && !target_clean.contains(':') && !target_clean.starts_with('/') {
        return Ok(target_clean);
    }

    // Direct match with work_dir
    if target_clean.eq_ignore_ascii_case(&work_clean) {
        return Ok(".".to_string());
    }

    let work_prefix = format!("{}/", work_clean);
    #[cfg(target_os = "windows")]
    let starts_with_work = target_clean
        .to_lowercase()
        .starts_with(&work_prefix.to_lowercase());
    #[cfg(not(target_os = "windows"))]
    let starts_with_work = target_clean.starts_with(&work_prefix);

    if starts_with_work {
        let rel_slice = &target_clean[work_prefix.len()..];
        return Ok(rel_slice.trim_start_matches('/').to_string());
    }

    // Attempt Path::strip_prefix
    if let Ok(rel) = abs_path.strip_prefix(work_dir) {
        let rel_clean = clean_path_str(rel);
        if !rel_clean.is_empty() {
            return Ok(rel_clean);
        }
    }

    // If outside work_dir and cannot be cleanly relative, return trimmed forward-slash path
    Ok(target_clean)
}

/// Convenience alias for `to_relative_work_path`.
pub fn to_relative_path(target_path: &Path, base_work_dir: &Path) -> Result<String, AppError> {
    to_relative_work_path(target_path, base_work_dir)
}

/// Re-anchors a relative path against the target workspace directory.
/// Also expands `${workDir}/` or `${repoDir}/` if present.
pub fn from_relative_work_path(rel_path: &str, work_dir: &Path) -> PathBuf {
    let trimmed = rel_path.trim();

    // Check for variable prefixes
    let path_part = if let Some(stripped) = trimmed.strip_prefix("${workDir}") {
        stripped.trim_start_matches('/').trim_start_matches('\\')
    } else if let Some(stripped) = trimmed.strip_prefix("${repoDir}") {
        stripped.trim_start_matches('/').trim_start_matches('\\')
    } else {
        trimmed
    };

    let p = Path::new(path_part);
    if p.is_absolute()
        || path_part.contains(':')
        || (path_part.starts_with('/') && !cfg!(target_os = "windows"))
    {
        PathBuf::from(path_part)
    } else {
        work_dir.join(path_part)
    }
}

/// Converts a relative path into canonical template syntax: `${workDir}/<relative_path>`.
pub fn to_template_path(rel_path: &str) -> String {
    let clean = clean_path_string(rel_path);
    let trimmed = clean.trim_start_matches('/').trim_start_matches('.');
    if trimmed.is_empty() {
        "${workDir}".to_string()
    } else {
        format!("${{workDir}}/{}", trimmed.trim_start_matches('/'))
    }
}

/// Expands a canonical template path `${workDir}/<relative_path>` using the provided work_dir.
pub fn expand_template_path(template_path: &str, work_dir: &Path) -> PathBuf {
    from_relative_work_path(template_path, work_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_path_string() {
        assert_eq!(
            clean_path_string(r"D:\work\Antigravity-Manager\"),
            "D:/work/Antigravity-Manager"
        );
        assert_eq!(clean_path_string(r"\\?\D:\work\app"), "D:/work/app");
    }

    #[test]
    fn test_to_relative_work_path() {
        let work = Path::new(r"D:\work");
        let target = Path::new(r"D:\work\Antigravity-Manager\src-tauri");
        let rel = to_relative_work_path(target, work).unwrap();
        assert_eq!(rel, "Antigravity-Manager/src-tauri");

        let exact = to_relative_work_path(work, work).unwrap();
        assert_eq!(exact, ".");
    }

    #[test]
    fn test_from_relative_work_path() {
        let work = Path::new(r"D:\work");
        let p = from_relative_work_path("Antigravity-Manager/src", work);
        assert_eq!(clean_path_str(&p), "D:/work/Antigravity-Manager/src");

        let p_tpl = from_relative_work_path("${workDir}/Antigravity-Manager", work);
        assert_eq!(clean_path_str(&p_tpl), "D:/work/Antigravity-Manager");
    }

    #[test]
    fn test_template_conversions() {
        assert_eq!(
            to_template_path("Antigravity-Manager/src"),
            "${workDir}/Antigravity-Manager/src"
        );
        assert_eq!(to_template_path("."), "${workDir}");
    }
}
