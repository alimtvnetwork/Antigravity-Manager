# Subtask 05: Test and Confirm Auto-Switching Prompt Restoration

- **Task**: Instance-Based Prompt and UI Functionality Verification
- **Subtask**: 05 of 06
- **Status**: Completed
- **Path**: `.ai-memory/plans/subtasks/241-instance-based-prompt-and-ui-functionality-verification/05-subtask-auto-switch-and-prompt-restoration.md`

## Audit Outcomes
- Verified `src-tauri/src/modules/repo_db/backup.rs` and `src-tauri/src/modules/auto_switcher/instances.rs`:
  - Complete 5-step lifecycle confirmed:
    1. In-flight prompts captured to `active_prompts` and `~/.antigravity_tools/post_switch_resume_state.json`.
    2. IDE process tree cleanly terminated with Win32 escalation and 200ms kernel settle delay.
    3. Google account credentials swapped in `state.vscdb`.
    4. IDE relaunched with switched profile; 10-second settling delay + CDP readiness ladder executed.
    5. Prompts re-injected into the same `session_id` in strict FIFO order (`ORDER BY created_at ASC, id ASC`).
