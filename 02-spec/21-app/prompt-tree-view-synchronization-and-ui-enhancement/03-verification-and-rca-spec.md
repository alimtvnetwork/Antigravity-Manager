# Verification and Root Cause Analysis Specification: Prompt Tree View Synchronization and UI Enhancement

## 1. Root Cause Analysis (RCA)

### 1.1 RCA Item 1: Erroneous Duration Calculation (`11h 39m` for `18:40 -> 22:19`)
- **Symptom**: In Screenshot 2 (`assets/screenshots/prompt-tree-view-synchronization-and-ui-enhancement-02.png`), the conversation header displays `11h 39m (18:40 -> 22:19)`.
- **Underlying Cause**:
  1. The start and end timestamps extracted from the conversation transcript or summary table may be mixed between UTC and local time (e.g. 8-hour timezone offset between UTC and GMT+8: `3h 39m + 8h = 11h 39m`).
  2. If the start time was interpreted in UTC while the end time was formatted in local time, an 8-hour error was injected directly into the subtraction `(end - start)`.
- **Fix**:
  1. Normalize both `start_time` and `end_time` to raw epoch milliseconds before subtraction.
  2. Calculate duration strictly as `(end_epoch_ms - start_epoch_ms)`.
  3. Format the duration using a shared duration formatter that never mixes timezones.

### 1.2 RCA Item 2: Missing Cross-Workspace Conversations in Tree View
- **Symptom**: In Screenshot 1 (`assets/screenshots/prompt-tree-view-synchronization-and-ui-enhancement-01.png`), the IDE lists conversations from `coding-guidelines` (`Letterly Prompt Length Optimization`, 30m ago) and older conversations from `gitmap` (`Read & Understand: agy-rec-real-217945324`, 6h ago), but AGM's tree view in Screenshot 2 only shows `gitmap` with one active conversation and omits `coding-guidelines`.
- **Underlying Cause**:
  1. AGM's workspace discovery queries only currently open/active instances or filtered workspaces, or misses projects whose `workspaceStorage` paths are not in the primary index.
  2. The SQLite prompt tree cache only updates on specific triggers and does not re-scan other folders if cached.
- **Fix**:
  1. Ensure `discovery.rs` iterates across all workspace directories registered in `projects.db` and Antigravity `workspaceStorage`.
  2. Ensure completed conversations from the past 24-48 hours are populated under their respective project node.

### 1.3 RCA Item 3: False Hierarchy Nesting (Top-Level Conversation Rendered Under C209)
- **Symptom**: In Screenshot 2, `C213 Read & Understand: agy-r...` is displayed nested underneath `C209 ENT Goal Migration And Email Commands`, even though C213 is a distinct conversation in the IDE history.
- **Underlying Cause**:
  1. The tree construction algorithm groups nodes by heuristic heuristics (e.g., if a conversation was started shortly after another or shares a prefix), falsely treating C213 as a child node.
  2. Only genuine subagent invocations (identified by `parent_conversation_id` or `invoke_subagent` tool calls) should be nested.
- **Fix**:
  1. Enforce strict check: only nest if `parent_conversation_id` is explicitly set and matches the parent conversation ID.
  2. All primary user conversations must render as siblings directly under the project node.

### 1.4 RCA Item 4: Non-Functional Reset Button & Stale Version Cache
- **Symptom**: Clicking "Reset" does not clear the cached tree or reload fresh data, and version upgrades retain stale tree layouts.
- **Underlying Cause**:
  1. The Reset button in the frontend only resets local React state (like search filters or expanded node set) rather than triggering a backend cache invalidation command.
  2. `prompt_tree_cache` does not track `app_version`.
- **Fix**:
  1. Bind Reset button to `cmd_reset_prompt_tree` / `/api/prompts/reset`, clearing `prompt_tree_cache` in SQLite.
  2. Track `cached_version` in `gui_config.json` or `repo_prompts.db`; auto-invalidate on version mismatch.

## 2. 4-Part Verification Plan

### Test 1: Mathematical Duration Verification
- Unit test feeding `18:40:00` and `22:19:00` timestamps:
  - Asserts that duration equals `3h 39m`, never `11h 39m`.
  - Asserts timezone consistency between start and end.

### Test 2: Hierarchy & Nesting Verification
- Unit test feeding two independent conversations `C209` and `C213`:
  - Asserts that `C213` has `parent_id = None`.
  - Asserts that both `C209` and `C213` render as direct children of the project node.

### Test 3: Relative Time Badge Formatting
- Unit test covering relative recency thresholds:
  - 1 min ago -> `now`
  - 30 mins ago -> `30m`
  - 6 hours ago -> `6h`
  - 12 hours ago -> `12h`
  - 28 hours ago -> `1d`

### Test 4: Reset Button & Version Cache Invalidation
- Integration test calling reset API:
  - Asserts `prompt_tree_cache` rows are deleted.
  - Asserts re-query populates fresh rows with updated timestamp and version.
