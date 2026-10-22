# Subtask 05 — AGM pending-plan closeout

Repo: Antigravity-Manager. Status: PENDING. Owner: Worker wave 2 (after restore fixes).

## Goal
`.ai-memory/plans/pending/` reflects reality: done work filed, real remainders implemented.

## Steps
1. Move the 7 DONE-IN-CODE plans (101, 106, 11, 12, 155, 156, boolean-and-complex-conditions-remediation) and 2 SUPERSEDED plans (09, 75) to `.ai-memory/plans/completed/` with a one-line closure note appended to each (evidence from research 01). Plan 75's closure note must record plan 87 as its successor.
2. Plan 91 (CI workflow deadlines): pin `ubuntu-24.04` in .github/workflows/ci.yml, release.yml, deploy-pages.yml, purge-actions-artifacts.yml (replace `ubuntu-latest`); apply the plan's remaining items (Node-24 action majors, release gate on green CI, cancel-in-progress, untrack temp JSON) where still applicable.
3. Plan 88: implement `agm supabase push-settings` (native CLI command) + email recipient dedupe, per the plan file.
4. Plan 139 remainder: expose `pid_refresh_seconds` in Settings UI + CLI (backend field exists in src-tauri/src/models/config.rs); record the agy `session_id` resume check result in the plan; the saved-PID ambiguity is a user decision — log it, do not guess.
5. Plan 86 (update integrity): implement skipped-version reporting in the update path (src-tauri/src/modules/delegate_updater/) per the plan; remaining release-gate items defer to subtask 08's release.
6. Plans 105 and 04: scope-refresh only — append a status note (105 = largest open UI wave, not started; 04 = installer script path drifted) and leave them pending with the note; do NOT start 105's UI wave in this task.
7. Plans 92-cli-instance-switch-e2e, 92-e2e-tests-plan, 93: execution is subtask 08's E2E phase; here, only verify the runbooks match current CLI commands and fix any drift in the documents.

## Verification
- Plans index (.ai-memory/plans/readme.md) updated by lead after completion.
- No builds/tests in this subtask (CI + subtask 08 verify).
