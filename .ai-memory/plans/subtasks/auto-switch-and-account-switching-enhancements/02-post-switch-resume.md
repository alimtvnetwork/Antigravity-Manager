# Task-02: Post-switch resume (10s timer)

## Status
PENDING (needs Task-01)

## Objective
~10 seconds after an auto-switch or account switch, resume the last running
prompt (re-dispatch directly or enqueue + send immediately).

## Requirements
- Hook into the single-point switch orchestration
  (`instance::switch_account_to_instance` per RCA-15 — verify via Task-01).
- Configurable delay (default 10s), cancellable (e.g. user sends something
  else first), and visible: UI indicator + audit entry.
- Resume path MUST honor RCA-15 remediation: status transitions
  (`running` → `backed_up` → `dispatched`), no duplicate reinjection,
  cache bypass for restored prompts.
- FIFO invariant untouched. No silent errors. CLI/GUI parity where the
  timer is concerned.

## Acceptance
- [ ] Switch → 10s → last running prompt resumes on the new account.
- [ ] No duplicate dispatches; no stuck prompts.
- [ ] `cargo fmt` + `clippy` clean.

## Constraints
- GitMap search only. Relative paths only.
