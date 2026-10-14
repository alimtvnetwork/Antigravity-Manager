# Task-01: RCA batch 1 — AGM incidents

## Status
PENDING

## Objective
Write detailed root-cause analyses for the three AGM incidents into
`.ai-memory/rca/` (create the directory; the lead owns `readme.md`).

## Incidents to document

### 01-agm-cargo-fmt-instance-rs.md
- Symptom: CI `Check Rust Code` failed on all 3 OS runners at step
  `Check Rust formatting` (exit 1) on commit `e04cc7b`.
- Root cause: `run_caught_prompt_task` in
  `src-tauri/src/commands/instance.rs:313` had a long single-line signature
  that `cargo fmt` wants wrapped.
- How found: `gitmap pe` → failed tree → `cargo fmt -- --check` locally showed
  the diff.
- Fix: ran `cargo fmt`; committed `src-tauri/src/commands/instance.rs`
  (4 insertions, 1 deletion); pushed as `9b968fb5`.
- Prevention: run `cargo fmt -- --check` before every push (repo AGENTS.md
  pre-flight).

### 02-agm-telegram-e0061.md
- Symptom: CI `Check Rust Code` → Clippy `error[E0061]: this function takes 5
  arguments but 4 arguments were supplied` on commit `d71ae907`.
- Root cause: `send_telegram_message_with_markup` grew a 5th param
  (`disable_preview: bool`); 3 call sites in
  `src-tauri/src/modules/telegram_inbound/daemon.rs` (lines 311, 459, 544)
  were missed — the author's search only caught the first.
- How found: `gitmap pe` → CI log → `gh run view --job ... --log-failed`
  listed the three sites.
- Fix: added `false,` at each site (behavior-preserving); verified every
  caller via repo-wide search; `cargo fmt` clean; commit `705bc024`.
- Prevention: after any signature change, search ALL call sites
  (`gitmap aum search`) and verify count before push; never rely on "CI will
  catch it".

### 03-agm-paquirk-branch-conflict.md
- Symptom: `gitmap pa` failed on `gitmap-v28` with merge conflicts.
- Root cause: local branch `fix/ci-gates-green` held a duplicate of the
  already-pushed remote fix (`83939b8ae`); `pa` tried to merge them.
- How found: `gitmap pull-error gitmap-v28` → conflict list; `git status`
  showed the stale branch.
- Fix: switched to `main`, `git pull --ff-only`; left the stale branch alone
  (never delete without asking).
- Prevention: check `git status`/branch before `pa`; don't keep stale
  work branches after the canonical fix is pushed.

## Format per RCA
Symptom (with CI log excerpt) → Root cause → How found (exact commands) →
Fix (file:line, old → new) → Verification evidence → Prevention checklist item.

## Constraints
- GitMap search only. Relative paths only. No secrets. Targeted commits only
  (lead commits; workers never run git).
