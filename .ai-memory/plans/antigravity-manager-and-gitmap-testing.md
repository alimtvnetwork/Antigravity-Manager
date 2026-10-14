# Plan: Antigravity Manager and GitMap testing

- **Slug:** `antigravity-manager-and-gitmap-testing`
- **Status:** `COMPLETED`
- **Spec:** `02-spec/21-app/antigravity-manager-and-gitmap-testing/01-root-cause.md`

## Completed

- Stopped the startup purge from deleting `prompt_tree_cache`.
- CLI tree reads that cache instead of forcing a rescan (`2402 ms` cold scan vs `183 ms` warm cached read).
- Raised prompt tree SQLite cache TTL from 5 seconds to 120 seconds and aligned CLI/GUI `max_words` (`2000`).
- Caught panics in `get_project_conversation_tree` and returned the backtrace.
- Added a Reset control on the prompt tree header and `agm prompts reset-tree`.
- Fixed the React `useEffect` render loop in `src/components/instances/PromptTreeView/usePromptTreeData.ts` that caused `msedgewebview2.exe` `0xe0000008` OOM crashes.
- Completed sequential E2E of Send, Enqueue, Copy, Resend, Backup, Restore, and Auto-Switch (`98%` test threshold restored to `15%`).

## Subtasks

- [01-prompt-tree-load-and-reset.md](./subtasks/antigravity-manager-and-gitmap-testing/01-prompt-tree-load-and-reset.md)
- [02-e2e-cases-for-ai.md](./subtasks/antigravity-manager-and-gitmap-testing/02-e2e-cases-for-ai.md)
