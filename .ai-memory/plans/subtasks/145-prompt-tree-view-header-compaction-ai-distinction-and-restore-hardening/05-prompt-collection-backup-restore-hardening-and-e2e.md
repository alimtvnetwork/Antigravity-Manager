# Subtask 05: Prompt Collection Backup Restore Hardening and E2E

**Parent Plan:** [145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md](../../145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md)  
**Status:** `IN PROGRESS`  

---

## Objectives
1. Harden backend prompt collection in `repo_db.rs`:
   - Dual-source transcript extraction (`transcript_full.jsonl` fallback).
   - Capture `latest_response` and tool activity.
   - Robust detection of running status from active transcript file modification times and process liveness.
2. Harden Backup & Restore:
   - Complete JSON backup serialization capturing projects, prompts, results, and origin metadata.
   - FIFO restoration order preserving prompt sequence when writing to `.antigravity_resume_task.json`.
3. Author comprehensive E2E test script and run local verification.
4. Pass all pre-flight checks: `cargo fmt`, `cargo clippy`, and `npm run build`.
