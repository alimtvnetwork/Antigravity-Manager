// v4a envelope (split from apply_patch_preflight.rs).
use super::cwd::{anchor_probe, has_cwd_candidate};
use super::read::read_patch_file;
use super::recover::last_op_is_add_file_code;
use super::repair::repair_update_section;
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

/// **Auto-complete for missing envelopes**: Models often only write `*** Add/Update File:` + content, leaving out `*** Begin Patch` /
/// `*** End Patch` Head and tail → Codex (and the V4A verification of this adapter) is judged incomplete → the model is forced to retry.
/// When the patch contains at least one `*** Add/Update/Delete File:` operation, the JSON is complete (the caller first gates
/// `detect_json_truncation` is None to use this function to ensure that it is not streaming truncation), but when the Begin/End envelope is missing,
/// **Pure complement mark** (do not change one byte of content, do not guess), return `(after completion, Some(Repair))`; inherently complete / non-patch body
/// Return `(as-is, None)`.
///
/// Safety: When Begin is missing, `*** Begin Patch` will be added to the front only if the first non-blank line is the operation line** (it will not change if there is leading prose,
/// Leave it to `repair_v4a_envelope` / Codex); when End is missing, remove the trailing blank and add `*** End Patch`.
pub fn ensure_v4a_envelope(input: &str) -> (String, Option<Repair>) {
    let is_op = |l: &str| {
        let t = l.trim_end();
        t.starts_with("*** Add File:")
            || t.starts_with("*** Update File:")
            || t.starts_with("*** Delete File:")
    };
    if !input.lines().any(is_op) {
        return (input.to_owned(), None); // Not a recognized patch body, do not touch
    }
    let has_begin = input.lines().any(|l| l.trim_end() == "*** Begin Patch");
    let has_end = input.lines().any(|l| l.trim_end() == "*** End Patch");
    if has_begin && has_end {
        return (input.to_owned(), None);
    }
    let mut body = input.to_owned();
    let mut added: Vec<&str> = Vec::new();
    if !has_begin {
        // It's only safe if the first non-blank line is the action line (no leading prose mixed into the envelope).
        let first_nonempty = input.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        if !is_op(first_nonempty) {
            return (input.to_owned(), None);
        }
        body = format!("*** Begin Patch\n{body}");
        added.push("Begin Patch");
    }
    if !has_end {
        let trimmed = body.trim_end();
        let last = trimmed.lines().last().unwrap_or("");
        // [MOC-268] **Only `+*** End Patch` (Add line prefix)** is the form of "the model mistakenly prefixes the terminator".
        // `*** End Patch`(context)/ `-*** End Patch`(deletion) is a legal Update hunk line** - e.g. model
        // Use Update to **delete** the previous remaining `*** End Patch` (`-*** End Patch`) in the file, or use it as a context anchor;
        // Treating them as terminators will **silently discard delete/destroy anchors**(chatgpt-codex-connector review)→ so ` `/`-` will always
        // Go to the following normal append (complement the terminator, leave the hunk line intact).
        // For `+*** End Patch` and then disambiguate by file type (user’s decision):
        //   · Code / structured configuration file (bare `*** End Patch` cannot be the last line of legal source code) → must be an incorrect prefix terminator → **Strip prefix**
        //     (`head` cuts to the starting point of the last line, retaining the newline before it; the last line is ASCII, boundary safe).
        //   · Document/Text/Unknown (maybe the last line of the text) → **No guess**: No stripping (no need to delete the text), no appending (no need to leave any residue), leaving incomplete
        //     The submitted data will be truncated by downstream judgment and the model will be reissued according to guidance rule 2. prompt is the cure, the middle layer only intervenes when it is safe.
        if last == "+*** End Patch" {
            if last_op_is_add_file_code(&body) {
                let head = &trimmed[..trimmed.len() - last.len()];
                body = format!("{head}*** End Patch");
                added.push("End Patch (code file·Patch plus prefix terminator)");
            } else {
                return (
                    body,
                    Some(Repair {
                        file: "(envelope)".to_owned(),
                        kind: "skipped:ambiguous_prefixed_end".to_owned(),
                        detail:
                            "The last line +*** End Patch and the target is a non-code file (maybe the text) → If you don’t guess or complete it, leave it incomplete"
                                .to_owned(),
                    }),
                );
            }
        } else {
            // Contains `*** End Patch` / `-*** End Patch` (legal hunk line) and the last line of normal content → normal complement terminator.
            body = format!("{trimmed}\n*** End Patch");
            added.push("End Patch");
        }
    }
    (
        body,
        Some(Repair {
            file: "(envelope)".to_owned(),
            kind: "repaired".to_owned(),
            detail: format!(
                "The model misses the envelope, auto-complete: {}",
                added.join(" + ")
            ),
        }),
    )
}

/// **Rule: **No prefix line in Update body**Complete based on file judgment** (Real machine seq235: single line missing prefix → validate reject →
/// The entire Update rewrite is a waste). grammar `change_line: ("+"|"-"|" ") /(.*)/` requires each line to be prefixed; model occasionally
/// Missing a line's prefix. **Non-destructive** repairs (only add prefixes / delete obsolete lines that can be proven to be duplicates, never lose content):
/// - The unprefixed line** is repeated** with the adjacent `+<same content>` line** (the model is written twice) → delete the useless line (the content is in the `+` line and will not be lost);
/// - Otherwise the unprefixed **non-empty** line** has exactly the same entire line in the target file** → it is the context line with missing spaces → fill in ` `
///   (Legal context and byte-exact; the least damaging explanation: row retention. If the original intention of the model is to delete, at most it will not be deleted and there will be no data loss);
/// - The rest (not in the file, non-repeating, blank lines) → pass it through as it is, submit an error to validate and let the model self-correct (no guessing).
///
/// Only applies to `*** Update File:` section (Add File leak `+` is managed by [`ensure_add_file_plus`]). Requires `cwd`.
pub(crate) fn fix_unprefixed_lines(v4a: &str, cwd: Option<&str>) -> (String, Vec<Repair>) {
    if !has_cwd_candidate(cwd) {
        return (v4a.to_owned(), Vec::new());
    }
    if !v4a.contains("*** Update File:") {
        return (v4a.to_owned(), Vec::new());
    }
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut in_update = false;
    let mut file_lines: Vec<String> = Vec::new();
    let mut drop_dups = 0usize;
    let mut add_ctx = 0usize;
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        if let Some(path) = l.strip_prefix("*** Update File: ") {
            in_update = true;
            // Parse target files by candidate cwd + anchor probe (MOC-263 P1/P2).
            let mut se = i + 1;
            while se < lines.len() && !lines[se].starts_with("*** ") {
                se += 1;
            }
            let probe = anchor_probe(&lines[i + 1..se]);
            file_lines = read_patch_file(path.trim(), cwd, &probe)
                .map(|(_, c)| c.lines().map(str::to_owned).collect())
                .unwrap_or_default();
            out.push(l.to_owned());
            i += 1;
            continue;
        }
        if l.starts_with("*** ") {
            in_update = false; // Any other control line ends Update body
            out.push(l.to_owned());
            i += 1;
            continue;
        }
        let first = l.chars().next();
        let valid = matches!(first, Some('+') | Some('-') | Some(' '))
            || l.starts_with("@@")
            || l.is_empty();
        if in_update && !valid {
            // case1: Duplicate waste lines adjacent to `+<same content>` → delete (the content is in the + line and will not be lost)
            let plus_dup = format!("+{l}");
            let next_dup = lines.get(i + 1).map(|n| *n == plus_dup).unwrap_or(false);
            let prev_dup = out.last().map(|o| o == &plus_dup).unwrap_or(false);
            if next_dup || prev_dup {
                drop_dups += 1;
                i += 1;
                continue;
            }
            // case2: There are exactly the same entire lines in the file → context leaks spaces → fills in ` `
            if file_lines.iter().any(|fl| fl == l) {
                out.push(format!(" {l}"));
                add_ctx += 1;
                i += 1;
                continue;
            }
            // else: through (no guessing)
        }
        out.push(l.to_owned());
        i += 1;
    }
    if drop_dups + add_ctx > 0 {
        repairs.push(Repair {
            file: "(unprefixed)".to_owned(),
            kind: "repaired".to_owned(),
            detail: format!(
                "Update unprefixed line repair: add context space {add_ctx} / delete duplicate and useless lines {drop_dups} (lark change_line)"
            ),
        });
    }
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}

/// Perform pre-flight repair on V4A patch. `cwd` is used to resolve the relative path of patch to a real file.
/// Return `(fixed V4A, processing record)`. No cwd / No `Update File` / V4A returns unchanged when the file cannot be read.
pub fn preflight_repair(v4a: &str, cwd: Option<&str>) -> (String, Vec<Repair>) {
    if !has_cwd_candidate(cwd) {
        return (v4a.to_owned(), Vec::new());
    }
    // There is no direct short-circuiting of Update File (Add/Delete File does not involve anchor matching).
    if !v4a.contains("*** Update File:") {
        return (v4a.to_owned(), Vec::new());
    }
    let mut repairs = Vec::new();
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(path) = line.strip_prefix("*** Update File: ") {
            out.push(line.to_owned());
            i += 1;
            // Collect the body of this Update File section (until the next `***` control action).
            let body_start = i;
            while i < lines.len() && !lines[i].starts_with("*** ") {
                i += 1;
            }
            let body = &lines[body_start..i];
            let (repaired_body, rep) = repair_update_section(path.trim(), body, cwd);
            out.extend(repaired_body);
            repairs.push(rep);
        } else {
            out.push(line.to_owned());
            i += 1;
        }
    }
    // Keep the trailing newline semantics: lines() discards the trailing newline, and adds it after join if the original text ends with \n.
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}
