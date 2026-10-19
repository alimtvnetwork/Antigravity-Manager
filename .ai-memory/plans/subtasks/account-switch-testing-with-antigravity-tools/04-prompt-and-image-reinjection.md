# Subtask 04: Prompt and Image Re-injection

## Objective
Verify that the backed-up prompt text and multimodal image attachment are successfully restored and re-injected post-switch without truncation or data loss.

## Execution Steps
1. Inspect `.antigravity_resume_task.<test_instance>.json` post-switch.
2. Verify database rehydration:
   - Check `backup-prompts.db:prompt_backups` where `is_restored = 1`.
   - Check `repo_prompts.db:active_prompts` where status transitioned to `dispatched` or `restored`.
3. Verify image attachment integrity:
   - Ensure the re-injected prompt preserves the Base64 image data URI / file path intact.
4. Record restoration latency and verification status.
