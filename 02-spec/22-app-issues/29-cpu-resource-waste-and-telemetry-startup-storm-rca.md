# 4-Part RCA 29: High CPU Consumption, GUI "Not Responding" Freeze, Runaway Child Processes & Telemetry Polling Storm

## Part 1: Symptoms and Blast Radius
1. **GUI Window Freeze & "Not Responding" State**:
   - As documented in runtime telemetry and user telemetry captures (`media_1790866964677.png`), the main `Antigravity Manager Tools` (agm-alim.exe) native Rust Tauri process consumed **30.6% total CPU** (100% of an entire CPU core on multi-core systems) and was marked by Windows as **"Not responding"**.
   - The Windows message loop was blocked/starved, causing UI unresponsiveness and stuttering during application launch and instance evaluation.
2. **Runaway Child Process Spawning (`agy.exe`)**:
   - In user telemetry captures (`media_1790867114753.png`), the application accumulated **30 child processes** (`Antigravity Manager Tools (30)`), including over **10 simultaneous `agy.exe` instances**, consuming **1,884.4 MB (~1.9 GB) of RAM** and compounding CPU starvation.
3. **Missing Initial Grace Period (Startup Resource Storm)**:
   - Within the first 0–5 seconds of startup, multiple background daemons (auto-profile switcher, email telemetry watcher, Telegram inbound listener, weekly reset scheduler, and frontend auto-refresh/auto-sync hooks) all fired simultaneously, contending for disk I/O and CPU cycles while the primary WebView2 window was attempting to initialize and render.
4. **Task Manager Process Version Obscurity**:
   - In Windows Task Manager, the process group and main window displayed solely the generic name `Antigravity Manager Tools` without any version identifier (`vX.Y.Z`), complicating multi-version diagnostics and live operational tracking.

---

## Part 2: Proximate Cause
1. **Unbounded Process Table Enumeration via `sysinfo::ProcessRefreshKind::everything()`**:
   - In `src-tauri/src/modules/instance.rs` (`find_pids_for_data_dir`), every call instantiated a new `sysinfo::System` and invoked `ProcessRefreshKind::everything()`. This caused Windows Win32 APIs (`EnumProcesses`, `OpenProcess`, `GetProcessTimes`, `GetEnvironmentStrings`) to execute for every single OS process on every check, without caching.
   - When evaluating 4–5 instances, this expensive enumeration executed repeatedly in tight succession.
2. **Missing Sleep & Full Process Refresh in Instance Close / Verification Loop**:
   - In `close_instance()`, process drain verification refreshed the entire process list (`ProcessesToUpdate::All`) repeatedly.
3. **Zero Startup Delay Across Background Services**:
   - `auto_switcher.rs` only waited 3 seconds before executing startup evaluation and rotation.
   - `email_watcher.rs` initialized `last_inbox` and `last_telemetry` timestamps to `0`, making `now - 0 >= interval` immediately true on the first 5-second tick.
   - `scheduler.rs` used `tokio::time::interval`, whose first tick completes at 0ms.
   - Frontend `BackgroundTaskRunner.tsx` triggered `refreshAllQuotas()` and `syncAccountFromDb()` immediately upon component mount.
4. **Missing Worker Concurrency Limit in `spawn_prompt_via_agy()`**:
   - In `src-tauri/src/modules/repo_db.rs`, `dispatch_running_prompts()` iterated through backed-up prompts and spawned `agy.exe` processes without verifying whether an active `agy` worker was already executing in that workspace, resulting in concurrent multi-process accumulation.

---

## Part 3: Root Cause Analysis
1. **Absence of Unified Startup Warmup Policy**:
   - The architecture lacked a mandatory 60-second (1-minute) quiet period. Background telemetry, network sync, and scheduled maintenance must remain dormant during the initial minute after launch, allowing the user interface to render and stabilize with 0% idle CPU overhead.
2. **Point-to-Point Polling vs. Broadcast Pattern Divergence**:
   - Several services defaulted to rapid 10s–30s polling intervals. Modern enterprise gateways require a broadcast architecture: the native backend evaluates state on relaxed 5-minute (300s) intervals and broadcasts push events (`account://auto-switched`, `accounts://refreshed`), while frontend consumers listen passively rather than polling.
3. **Heavy Synchronous Operations on Setup Threads**:
   - Startup update notification dispatch performed synchronous email delivery, risking socket hang and blocking the Tauri setup sequence.
4. **Static Window Title Configuration**:
   - `tauri.conf.json` and Tauri window initialization lacked dynamic version string injection into the OS top-level window title.

---

## Part 4: Preventive and Remediating Actions
1. **Enforce Mandatory 60-Second Startup Quiet Period**:
   - In `src-tauri/src/modules/auto_switcher.rs`: Deferred startup rotation check by 60 seconds.
   - In `src-tauri/src/modules/email_watcher.rs`: Initialized last run timestamps to `now + 60` and enforced a 60-second initial sleep.
   - In `src-tauri/src/modules/telegram_inbound.rs`: Added 60-second startup delay and 60-second idle poll when disabled.
   - In `src-tauri/src/modules/scheduler.rs`: Added 60-second delay prior to entering the interval loop.
   - In `src/components/common/BackgroundTaskRunner.tsx`: Defer all auto-refresh, auto-sync, and smart fast-forward checks until at least 60 seconds after launch.
2. **5-Minute Telemetry Alignment & Broadcast Pattern**:
   - Default all routine telemetry, Supabase sync, and health checks to 300 seconds (5 minutes).
   - Leverage Tauri desktop event broadcasting (`emit`) for UI synchronization with zero unneeded polling.
3. **Optimized Targeted Process Inspection & 4-Second Cache**:
   - In `find_pids_for_data_dir`: Replaced `ProcessRefreshKind::everything()` with minimal `.with_cmd(OnlyIfNotSet).with_exe(OnlyIfNotSet)`.
   - Introduced a 4-second thread-safe process snapshot cache so multi-instance batch evaluation reuses a single scan.
   - In `is_instance_running`: Switched saved PID verification to targeted `ProcessesToUpdate::Some(&[pid])`.
   - In `close_instance`: Added targeted PID refresh and guaranteed 100ms yield between checks.
4. **Active Worker Concurrency Singleton in `repo_db.rs`**:
   - Added `ACTIVE_AGY_WORKERS` tracking in `spawn_prompt_via_agy()` to intercept and prevent duplicate `agy.exe` process spawning for active workspaces.
5. **Windows Task Manager Version Identification**:
   - In `src-tauri/src/lib.rs`: Injected dynamic window title formatting (`Antigravity Manager Tools v{version}`) and Win32 `SetWindowTextW` call so Windows Task Manager immediately reflects the exact version number in process trees and application lists.
   - Synchronized `tauri.conf.json` title in `scripts/bump-version.mjs`.
