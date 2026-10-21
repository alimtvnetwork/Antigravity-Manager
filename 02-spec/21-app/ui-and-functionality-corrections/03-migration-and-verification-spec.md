# Migration & Verification Specification: UI and Functionality Corrections

## 1. Migration Requirements
- No breaking database migrations required. Schema for `running_projects` and `active_prompts` already supports `status` and `is_running`.
- Width preferences for PromptTreeView can persist in `localStorage` (`agm_prompt_tree_width`).

## 2. Verification Protocol (End-to-End Tests)
1. **Send Button Preemption Test**:
   - Verify clicking Send triggers cancellation on active task and immediately launches target prompt.
2. **Enqueue Button Preservation Test**:
   - Verify clicking Enqueue leaves active running task untouched and enters prompt into queue (`status = 'queued'`).
3. **Resizable Tree View Test**:
   - Verify drag handle or width control allows reducing panel width to `< 260px` and expanding to `> 450px`.
4. **IDE Rename Sync Test**:
   - Verify renaming conversation in Prompt Tree updates both AGM tree node and the corresponding entry in `state.vscdb`.
5. **Theme & Hover Verification**:
   - Verify switching to One Dark Pro renders without hardcoded purple middle pane.
   - Verify hovering over instance cards does not cause layout jitter or coordinate translation.
6. **Scoreboard Popover Test**:
   - Verify hovering over Scoreboard formula cell does not collide with or get pinned under the sticky table header.
7. **Release Gate**:
   - Run Rust formatting & clippy (`cargo fmt -- --check`, `cargo clippy --all-targets --all-features`).
   - Run frontend build (`npm run build`).
   - Execute minor version bump (`npm run bump minor`) following strict `@aukgit` attribution.
