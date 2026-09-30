# Plan 88: Email Recipient Dedupe, Supabase push-settings, and Hardened Vault Scripts

Status: pending
Raised: 2026-09-30
Problem class (one PR scope): Data hygiene (email) and Supabase/vault tooling
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Subtasks: [index](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/index.md)
Related: [85](./85-ui-tier-badge-priority-highlight-focus-and-stack-trace.md), [86](./86-update-integrity-asset-verification-and-release-gates.md), [87](./87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler.md); completed plans [71](../completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md), [74](../completed/74-ui-email-telegram-fixes-revisit.md)

## Context

**Request.** The full verbatim user request is in the spec under "User Request (Verbatim)". This plan covers the tasks listed below; sibling plans cover the rest.

Stops notification emails from being re-added (T09), adds native `agm supabase push-settings` (T08), removes committed default keys, and hardens the vault scripts (`connect-supabase.ps1`, `push-settings.ps1`) with a shared guard library and a verified one-liner (T11).

**Two repositories.** Steps 007-010 edit the sibling `../repo-secrets/` repository (separate git history, commit there separately). Steps 001-006 edit this repository. Secret values are never printed, logged, or committed.

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

**Open ambiguities.**
- [02-vault-folder-and-machine-alias-for-supabase.md](../../ambiguous-questions/01-new-ambiguity/02-vault-folder-and-machine-alias-for-supabase.md)
- [06-beta-staging-before-main.md](../../ambiguous-questions/01-new-ambiguity/06-beta-staging-before-main.md)

## Execution Model (RULE 0C)

- One step per run. Exactly one step is executed per run. Never batch two steps into a single run.
- Every step is standalone: read the step file plus the files it cites; nothing else is assumed.
- Self-loop after verify: when a step's `## 6. Verify` passes and `## 7. Done When` is satisfied, mark the step `done` in the status table below, then re-read this file, pick the next unstarted step, and begin a fresh run.
- Concurrency ceiling: at most 2 spawned agents, each with at most 3 parallel threads.
- Pre-flight for the plan (once, before the PR, not per step): `cd src-tauri && cargo fmt -- --check`, `cd src-tauri && cargo clippy --all-targets --all-features`, `npm run build` when `src/` changed. CI compiles tests without running them.

## Status Table

| # | Step | Title | Status |
| :--- | :--- | :--- | :--- |
| `001` | [001-recipient-normalized-upsert-and-unique-index.md](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/001-recipient-normalized-upsert-and-unique-index.md) | Normalized upsert-or-skip for notification recipients | `pending` |
| `002` | [002-route-all-email-paths-through-helper.md](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/002-route-all-email-paths-through-helper.md) | Every add/import path uses the helper and reports skipped counts | `pending` |
| `003` | [003-existing-duplicates-report-and-merge.md](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/003-existing-duplicates-report-and-merge.md) | `agm email dedupe` for rows that already exist | `pending` |
| `004` | [004-supabase-load-json-merge-node-alias-and-flags.md](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/004-supabase-load-json-merge-node-alias-and-flags.md) | `load-json` applies `node_alias` and sync flags | `pending` |
| `005` | [005-agm-supabase-push-settings.md](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/005-agm-supabase-push-settings.md) | `agm supabase push-settings` | `pending` |
| `006` | [006-remove-committed-default-keys-from-repo-scripts.md](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/006-remove-committed-default-keys-from-repo-scripts.md) | Remove endpoint keys from `scripts/setup-supabase.ps1` and `scripts/supabase-setup.ps1` | `pending` |
| `007` | [007-connect-supabase-ps1-hardening.md](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/007-connect-supabase-ps1-hardening.md) | Harden `connect-supabase.ps1` in the vault repo | `pending` |
| `008` | [008-push-settings-ps1-fixes.md](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/008-push-settings-ps1-fixes.md) | Fix `push-settings.ps1` remote transfer and verification | `pending` |
| `009` | [009-shared-script-guard-helpers.md](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/009-shared-script-guard-helpers.md) | Shared guard helpers and vault self-check | `pending` |
| `010` | [010-one-liner-and-live-verification.md](../subtasks/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts/010-one-liner-and-live-verification.md) | Verified one-liner for Supabase on this machine | `pending` |

## Order and Dependencies
001 -> 002 -> 003 (email). 004 -> 005 (CLI). 006 after 005. 007 -> 008 -> 009 -> 010 (scripts, in the vault repo); 010 is the live proof.

## Acceptance
See the spec section "Acceptance Criteria"; this plan owns the items named in its summary.

## Attachments
Screenshots from the original request are not stored in the repo (an email is visible in at least one). See the spec "Attachments" section.

## Rollback
Each subtask is one commit that can be reverted independently. Configuration and database migrations introduced here are additive; the dedupe `--apply` path writes a timestamped DB backup first.
