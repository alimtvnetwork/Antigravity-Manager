# Task-05: Audit stack traces

## Status
PENDING

## Objective
Every audit entry captures a stack trace: function name + file (+ line
where cheap) for the recording code path.

## Requirements
- Rust (`src-tauri/src/modules/audit/`): capture with
  `std::backtrace::Backtrace` (or the repo's tracing setup); filter frames
  to repo crates; store compactly (top N relevant frames, not the full
  dump). Must be cheap enough for per-interaction recording (consider
  capturing only on demand or sampling if profiling shows cost).
- Frontend (`src/components/audit/` + `useAuditRecorder.ts`):
  `new Error().stack` parsed to function/file; privacy-safe (no input
  values — consistent with the existing recorder).
- Audit panel displays the trace (collapsible per entry); export includes it.
- Schema migration: add the trace column/field without breaking existing
  entries (nullable/backfilled as "unavailable").

## Acceptance
- [ ] Entries carry func + file (+ line) traces.
- [ ] Panel shows them; export includes them.
- [ ] `cargo fmt` + `clippy` clean; `npm run build` passes.

## Constraints
- GitMap search only. Relative paths only. No silent errors.
