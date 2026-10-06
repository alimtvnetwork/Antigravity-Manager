# Plan 87: Instance Switch Prompt Continuity and Auto-Switch CLI with Scheduler

Status: pending
Raised: 2026-09-30
Problem class (one PR scope): Multi-instance switching and auto-switch
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Subtasks: [index](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/index.md)
Related: [85](../completed/85-ui-tier-badge-priority-highlight-focus-and-stack-trace.md), [86](./86-update-integrity-asset-verification-and-release-gates.md), [88](./88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md); pending plan [75](./75-multi-instance-switching-prompt-backup-and-hygiene.md) overlaps on prompt backup and E2E

## Context

**Request.** The full verbatim user request is in the spec under "User Request (Verbatim)". This plan covers the tasks listed below; sibling plans cover the rest.

Fixes the most pressing defect (T07): a running prompt must survive a switch or fast-forward on any instance. The pipeline is per instance: back up to SQLite through the CLI, close only that instance (PID to `--user-data-dir`), switch credentials, relaunch, wait until ready, inject once, verify. Also fixes auto-switch (T10): strict 100% candidate, one CLI actor, two triggers (reactive and a 1-minute scheduled checker).

**Overlap with plan 75.** Plan 75 (pending) covers multi-instance isolation, prompt backup, hygiene script, E2E, and a minor release ceremony. Plan 87 supersedes its prompt-backup, switch, and E2E parts. Before starting 87, decide with the user whether to close plan 75 as superseded or to trim it to the dev-tool-clear hygiene item and its own release step. No file in plan 75 is changed by this planning step.

**Mandatory reading.** Before any step, read `.ai-memory/coding-guidelines.md`, `.ai-memory/strictly-avoid.md`, and `.ai-memory/folder-structure.md`. Project rules in `AGENTS.md` apply: pipeline-first fixes, headless/CLI parity, cross-platform, root-cause fixes, no release except on explicit command, PR = one problem class with individually revertable commits.

**Release policy (verbatim, law).**

Individual task runs NEVER release. No version bump, no changelog entry, no
release-notes update, no root `readme.md` version pin on a per-task basis. A run
that touches the version while sibling tasks are pending is auto-reject.

The release fires ONLY when the ENTIRE plan is finished, meaning every task and
subtask of this plan has moved out of `.ai-memory/plans/pending/` into
`.ai-memory/plans/completed/` with `Status: completed`, AND the user explicitly commands the release. At that moment, and only
then:

- Bump the MINOR version per the release ceremony in `01-prompts/17-release-management/04-release.md`.
- Add one changelog entry covering the whole plan, never a single task.
- Update release notes.
- Pin the new version in the root `readme.md`.

**Ambiguities (all resolved 2026-09-30).**
- [03-native-ide-prompt-detection-scope-and-acceptance-signal.md](../../ambiguous-questions/02-ambiguity-resolved/03-native-ide-prompt-detection-scope-and-acceptance-signal.md)
- [06-beta-staging-before-main.md](../../ambiguous-questions/02-ambiguity-resolved/06-beta-staging-before-main.md)

## Execution Model (RULE 0C)

- One step per run. Exactly one step is executed per run. Never batch two steps into a single run.
- Every step is standalone: read the step file plus the files it cites; nothing else is assumed.
- Self-loop after verify: when a step's `## 6. Verify` passes and `## 7. Done When` is satisfied, mark the step `done` in the status table below, then re-read this file, pick the next unstarted step, and begin a fresh run.
- Concurrency ceiling: at most 2 spawned agents, each with at most 3 parallel threads.
- Pre-flight for the plan (once, before the PR, not per step): `cd src-tauri && cargo fmt -- --check`, `cd src-tauri && cargo clippy --all-targets --all-features`, `npm run build` when `src/` changed. CI compiles tests without running them.

## Status Table

| # | Step | Title | Status |
| :--- | :--- | :--- | :--- |
| `001` | [001-instance-scope-resolver.md](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/001-instance-scope-resolver.md) | One InstanceScope resolver (PIDs, data dir, home dir, storage layouts) | `pending` |
| `002` | [002-per-instance-prompt-discovery.md](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/002-per-instance-prompt-discovery.md) | Discover running prompts inside the instance's own directories | `pending` |
| `003` | [003-backup-result-typed-and-cli.md](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/003-backup-result-typed-and-cli.md) | Typed backup result and `agm brp --instance` | `pending` |
| `004` | [004-close-only-target-instance.md](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/004-close-only-target-instance.md) | Close only the target instance's processes after a verified backup | `pending` |
| `005` | [005-readiness-wait-and-single-injection.md](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/005-readiness-wait-and-single-injection.md) | Wait for IDE readiness, inject once | `pending` |
| `006` | [006-per-instance-verification.md](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/006-per-instance-verification.md) | Verify per instance and expose in CLI JSON | `pending` |
| `007` | [007-switch-actor-cli-with-lock.md](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/007-switch-actor-cli-with-lock.md) | `agm switch-if-low-credit` as the single instance-aware actor | `pending` |
| `008` | [008-auto-tick-checker-and-schedule.md](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/008-auto-tick-checker-and-schedule.md) | `agm auto tick` and `agm auto schedule` | `pending` |
| `009` | [009-strict-candidate-and-unified-triggers.md](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/009-strict-candidate-and-unified-triggers.md) | Strict 100% candidate, one trigger path, consistent model | `pending` |
| `010` | [010-e2e-real-instances-live-prompt.md](../subtasks/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler/010-e2e-real-instances-live-prompt.md) | Local E2E: two real instances, live prompt, two switches, teardown | `pending` |

## Order and Dependencies
Strict chain 001 -> 006 (each builds on the previous). 007 then 008 then 009. 010 is last and is the proof for the whole plan.

## Acceptance
See the spec section "Acceptance Criteria"; this plan owns the items named in its summary.

## Attachments
Screenshots from the original request are not stored in the repo (an email is visible in at least one). See the spec "Attachments" section.

## Rollback
Each subtask is one commit that can be reverted independently. Configuration and database migrations introduced here are additive; the dedupe `--apply` path writes a timestamped DB backup first.
