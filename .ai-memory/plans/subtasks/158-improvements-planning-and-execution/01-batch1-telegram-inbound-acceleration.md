# Subtask 01: Batch 1 Execution (Improvements 31–36: Telegram Inbound Control & Latency Optimization)

## Improvements In Scope
- **Imp-31**: Telegram Startup Sleep Reduction (`src-tauri/src/modules/telegram_inbound/daemon.rs`)
- **Imp-32**: Immediate Callback Spinner Dismissal (`src-tauri/src/modules/telegram_inbound/daemon.rs`)
- **Imp-33**: Telegram Instant Prompt Confirmation (`src-tauri/src/modules/telegram_inbound/send_flow.rs`)
- **Imp-34**: Detached Background Proof Capture (`src-tauri/src/modules/telegram_inbound/send_flow.rs`)
- **Imp-35**: Cluster Node Discovery 30s TTL Cache (`src-tauri/src/modules/telegram_inbound/cluster.rs`)
- **Imp-36**: Telegram Markdown Chunking Buffer (`src-tauri/src/modules/telegram_inbound/prompt_history.rs`)

## Status
`[COMPLETED]`

## Verified Changes
- **Imp-31**: Telegram daemon startup sleep reduced to 500ms (`tokio::time::sleep(Duration::from_millis(500))`) in `src-tauri/src/modules/telegram_inbound/daemon.rs`.
- **Imp-32**: `answerCallbackQuery` invoked immediately upon receipt of callback queries in `src-tauri/src/modules/telegram_inbound/daemon.rs`.
- **Imp-33**: Instant text confirmation dispatched to chat prior to proof capture in `src-tauri/src/modules/telegram_inbound/send_flow.rs`.
- **Imp-34**: `capture_and_send_proof_shot` detached into background Tokio tasks in `src-tauri/src/modules/telegram_inbound/send_flow.rs`.
- **Imp-35**: GitMap cluster node discovery results cached with 30s TTL in `src-tauri/src/modules/telegram_inbound/cluster.rs`.
- **Imp-36**: 4000-character Telegram boundary guard added to `src-tauri/src/modules/telegram_inbound/prompt_history.rs`.
