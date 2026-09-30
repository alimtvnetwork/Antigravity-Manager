# Subtasks — Plan 88 email dedupe, Supabase push-settings, one-liner

Parent plan: [88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md](../../pending/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md)

Order follows data flow: contracts and scaffolds first, callers after, UI and scripts last. Execute **one step per run**; mark the row `done` in the parent plan's status table, then self-loop.

| # | File | Title | Status |
| :--- | :--- | :--- | :--- |
| `001` | [001-recipient-normalized-upsert-and-unique-index.md](./001-recipient-normalized-upsert-and-unique-index.md) | Normalized upsert-or-skip for notification recipients | `pending` |
| `002` | [002-route-all-email-paths-through-helper.md](./002-route-all-email-paths-through-helper.md) | Every add/import path uses the helper and reports skipped counts | `pending` |
| `003` | [003-existing-duplicates-report-and-merge.md](./003-existing-duplicates-report-and-merge.md) | `agm email dedupe` for rows that already exist | `pending` |
| `004` | [004-supabase-load-json-applies-sync-flag-from-vault.md](./004-supabase-load-json-applies-sync-flag-from-vault.md) | `load-json` imports keys and applies the sync flag from the vault file | `pending` |
| `005` | [005-agm-supabase-push-settings.md](./005-agm-supabase-push-settings.md) | `agm supabase push-settings --vault <path>` | `pending` |
| `006` | [006-one-liner-powershell-file-uses-agm-cli.md](./006-one-liner-powershell-file-uses-agm-cli.md) | One-liner PowerShell file that imports keys through the AGM CLI | `pending` |
| `007` | [007-live-verification-of-supabase-one-liner.md](./007-live-verification-of-supabase-one-liner.md) | Run and verify the one-liner on this machine | `pending` |

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
