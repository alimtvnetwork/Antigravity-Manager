# Subtasks for plan 140 - Per-Instance Prompt Attribution

Parent plan: `.ai-memory/plans/140-per-instance-prompt-attribution-backup-queue.md`
Spec: `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/`

| # | File | Scope | Status |
|---|---|---|---|
| 01 | [01-canonical-instance-key-and-per-instance-resume-file.md](./01-canonical-instance-key-and-per-instance-resume-file.md) | Canonical instance key, identity triple, schema migration, per-instance resume hand-off | pending |
| 02 | [02-backup-queue-restore-pipeline-fixes.md](./02-backup-queue-restore-pipeline-fixes.md) | Status state machine, one restore path, running detection, switch flows | pending |
| 03 | [03-cli-and-ipc-parity.md](./03-cli-and-ipc-parity.md) | CLI `-i` and `--json` everywhere, NEW enqueue/queue/trace commands, NEW IPC wrappers, UI instance badge | pending |
| 04 | [04-e2e-multi-instance-test-catalog.md](./04-e2e-multi-instance-test-catalog.md) | E2E-00..E2E-19 catalog and coverage of AC-01..AC-32 | pending |
| 05 | [05-e2e-runbook-and-evidence-capture.md](./05-e2e-runbook-and-evidence-capture.md) | Step-by-step PowerShell runbook, evidence capture, stop rules | pending |

| 06 | [06-blind-executor/00-start-here.md](./06-blind-executor/00-start-here.md) | Steps 01 to 22 cut from subtasks 01 to 03 for an executor with no repo context: settled decisions, verbatim current code, replacement code, tests, one gate and one commit per step | pending |

Order: 05 (E2E-00, E2E-01 only), then 01, 02, 03, then 04 and 05 in full.

Executor without repo context (any model): follow `06-blind-executor/00-start-here.md` and its steps 01 to 22 instead of subtasks 01 to 03. Its decisions DR-1 to DR-7 override the open questions in the spec.
