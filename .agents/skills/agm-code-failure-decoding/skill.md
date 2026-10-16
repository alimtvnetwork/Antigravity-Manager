---
name: agm-code-failure-decoding
description: Mandatory before editing any code in Antigravity-Manager. Ten-minute codebase orientation, failure taxonomy from real CI and logic failures in this repo, the learn-from-failure loop, syntax-writing rules for Rust and JSX, and the local fmt, clippy, test, and build gate.
---

# AGM Code Failure Decoding & Fast Codebase Learning

The full skill lives in `.ai-memory/memory/learned/24-code-failure-decoding-and-fast-codebase-learning-skill.md`. Read it completely before your first edit. This file is only the checklist.

## Before editing

1. `git log -n 30 --oneline`, then `git show <sha> -- <file>` for recent changes to the file you will edit.
2. Read `.ai-memory/what-to-read.md`, `.ai-memory/cicd-index.md` "Recurring Failure Classes", and the matching sections of `.ai-memory/strictly-avoid.md`.
3. Grep the symbol, its callers, and its types. Read 30 lines around the edit point in large files (`instance.rs`, `repo_db.rs`, `auto_switcher.rs`, `agm.rs`).
4. Confirm which key you hold: instance id (`"default"`, uuid) vs IDE flavor (`target_ide`).

## While editing

- Anchor replacements on a unique line such as the `fn` signature. Never cut mid-block.
- Inside `mod tests`, keep exactly one `#[test]` plus the `fn name() {` line per test.
- Grep every use before deleting a `let`. Grep every `StructName {` before adding a field. Grep every caller before changing a signature or visibility.
- Read real return types (`PathBuf` vs `String`). Clone fields out of `Drop` types. Drop rusqlite statements before reusing the connection. Add `_ =>` for `#[non_exhaustive]` enums. Use `tauri::async_runtime` in Tauri commands.
- Balance JSX tags inside the edited block.

## Before pushing

`cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, targeted `cargo test --lib -- --test-threads=1 <name>`, and `npm run build` for frontend edits. Stage explicit paths in the case git tracks.

## After any failure

Exact error text, class, one-sentence root cause, fix the whole class, add a test, rerun the full gate, record the RCA and the ban.
