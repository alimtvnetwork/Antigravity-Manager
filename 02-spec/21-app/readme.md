# Application Specifications Index (`02-spec/21-app/`)

| # | Specification File | Summary |
| :--- | :--- | :--- |
| `01` | [01-unified-switch-fast-forward-autoswitch-spec.md](./01-unified-switch-fast-forward-autoswitch-spec.md) | Unified 5-step account switch pipeline (`Backup Prompts -> Close IDE -> Switch Credentials -> Re-open IDE -> Re-inject Prompts`), normalized `/ 1000` candidate scoring (`< 100%` = `0`), and configurable `6h`–`10h` stale instance/Supabase lock expiration. |
| `02` | [02-gitmap-agm-tree-instance-swap-spec.md](./02-gitmap-agm-tree-instance-swap-spec.md) | Latest GitMap CLI parity (`agy`, `ssh`, `update`, `prompt`, `backup/restore`), bracketed Project -> Conversation -> 200-Word Prompt tree view with dual AGM/GitMap sequence IDs (`[AGM:P001 \| GM:#1]`, `[AGM:C001 \| GM:<cid>]`), instance/machine-scoped prompt injection, and isolated multi-instance/multi-project account swapping. |
| `71` | [71-supabase-multi-machine-hierarchy-and-cli-e2e.md](./71-supabase-multi-machine-hierarchy-and-cli-e2e.md) | Supabase multi-machine relational hierarchy (`nodes` parent to `instance_profiles` child), workspace leases cross-machine synchronization, native CLI commands (`agm supabase help/set/status/test`), PowerShell one-liners, and repo-secrets vault update. |

