# Subtasks — Plan 88 email dedupe, Supabase push-settings, vault scripts

Parent plan: [88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md](../../pending/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md)

Order follows data flow: contracts and scaffolds first, callers after, UI and scripts last. Execute **one step per run**; mark the row `done` in the parent plan's status table, then self-loop.

| # | File | Title | Status |
| :--- | :--- | :--- | :--- |
| `001` | [001-recipient-normalized-upsert-and-unique-index.md](./001-recipient-normalized-upsert-and-unique-index.md) | Normalized upsert-or-skip for notification recipients | `pending` |
| `002` | [002-route-all-email-paths-through-helper.md](./002-route-all-email-paths-through-helper.md) | Every add/import path uses the helper and reports skipped counts | `pending` |
| `003` | [003-existing-duplicates-report-and-merge.md](./003-existing-duplicates-report-and-merge.md) | `agm email dedupe` for rows that already exist | `pending` |
| `004` | [004-supabase-load-json-merge-node-alias-and-flags.md](./004-supabase-load-json-merge-node-alias-and-flags.md) | `load-json` applies `node_alias` and sync flags | `pending` |
| `005` | [005-agm-supabase-push-settings.md](./005-agm-supabase-push-settings.md) | `agm supabase push-settings` | `pending` |
| `006` | [006-remove-committed-default-keys-from-repo-scripts.md](./006-remove-committed-default-keys-from-repo-scripts.md) | Remove endpoint keys from `scripts/setup-supabase.ps1` and `scripts/supabase-setup.ps1` | `pending` |
| `007` | [007-connect-supabase-ps1-hardening.md](./007-connect-supabase-ps1-hardening.md) | Harden `connect-supabase.ps1` in the vault repo | `pending` |
| `008` | [008-push-settings-ps1-fixes.md](./008-push-settings-ps1-fixes.md) | Fix `push-settings.ps1` remote transfer and verification | `pending` |
| `009` | [009-shared-script-guard-helpers.md](./009-shared-script-guard-helpers.md) | Shared guard helpers and vault self-check | `pending` |
| `010` | [010-one-liner-and-live-verification.md](./010-one-liner-and-live-verification.md) | Verified one-liner for Supabase on this machine | `pending` |

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
