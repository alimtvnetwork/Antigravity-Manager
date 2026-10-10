//! Repo DB: text utils

use rusqlite::params;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

/// Extract clean prompt text by stripping <USER_REQUEST> / <ADDITIONAL_METADATA> tags if present
pub fn extract_clean_user_prompt(raw: &str) -> String {
    let mut text = raw.trim();
    if let Some(start) = text.find("<USER_REQUEST>") {
        let after = &text[start + "<USER_REQUEST>".len()..];
        if let Some(end) = after.find("</USER_REQUEST>") {
            text = &after[..end];
        } else {
            text = after;
        }
    }
    if let Some(end) = text.find("<ADDITIONAL_METADATA>") {
        text = &text[..end];
    }
    if let Some(end) = text.find("<CONTEXT_SUMMARY>") {
        text = &text[..end];
    }
    text.trim().to_string()
}

/// Extract smart, readable summary of a prompt for display in CLI and Telegram.
/// Strips XML tags, skips leading commit SHAs, file paths, markdown headers, and extracts the primary human directive.
pub fn extract_smart_prompt_summary(raw: &str, max_len: usize) -> String {
    let cleaned = extract_clean_user_prompt(raw);
    let mut substantive_lines = Vec::new();

    for line in cleaned.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if line is a commit hash or list of hashes (e.g. 40 hex chars or 10-64 hex chars)
        let is_hex_sha = (trimmed.len() == 40 || (trimmed.len() >= 8 && trimmed.len() <= 64))
            && trimmed.chars().all(|c| c.is_ascii_hexdigit());
        if is_hex_sha {
            continue;
        }

        // Check if line is a pure file path (e.g. D:\work\... or C:\... or /path/...)
        if trimmed.starts_with("D:\\")
            || trimmed.starts_with("C:\\")
            || trimmed.starts_with("file:///")
            || (trimmed.starts_with('/') && !trimmed.starts_with("/prompt"))
        {
            continue;
        }

        // Check if line is a markdown header (# Title), horizontal rule (---, ***), or blockquote (> ...)
        if trimmed.starts_with('#')
            || trimmed.starts_with('>')
            || trimmed.starts_with("---")
            || trimmed.starts_with("***")
        {
            continue;
        }

        // Check for common prompt preamble / telemetry boilerplates
        let lower = trimmed.to_lowercase();
        if lower.starts_with("prompt version:")
            || lower.starts_with("synchronization:")
            || lower.starts_with("top-instruction priority mandate")
            || lower.starts_with("application:")
            || lower.starts_with("error id:")
            || lower.starts_with("severity level:")
            || lower.starts_with("captured at:")
            || lower.starts_with("route / page:")
            || lower.starts_with("source:")
            || lower.starts_with("code:")
            || lower.starts_with("traceability id:")
        {
            continue;
        }

        substantive_lines.push(trimmed);
    }

    let candidate = if !substantive_lines.is_empty() {
        substantive_lines.join(" ")
    } else {
        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    };

    let collapsed = candidate.split_whitespace().collect::<Vec<_>>().join(" ");
    let limit = if max_len == 0 { 80 } else { max_len };

    if collapsed.chars().count() > limit {
        let truncated: String = collapsed.chars().take(limit).collect();
        format!("{}...", truncated.trim_end())
    } else {
        collapsed
    }
}

/// Resolve the current Git branch for a given repository directory
pub fn get_git_branch_for_path(repo_path: &str) -> Option<String> {
    let p = Path::new(repo_path);
    if !p.exists() {
        return None;
    }
    let git_head = p.join(".git").join("HEAD");
    if git_head.exists() {
        if let Ok(content) = fs::read_to_string(&git_head) {
            let line = content.trim();
            if let Some(branch) = line.strip_prefix("ref: refs/heads/") {
                return Some(branch.trim().to_string());
            }
        }
    }
    // Worktree or submodule where .git is a file
    let git_file = p.join(".git");
    if git_file.is_file() {
        if let Ok(content) = fs::read_to_string(&git_file) {
            if let Some(gitdir_rel) = content.trim().strip_prefix("gitdir:") {
                let gitdir_path = p.join(gitdir_rel.trim());
                let sub_head = gitdir_path.join("HEAD");
                if sub_head.exists() {
                    if let Ok(c) = fs::read_to_string(&sub_head) {
                        let line = c.trim();
                        if let Some(branch) = line.strip_prefix("ref: refs/heads/") {
                            return Some(branch.trim().to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Query conversation title from Antigravity conversation_summaries.db
pub fn lookup_conversation_title(prefix_or_id: &str) -> Option<String> {
    let clean = prefix_or_id.trim();
    if clean.is_empty() {
        return None;
    }
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")))?;
    let summaries_db = base_dir.join("conversation_summaries.db");
    if !summaries_db.exists() {
        return None;
    }
    let conn = Connection::open_with_flags(
        &summaries_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .or_else(|_| {
        let uri = format!(
            "file:{}?immutable=1",
            summaries_db.to_string_lossy().replace('\\', "/")
        );
        Connection::open_with_flags(
            &uri,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
    })
    .ok()?;
    // Justification: busy_timeout on a third-party read-only database is a nicety; reads continue with the default
    crate::error::record_ignored(
        conn.pragma_update(None, "busy_timeout", 2000),
        "set busy_timeout on Antigravity summaries database",
    );

    let pattern = format!("%{}%", clean);
    let mut stmt = conn
        .prepare("SELECT title FROM conversation_summaries WHERE conversation_id LIKE ? OR project_id LIKE ? LIMIT 1")
        .ok()?;
    let mut rows = stmt
        .query_map(params![pattern, pattern], |r| r.get::<_, String>(0))
        .ok()?;
    if let Some(Ok(title)) = rows.next() {
        let t = title.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    None
}

/// Format human-friendly workspace label (e.g. `Antigravity-Manager (main) [AGM]`)
pub fn format_friendly_workspace_label(
    project_id: &str,
    repo_name: &str,
    repo_path: &str,
) -> String {
    // 1. Resolve repository name cleanly
    let base_name = if !repo_name.trim().is_empty()
        && (!repo_name.contains('-') || !repo_name.chars().any(|c| c.is_ascii_digit()))
    {
        repo_name.trim().to_string()
    } else {
        Path::new(repo_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                if let Some(idx) = project_id.rfind('-') {
                    let suffix = &project_id[idx + 1..];
                    if suffix.len() >= 8 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
                        return project_id[..idx].to_string();
                    }
                }
                project_id.to_string()
            })
    };

    // 2. Resolve Git branch
    let branch_opt = get_git_branch_for_path(repo_path);

    // 3. Resolve Conversation title
    let id_part = if let Some(idx) = project_id.rfind('-') {
        &project_id[idx + 1..]
    } else {
        project_id
    };
    let title_opt = lookup_conversation_title(id_part);

    // 4. Construct composite label
    let mut label = base_name;
    if let Some(b) = branch_opt {
        label.push_str(&format!(" ({})", b));
    }
    if let Some(t) = title_opt {
        let t_trim = t.trim();
        if !t_trim.is_empty() && !label.to_lowercase().contains(&t_trim.to_lowercase()) {
            label.push_str(&format!(" [{}]", t_trim));
        }
    }
    label
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_clean_user_prompt() {
        let raw = "<USER_REQUEST>\nFix the auto-switch prompt resumption bug\n</USER_REQUEST>\n<ADDITIONAL_METADATA>...</ADDITIONAL_METADATA>";
        let clean = extract_clean_user_prompt(raw);
        assert_eq!(clean, "Fix the auto-switch prompt resumption bug");

        let simple = "Simple task prompt";
        assert_eq!(extract_clean_user_prompt(simple), "Simple task prompt");
    }

    #[test]
    fn test_extract_smart_prompt_summary() {
        let raw = "<USER_REQUEST>\n808e01722752a5c3b788899692cb8e83ade8b09a\n5f667758711619c8bb4f30304ab16a682659fd18\n\nD:\\work\\repo-secrets\n\nCan you please keep the secrets of the Telegram and other stuff?\n</USER_REQUEST>";
        let summary = extract_smart_prompt_summary(raw, 60);
        assert!(summary.starts_with("Can you please keep the secrets of the Telegram"));
        assert!(!summary.contains("808e01722752a5c3b788899692cb8e83ade8b09a"));
        assert!(!summary.contains("D:\\work\\repo-secrets"));
        assert!(!summary.contains("<USER_REQUEST>"));
    }

    #[test]
    fn test_format_friendly_workspace_label() {
        let label = format_friendly_workspace_label(
            "antigravity-manager-d58c5517",
            "antigravity-manager-d58c5517",
            "d:/work/Antigravity-Manager",
        );
        assert!(label.starts_with("Antigravity-Manager"));
    }
}
