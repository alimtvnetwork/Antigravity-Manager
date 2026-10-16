# Spec: Auto Switch and Account Switching Enhancements

## slug
auto-switch-and-account-switching-enhancements

## Status
ACTIVE — plan enqueued at `.ai-memory/plans/auto-switch-and-account-switching-enhancements.md`

## 1. Objective

Make auto-switch and account switching reliable and high-priority:
post-switch prompt resumption, durable prompt backups re-sendable via CLI/UI,
a rerun section in the prompts tree, audit stack traces, and a minor release
with CI green for morning testing.

## 2. Background — read before coding
- `02-spec/22-app-issues/15-auto-switch-prompt-resumption-stuck-and-reinjection-rca.md`
  — the 4-part RCA (in-flight status misalignment, stale backup ingestion,
  dispatch cache lockout, dual backup/restore cycles) and its remediation.
  Don't regress it.
- `02-spec/22-app-issues/12-auto-switcher-quota-and-instance-rotation-rca.md`
  — quota evaluation, candidate fallback, switch telemetry.
- `02-spec/22-app-issues/36-auto-switch-fast-forward-downgrade-regression-rca.md`
  — fast-forward regressions.
- `.ai-memory/what-to-read.md` (E2E conventions, prompt-tree send rules).
- AGENTS.md: FIFO invariant (`ORDER BY created_at ASC, id ASC`), No Silent
  Errors, Headless & CLI parity, release workflow.

## 3. Requirements

### 3.1 Post-switch resume (10s)
- After an auto-switch or account switch completes, ~10 seconds later the
  system must resume the last running prompt: either re-dispatch it directly
  to the new account/instance, or enqueue it and send it immediately.
- Must respect the RCA-15 remediation: status transitions
  (`running` → `backed_up` → `dispatched`), no duplicate reinjection, cache
  bypass for restored prompts, single-point orchestration in
  `switch_account_to_instance`.
- The delay must be configurable (default 10s), cancellable, and visible
  (UI indicator + audit entry).

### 3.2 Prompt backup for later re-send
- Running and enqueued prompts are backed up durably so that at any later
  time the user can re-send all enqueued prompts via CLI (`agm` command)
  and via the UI (button in the prompts tree / queue view).
- Backup must include prompt text, destination (instance/conversation),
  images/attachments references, and enqueue order (FIFO preserved).

### 3.3 Rerun section in prompts tree
- In the send section of the prompts tree, add a rerun section:
  - Rerun count selector (how many times).
  - Message options: "is it done?", "is it published?", or randomized
    messages (from a small built-in set, clearly labeled as randomized).
  - Dispatches the chosen message N times (user said "two, three times").
- Rerun uses the existing dispatch path (no forked logic).

### 3.4 Audit stack traces
- Every audit entry (from the `src-tauri/src/modules/audit/` recorder)
  captures a stack trace: function name + file (+ line where cheap) for the
  code path that recorded the interaction.
- Rust: use `std::backtrace::Backtrace` (or the repo's existing tracing
  setup) — capture frames, filter to repo crates, store compactly.
- Frontend: `new Error().stack` parsed to function/file — privacy-safe
  (no input values, consistent with existing recorder).

### 3.5 Release
- Minor bump via repo flow (`npm run bump minor`, changelog with
  `(Thanks to @aukgit)` only, README sync), tag, push tag,
  `gh release create -R alimtvnetwork/Antigravity-Manager`.
- CI watched until green; fix failures (minor bumps authorized).
- E2E validation documented (headless limits noted).

## 4. Out of scope
- `.github/workflows/` changes (standing rule).
- Deletions (standing rule).
- Progress bar (standing off-limits).

## 5. Constraints (non-negotiable)
- GitMap-only search. Relative paths only. Lowercase filenames.
- Targeted `git commit <exact paths>` + `git push` by the lead only.
  NEVER `gitmap cpf`/`cpb`.
- No silent errors (AGENTS.md). FIFO invariant untouched.
- `cargo fmt` clean; clippy clean; `npm run build` for frontend changes.
- Don't regress RCA-15 remediation or RCAs 14–16.
