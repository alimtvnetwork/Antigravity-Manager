# Issue 50: Instance switch or fast-forward loses the running prompt

Status: the failure observed on 2026-10-02 is [issue 57](./57-switch-marks-prompt-restored-before-ide-ready-rca.md). The diagnosis below was the 2026-09-30 hypothesis. Do not implement it as the current cause.
Raised: 2026-09-30
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Plan: [87](../plans/pending/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler.md)

## 1. Symptom
After a switch/fast-forward on a cloned instance, the prompt that was running is not detected, not backed up, and not re-injected.

## 2. Trigger
A non-default instance with its own `data_dir`/`home_dir` and a prompt running in its IDE.

## 3. Root cause
Prompt discovery and project detection in `repo_db.rs` read the default/global state instead of the instance's own directories. PID to instance mapping exists (`instance.rs::find_pids_for_data_dir`) but is not the single scope source. Backup errors are swallowed, resend may run up to three times from different callers, and nothing waits for IDE readiness before injecting.

## 4. Why it escaped
E2E ran against the default instance and with seeded prompts, never with a real prompt in a second real instance.

## 5. Fix (planned)
One `InstanceScope` resolver (PIDs, data/home dirs, both AppData layouts); per-instance discovery and backup with typed results; close only that instance; wait for readiness; inject once; per-instance verification; E2E with two real test instances and teardown. See plan 87.

## 6. Prevention
No global fallback for non-default instances; backup result type cannot be ignored; E2E in the release checklist for this area.

## 7. Regression check
`scripts/test-instance-e2e.ps1` with two instances, a live prompt, two consecutive switches.
