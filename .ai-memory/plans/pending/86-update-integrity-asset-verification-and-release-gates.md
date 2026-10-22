# Plan 86: Update Integrity: Verify Release Assets Before Announcing, Installer Pre-Check, Release Gates

Status: pending
Raised: 2026-09-30
Problem class (one PR scope): Update and release integrity
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Subtasks: [index](../subtasks/86-update-integrity-asset-verification-and-release-gates/index.md)
Related: [85](../completed/85-ui-tier-badge-priority-highlight-focus-and-stack-trace.md), [87](./87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler.md), [88](./88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md); CI/CD issue [39](../../cicd-issues/39-release-published-with-missing-artifacts-rca.md); spec [22-app-issues/20](../../../02-spec/22-app-issues/20-installer-upstream-fork-inversion-rca.md)

## Context

**Request.** The full verbatim user request is in the spec under "User Request (Verbatim)". This plan covers the tasks listed below; sibling plans cover the rest.

Makes the update checker, `agm update`, both installers, and the release workflow agree that an update exists only when an installable asset exists for this platform (T03). The user is told which versions were skipped and why, and another verified release is offered.

**Branch.** Work directly on `main` (resolved ambiguity 06). This plan changes the update path that stable users depend on, so the post-publish verification job (step 009) is the safety net.

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
| `001` | [001-update-info-contract.md](../subtasks/86-update-integrity-asset-verification-and-release-gates/001-update-info-contract.md) | Extend UpdateInfo with asset verification fields | `implemented - verification deferred` |
| `002` | [002-release-asset-resolver.md](../subtasks/86-update-integrity-asset-verification-and-release-gates/002-release-asset-resolver.md) | Resolve the newest release that has a platform asset | `implemented (core) - verification deferred` |
| `003` | [003-aggregate-source-errors-and-error-code.md](../subtasks/86-update-integrity-asset-verification-and-release-gates/003-aggregate-source-errors-and-error-code.md) | Aggregate update source failures and use a registered code | `partial - contract field only` |
| `004` | [004-update-ui-skipped-versions-notice.md](../subtasks/86-update-integrity-asset-verification-and-release-gates/004-update-ui-skipped-versions-notice.md) | Tell the user which versions were skipped and install the resolved one | `implemented except candidate-install action` |
| `005` | [005-install-ps1-precheck-and-ladder.md](../subtasks/86-update-integrity-asset-verification-and-release-gates/005-install-ps1-precheck-and-ladder.md) | install.ps1 verifies assets before selecting a version | `deferred to release phase (190-08)` |
| `006` | [006-install-sh-precheck-parity.md](../subtasks/86-update-integrity-asset-verification-and-release-gates/006-install-sh-precheck-parity.md) | install.sh gets the same pre-check contract | `deferred to release phase (190-08)` |
| `007` | [007-agm-update-uses-resolved-version.md](../subtasks/86-update-integrity-asset-verification-and-release-gates/007-agm-update-uses-resolved-version.md) | agm update and the delegate updater use the resolved version | `partial - checker and CLI resolver wired` |
| `008` | [008-release-workflow-artifact-gate.md](../subtasks/86-update-integrity-asset-verification-and-release-gates/008-release-workflow-artifact-gate.md) | release.yml publishes only with every expected artifact | `deferred to release phase (190-08)` |
| `009` | [009-post-publish-asset-url-verification.md](../subtasks/86-update-integrity-asset-verification-and-release-gates/009-post-publish-asset-url-verification.md) | Post-publish job re-fetches every URL in updater.json | `deferred to release phase (190-08)` |

## Order and Dependencies
001 then 002, then 003 and 004. 005 then 006 then 007. 008 then 009 are CI-only and independent of the Rust and installer steps.

## Acceptance
See the spec section "Acceptance Criteria"; this plan owns the items named in its summary.

## Attachments
Screenshots from the original request are not stored in the repo (an email is visible in at least one). See the spec "Attachments" section.

## Rollback
Each subtask is one commit that can be reverted independently. Configuration and database migrations introduced here are additive; the dedupe `--apply` path writes a timestamped DB backup first.

## Progress (2026-10-23, subtask 190-05)

- Subtask 001 implemented: `UpdateInfo` now carries `asset_verified`, `asset_url`, `candidates`, `skipped_versions`, and `source_errors` (all serde-defaulted for older payloads), with shared `SkippedVersion` and `SourceError` types.
- Subtask 002 core implemented: a single platform-asset resolver is now the source of truth for the updater.json path, the GitHub releases path, and the static fallback. Drafts, stable-channel prereleases, and releases without an installer asset for the current platform are skipped with a recorded reason; the newest installable release is selected and the remaining installable versions are reported as candidates. Fixture tests were added for the code paths, but were not executed in this implementation-only pass.
- Subtask 003 is only partial: the `source_errors` contract field exists, but the update sources currently populate it as empty; aggregation of real source failures and the registered error code remain open.
- Subtask 004 implemented with one explicit limitation: the update notification now shows a “No installable update” state when newer versions were skipped, lists each skipped version and reason (with a details view of candidates), and offers no Install action in that state; the same notice also appears beside a normal offer. The “install a specific candidate” action from the subtask is **not** implemented: the existing installer entry point takes no target-version argument, so candidates are listed but cannot be installed from the banner without a backend change. This was not faked.
- Path drift noted: `src/pages/Settings.tsx` no longer consumes the update store; the live update surfaces are the update notification and the navigation menu.
- Subtask 007 is partial: the GUI update checker and `agm update` both resolve through the releases list and report skipped versions (`[!] Skipped v<version>: <reason>` interactively and `skipped_versions` in the JSON payload). End-to-end delegate-updater verification remains for the E2E phase.
- Subtasks 005, 006, 008, and 009 are release-gate/installer items and were intentionally not built in this pass; they are deferred to subtask 190-08's release phase. This plan remains pending.
