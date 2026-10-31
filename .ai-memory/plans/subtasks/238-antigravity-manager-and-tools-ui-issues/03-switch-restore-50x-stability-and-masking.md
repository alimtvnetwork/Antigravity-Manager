# Subtask 03: Switch Restore 50x Stability and Post-Switch Privacy Masking

- **Parent Plan**: `.ai-memory/plans/pending/238-antigravity-manager-and-tools-ui-issues.md`
- **Slug**: `antigravity-manager-and-tools-ui-issues - Task 03`
- **Status**: PENDING

## Scope
1. Fix `src-tauri/src/modules/repo_db/resend.rs` and `src-tauri/src/modules/instance/workspace.rs` post-switch restoration:
   - Ensure reliable CDP injection of backed-up prompts after account switch instead of relying solely on headless background `agy` execution.
   - Guard against timing races during IDE relaunch across 50 consecutive switches.
2. Implement 5–10s post-switch prompt conversation privacy masking in `src/pages/instances/useInstancePage.ts` and `src/pages/instances/InstanceCard.tsx`:
   - During the 10s settling window, display privacy shield shimmer / placeholder over conversation turns.
   - Unmask automatically once settling timer completes and restored prompt is confirmed.
