// Patch normalization (split from apply_patch_preflight.rs).
use super::cwd::recall_cwd_candidates;
// apply_patch **pre-flight automatic repair**: Before sending the V4A patch to Codex apply, read the target file for comparison.
// Automatically align **safe** context mismatch (trailing spaces / leading and trailing whitespace differences), eliminating the number one failure of V4A
// `apply_patch verification failed: Failed to find expected lines`。
//
// ## Why is it needed?
// Weaker chat models (non-OpenAI) are often unable to reproduce the context/delete lines of `Update File` byte-by-byte on large files.
// (Trailing spaces, indentation, memory deviation) → Codex cannot find the anchor point → apply fails → the entire model file is rewritten, wasting time and tokens.
// The error reported by the actual machine (rollout ground truth) is exactly this type.
//
// ## Safety boundary (never damage files - align users with the hard rule of "no destructive downgrades")
// - **Move anchor point only**: context(space prefix)/ delete (`-`) line in `Update File`. `+New` lines **never change**.
// - **Only fix when there is a unique match**: The anchor block is found in the file by "Ignore trailing spaces/leading and trailing blanks" to find candidates, and **exactly one** position will be aligned;
//   0 (the model is really wrong) or ≥2 (ambiguous) will be **released as is** and handed over to Codex parse_patch to expose the real bad, never rely on guessing.
// - **Add File / Delete File does not touch** (no anchor, no matching involved). Unable to read file / no cwd → release as is.
// - Each repair/release is recorded in the apply-patch diagnostic page and can be audited.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::proxy::adapters::apply_patch_preflight::Repair;
use serde_json::Value;

/// A record of pre-flight processing (for diagnostic pages/logs).

pub(crate) fn diagnose_absolute_paths(v4a: &str, primary: Option<&str>) -> Vec<Repair> {
    let mut known_cwds: Vec<String> = Vec::new();
    if let Some(cwd) = primary {
        if !cwd.is_empty() {
            known_cwds.push(cwd.to_string());
        }
    }
    known_cwds.extend(recall_cwd_candidates());
    known_cwds.sort();
    known_cwds.dedup();

    if known_cwds.is_empty() {
        return Vec::new();
    }

    let cwd_paths: Vec<PathBuf> = known_cwds.iter().map(PathBuf::from).collect();
    let mut repairs = Vec::new();
    for line in v4a.lines() {
        let Some(file) = line
            .strip_prefix("*** Update File: ")
            .or_else(|| line.strip_prefix("*** Add File: "))
            .or_else(|| line.strip_prefix("*** Delete File: "))
        else {
            continue;
        };
        let file = file.trim();
        let path = Path::new(file);
        if !path.is_absolute() {
            continue;
        }
        let inside_known_cwd = cwd_paths.iter().any(|cwd| path.starts_with(cwd));
        if !inside_known_cwd {
            repairs.push(Repair {
                file: file.to_string(),
                kind: "diagnostic:absolute_path_outside_known_cwd".to_string(),
                detail: format!(
                    "absolute patch target is outside known cwd candidates: {}",
                    known_cwds.join(" | ")
                ),
            });
        }
    }
    repairs
}

/// **Rules: Bilateral `@@ … @@` → Unilateral `@@ …`**(prompt gotcha #1 / chat-path #1). V4A's `@@` is
/// **One-sided** anchor(`@@ <header>`); the model is often written as two-sided `@@ <header> @@`, Codex treats the trailing `@@` as literal text
/// → `Failed to find context '... @@'`. Only process the `@@` header line of **column 0** (the text line has `+`/`-`/space prefix, do not touch),
/// Remove the trailing `@@` and its leading whitespace; **the naked `@@` (section separation) remains unchanged**.
pub(crate) fn strip_trailing_at(v4a: &str) -> (String, Vec<Repair>) {
    let mut changed = 0usize;
    let out: Vec<String> = v4a
        .lines()
        .map(|l| {
            if l.starts_with("@@") {
                let t = l.trim_end();
                // Naked `@@`(len==2) is legal section separation and skipped; `@@ x @@` is used to remove the tail.
                if t.len() > 2 && t.ends_with("@@") {
                    let body = t[..t.len() - 2].trim_end();
                    if !body.is_empty() && body != "@@" {
                        changed += 1;
                        return body.to_owned();
                    }
                }
            }
            l.to_owned()
        })
        .collect();
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    let repairs = if changed > 0 {
        vec![Repair {
            file: "(@@ header)".to_owned(),
            kind: "repaired".to_owned(),
            detail: format!("Bilateral @@ → Unilateral: {changed} row (prompt gotcha #1)"),
        }]
    } else {
        Vec::new()
    };
    (joined, repairs)
}

fn normalized_diff_path(path: &str) -> Option<String> {
    let mut p = path.trim();
    if p == "/dev/null" || p.is_empty() {
        return None;
    }
    if (p.starts_with("a/") || p.starts_with("b/")) && p.len() > 2 {
        p = &p[2..];
    }
    Some(p.to_string())
}

/// Gemini often writes patches as unified diff on the non-native OpenAI apply_patch tool:
/// `--- path` / `+++ path` / `@@ -a,+b`. Codex V4A requires file operation headers
/// `*** Update File: path`, so only header conversion without semantic loss is done here.
pub(crate) fn convert_unified_file_headers(v4a: &str) -> (String, Vec<Repair>) {
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut i = 0usize;
    let mut converted = 0usize;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if let Some(old_path) = line.strip_prefix("--- ") {
            if let Some(next) = lines.get(i + 1) {
                if let Some(new_path) = next.strip_prefix("+++ ") {
                    if let Some(path) =
                        normalized_diff_path(new_path).or_else(|| normalized_diff_path(old_path))
                    {
                        out.push(format!("*** Update File: {path}"));
                        repairs.push(Repair {
                            file: path,
                            kind: "repaired".to_string(),
                            detail: "unified diff file headers -> V4A Update File".to_string(),
                        });
                        converted += 1;
                        i += 2;
                        continue;
                    }
                }
            }
        }

        if let Some(path) = trimmed
            .strip_prefix("file: ")
            .or_else(|| trimmed.strip_prefix("File: "))
            .and_then(normalized_diff_path)
        {
            out.push(format!("*** Update File: {path}"));
            repairs.push(Repair {
                file: path,
                kind: "repaired".to_string(),
                detail: "file: header -> V4A Update File".to_string(),
            });
            converted += 1;
            i += 1;
            continue;
        }

        out.push(line.to_string());
        i += 1;
    }

    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    if converted == 0 {
        (v4a.to_owned(), Vec::new())
    } else {
        (joined, repairs)
    }
}

fn is_unified_range_token(token: &str, sign: char) -> bool {
    let Some(rest) = token.strip_prefix(sign) else {
        return false;
    };
    let mut pieces = rest.split(',');
    let Some(start) = pieces.next() else {
        return false;
    };
    if start.is_empty() || !start.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if let Some(count) = pieces.next() {
        if count.is_empty() || !count.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    pieces.next().is_none()
}

pub(crate) fn is_unified_hunk_range_header(line: &str) -> bool {
    if !line.starts_with("@@") {
        return false;
    }
    let mut body = line[2..].trim();
    if let Some(stripped) = body.strip_suffix("@@") {
        body = stripped.trim_end();
    }
    let mut parts = body.split_whitespace();
    let Some(old_range) = parts.next() else {
        return false;
    };
    let Some(new_range) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && is_unified_range_token(old_range, '-')
        && is_unified_range_token(new_range, '+')
}

/// The `@@ -1,2 +1,3` line number of unified diff is not a V4A anchor. Codex will convert `@@ <text>`
/// It is regarded as the text context to be searched, so the pure line number range is adjusted to bare `@@`.
pub(crate) fn strip_unified_hunk_ranges(v4a: &str) -> (String, Vec<Repair>) {
    let mut changed = 0usize;
    let out: Vec<String> = v4a
        .lines()
        .map(|line| {
            if is_unified_hunk_range_header(line) {
                changed += 1;
                "@@".to_string()
            } else {
                line.to_string()
            }
        })
        .collect();
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    let repairs = if changed > 0 {
        vec![Repair {
            file: "(@@ range header)".to_owned(),
            kind: "repaired".to_owned(),
            detail: format!("unified @@ line number range -> V4A naked @@: {changed} line"),
        }]
    } else {
        Vec::new()
    };
    (joined, repairs)
}
