// Patch file reading (split from apply_patch_preflight.rs).
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

use serde_json::Value;

/// A record of pre-flight processing (for diagnostic pages/logs).

pub(crate) fn read_patch_file(
    relpath: &str,
    primary: Option<&str>,
    probe: &[(bool, &str)],
) -> Option<(PathBuf, String)> {
    let p = Path::new(relpath);
    if p.is_absolute() {
        return std::fs::read_to_string(p)
            .ok()
            .map(|c| (p.to_path_buf(), c));
    }
    // ① fresh primary authority: the current request comes with cwd and the file is readable → use it directly and hand it over to the downstream to determine matching (including align_at_headers
    //    partial `@@` substring fix). **probe only does tie-breaker between multiple candidates with the same name, never gate** - otherwise
    //    Incomplete `@@` header / single candidate will be misjudged as unreadable due to probe 0 hit (chatgpt-codex-connector review P2 second round).
    if let Some(c) = primary {
        if !c.is_empty() {
            let abs = Path::new(c).join(p);
            if let Ok(content) = std::fs::read_to_string(&abs) {
                return Some((abs, content));
            }
        }
    }
    // ② Otherwise, use the most recent cwd candidate history (most-recent-first) to read out all existing files with the same name.
    let mut readable: Vec<(PathBuf, String)> = Vec::new();
    for c in recall_cwd_candidates() {
        let abs = Path::new(&c).join(p);
        if let Ok(content) = std::fs::read_to_string(&abs) {
            readable.push((abs, content));
        }
    }
    match readable.len() {
        0 => return None,
        // Single candidate → used directly (downstream determines matching, only partial header substring repair has a chance); will not be skipped due to probe 0 hit.
        1 => return readable.into_iter().next(),
        _ => {}
    }
    // ③ Multiple candidates with the same name (concurrent sessions share README.md/package.json, etc.) → Press the anchor point probe to select the file that the patch actually targets.
    //    Scoring: context/delete line (non-header) is hit by **whole line exact**(trim); `@@` header (header) is hit by **substring**
    //    The real entire line (the incomplete header is the entire line substring, such as `System Architecture Suggestion` ⊂ `## 6. System Architecture Suggestion`). Two categories of combined scoring, ** unique
    //    Only the highest score** will be selected; tie / all 0 → None (ambiguity will not be guessed, violation of "no guessing, no loss" will align the stale files).
    //    The header does not enter exact: otherwise the entire line of stale with the same name (exactly = incomplete header) will use exact to beat the substring of real (review).
    let probe: Vec<(bool, &str)> = probe
        .iter()
        .map(|&(h, t)| (h, t.trim()))
        .filter(|(_, t)| !t.is_empty())
        .collect();
    if probe.is_empty() {
        return readable.into_iter().next(); // No anchor point (purely new patch) → most recent (no alignment required, downstream no-op)
    }
    let scores: Vec<usize> = readable
        .iter()
        .map(|(_, c)| {
            let fl: Vec<&str> = c.lines().map(str::trim).collect();
            probe
                .iter()
                .filter(|&&(is_header, t)| {
                    if is_header {
                        fl.iter().any(|line| !line.is_empty() && line.contains(t))
                    } else {
                        fl.iter().any(|line| *line == t)
                    }
                })
                .count()
        })
        .collect();
    let max = scores.iter().copied().max().unwrap_or(0);
    if max == 0 {
        return None; // No candidates contain any anchor points → none are targets → skip (safe)
    }
    if scores.iter().filter(|s| **s == max).count() != 1 {
        return None; // Tied for the highest = Ambiguity → Not guessing
    }
    let best_idx = scores.iter().position(|s| *s == max).unwrap();
    Some(readable.swap_remove(best_idx))
}
