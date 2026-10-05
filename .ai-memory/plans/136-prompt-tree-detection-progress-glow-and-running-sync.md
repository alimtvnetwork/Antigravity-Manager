# Master Execution Plan: 136-prompt-tree-detection-progress-glow-and-running-sync

**Status:** IN PROGRESS  
**Created:** 2026-10-05  
**Target Version:** v4.157.0

## Problem Summary

1. **Empty Prompt Tree**: `workspaceStorage` is empty on all instances. Projects live in `conversation_summaries.db`. Tree view shows 0 projects across all instances.
2. **Wrong Progress Bar Colors**: Cyan/teal palette (introduced in task 134) must be reverted to vibrant neon green `#1af18d` with `shadow-[0_0_12px_rgba(26,241,141,0.85)]`.
3. **Missing Modal Header Identity**: `PromptTreeViewModal` must show `[#seq] [Profile Name] [...\Antigravity.exe]`.
4. **Harsh Green Button Chrome**: Instance action buttons have aggressive green/emerald styling.
5. **In-flight Auto-Switch**: Must verify prompt re-injection after account rotation.

## Subtask Checklist

- [ ] Task-01: Empty prompt tree fix — multi-source project discovery from `conversation_summaries.db` workspace URIs
- [ ] Task-02: Neon glowing green progress bar restoration + checkpoint node glow
- [ ] Task-03: PromptTreeViewModal header identity standardization  
- [ ] Task-04: Instances.tsx wiring + button de-greening
- [ ] Task-05: Auto-switcher in-flight quota monitoring verification
- [ ] Task-06: Atomic GitMap commit

## Worker Allocation

| Worker | Tasks | Files |
|--------|-------|-------|
| Worker 01 | Task-02, Task-03, Task-04 | `QuotaProgressBar.tsx`, `WaterDrainProgressBar.tsx`, `PromptTreeViewModal.tsx`, `Instances.tsx`, `InstanceTable.tsx` |
| Worker 02 | Task-01, Task-05 | `repo_db.rs`, `auto_switcher.rs` |

## Research Findings

- `detect_running_projects()` relies on `workspaceStorage/*/workspace.json` — all empty
- `compute_project_conversation_tree()` reads `conversation_summaries.db` correctly at lines 4162–4312
- The disconnect: `list_running_projects()` at line 4071 returns projects from `running_projects` SQLite table which is only populated by `detect_running_projects()` — which scans empty workspaceStorage
- Fix: Build `projects` list from `conversation_summaries.db` workspace_uris directly, bypassing the `running_projects` table dependency
