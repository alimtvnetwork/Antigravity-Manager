# Master Plan: Task 110 - Running Instance Auto-Sync, Auto-Selection, Project Archive & Stale Prompts Demotion

## 1. Plan Overview
- **Task ID**: `110-running-instance-auto-sync-and-project-archive`
- **Scope**:
  - Subtask 1: Auto-Selection on Modal Open & Configurable Background Interval Sync (`src/components/instances/PromptTreeViewModal.tsx`).
  - Subtask 2: Per-Instance Project Archive / Less Favorite & Stale Prompts Demotion (`src/components/instances/PromptTreeViewModal.tsx`).
  - Subtask 3: E2E Verification, Pre-flight Checks & Minor Release Ceremony (`v4.137.0` -> `v4.138.0`).

---

## 2. Work Breakdown & Subtasks

### Subtask 1: Auto-Selection on Modal Open & Configurable Interval Sync
- **File**: `src/components/instances/PromptTreeViewModal.tsx`
- **Goal**:
  1. Detect actively running conversation/project on modal open and auto-select/auto-expand it. Fallback to newest conversation of the first active project with latest prompt turn.
  2. Implement configurable interval selector capsule (`15s`, `30s`, `1m`, `2m`, `Off`) with minimum 15s safety floor and `localStorage` persistence.
  3. Non-destructive timer loop with `force: true` cache bypass.

### Subtask 2: Per-Instance Project Archive & Stale Prompts Demotion
- **File**: `src/components/instances/PromptTreeViewModal.tsx`
- **Goal**:
  1. Add Project Archive / Less Favorite toggle button on project rows with `localStorage` persistence (`agm_archived_projects_{instanceId}`).
  2. Partition active vs archived projects with bottom collapsible `📁 Archived Projects ({count})` and `[Archived]` filter pill.
  3. Demote empty/stale conversations to bottom under `📁 Archived / Stale Prompts ({count})`.

### Subtask 3: Pre-flight Verification & Minor Release Ceremony
- Run `cd src-tauri && cargo fmt -- --check`.
- Run `npm run build`.
- Execute minor bump (`v4.137.0` -> `v4.138.0`).
- Update `CHANGELOG.md`, `CHANGELOG_EN.md`, `README.md`, `README_EN.md` with strict `@aukgit` attribution.
- Commit via `gitmap cpf` and tag `v4.138.0`.
