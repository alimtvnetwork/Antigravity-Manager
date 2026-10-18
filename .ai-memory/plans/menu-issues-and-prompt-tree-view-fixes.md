# Master Plan: Menu Issues & Prompt Tree View Fixes

## User Request (Verbatim)
```text
Rerun that. Our menu has several issues in the prompt tree view. Make sure you fix those. The menu does not go away, there is flickering, I do not see the enqueue items, and the image or event does not get copied. Several things we have discussed several times, it's not fixed. You should be more careful and fix it. It does not look very good.
```

## High-Level Goals & Deliverables
1. **Task-01**: Centralized Singleton Context Menu & Comprehensive Dismissal Guard (`.ai-memory/plans/subtasks/menu-issues-and-prompt-tree-view-fixes/01-singleton-context-menu-and-dismissal.md`) — **COMPLETED**
2. **Task-02**: Enqueued Prompt Visibility & Auto-Expansion in Tree View (`.ai-memory/plans/subtasks/menu-issues-and-prompt-tree-view-fixes/02-enqueued-items-visibility-and-auto-expand.md`) — **COMPLETED**
3. **Task-03**: Taint-Free Image & Event Clipboard Copying Pipeline (`.ai-memory/plans/subtasks/menu-issues-and-prompt-tree-view-fixes/03-clipboard-image-and-event-copy-fix.md`) — **COMPLETED**
4. **Task-04**: Pre-flight Verification, SemVer Bump, `@aukgit` Attribution & Atomic Push (`.ai-memory/plans/subtasks/menu-issues-and-prompt-tree-view-fixes/04-verification-and-release.md`) — **COMPLETED**

## Concurrency & Ownership Box
- **Worker 01 (Task-01)**: Owned `TreePanel.tsx` & `PromptTreeContextMenu.tsx`.
- **Worker 02 (Task-02)**: Owned `ConversationNode.tsx`, `ProjectNode.tsx`, `conversationGuards.ts`, and `tierClassification.ts`.
- **Worker 03 (Task-03)**: Owned `cmd_files.rs`, `commands/mod.rs`, `run.rs`, and `clipboard.ts`.
- **Release Ceremony (Task-04)**: Author changelogs, READMEs, run pre-flight gates, and atomic commit/push.
