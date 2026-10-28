# Subtask 01 — Per-instance Telegram status cards

Implement `src-tauri/src/modules/telegram_inbound/instance_cards.rs`. Fan-out `/status` and `/observe` to one fleet header plus one message per instance with ~100-word prompt preview and `i:` action buttons (copy/full/rerun/done/released/auto/ff/shot/refresh). Wire callbacks in `daemon.rs`. Scope `/ff` to an instance id.
