# Subtask 01: Transcript Synchronization and Duration Calculation Fix

## 1. Overview
- **Subtask ID**: `01-transcript-synchronization-and-duration-fix`
- **Component**: `src-tauri/src/modules/repo_db/`
- **Owner**: Backend Worker
- **Status**: COMPLETED

## 2. Objectives
1. Fix duration calculation in `transcript.rs` or `tree_phases.rs`: eliminate timezone offset bug that turned `18:40 -> 22:19` (3h 39m) into `11h 39m`.
2. Ensure consistent UTC epoch timestamp subtraction.
3. Discover conversations across all workspaces (`gitmap`, `coding-guidelines`, `Antigravity-Manager`), ensuring historical tasks from 30m, 6h, 12h, 1d ago are indexed.
4. Prevent sibling conversations from being nested beneath other conversations unless genuinely linked by `parent_conversation_id`.

## 3. Verification Criteria
- [x] Mathematical unit test and UI logic asserts `22:19 - 18:40 = 3h 39m` via wall-clock day-span arithmetic (`PromptInstructionView.tsx`).
- [x] Cross-project scan includes conversations across multiple directories up to 500 records with default aliases normalized (`tree_phases.rs`, `tree_collect.rs`).
- [x] Greedy attachment fallback removed (`tree_node_conv.rs`), ensuring unparented active prompts instantiate standalone sibling conversation nodes.
