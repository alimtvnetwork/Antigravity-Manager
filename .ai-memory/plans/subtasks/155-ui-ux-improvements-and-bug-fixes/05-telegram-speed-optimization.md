# Subtask 05: Telegram Communication Speed Optimization

## Objective
Accelerate Telegram prompt inspection and sending turnaround to sub-second responses.

## Target Files
- `src-tauri/src/modules/telegram_inbound/daemon.rs`
- `src-tauri/src/modules/telegram_inbound/send_flow.rs`
- `src-tauri/src/modules/telegram_inbound/prompt_history.rs`
- `src-tauri/src/modules/telegram_inbound/cluster.rs`
- `src-tauri/src/modules/telegram_inbound/node_label.rs`

## Technical Details
1. Drop the 60s startup sleep in `daemon.rs:686` to 500ms for immediate polling.
2. In `dispatch_send_prompt` and `rerun_prompt_by_id`, send immediate text confirmation (`Prompt dispatched!`) to the user (<100ms) and spawn screenshot proof capture as an asynchronous background task.
3. Fire `sendChatAction` with `"typing"` immediately upon receiving incoming commands.
4. Add in-memory caching with 30s TTL for `query_gitmap_cluster_nodes` to eliminate 15–25 redundant sequential `gitmap.exe` process spawns per command.
