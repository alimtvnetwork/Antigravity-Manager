# Plan: System-Wide Broadcasting Design Pattern & 30-Second Minimum Interval Guard

## Executive Summary
Audit, standardize, and enforce the system-wide broadcasting design pattern across both **Antigravity Manager (AGM)** and **GitMap**, ensuring all recurring schedules, background polling loops, notification broadcasts, and worker tickers operate at intervals of **not less than 30 seconds** (`>= 30s`), with startup quiet delays and deduplication guards.

## Subtasks
- [x] **Subtask 1: AGM Invariant Clamping & Deduplication Expansion**
  - In `src-tauri/src/modules/repo_db/goal_heartbeat.rs`: Clamped `interval_secs` to `interval_secs.max(30)` (default 30s instead of 5s).
  - In `src-tauri/src/modules/notification_hub/channels.rs`: Expanded duplicate switch notification suppression window from 5s to 30s (`(now_ts - guard.1).abs() <= 30`) to eliminate notification broadcast storms.
  - Verified all AGM schedulers in `modules/scheduler.rs`, `modules/auto_switcher/`, `modules/email_watcher/`, `modules/telegram_inbound/`, and `modules/supabase_sync/` maintain >= 30s intervals.
- [x] **Subtask 2: GitMap Interval Clamping & Concurrency Safety**
  - In `d:/work/gitmap/cli/constants/constants_task.go`: Updated `TaskMinInterval = 30` (up from 2s).
  - In `d:/work/gitmap/cli/cmd/async_cmd.go`: Clamped parsed interval to `opts.intervalS = max(opts.intervalS, 30)`.
  - In `d:/work/gitmap/cli/macro/async_runner.go`: Clamped `opts.IntervalSec = max(opts.IntervalSec, 30)`.
  - Verified cluster worker pool bounded concurrency (`maxWorkers = 10`) and SSH fleet pull 30s elapsed gate.
- [x] **Subtask 3: Quality Gates & Verification**
  - Ran `cargo fmt -- --check`: PASS (0 diffs).
  - Ran `cargo check --lib`: PASS (exit code 0).
  - Executed targeted tests `modules::telegram_inbound::tests`: PASS (15/15 passed).
  - Verified Go compilation & test pass in GitMap (`cmd`, `macro`, `cluster`, `cmdtask`). Pushed to GitMap `origin/main` (`dcb0577ec`).
