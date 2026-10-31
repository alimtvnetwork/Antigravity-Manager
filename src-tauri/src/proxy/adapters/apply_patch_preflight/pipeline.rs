// Patch optimization pipeline (split from apply_patch_preflight.rs).
use super::cwd::remember_cwd;
use super::envelope::{ensure_v4a_envelope, fix_unprefixed_lines, preflight_repair};
use super::normalize::{
    convert_unified_file_headers, diagnose_absolute_paths, strip_trailing_at,
    strip_unified_hunk_ranges,
};
use super::recover::{recover_empty_move, recover_update_empty_file};
use super::repair::repair_update_section;
use super::validate::{align_at_headers, ensure_add_file_plus};
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

/// apply_patch **General entrance to the middle layer**: **restore known format errors one by one according to the whitelist rules** so that the model does not follow the prompt
/// The generated malformed patch can still be applied correctly by Codex. **Only the known pits that are certain will be touched; unknown pits will be released as they are (no guessing, no throwing away). **
///
/// Two-layer structure (aligned [[MOC-194]] scheme):
///
/// **Tier A syntax adjustment** (mirroring Codex's lark syntax for GPT, pure string, no disk reading - GPT is generated based on syntax constraints
/// Guaranteed legality, guaranteed after the third party chat path):
/// - [`strip_trailing_at`] — bilateral `@@ … @@` → unilateral (grammar `change_context: "@@" | "@@ " /(.+)/`; measured 18×).
/// - [`convert_unified_file_headers`] — `--- old` / `+++ new` or `file: path` → `*** Update File: path`.
/// - [`ensure_add_file_plus`] — Add File content line missing `+` → completion (grammar `add_line: "+" /(.*)/`, Add File unambiguous).
/// - [`ensure_v4a_envelope`] — missing `*** Begin/End Patch` → completion (grammar `start: begin_patch hunk+ end_patch`;
///   gotcha #6 + real machine seq230). **Only done when `json_complete` (non-streaming truncation)**, and **put last** to wrap Tier B products.
///
/// **Tier B semantic recovery** (file status/content layer that grammar cannot control, requires `cwd` to read the disk):
/// - [`recover_update_empty_file`] — Update empty file → Delete+Add (actual measurement: 50×, lossless).
/// - [`align_at_headers`] — `@@ <header>` Failed anchor → Align the file to the entire line (`Failed to find context`).
/// - [`fix_unprefixed_lines`] — There are no prefixed lines in Update → Fill context spaces according to file judgment / delete duplicate and useless lines (seq235).
/// - [`recover_empty_move`] — empty Update+Move(rename-only)→ Delete+Add to copy the original content (actual measurement: 76×).
/// - [`preflight_repair`] — Update context byte-exact mismatch → read disk alignment (measured 134×).
///
/// Uncovered error points: **pass it as it is** and submit it to the Codex applier to report an error (no guessing, no loss).
/// `json_complete`: The caller passes `detect_json_truncation(args).is_none()`(chat); gemini args passes `true` completely in one go.
pub fn optimize_patch(v4a: &str, cwd: Option<&str>, json_complete: bool) -> (String, Vec<Repair>) {
    // [MOC-194/MOC-263] **Two types of cwd, used separately**:
    // - `fresh_cwd` = `<cwd>` that comes with the current request (usually None for apply_patch requests). **Judgment File == Codex Application
    //   The document is safe and trustworthy.
    // - Candidate history = the last N distinct cwd([`recall_cwd_candidates`]) remembered across requests. Codex only on turn-start
    //   The request is sent with `<cwd>`, and the apply_patch tool loops subsequent requests without → and relies on it to roll back. MOC-263: Change candidate list from single slot,
    //   Concurrent multi-session is no longer covered by stale cwd of other projects (when reading the disk, try one by one according to the candidates and select the first one that exists).
    //
    // **Status rewriting rules** (`recover_update_empty_file` / `recover_empty_move`: Convert Update to Delete+Add)
    // The judgment file and application file (Codex uses patch relative path in real cwd application) **may not be the same** → Wrong cwd will happen
    // Deleting the file with the same name of the wrong project (destructive). Therefore, these two ** only use fresh_cwd** (it is safe to determine == application), and ** do not check the candidate history**;
    // apply_patch request no fresh cwd → automatically skip through (safe).
    // **byte-exact alignment rule** (align/preflight/fix_unprefixed) passes `fresh_cwd` as primary, internal manager
    // [`read_patch_file`] Check the candidate history again: the worst case of hitting the wrong file is "non-unique match/byte mismatch" → safe no-op.
    let fresh_cwd = cwd;
    // If the current request contains cwd, it will be recorded in the candidate history (the cwd of turn-start is mainly used by the converter `remember_cwd_from_request`
    // Accounted on every request; here's the catch: in case apply_patch request comes with cwd it is also accounted for).
    if let Some(c) = cwd {
        remember_cwd(c);
    }
    let mut repairs = Vec::new();
    let mut s = v4a.to_owned();
    repairs.extend(diagnose_absolute_paths(&s, fresh_cwd));

    // ── Tier A syntax correction (pure string)──
    let (s1, r1) = strip_trailing_at(&s);
    s = s1;
    repairs.extend(r1);

    let (s_d, r_d) = convert_unified_file_headers(&s);
    s = s_d;
    repairs.extend(r_d);

    let (s_r, r_r) = strip_unified_hunk_ranges(&s);
    s = s_r;
    repairs.extend(r_r);

    let (s_g, r_g) = ensure_add_file_plus(&s);
    s = s_g;
    repairs.extend(r_g);

    // ── Tier B semantic recovery ──
    // Note: `Add File already exists → Delete+Add overwrite` rule **revoked** (2026-06-09). It will overwrite existing files,
    // It is possible to lose existing content that is not in the Add content (destructive downgrade); and it will rob the model after receiving `already exists`
    // Self-correction is an opportunity for **Targeted Update** (lossless). Instead, pass it as is and submit the Codex report `already exists` to let the model self-correct.
    //
    // Status rewrite rules → **fresh_cwd** (to prevent stale from deleting the wrong file, see above).
    let (s_f, r_f) = recover_update_empty_file(&s, fresh_cwd);
    s = s_f;
    repairs.extend(r_f);

    let (s3, r3) = recover_empty_move(&s, fresh_cwd);
    s = s3;
    repairs.extend(r3);

    // byte-exact alignment rule → pass fresh_cwd as primary, internally read_patch_file and then check the candidate history (worst-case security no-op).
    let (s_h, r_h) = align_at_headers(&s, fresh_cwd);
    s = s_h;
    repairs.extend(r_h);

    let (s_u, r_u) = fix_unprefixed_lines(&s, fresh_cwd);
    s = s_u;
    repairs.extend(r_u);

    let (s2, r2) = preflight_repair(&s, fresh_cwd);
    s = s2;
    repairs.extend(r2);

    // ── Put the envelope completion at the end: wrap up Tier B’s possible new Delete+Add and other structures ──
    if json_complete {
        let (s4, r4) = ensure_v4a_envelope(&s);
        s = s4;
        if let Some(r) = r4 {
            repairs.push(r);
        }
    }
    (s, repairs)
}
