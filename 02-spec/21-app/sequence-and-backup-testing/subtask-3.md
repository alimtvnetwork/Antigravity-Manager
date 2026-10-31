# Subtask 3: End-to-End Prompt Backup & Restore Testing with Screenshot IDs

## 1. Context & Invariants
- **Target**: `src-tauri/src/modules/repo_db/`, `src-tauri/src/commands/prompt_cmd.rs`, `src-tauri/src/bin/agm.rs`.
- **Core Invariant**:
  - Prompt queueing (`agm prompt queue <text>`) must register prompts into SQLite `active_prompts` with status `queued` in strict FIFO order.
  - Prompt backup (`agm prompt backup` / UI Export) must snapshot queued and active prompts into `prompt_backups` with complete metadata.
  - Backed-up prompts containing screenshot identifiers must preserve those identifiers verbatim.
  - Prompt restore (`agm prompt restore` / UI Import) must re-inject backed-up prompts in strict FIFO order (`ORDER BY created_at ASC, id ASC`) into `active_prompts` without data loss.

## 2. Testing Procedure
1. Generate a synthetic test prompt with an explicit Screenshot ID token:
   `TEST_PROMPT="[TEST-VERIFY] Automated prompt backup verification with Screenshot ID: SS_ID_987654321"`
2. Queue the prompt via CLI:
   `agm prompt queue "$TEST_PROMPT"`
3. Execute prompt backup:
   `agm prompt backup`
4. Query `prompt_backups` to confirm the prompt snapshot was created.
5. Simulate queue eviction / restoration need:
   Run `agm prompt restore`.
6. Assert that the prompt with `Screenshot ID: SS_ID_987654321` is present in active prompts, ready for execution.
