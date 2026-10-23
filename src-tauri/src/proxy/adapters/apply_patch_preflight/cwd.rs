// CWD tracking (split from apply_patch_preflight.rs).
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

use serde_json::Value;

/// A record of pre-flight processing (for diagnostic pages/logs).
const CWD_CANDIDATES_CAP: usize = 12;
static CWD_HISTORY: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn cwd_history() -> &'static Mutex<VecDeque<String>> {
    CWD_HISTORY.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// Remember a recently seen cwd (to reset the top, super [`CWD_CANDIDATES_CAP`] to eliminate the oldest). Empty strings are ignored.
pub(crate) fn remember_cwd(cwd: &str) {
    if cwd.is_empty() {
        return;
    }
    if let Ok(mut q) = cwd_history().lock() {
        if let Some(pos) = q.iter().position(|c| c == cwd) {
            q.remove(pos);
        }
        q.push_front(cwd.to_owned());
        while q.len() > CWD_CANDIDATES_CAP {
            q.pop_back();
        }
    }
}

/// The most recently seen cwd candidate (most-recent-first).
pub(crate) fn recall_cwd_candidates() -> Vec<String> {
    cwd_history()
        .lock()
        .map(|q| q.iter().cloned().collect())
        .unwrap_or_default()
}

/// Whether there are any available cwds (primary or historical candidates for the current request). The byte-exact rules are short-circuited accordingly.
pub(crate) fn has_cwd_candidate(primary: Option<&str>) -> bool {
    primary.map(|c| !c.is_empty()).unwrap_or(false) || !recall_cwd_candidates().is_empty()
}

/// "Anchor probe" of patch section, each item `(is_header, text)`:
/// - context(` `)/ Delete (`-`) lines to remove the prefix → `(false, line content)`, and compare the **entire line exact**(trim) in the candidate file;
/// - The header text of `@@ <header>` → `(true, header)`, compared by **substring** (the incomplete header is the substring of the real entire line, such as
///   `System architecture recommendations` ⊂ `## 6. System architecture recommendations`). The two categories are scored separately: if the exact header is hit by the entire line of the same name as stale, it will be mistakenly selected.
///   Therefore, the header does not enter exact(chatgpt-codex-connector review). Provides [`read_patch_file`] to select a target file among candidates with the same name.
pub(crate) fn anchor_probe<'a>(body: &[&'a str]) -> Vec<(bool, &'a str)> {
    let mut probe = Vec::new();
    for l in body {
        match l.chars().next() {
            Some(' ') | Some('-') => probe.push((false, &l[1..])),
            Some('+') => {} // Add a new line - not in the target file, do not probe
            _ => {
                if let Some(h) = l.strip_prefix("@@ ") {
                    let h = h.trim();
                    if !h.is_empty() {
                        probe.push((true, h));
                    }
                } else if !l.is_empty() && !l.starts_with("@@") && !l.starts_with("*** ") {
                    // Unprefixed lines (the model misses the prefix, fix_unprefixed_lines must be fixed based on exact matching of the entire line of the file) → do the entire line
                    // exact probe, so that the empty-probe path (when the missing prefix is the only anchor point) can also select the right file among candidates with the same name.
                    // (chatgpt-codex-connector review)。
                    probe.push((false, l));
                }
            }
        }
    }
    probe
}

/// Parse and read patch object files (MOC-263 P1 + P2) by candidate cwd. `primary`(current request cwd,apply_patch
/// Requests are usually None) given priority, and then tried one by one according to the recent cwd history. `probe` = patch context/remove anchor line content
/// ([`anchor_probe`]): When multiple candidate cwds have relative files with the same name (concurrent sessions share `README.md`/`package.json`
/// etc.), **select the candidate with the most probe anchor lines in the content** (= the file that patch actually targets), rather than the first readable one
/// (chatgpt-codex-connector review P2: Taking the first one will align the wrong files). All candidate probe hits are 0 → None
/// Candidate is target → return None(skip,safe). `probe` is empty (purely new patch without anchor point) → return to the first readable one
/// (No way to tell, best to try harder). Absolute paths are read directly.

/// Extract `<cwd>...</cwd>` (the environment_context block injected by Codex from the Codex Responses request,
/// Shaped like `<environment_context>\n <cwd>/abs/path</cwd>\n <shell>zsh</shell>...`).
///
/// **Traverse the Value tree** to find the string node containing `<cwd>` (its value is already the original text after serde escaping) and then extract - **cannot**
/// First `serde_json::to_string(whole request)` and then search: that will re-JSON escape the string value**, Windows path
/// The backslashes in `C:\Users\...` are doubled to `C:\\Users\\...`, resolve_path gets the wrong path (codex-connector #435 P2).
/// It does not depend on whether `<cwd>` falls in instructions or a certain input message (string nodes at any level are scanned).
pub fn extract_cwd(request: Option<&Value>) -> Option<String> {
    fn find_in_value(v: &Value) -> Option<String> {
        match v {
            Value::String(s) => extract_cwd_from_str(s),
            Value::Array(a) => a.iter().find_map(find_in_value),
            Value::Object(o) => o.values().find_map(find_in_value),
            _ => None,
        }
    }
    find_in_value(request?)
}

/// Extract `<cwd>...</cwd>` from a single (unescaped) string.
fn extract_cwd_from_str(s: &str) -> Option<String> {
    let start = s.find("<cwd>")? + "<cwd>".len();
    let rest = &s[start..];
    let end = rest.find("</cwd>")?;
    let cwd = rest[..end].trim();
    if cwd.is_empty() {
        None
    } else {
        Some(cwd.to_owned())
    }
}

/// [MOC-194 Critical] Store `<cwd>` in the request into the process-level cache. **Must be called on every request** (not just apply_patch):
/// With `<cwd>` is a **turn-start request** (does not generate apply_patch, does not adjust [`optimize_patch`]), and apply_patch
/// Appears in tool loop subsequent requests without cwd. Only memorize in `optimize_patch` → you will never learn cwd (actual test:
/// `LAST_CWD` is always None, all Tier B disk reading rules are no-op). Therefore, the memory point must be where every request passes
/// (Converter `with_original_request`), the cwd of turn-start can be rolled back to by subsequent apply_patch requests.
pub fn remember_cwd_from_request(request: Option<&Value>) {
    if let Some(cwd) = extract_cwd(request) {
        remember_cwd(&cwd);
    }
}

pub fn remember_cwd_from_text(text: &str) -> bool {
    let Some(cwd) = extract_cwd_from_str(text) else {
        return false;
    };
    remember_cwd(&cwd);
    true
}
