# Plan: 125-macos-dmg-install-quota-ui-and-instance-prompt-resend-fix

## Status: ACTIVE
- Parent Task: macOS DMG Installation, Quota Progress Bar UI Overhaul, Instance Weekly Quota Capture, Prompt Tree View Line Gaps, and Instance Prompt Dispatch Engine
- Architecture Spec: `02-spec/21-app/125-macos-dmg-install-quota-ui-and-instance-prompt-resend-fix/01-architecture-spec.md`
- SQLite Task DB: `.ai-memory/temp-agents/127-125-macos-dmg-install-quota-ui/agent-task.db`
- Ledger: `.ai-memory/temp-agents/127-125-macos-dmg-install-quota-ui/ledger.md`

## Subtask Execution Table
| Subtask ID | Task Name | Worker | Owned Files | Status | Evidence |
|---|---|---|---|---|---|
| Task-01 | Prompt Tree View Line Gaps (`<br />`), Dot-Dot Expand & Header Metadata Trio | Worker 01 (Frontend) | `src/components/instances/PromptTreeViewModal.tsx` | PENDING | - |
| Task-02 | Prompt Dispatch Engine (N Key Hotkey & Live IDE Injection) | Worker 01 (Frontend) | `src/components/instances/PromptTreeViewModal.tsx` | PENDING | - |
| Task-03 | Project Activity Scoping (`white-presentation-v1`) & Ghost Conversation Purge | Worker 02 (Backend) | `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/instance.rs` | PENDING | - |
| Task-04 | Quota Progress Bar UI Overhaul (Equal Widths, Fatty Height, Same-Column Time+%, Orange-to-Red) | Worker 02 (Frontend) | `src/components/accounts/QuotaProgressBar.tsx`, `src/pages/Accounts.tsx` | PENDING | - |
| Task-05 | Weekly Quota Capture & Progress Display in Instance Mode | Worker 02 (Frontend) | `src/pages/Instances.tsx`, `src/components/instances/InstanceTable.tsx` | PENDING | - |
| Task-06 | macOS DMG Installation, Auto-Install Script & Code Signature Hardening | Worker 01 (DevOps) | `install.sh`, `scripts/Fix_Damaged.command`, `scripts/package_dmg.sh` | COMPLETED | Verified recursive quarantine removal, trash cleanup, lsregister refresh, stack trace capture, and bundle overwriting. |

## Acceptance Criteria
1. Explicit `<br />` line break tags between lines and paragraphs in Raw and Markdown modes.
2. Clicking "..." expands full prompt instruction.
3. 'N' key and "Send Now" button inject `.antigravity_resume_task.json` and resume IDE with toast notification.
4. Default instance does not report `white-presentation-v1` as RUNNING.
5. Ghost 0-word untitled conversations are hidden/merged.
6. 4H and Weekly quota bars have equal balanced widths; time and percentage are in the same column; height is increased; color transitions to red after final item.
7. Weekly quota progress bar displays inside instance cards and table.
8. macOS install script handles signature and installs automatically.
