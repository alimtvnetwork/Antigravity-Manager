# Migration and Verification Spec: Running and Queued Prompt Detection

## 1. Migration Steps
1. **Schema & Database State**:
   - No breaking database migrations required. Existing `active_prompts` table in `repo_prompts.db` retains its existing schema (`id`, `instance_id`, `repo_path`, `prompt_content`, `status`, `updated_at`, `created_at`, `session_id`).
   - Any rows in `active_prompts` with `status = 'dispatched'` will remain retained under the extended 3600s TTL.
2. **IPC Command Registration**:
   - Register `commands::invalidate_prompt_tree_cache` in `src-tauri/src/lib/run.rs` and `src-tauri/src/commands/instance.rs`.

## 2. Verification Checklist
- [ ] **Liveness Detection**:
  - Run Antigravity IDE and send a long prompt.
  - Open Prompt Tree View in Antigravity Manager.
  - Verify the conversation displays as `RUNNING` with an animated/green running badge (NOT `IDLE`).
  - Verify the active turn displays `RUNNING` (NOT `COMPLETED`).
- [ ] **Queued Messages Visibility**:
  - While the agent is working, type 1 or more messages into the IDE's prompt box and press Enter (so they show under `Queued Messages` in the IDE).
  - Open or refresh Prompt Tree View in Antigravity Manager.
  - Verify all queued messages display under the active conversation as `QUEUED` turns.
  - Verify the FIFO queue banner shows `N prompt(s) enqueued in FIFO queue for this conversation`.
- [ ] **FIFO Enqueue Flow**:
  - Enqueue a prompt via Antigravity Manager's "Enqueue" button.
  - Verify the prompt is injected into the IDE via CDP and recorded in `active_prompts`.
  - Verify it does NOT disappear after 120 seconds.
- [ ] **Cache Invalidation Gate**:
  - Verify `invoke('invalidate_prompt_tree_cache')` executes without `E9001: Command not found` errors.
- [ ] **Pre-flight & Release**:
  - `cargo fmt -- --check`
  - `node scripts/run-frontend-tests.mjs`
  - `node node_modules/typescript/bin/tsc --noEmit`
  - `node --max-old-space-size=4096 ./node_modules/vite/bin/vite.js build`
  - Minor version bump to `v4.282.0`.
