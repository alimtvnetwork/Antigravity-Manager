//! Apply-patch preflight repair (split from apply_patch_preflight.rs).
//! Facade: implementation lives in submodules, each <= 500 lines.

//! apply_patch **pre-flight automatic repair**: Before sending the V4A patch to Codex apply, read the target file for comparison.
//! Automatically align **safe** context mismatch (trailing spaces / leading and trailing whitespace differences), eliminating the number one failure of V4A
//! `apply_patch verification failed: Failed to find expected lines`。
//!
//! ## Why is it needed?
//! Weaker chat models (non-OpenAI) are often unable to reproduce the context/delete lines of `Update File` byte-by-byte on large files.
//! (Trailing spaces, indentation, memory deviation) → Codex cannot find the anchor point → apply fails → the entire model file is rewritten, wasting time and tokens.
//! The error reported by the actual machine (rollout ground truth) is exactly this type.
//!
//! ## Safety boundary (never damage files - align users with the hard rule of "no destructive downgrades")
//! - **Move anchor point only**: context(space prefix)/ delete (`-`) line in `Update File`. `+New` lines **never change**.
//! - **Only fix when there is a unique match**: The anchor block is found in the file by "Ignore trailing spaces/leading and trailing blanks" to find candidates, and **exactly one** position will be aligned;
//!   0 (the model is really wrong) or ≥2 (ambiguous) will be **released as is** and handed over to Codex parse_patch to expose the real bad, never rely on guessing.
//! - **Add File / Delete File does not touch** (no anchor, no matching involved). Unable to read file / no cwd → release as is.
//! - Each repair/release is recorded in the apply-patch diagnostic page and can be audited.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde_json::Value;

/// A record of pre-flight processing (for diagnostic pages/logs).
#[derive(Debug, Clone, PartialEq)]

pub struct Repair {
    /// The file path in patch (relative, as is).
    pub file: String,
    /// `repaired` (aligned anchor points)/ `clean` (exact match, unchanged)/ `skipped: <reason>` (released without repair).
    pub kind: String,
    /// Human readable details (a few lines changed/why it was released).
    pub detail: String,
}

/// [MOC-194/MOC-263] Process-level "most recently seen cwd" candidate history (most-recent-first, deduplication, capacity limit).
///
/// **Why was it changed from single slot to candidate list (MOC-263 P1)**: Codex only sends `<cwd>`,apply_patch in turn-start request
/// The tool loops subsequent requests without cwd → relies on cross-request memory. The old implementation is **process-level single slot**, when multiple Codex sessions are concurrent (real machine
/// Normal state: open N conversations at the same time to change different projects) a single slot is continuously covered by the turn-start cwd of other conversations → apply_patch
/// The request is returned to the stale cwd of a specific project → Tier B disk reading rules are parsed to the wrong directory → the whole process is `skipped:unreadable`
/// (The actual measured phase-1:5/5 section is completely useless). Change to **Candidate list of the last N different cwds**: Try each candidate when reading the disk
/// Whether `cwd/relative path` exists, select the first one that exists (only real project cwd has this file, stale cwd does not → automatically select the right one).
/// Hitting the file with the same name as the wrong cwd will at worst cause subsequent anchor point matching to fail → safely skip and never change by mistake (keep "no guessing, no losing").
pub mod cwd;
pub mod envelope;
pub mod normalize;
pub mod pipeline;
pub mod read;
pub mod recover;
pub mod repair;
pub mod validate;

#[cfg(test)]
mod tests_cwd;
#[cfg(test)]
mod tests_repair;

pub use cwd::{extract_cwd, remember_cwd_from_request, remember_cwd_from_text};
pub use envelope::{ensure_v4a_envelope, preflight_repair};
pub use pipeline::optimize_patch;
pub use validate::validate_v4a_for_codex;

#[cfg(test)]
pub(crate) use cwd::*;
#[cfg(test)]
pub(crate) use envelope::*;
#[cfg(test)]
pub(crate) use normalize::*;
#[cfg(test)]
pub(crate) use read::*;
#[cfg(test)]
pub(crate) use recover::*;
#[cfg(test)]
pub(crate) use repair::*;
#[cfg(test)]
pub(crate) use validate::*;
