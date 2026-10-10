use super::lock::{
    ANTIGRAVITY_ACCOUNTS_FILE, ANTIGRAVITY_CONFIG_FILE, OPENCODE_CONFIG_FILE,
    OPENCODE_CONFIG_FILE_JSONC, OPENCODE_DIR,
};
use serde_json::Value;
#[cfg(test)]
use std::fs;
use std::path::PathBuf;

pub(crate) fn get_opencode_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(OPENCODE_DIR))
}

/// Resolve the active OpenCode config file name for the given directory.
///
/// OpenCode accepts both `opencode.json` and `opencode.jsonc`. To avoid creating a
/// parallel `opencode.json` next to a user's existing `opencode.jsonc`, we probe the
/// directory: if a `.jsonc` file already exists we keep using it, otherwise we fall
/// back to the default `opencode.json`. This keeps each user on the file they
/// already use.
///
/// Pass `Some(dir)` to probe a real directory, or `None` to get the default file
/// name without filesystem access (used by pure-function helpers).
pub(crate) fn resolve_active_config_file_name(dir: Option<&PathBuf>) -> &'static str {
    if let Some(dir) = dir {
        let jsonc = dir.join(OPENCODE_CONFIG_FILE_JSONC);
        if jsonc.exists() {
            return OPENCODE_CONFIG_FILE_JSONC;
        }
    }
    OPENCODE_CONFIG_FILE
}

pub(crate) fn get_config_paths() -> Option<(PathBuf, PathBuf, PathBuf)> {
    get_opencode_dir().map(|dir| {
        let config_file = resolve_active_config_file_name(Some(&dir));
        (
            dir.join(config_file),
            dir.join(ANTIGRAVITY_CONFIG_FILE),
            dir.join(ANTIGRAVITY_ACCOUNTS_FILE),
        )
    })
}

/// Strip JSONC comments (both `// line` and `/* block */`) so the remainder can be
/// parsed by `serde_json`, which only understands strict JSON.
///
/// This is intentionally a small, conservative scanner: it tracks whether the current
/// position is inside a string literal (respecting `\"` escapes) so that `//` or `/*`
/// appearing inside a string value is never mistaken for a comment. Trailing commas are
/// handled separately by [`strip_jsonc_trailing_commas`].
pub(crate) fn strip_jsonc_comments(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    let mut in_string = false;

    while i < bytes.len() {
        let c = bytes[i];

        if in_string {
            out.push(c);
            if c == b'\\' && i + 1 < bytes.len() {
                // Keep the escaped char verbatim (e.g. \", \\, \/).
                out.push(bytes[i + 1]);
                i += 2;
                continue;
            }
            if c == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match c {
            b'"' => {
                in_string = true;
                out.push(b'"');
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                // Line comment: skip until newline.
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                // Block comment: skip until closing */.
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }

    // Only ASCII syntax is removed; all UTF-8 string bytes remain intact.
    String::from_utf8(out).expect("JSONC normalization preserves UTF-8")
}

/// Remove trailing commas (a `,` followed, after optional whitespace, by a closing
/// `}` or `]`) so `serde_json` can parse JSONC that permits them. Like the comment
/// stripper, this respects string literals so a comma at the end of a string value is
/// never touched.
pub(crate) fn strip_jsonc_trailing_commas(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    let mut in_string = false;

    while i < bytes.len() {
        let c = bytes[i];

        if in_string {
            out.push(c);
            if c == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1]);
                i += 2;
                continue;
            }
            if c == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if c == b'"' {
            in_string = true;
            out.push(b'"');
            i += 1;
            continue;
        }

        // When we hit a comma outside a string, look ahead past whitespace. If the next
        // significant character closes an object/array, this is a trailing comma — drop
        // it. Otherwise keep the comma (it separates real elements).
        if c == b',' {
            let mut j = i + 1;
            let mut is_trailing = false;
            while j < bytes.len() {
                match bytes[j] {
                    b' ' | b'\t' | b'\n' | b'\r' => j += 1,
                    b'}' | b']' => {
                        is_trailing = true;
                        break;
                    }
                    _ => break,
                }
            }
            if is_trailing {
                // Skip the comma (don't push it); leave the whitespace to be pushed normally.
                i += 1;
            } else {
                out.push(b',');
                i += 1;
            }
            continue;
        }

        out.push(c);
        i += 1;
    }

    // Only ASCII syntax is removed; all UTF-8 string bytes remain intact.
    String::from_utf8(out).expect("JSONC normalization preserves UTF-8")
}

/// Read and parse an OpenCode config file, tolerating JSONC comments and trailing commas.
#[cfg(test)]
pub(crate) fn parse_config_file(path: &PathBuf) -> Option<Value> {
    let content = fs::read_to_string(path).ok()?;
    parse_jsonc(&content)
}

/// Parse a JSON/JSONC string: try strict JSON first, then normalize comments and
/// trailing commas and retry. Used everywhere a user opencode config is read.
pub(crate) fn parse_jsonc(content: &str) -> Option<Value> {
    serde_json::from_str(content)
        .or_else(|_| {
            let normalized = strip_jsonc_trailing_commas(&strip_jsonc_comments(content));
            serde_json::from_str(&normalized)
        })
        .ok()
}

pub(crate) fn extract_version(raw: &str) -> String {
    let trimmed = raw.trim();

    // Try to extract version from formats like "opencode/1.2.3" or "codex-cli 0.86.0"
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    for part in parts {
        // Check for format like "opencode/1.2.3"
        if let Some(slash_idx) = part.find('/') {
            let after_slash = &part[slash_idx + 1..];
            if is_valid_version(after_slash) {
                return after_slash.to_string();
            }
        }
        // Check if part itself looks like a version
        if is_valid_version(part) {
            return part.to_string();
        }
    }

    // Fallback: extract last sequence of digits and dots
    let version_chars: String = trimmed
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    if !version_chars.is_empty() && version_chars.contains('.') {
        return version_chars;
    }

    "unknown".to_string()
}

pub(crate) fn is_valid_version(s: &str) -> bool {
    // A valid version should start with digit and contain at least one dot
    s.chars().next().map_or(false, |c| c.is_ascii_digit())
        && s.contains('.')
        && s.chars().all(|c| c.is_ascii_digit() || c == '.')
}
