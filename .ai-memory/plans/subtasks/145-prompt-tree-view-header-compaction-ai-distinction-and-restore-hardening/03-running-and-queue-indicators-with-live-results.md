# Subtask 03: Running and Queue Indicators with Live Results

**Parent Plan:** [145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md](../../145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md)  
**Status:** `IN PROGRESS`  

---

## Objectives
1. Prominent running and queued visual indicators:
   - Glowing emerald pulse badge for active prompts: `● RUNNING (PID · mm:ss)`.
   - Amber badge for queued prompts: `⏳ QUEUED`.
   - Project nodes display glowing count: `N RUNNING`.
2. Extract live execution results and AI responses:
   - Backend `repo_db.rs` extracts `latest_response`, `tool_calls`, and `latest_step_summary`.
   - In UI, add `[ Prompt Instruction ]` vs `[ AI Results & Tool Outputs ]` tabs.
   - For running prompts, live step output streams in real-time.
3. Harden "Focus IDE" / "Open IDE Window":
   - Ensure the Antigravity window is restored, un-minimized, and brought to foreground.
   - Pass target repository directory so the user is brought directly to the relevant workspace.
