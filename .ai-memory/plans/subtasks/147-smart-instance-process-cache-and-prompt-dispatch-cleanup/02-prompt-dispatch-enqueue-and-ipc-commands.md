# Subtask 02: Prompt Dispatch & Enqueue IPC Commands

## Objective
Implement dedicated, reliable IPC commands `send_prompt_now` and `enqueue_prompt` in `src-tauri/src/commands/instance.rs`, registered in `src-tauri/src/lib.rs`, and backed by `src-tauri/src/modules/repo_db.rs`.

## Actions
1. In `src-tauri/src/modules/repo_db.rs`:
   - Implement `send_prompt_now_for_instance(instance_id, repo_path, prompt_content, conversation_id)`:
     - Uses `ensure_instance_running_smart` to verify IDE process is running (reusing without re-launching).
     - Saves/updates row in `active_prompts` with status `'running'`.
     - Writes `.antigravity_resume_task.json` and `.antigravity_resume_task.<instance_id>.json`.
     - Executes prompt via `spawn_prompt_via_agy(&active_prompt)`.
   - Implement `enqueue_prompt_for_instance(instance_id, repo_path, prompt_content, conversation_id)`:
     - Saves/updates row in `active_prompts` with status `'queued'`.
     - Writes `.antigravity_resume_task.json` with status `'queued'`.
2. In `src-tauri/src/commands/instance.rs`:
   - Expose `send_prompt_now`, `enqueue_prompt`, and `get_running_instances_process_count`.
3. In `src-tauri/src/lib.rs`:
   - Register `commands::send_prompt_now`, `commands::enqueue_prompt`, and `commands::get_running_instances_process_count` in `generate_handler![]`.
