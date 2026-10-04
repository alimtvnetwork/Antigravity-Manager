# Plan: 124-gitmap-prompt-tree-ui-gap-expansion-and-liveness-fix

## Status: ACTIVE
- Parent Task: GitMap Prompt Tree View Line Gap, Dot-Dot Expansion, N Key / Send Now, Project Liveness False-Positive Fix, and Header Metadata Trio
- Architecture Spec: `02-spec/21-app/124-gitmap-prompt-tree-ui-gap-expansion-and-liveness-fix/01-architecture-spec.md`
- SQLite Task DB: `.ai-memory/temp-agents/126-124-gitmap-prompt-tree-ui-gap/agent-task.db`
- Ledger: `.ai-memory/temp-agents/126-124-gitmap-prompt-tree-ui-gap/ledger.md`

## Subtask Execution Table
| Subtask ID | Task Name | Worker | Owned Files | Status | Evidence |
|---|---|---|---|---|---|
| Task-01 | Frontend: Line gaps with `<br />` tags and Markdown formatting | Worker 01 (Frontend) | `src/components/instances/PromptTreeViewModal.tsx` | PENDING | - |
| Task-02 | Frontend: Dot-dot click-to-expand, N key hotkey & Send Now dispatch | Worker 01 (Frontend) | `src/components/instances/PromptTreeViewModal.tsx` | PENDING | - |
| Task-03 | Frontend: Header metadata trio, tail snippet, and empty conversation filter | Worker 01 (Frontend) | `src/components/instances/PromptTreeViewModal.tsx` | PENDING | - |
| Task-04 | Backend: Project liveness scoping & recency gates (white-presentation-v1 fix) | Worker 02 (Backend) | `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/instance.rs` | PENDING | - |
| Task-05 | Backend: Filter & merge empty untitled conversations (0 words) | Worker 02 (Backend) | `src-tauri/src/modules/repo_db.rs` | PENDING | - |
| Task-06 | Backend: Include instance trio and tail snippet in tree nodes | Worker 02 (Backend) | `src-tauri/src/modules/repo_db.rs` | PENDING | - |

## Verification Criteria
1. Prompt instruction text renders explicit `<br />` tags between lines/paragraphs in both Raw and Markdown Preview modes.
2. Clicking "..." expands full prompt instruction.
3. Pressing 'N' hotkey or clicking "Send Now" button triggers resume task and displays toast confirmation.
4. Default instance does not falsely mark `white-presentation-v1` as RUNNING.
5. 0-word untitled conversations are filtered out from the tree.
6. Prompt view header displays prompt sequence, tail excerpt snippet, and instance trio `[#seq · exe · name]`.
