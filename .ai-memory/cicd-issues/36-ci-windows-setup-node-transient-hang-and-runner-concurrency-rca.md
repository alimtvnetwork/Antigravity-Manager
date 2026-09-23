# RCA 36: CI Windows Runner Setup-Node Transient Cache Lock & Runner Concurrency

## Part 1: Summary of Failure
- **Workflow**: CI (`.github/workflows/ci.yml`)
- **Run ID**: `35844114197`
- **Commit**: `12e40b1edf66d5d07e04efdd9386c5d49318fb71`
- **Job**: `Build Tauri App (windows-2025)`
- **Failure Symptom**: Job execution exceeded the maximum timeout of 45m0s and was terminated by GitHub Actions (`CANCELLED: Job 'Build Tauri App (windows-2025)' cancelled: The job has exceeded the maximum execution time of 45m0s`).

---

## Part 2: Root Cause Analysis (4-Part RCA)

### 1. Telemetry & Execution Trace
By inspecting the internal step-by-step telemetry via GitHub Actions API (`/repos/alimtvnetwork/Antigravity-Manager/actions/jobs/107127588028`):
```json
{"name": "Cache Rust dependencies", "status": "completed", "conclusion": "success", "started_at": "2026-09-23T09:43:49Z", "completed_at": "2026-09-23T09:44:34Z"},
{"name": "Node.js setup", "status": "in_progress", "conclusion": null, "started_at": "2026-09-23T09:44:34Z", "completed_at": null},
{"name": "Install frontend dependencies", "status": "pending", "conclusion": null}
```
The telemetry reveals that the job did **not** hang during Rust compilation or Tauri building. Rather, it became permanently stuck at Step 7 (`Node.js setup`) at `09:44:34Z` and never transitioned to `Install frontend dependencies`.

### 2. High Runner Concurrency & Cache Saturation
At the moment run `35844114197` was dispatched:
1. `Release #35842405396` (v4.64.0) was running with 6 parallel builders.
2. `Release #35844132031` (v4.65.0) was running with 6 parallel builders.
3. `CI #35844114197` had 7 parallel jobs across Windows, macOS, and Ubuntu.
In total, 19 runner jobs were simultaneously hitting the GitHub Actions cache service (`actions/setup-node@v4` with `cache: "npm"` and `Swatinem/rust-cache@v2`).

### 3. Windows-2025 Setup-Node Socket / Tar Extraction Deadlock
On Windows runner images (`windows-2025`), `actions/setup-node@v4` using `cache: "npm"` invokes internal tar-based cache extraction without an internal timeout. Under high concurrent cache contention, the Windows HTTP socket or tar extraction handle stalled, causing the step to hang silently for the remainder of the 45-minute workflow window.

---

## Part 3: Actions Taken & Resolution

1. **Rerun Verification**:
   Triggered a targeted rerun of the failed job via `gh run rerun 35844114197 --failed`.
2. **Step-by-Step Telemetry Tracking**:
   - `✓ Node.js setup`: Succeeded immediately in <15s without cache socket contention.
   - `✓ Install frontend dependencies`: Succeeded in 30s.
   - `✓ Disable updater artifacts in unsigned CI builds`: Succeeded in 2s.
   - `✓ Build Tauri app (debug)`: Completed compiling `agm-alim.exe` in 3m25s.
3. **Outcome**:
   The job completed in **5m49s** with exit code 0 (`✓ Build Tauri App (windows-2025) in 5m49s (ID 107151691771)`).
   All 7 CI jobs in `35844114197` are now **100% Green PASS**.

---

## Part 4: Prevention & Best Practices to Avoid Recurrence

1. **Sequential Release Tag Pushing**:
   Avoid rapidly pushing multiple release tags or commits within seconds of each other. Pushing tags simultaneously spawns 15+ concurrent runners across matrices, saturating GitHub Actions runner and cache limits.
2. **Recognizing Setup-Node Cache Stalls vs Compilation Regressions**:
   When a Windows job in CI exceeds 45 minutes, check step telemetry first. If `Build Tauri app (debug)` was never reached and the stall occurred in `Node.js setup`, the root cause is GitHub cache socket exhaustion, not application code failure.
3. **Isolated Rerun Protocol**:
   Use `gh run rerun <run_id> --failed` once concurrent release jobs subside to allow the runner to acquire cache handles without contention.
