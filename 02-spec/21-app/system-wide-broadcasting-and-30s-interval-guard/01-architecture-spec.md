# Specification: System-Wide Broadcasting Design Pattern & 30-Second Minimum Interval Guard

## 1. Overview & Problem Definition

In distributed and multi-process architectures such as **Antigravity Manager (AGM)** and **GitMap**, background timers, schedulers, notifications, and broadcasting mechanisms are critical for real-time synchronization, quota tracking, and cluster coordination. However, if background routines, notification dispatchers, or file synchronization loops fire at unconstrained or sub-second frequencies, the system suffers severe degradation:
- High CPU utilization and battery drain from spinning loops
- SQLite database lock contention and WAL write thrashing
- Remote API rate-limiting (Telegram 429 errors, Google OAuth quota exhaustion)
- Terminal clutter and network socket starvation

To prevent these failure modes, both Antigravity Manager and GitMap mandate a **System-Wide Invariant: No periodic background broadcast, polling scheduler, or automated sync loop may execute faster than once every 30 seconds (`interval >= 30s`)**, coupled with exponential backoffs, quiet startup periods (60s), and sliding debounces.

---

## 2. Antigravity Manager (AGM) Audit & Interval Inventory

AGM centralizes its background execution across specialized modules in `src-tauri/src/modules/` and `src-tauri/src/proxy/`.

| Subsystem / Daemon | Location | Configured Interval | 30s Minimum Invariant Enforcement | Startup Quiet Period |
|---|---|---|---|---|
| **Auto-Profile Switcher** | `modules/auto_switcher/status.rs` | 30s – 120s adaptive ladder | Clamped strictly: `interval_secs.clamp(30, 50)` when instances run; 120s when idle | 3s initial pause |
| **IDE Crash Watchdog** | `modules/auto_switcher/status.rs` | 60s – 120s | Clamped to `.max(60)` | 60s quiet period |
| **Weekly Reset Warmup** | `modules/scheduler.rs` | 300s (5 min) | Fixed `Duration::from_secs(300)` | 60s quiet period |
| **Prompt FIFO Queue Ticker** | `modules/scheduler.rs` | 30s (idle) / 5s (in-flight dispatch) | Backs off to 30s when empty; 5s only when user explicitly enqueues tasks waiting for idle IDE | 5s initial pause |
| **Instance PID & Quota Sync** | `modules/scheduler.rs` | 600s (10 min) | Fixed `Duration::from_secs(600)` | 35s quiet period |
| **Email Inbound / Watcher** | `modules/email_watcher/heartbeat.rs` | 30s (tick), 300s (telemetry) | Main tick loop `Duration::from_secs(30)`. Quota & idle alerts enforce **600s cooldown** | 60s quiet period |
| **Telegram Inbound Poller** | `modules/telegram_inbound/daemon.rs` | 25s server long-poll, 30s-60s idle | Long-polling HTTP wait of 25s; 60s sleep if disabled, 30s if token empty. 3s tap debounce | 60s quiet period |
| **Supabase Fleet Heartbeat** | `modules/supabase_sync/fleet.rs` | 300s (5 min) | Fixed `Duration::from_secs(300)` | 60s quiet period |
| **Supabase Auto-Pruner** | `modules/supabase_pruner.rs` | 300s (5 min) | Fixed `Duration::from_secs(300)` | N/A |
| **Conversation Cleanup** | `modules/agy_cleaner/daemon.rs` | 3600s – 86400s (1–24 hr) | Clamped to `interval_hours.max(1) * 3600` | N/A |
| **Notification Hub Broadcast** | `modules/notification_hub/channels.rs` | Event-driven | **30s duplicate switch suppression** (`LAST_SWITCH_DISPATCH`) | Immediate on switch event |
| **Prompt Goal Heartbeat Worker** | `modules/repo_db/goal_heartbeat.rs` | User CLI / detached | Clamped to `interval_secs.max(30)` (default 30s) | N/A |

---

## 3. GitMap Audit & Interval Inventory

GitMap governs cluster execution, automated repository synchronization, and background monitors in `d:/work/gitmap/cli/`.

| Subsystem / CLI Feature | Location | Configured Interval | 30s Minimum Invariant Enforcement | Concurrency & Resource Guard |
|---|---|---|---|---|
| **Task Sync Loop (`task-run`)** | `cli/cmdtask/tasksync.go` | User defined | `enforceMinInterval(interval)` clamped to `TaskMinInterval = 30` | Single goroutine per task, `filepath.Walk` with ignore filter |
| **Async Monitor (`gitmap async`)** | `cli/cmd/async_exec.go` | `-t <seconds>` | Clamped to `opts.intervalS = max(opts.intervalS, 30)` | Bounded subprocess execution with timestamp logging |
| **Macro Async Runner** | `cli/macro/async_runner.go` | `opts.IntervalSec` | Clamped to `opts.IntervalSec = max(opts.IntervalSec, 30)` | Asynchronous non-blocking worker |
| **Cluster Distribution Loop** | `cli/cluster/distribution.go` | `interval time.Duration` | Clamped to `interval.Truncate(time.Second).Seconds() >= 30` | Runs only on topology change |
| **SSH Fleet Pull Heartbeat** | `cli/cmdssh/ssh_pull_fleet.go` | 5s ticker | Only emits user notification when `elapsed >= 30*time.Second` | Suppressed in JSON mode; single background heartbeat |
| **Cluster Fan-Out (`sc`, `pool`)** | `cli/cluster/pool.go` | Event-driven / On-demand | Bounded worker pool (`maxWorkers = 10`) | Context cancellation on `Ctrl+C`, mutex-locked spinner UI |
| **Read Memory Prompt Broadcast** | `cli/cmdagy/agy_read_all_projects_broadcast.go` | Interactive / CLI | Requires interactive confirmation (`[y/N]`) or `--yes` | Dry-run preview (`-d`) supported |

---

## 4. Architectural Invariants

1. **Hard Floor of 30 Seconds**: Any loop intended for recurring polling or telemetry broadcast must reject or clamp any parameter lower than 30 seconds (`val >= 30`).
2. **Startup Quiet Period**: Heavy background tasks (telemetry synchronization, database maintenance, email watchers) must sleep for at least 30–60 seconds after application boot, allowing UI rendering and core initialization to finish without CPU spikes.
3. **Sliding Debounce & Deduplication**: Outbound broadcast channels (Email, Telegram, Supabase) must maintain in-memory timestamp caches to drop duplicate events arriving within 30 seconds of an identical payload.
4. **Bounded Worker Pools**: Fleet and cluster operations must restrict concurrency (e.g. `maxWorkers = 10`) rather than spawning unbounded goroutines or OS threads.
