# Master Plan: Smart Instance Process Cache & Prompt Enqueue Fix

- **Slug**: `149-smart-instance-process-cache-and-prompt-enqueue-fix`
- **Protocol**: `execute-parent-task-with-n-steps-v6` (N = 300, A = 2, H = 2)
- **Status**: `[IN PROGRESS]`
- **Created**: 2026-10-09

---

## 1. Executive Summary & Goals

The user reported three interconnected issues in Antigravity Manager:
1. **Prompt Sending / Enqueue Failure & Unwanted Relaunch**: Sending or enqueueing prompts from the instance section fails to inject into running Antigravity IDE instances and repeatedly opens/reopens new IDE instances instead.
2. **False-Positive Running Items**: "Too many running items, which is not running", caused by historical prompts being resurrected and loose mtime detection.
3. **UI Visual Noise**: Excessive tags and bracket tokens `[...]` cluttering the prompt view.

---

## 2. Planned Waves & Subtask Mapping

### Wave 1: Core Engine & Dispatch Wiring
- **Subtask 01**: Smart Process Cache Unification & Closed-PID Re-Scan (`src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/process.rs`)
- **Subtask 02**: Prompt Dispatch & Unified FIFO Enqueue Pipeline (`src-tauri/src/commands/instance.rs`, `src-tauri/src/modules/repo_db.rs`, `src/services/instanceService.ts`)

### Wave 2: State Hardening & UI Compaction
- **Subtask 03**: Ghost Running Eradication & Idle State Protection (`src-tauri/src/modules/repo_db.rs`)
- **Subtask 04**: Prompt Tree View Tag Compaction & Bracket Noise Stripping (`src/components/instances/PromptTreeViewModal.tsx`, `src/components/instances/InstanceTable.tsx`)

### Wave 3: Testing, Pre-flight & Release Ceremony
- **Subtask 05**: Host-Shielded E2E Tests, CLI Parity & Minor Version Bump (`03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`, manifests, changelogs, READMEs)

---

## 3. Strict Operating Invariants
- Total ban on raw search tools (`rg`, `ripgrep`, `grep`, `git grep`, `Select-String`). Search exclusively via GitMap.
- Strictly relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`).
- Attribution strictly `@aukgit` (`(Thanks to @aukgit)`).
- Never terminate or relaunch running Antigravity IDE instances during prompt operations.
