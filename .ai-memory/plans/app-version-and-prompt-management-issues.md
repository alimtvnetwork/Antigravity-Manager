# Master Plan: App Version and Prompt Management Issues

## Plan Information
- **Title**: App Version and Prompt Management Issues
- **Slug**: app-version-and-prompt-management-issues
- **Author**: @aukgit
- **Created**: 2026-10-19
- **Status**: Completed
- **Specification**: `02-spec/21-app/app-version-and-prompt-management-issues/spec.md`

---

## Subtasks Breakdown

- [x] **Subtask 1**: Fix Prompt Tree View Selection Glitching & Modal Backdrop Dismissal (`src/components/instances/PromptTreeView/`)
  - Subtask Plan: `.ai-memory/plans/subtasks/app-version-and-prompt-management-issues/subtask-1-tree-glitch-and-backdrop.md`
- [x] **Subtask 2**: Attached Images Visibility, Thumbnail Rendering & Clipboard Copy (`PromptInstructionView.tsx`, `clipboard.ts`)
  - Subtask Plan: `.ai-memory/plans/subtasks/app-version-and-prompt-management-issues/subtask-2-image-visibility-and-copy.md`
- [x] **Subtask 3**: Import/Export Menu Options: Combined Running+Queued Prompts & Direct Restore Action (`DetailHeader.tsx`, `usePromptTreeCoreActions.ts`)
  - Subtask Plan: `.ai-memory/plans/subtasks/app-version-and-prompt-management-issues/subtask-3-import-export-combined-restore.md`
- [x] **Subtask 4**: Telegram Multimodal Prompt Dispatch & Verification (`src-tauri/src/modules/telegram_bot/`)
  - Subtask Plan: `.ai-memory/plans/subtasks/app-version-and-prompt-management-issues/subtask-4-telegram-prompt-testing.md`
- [x] **Subtask 5**: App Version Publishing Gate & Post-Restore Account Switch Timer HUD (`use-update-store.ts`, `updater.rs`)
  - Subtask Plan: `.ai-memory/plans/subtasks/app-version-and-prompt-management-issues/subtask-5-app-version-and-switch-hud.md`

---

## Non-Negotiable Invariants & Guardrails
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Git Paths**: Zero absolute filesystem paths (`D:\...`, `C:\...`).
- **Developer Session Immunity**: Never terminate or disrupt PID 32 (`Antigravity.exe`).
- **Strictly Attribute @aukgit**: All release notes and changelogs attribute only `@aukgit`.
