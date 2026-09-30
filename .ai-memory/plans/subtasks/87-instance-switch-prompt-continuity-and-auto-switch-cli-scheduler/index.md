# Subtasks — Plan 87 instance switch, prompt continuity, auto-switch CLI and scheduler

Parent plan: [87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler.md](../../pending/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler.md)

Order follows data flow: contracts and scaffolds first, callers after, UI and scripts last. Execute **one step per run**; mark the row `done` in the parent plan's status table, then self-loop.

| # | File | Title | Status |
| :--- | :--- | :--- | :--- |
| `001` | [001-instance-scope-resolver.md](./001-instance-scope-resolver.md) | One InstanceScope resolver (PIDs, data dir, home dir, storage layouts) | `pending` |
| `002` | [002-per-instance-prompt-discovery.md](./002-per-instance-prompt-discovery.md) | Discover running prompts inside the instance's own directories | `pending` |
| `003` | [003-backup-result-typed-and-cli.md](./003-backup-result-typed-and-cli.md) | Typed backup result and `agm brp --instance` | `pending` |
| `004` | [004-close-only-target-instance.md](./004-close-only-target-instance.md) | Close only the target instance's processes after a verified backup | `pending` |
| `005` | [005-readiness-wait-and-single-injection.md](./005-readiness-wait-and-single-injection.md) | Wait for IDE readiness, inject once | `pending` |
| `006` | [006-per-instance-verification.md](./006-per-instance-verification.md) | Verify per instance and expose in CLI JSON | `pending` |
| `007` | [007-switch-actor-cli-with-lock.md](./007-switch-actor-cli-with-lock.md) | `agm switch-if-low-credit` as the single instance-aware actor | `pending` |
| `008` | [008-auto-tick-checker-and-schedule.md](./008-auto-tick-checker-and-schedule.md) | `agm auto tick` and `agm auto schedule` | `pending` |
| `009` | [009-strict-candidate-and-unified-triggers.md](./009-strict-candidate-and-unified-triggers.md) | Strict 100% candidate, one trigger path, consistent model | `pending` |
| `010` | [010-e2e-real-instances-live-prompt.md](./010-e2e-real-instances-live-prompt.md) | Local E2E: two real instances, live prompt, two switches, teardown | `pending` |

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
