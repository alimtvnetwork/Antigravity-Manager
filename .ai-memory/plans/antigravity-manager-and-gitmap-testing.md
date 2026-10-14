# Plan: Antigravity Manager and GitMap testing

- **Slug:** `antigravity-manager-and-gitmap-testing`
- **Status:** `IN PROGRESS`
- **Spec:** `02-spec/21-app/antigravity-manager-and-gitmap-testing/01-root-cause.md`

## Done in this repo

- Stopped the startup purge from deleting `prompt_tree_cache`.
- CLI tree reads that cache instead of forcing a rescan.
- Raised prompt tree SQLite cache TTL from 5 seconds to 120 seconds.
- Caught panics in `get_project_conversation_tree` and returned the backtrace.
- Added a Reset control on the prompt tree header and `agm prompts reset-tree`.

## Not done

- Live click-through of Send, Enqueue, backup, restore, and account switch at 98 percent.
- GitMap login email prompt, CLI color, and auto-commit. `gitmap` did not start on this machine.
- Ubuntu pass.

## Subtasks

- [01-prompt-tree-load-and-reset.md](./subtasks/antigravity-manager-and-gitmap-testing/01-prompt-tree-load-and-reset.md)
- [02-e2e-cases-for-ai.md](./subtasks/antigravity-manager-and-gitmap-testing/02-e2e-cases-for-ai.md)
