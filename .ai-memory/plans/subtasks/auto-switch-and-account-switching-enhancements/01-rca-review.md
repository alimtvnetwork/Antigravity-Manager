# Task-01: RCA review (research, read-only)

## Status
PENDING

## Objective
Read the auto-switch/account-switch/fast-forward RCAs and map the current
code state, so Tasks 02–05 don't regress fixed bugs.

## Read
- `02-spec/22-app-issues/15-auto-switch-prompt-resumption-stuck-and-reinjection-rca.md`
  (4-part RCA + remediation — the primary reference).
- `02-spec/22-app-issues/12-auto-switcher-quota-and-instance-rotation-rca.md`.
- `02-spec/22-app-issues/36-auto-switch-fast-forward-downgrade-regression-rca.md`.
- Current: `src-tauri/src/modules/auto_switcher.rs`,
  `src-tauri/src/modules/instance.rs` (switch orchestration),
  `src-tauri/src/modules/repo_db.rs` (backup/resend),
  `src-tauri/src/modules/backup_prompts_db.rs`.

## Output
Report via send_message: for each RCA remediation point, whether the
current code still honors it (file:line evidence), plus the exact
functions/files Tasks 02–05 must touch. Do not write files. Do not run git.

## Constraints
- Read-only. GitMap search only. Relative paths only.
