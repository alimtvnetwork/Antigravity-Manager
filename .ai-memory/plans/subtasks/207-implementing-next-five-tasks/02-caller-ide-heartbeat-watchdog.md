# Subtask 02: Caller IDE Vitality Heartbeat & Eviction Watchdog (`Imp-661`)
Status: COMPLETED
Parent Plan: 207-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Caller IDE Self-Preservation & Transient Test Sandboxing:
"Automated tests, CLI commands, and auto-switcher daemons MUST detect the caller's active IDE process (`detect_caller_instance_id()`) to prevent terminating the developer's working IDE session. Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided."

Implement `src-tauri/src/modules/instance/caller_ide_heartbeat_watchdog.rs` and unit tests in `src-tauri/src/modules/instance/tests_caller_ide_heartbeat_watchdog.rs`.

## Requirements
- Maintain background heartbeat state tracking caller instance PID and timestamp.
- Evaluate auto-switcher candidate evictions to protect the caller session from termination.
- Detect caller process termination/liveness via PID check.
- Return structured `CallerIdeHeartbeatReport` with liveness verdict, eviction immunity status, and heartbeat duration.
