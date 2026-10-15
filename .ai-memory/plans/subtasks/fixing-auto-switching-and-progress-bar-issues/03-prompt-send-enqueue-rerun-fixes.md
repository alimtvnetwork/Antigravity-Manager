# Subtask 03: Prompt Send, Enqueue & Rerun Fixes

- Update `src-tauri/src/modules/repo_db/agy.rs` (`spawn_or_replace_prompt_via_agy`) so explicit user `Send` / `Rerun` politely terminates any lingering headless `agy.exe` worker for that workspace before spawning the new turn.
- Trigger automatic FIFO queue drain (`dispatch_queued_prompts`) inside the `agy-monitor` thread when a worker exits.
- Invalidate prompt tree cache on send and enqueue.
