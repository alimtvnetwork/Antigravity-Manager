// v4a validation (split from apply_patch_preflight.rs).
use super::cwd::{anchor_probe, has_cwd_candidate};
use super::normalize::is_unified_hunk_range_header;
use super::read::read_patch_file;
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

/// Post-verification before sending to Codex to customize apply_patch. When an explicitly illegal V4A is found, the caller should replace the tool item
/// Mark it as incomplete to avoid feeding the failure history back to the model after the Codex is executed to form a loop.
pub fn validate_v4a_for_codex(v4a: &str) -> Option<(usize, String)> {
    let meaningful: Vec<(usize, &str)> = v4a
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .collect();
    let Some((first_line_no, first)) = meaningful.first() else {
        return Some((1, "empty apply_patch input".to_string()));
    };
    if first.trim() != "*** Begin Patch" {
        return Some((
            first_line_no + 1,
            "apply_patch input must start with *** Begin Patch".to_string(),
        ));
    }
    let Some((last_line_no, last)) = meaningful.last() else {
        return Some((1, "empty apply_patch input".to_string()));
    };
    if last.trim() != "*** End Patch" {
        return Some((
            last_line_no + 1,
            "apply_patch input must end with *** End Patch".to_string(),
        ));
    }

    let mut has_hunk = false;
    for (idx, line) in v4a.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "*** Begin Patch" && idx != *first_line_no {
            return Some((
                idx + 1,
                "apply_patch input contains a nested or repeated *** Begin Patch".to_string(),
            ));
        }
        if trimmed == "*** End Patch" && idx != *last_line_no {
            return Some((
                idx + 1,
                "apply_patch input contains an early or repeated *** End Patch".to_string(),
            ));
        }
        if trimmed.starts_with("*** Add File:")
            || trimmed.starts_with("*** Update File:")
            || trimmed.starts_with("*** Delete File:")
        {
            has_hunk = true;
        }
        if line.starts_with("--- ") || line.starts_with("+++ ") {
            return Some((
                idx + 1,
                "V4A apply_patch does not accept unified diff file header lines (---/+++)"
                    .to_string(),
            ));
        }
        if is_unified_hunk_range_header(line) {
            return Some((
                idx + 1,
                "V4A apply_patch uses bare @@ or @@ text anchors, not unified @@ -a,+b ranges"
                    .to_string(),
            ));
        }
        if idx != *first_line_no && idx != *last_line_no && !trimmed.is_empty() {
            match line.chars().next() {
                Some('+') | Some('-') | Some(' ') => {}
                _ if trimmed.starts_with("@@") => {}
                _ if trimmed.starts_with("*** Add File:")
                    || trimmed.starts_with("*** Update File:")
                    || trimmed.starts_with("*** Delete File:")
                    || trimmed.starts_with("*** Move to:")
                    || trimmed == "*** End of File" => {}
                _ if trimmed.starts_with("***") => {
                    return Some((
                        idx + 1,
                        format!(
                            "unrecognized V4A operation: {}",
                            trimmed.chars().take(80).collect::<String>()
                        ),
                    ));
                }
                _ => {
                    return Some((
                        idx + 1,
                        "line missing V4A prefix (expected '+', '-', ' ', '@@', or '*** ' marker)"
                            .to_string(),
                    ));
                }
            }
        }
    }
    if !has_hunk {
        return Some((
            1,
            "apply_patch input has no file operation hunk".to_string(),
        ));
    }
    None
}

/// **Rule G:Add File content line omission `+` prefix → completion**(grammar `add_hunk: … add_line+`,
/// `add_line: "+" /(.*)/`). Add File semantics = Each subsequent line is the **literal content** of the new file and must be prefixed with `+`;
/// The model occasionally misses writing `+` → Codex does not recognize it as content. **No ambiguity** in Add File section (all new), give non-`+` lines
/// Unified complement `+` (blank line → bare `+`); it is already the fixed version of `+` (not repeated into `++`). Pure string, no disk reading.
pub(crate) fn ensure_add_file_plus(v4a: &str) -> (String, Vec<Repair>) {
    if !v4a.contains("*** Add File:") {
        return (v4a.to_owned(), Vec::new());
    }
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if let Some(path) = lines[i].strip_prefix("*** Add File: ") {
            out.push(lines[i].to_owned()); // header
            i += 1;
            let mut fixed = 0usize;
            // body to the next `*** ` control line / EOF;Add File body is all `+` content lines.
            while i < lines.len() && !lines[i].starts_with("*** ") {
                if lines[i].starts_with('+') {
                    out.push(lines[i].to_owned());
                } else {
                    out.push(format!("+{}", lines[i]));
                    fixed += 1;
                }
                i += 1;
            }
            if fixed > 0 {
                repairs.push(Repair {
                    file: path.trim().to_owned(),
                    kind: "repaired".to_owned(),
                    detail: format!(
                        "Add File {fixed} line missing `+` prefix → completion (lark add_line)"
                    ),
                });
            }
        } else {
            out.push(lines[i].to_owned());
            i += 1;
        }
    }
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}

/// **Rule: `@@ <header>` Anchor alignment file real line** (real machine seq181: `Failed to find context 'X'`).
/// V4A's `@@ <header>` is a single-sided anchor point. Codex matches the section line in the file according to **exact whole line**; the model often writes **incomplete**
/// Header (such as `@@ System Architecture Suggestion`, and the real line of the file is `## 6. System Architecture Suggestion`) → The anchor point cannot be found. When `<header>` is not
/// When any **whole line** in the file happens to be uniquely contained in a certain file line, align `@@ <header>` to `@@ <whole line of the file>`;
/// 0/Multiple Contains → Ambiguity, leave it as is (no guessing). Naked `@@` (no header) does not move. Requires `cwd`.
pub(crate) fn align_at_headers(v4a: &str, cwd: Option<&str>) -> (String, Vec<Repair>) {
    if !has_cwd_candidate(cwd) {
        return (v4a.to_owned(), Vec::new());
    }
    if !v4a.contains("*** Update File:") {
        return (v4a.to_owned(), Vec::new());
    }
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut file_lines: Vec<String> = Vec::new();
    let mut have_file = false;
    let mut fixed = 0usize;
    let mut i = 0;
    while i < lines.len() {
        if let Some(path) = lines[i].strip_prefix("*** Update File: ") {
            // Switch to the new Update File section → parse the target file by candidate cwd + anchor probe (MOC-263 P1/P2)
            let mut se = i + 1;
            while se < lines.len() && !lines[se].starts_with("*** ") {
                se += 1;
            }
            let probe = anchor_probe(&lines[i + 1..se]);
            file_lines = read_patch_file(path.trim(), cwd, &probe)
                .map(|(_, c)| c.lines().map(str::to_owned).collect())
                .unwrap_or_default();
            have_file = !file_lines.is_empty();
            out.push(lines[i].to_owned());
            i += 1;
            continue;
        }
        // `@@ <header>` anchor (not naked `@@`), and the file has been loaded
        if have_file {
            if let Some(header) = lines[i].strip_prefix("@@ ") {
                let h = header.trim();
                if !h.is_empty() && !file_lines.iter().any(|fl| fl == h) {
                    let hits: Vec<&String> =
                        file_lines.iter().filter(|fl| fl.contains(h)).collect();
                    if hits.len() == 1 {
                        out.push(format!("@@ {}", hits[0]));
                        fixed += 1;
                        i += 1;
                        continue;
                    }
                }
            }
        }
        out.push(lines[i].to_owned());
        i += 1;
    }
    if fixed > 0 {
        repairs.push(Repair {
            file: "(@@ anchor)".to_owned(),
            kind: "repaired".to_owned(),
            detail: format!("@@ Anchor point is incomplete → Align the entire line of the file: {fixed} (Failed to find context)"),
        });
    }
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}
