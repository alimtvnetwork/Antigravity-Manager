//! Repository-Relative Git Path Hygiene Linter.
//! Enforces:
//! - All documentation, changelogs, specs, and release artifacts must use strictly repository-relative Git paths.
//! - Forbids absolute Windows drive letters (e.g. `C:\`, `D:\`) and Linux root paths (`/home/...`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathHygieneReport {
    pub is_clean: bool,
    pub absolute_path_violations: Vec<String>,
}

/// Scans text lines and identifies forbidden absolute filesystem paths.
pub fn lint_path_hygiene(content: &str) -> PathHygieneReport {
    let mut violations = Vec::new();

    for line in content.lines() {
        for word in line.split_whitespace() {
            let trimmed = word.trim_matches(|c: char| {
                c == '`'
                    || c == '('
                    || c == ')'
                    || c == '['
                    || c == ']'
                    || c == '<'
                    || c == '>'
                    || c == '"'
                    || c == '\''
            });

            // Check Windows absolute paths: e.g. C:\ or D:/
            if trimmed.len() >= 3 {
                let first_char = trimmed.chars().next().unwrap_or('\0');
                let second_char = trimmed.chars().nth(1).unwrap_or('\0');
                let third_char = trimmed.chars().nth(2).unwrap_or('\0');

                if first_char.is_ascii_alphabetic()
                    && second_char == ':'
                    && (third_char == '\\' || third_char == '/')
                {
                    violations.push(trimmed.to_string());
                    continue;
                }
            }

            // Check Unix home paths: /home/ or /Users/
            if trimmed.starts_with("/home/") || trimmed.starts_with("/Users/") {
                violations.push(trimmed.to_string());
            }
        }
    }

    PathHygieneReport {
        is_clean: violations.is_empty(),
        absolute_path_violations: violations,
    }
}
