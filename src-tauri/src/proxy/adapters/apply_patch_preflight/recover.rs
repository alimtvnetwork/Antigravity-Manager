// Patch recovery (split from apply_patch_preflight.rs).
use super::repair::resolve_path;
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

/// **Rule: `Update File` target is an empty file → `Delete File + Add File`** (prompt gotcha #3, lossless).
/// `*** Update File:` cannot operate on an empty file (Codex reports `cannot operate on a completely empty file`).
/// When the target file exists and is empty (really 0 bytes, not pure blank), and the Update body is a pure `+` line** (plain writing content, no `-`/context is available
/// When matching), it is converted into `*** Delete File: X` + `*** Add File:
/// body contains `-`/context (the model writes matching lines on an empty file, which is inherently contradictory)/ contains Move (leave it to the empty-move rule) → does not move. Requires `cwd`.
pub(crate) fn recover_update_empty_file(v4a: &str, cwd: Option<&str>) -> (String, Vec<Repair>) {
    let Some(cwd) = cwd else {
        return (v4a.to_owned(), Vec::new());
    };
    if !v4a.contains("*** Update File:") {
        return (v4a.to_owned(), Vec::new());
    }
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if let Some(path) = lines[i].strip_prefix("*** Update File: ") {
            let p = path.trim();
            // Only recognize **real 0 bytes** (Codex only reports an error for `completely empty file`; pure blank files are still readable content,
            // can update normally). Using `c.trim().is_empty()` will also convert pure blank files Delete+Add → throw away those
            // Blank bytes (destructive, codex-connector #435 P2).
            let is_empty = std::fs::read_to_string(resolve_path(p, cwd))
                .map(|c| c.is_empty())
                .unwrap_or(false);
            if is_empty {
                let body_start = i + 1;
                let mut j = body_start;
                while j < lines.len() && !lines[j].starts_with("*** ") {
                    j += 1;
                }
                let body = &lines[body_start..j];
                let has_move = body
                    .first()
                    .map(|l| l.starts_with("*** Move to:"))
                    .unwrap_or(false);
                let content: Vec<&&str> = body
                    .iter()
                    .filter(|l| !l.trim().is_empty() && !l.starts_with("@@"))
                    .collect();
                let all_plus = !content.is_empty() && content.iter().all(|l| l.starts_with('+'));
                if !has_move && all_plus {
                    out.push(format!("*** Delete File: {p}"));
                    out.push(format!("*** Add File: {p}"));
                    for b in body {
                        if b.starts_with('+') {
                            out.push((*b).to_owned());
                        }
                    }
                    repairs.push(Repair {
                        file: p.to_owned(),
                        kind: "repaired".to_owned(),
                        detail: "Update empty file → Delete+Add write (prompt gotcha #3)"
                            .to_owned(),
                    });
                    i = j;
                    continue;
                }
            }
        }
        out.push(lines[i].to_owned());
        i += 1;
    }
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}

/// **Rule: Empty `Update File + Move to`(rename-only)→ `Delete File + Add File`**(prompt gotcha #7).
/// I want to simply rename the model but write `*** Update File: X` + `*** Move to: Y` and **no hunk** → Codex report
/// `Update file hunk for path 'X' is empty`. Press prompt **My own suggestion** to restore: Read X original content, convert to
/// `*** Delete File: Unable to read X → Leave as is.
pub(crate) fn recover_empty_move(v4a: &str, cwd: Option<&str>) -> (String, Vec<Repair>) {
    let Some(cwd) = cwd else {
        return (v4a.to_owned(), Vec::new());
    };
    if !v4a.contains("*** Move to:") {
        return (v4a.to_owned(), Vec::new());
    }
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        // Matches `*** Update File: X` immediately followed by `*** Move to: Y`, and there is no hunk line between the Move and the next `*** ` control line.
        if let Some(old) = lines[i].strip_prefix("*** Update File: ") {
            if i + 1 < lines.len() {
                if let Some(new) = lines[i + 1].strip_prefix("*** Move to: ") {
                    // See if there is a hunk content line after Move and before the next **file operation** control line.
                    // Note: `*** End of File` is a documented **hunk internal mark** (prompt RENAME/MOVE section), not
                    // Section boundary - cannot stop at it (otherwise rename+EOF append will be misjudged as empty rename and converted into missing content)
                    // Delete+Add,codex-connector #435 P1). It itself means "there is hunk", keep scanning back.
                    let mut j = i + 2;
                    let mut has_hunk = false;
                    while j < lines.len() {
                        let t = lines[j];
                        if t.trim_end() == "*** End of File" {
                            has_hunk = true;
                            j += 1;
                            continue;
                        }
                        if t.starts_with("*** ") {
                            break; // True next file operation/End Patch boundary
                        }
                        if t.starts_with('+')
                            || t.starts_with('-')
                            || t.starts_with(' ')
                            || t.starts_with("@@")
                        {
                            has_hunk = true;
                        }
                        j += 1;
                    }
                    if !has_hunk {
                        // Empty rename-only → read the original file and transfer to Delete+Add. Unable to read/The content is empty → Do not transfer (empty Add File
                        // The body may be rejected by Codex)→ Release it as is and hand it over to Codex for processing.
                        let abs = resolve_path(old.trim(), cwd);
                        match std::fs::read_to_string(&abs) {
                            Ok(content) if !content.is_empty() => {
                                out.push(format!("*** Delete File: {}", old.trim()));
                                out.push(format!("*** Add File: {}", new.trim()));
                                for cl in content.lines() {
                                    out.push(format!("+{cl}"));
                                }
                                repairs.push(Repair {
                                    file: old.trim().to_owned(),
                                    kind: "repaired".to_owned(),
                                    detail: format!(
                                        "Empty Update+Move(rename-only)→ Delete+Add Copy original content → {} (prompt gotcha #7)",
                                        new.trim()
                                    ),
                                });
                                i = j; // Skip the original Update/Move(+empty body)
                                continue;
                            }
                            _ => {
                                repairs.push(Repair {
                                    file: old.trim().to_owned(),
                                    kind: "skipped:unreadable_or_empty".to_owned(),
                                    detail: "Empty Update+Move but the original file cannot be read / is empty → release as is"
                                        .to_owned(),
                                });
                            }
                        }
                    }
                }
            }
        }
        out.push(lines[i].to_owned());
        i += 1;
    }
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}

/// Whether the last patch operation is "`*** Add File:` + code/structured configuration file target". Used for [`ensure_v4a_envelope`] determination
/// Can the last line `+*** End Patch` be safely stripped into a terminator: **Only Add File** (new file, bare `*** End Patch` cannot be legal
/// The **last line** of the source code → must be an incorrect prefix terminator) before stripping; the `+*** End Patch` of `*** Update File:` is a **new line** (maybe to
/// Add this string of words to string / fixture), stripped = lost and added → not stripped (chatgpt-codex-connector review: limited to Add File).
/// Document/text/Unknown extensions will not be stripped (it may be the main text, leave incomplete and do not guess). allowlist is conservative. MOC-268.
pub(crate) fn last_op_is_add_file_code(body: &str) -> bool {
    let last_op = body.lines().rev().find(|l| {
        let t = l.trim_end();
        t.starts_with("*** Add File: ")
            || t.starts_with("*** Update File: ")
            || t.starts_with("*** Delete File: ")
    });
    let Some(path) = last_op.and_then(|l| l.trim_end().strip_prefix("*** Add File: ")) else {
        return false; // No operation, or the last operation is Update/Delete (not Add File) → No stripping
    };
    let ext = std::path::Path::new(path.trim())
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase());
    matches!(
        ext.as_deref(),
        Some(
            "rs" | "ts"
                | "tsx"
                | "js"
                | "jsx"
                | "mjs"
                | "cjs"
                | "py"
                | "go"
                | "java"
                | "kt"
                | "kts"
                | "c"
                | "h"
                | "cc"
                | "cpp"
                | "cxx"
                | "hpp"
                | "hh"
                | "cs"
                | "rb"
                | "php"
                | "swift"
                | "scala"
                | "lua"
                | "sql"
                | "sh"
                | "bash"
                | "zsh"
                | "css"
                | "scss"
                | "sass"
                | "less"
                | "html"
                | "htm"
                | "xml"
                | "vue"
                | "svelte"
                | "json"
                | "toml"
                | "yaml"
                | "yml"
                | "gradle"
                | "cmake"
                | "proto"
                | "graphql"
                | "dart"
                | "r"
        )
    )
}
