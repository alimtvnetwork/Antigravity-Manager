# Root Cause Analysis 46: macOS Hosted Runner Capacity Constraints & Queue Acquisition Cancellation

## Incident Overview
- **Run ID:** `39287955258` (CI), `39286465530` (Release)
- **Commit:** `27706b2` (`main`), `v4.238.4`
- **Workflow:** CI / Release
- **Status:** Cancelled / Failed after queue acquisition timeout

---

## 4-Part Root Cause Analysis (RCA)

### Part 1: Symptom
In GitHub Actions CI run `39287955258`, all other jobs completed successfully:
- `Build Frontend`: Pass (1m 31s)
- `Check Rust Code (windows-2022)`: Pass (5m 16s)
- `Check Rust Code (ubuntu-latest)`: Pass (2m 49s)
- `Check Rust Code (macos-latest)`: Pass (6m 30s)
- `Build Tauri App (windows-2022)`: Pass (5m 1s)
- `Build Tauri App (ubuntu-latest)`: Pass (3m 39s)

However, `Build Tauri App (macos-latest)` failed after 21m 30s in the queue with:
```text
CANCELED: Job 'Build Tauri App (macos-latest)' cancelled: The job was not acquired by Runner of type hosted even after multiple attempts
Annotation: Due to capacity constraints, jobs targeting macOS arm64 runners may experience longer queue times.
```
Similarly, in the concurrent `v4.238.4 Release` workflow (`39286465530`), two concurrent `macos-latest` matrix legs timed out with the same runner acquisition failure.

### Part 2: Root Cause
GitHub Actions recently transitioned `macos-latest` to Apple Silicon (`macos-14` / arm64 architecture). Free and standard tier GitHub accounts have strict concurrency limits (typically 1–2 concurrent macOS runners across all active workflow runs).
When the `Release` workflow triggered 3 parallel `macos-latest` jobs while the `CI` workflow simultaneously triggered 2 parallel `macos-latest` jobs (`check-rust` and `build-tauri`), all 5 jobs saturated GitHub's macOS hosted runner pool. The jobs that could not acquire an idle runner within GitHub's hosted queue timeout limit were cancelled by the infrastructure allocator with `The job was not acquired by Runner of type hosted even after multiple attempts`.

There was no compilation error, code syntax error, or test failure in the codebase.

### Part 3: Surgical Fix & Remediation Strategy
1. **Immediate Action:** Rerun failed job using GitHub CLI (`gh run rerun 39287955258 --failed`) once the initial burst of concurrent release jobs cleared.
2. **Infrastructure Resilience:**
   - In `.github/workflows/ci.yml`, `build-tauri` can add `needs: [check-rust, build-frontend]` or stagger macOS runner demand so that `check-rust (macos-latest)` and `build-tauri (macos-latest)` do not fight for the same single hosted macOS runner slot simultaneously.
3. **Continuous Monitoring:** Utilize `gitmap pe -t` and `gitmap pipeline-ai status` to dynamically track pipeline ETA and automatically detect capacity exhaustion.

### Part 4: Verification
- Command: `gh run rerun 39287955258 --failed`
- Live Observation: `gh run view 39287955258` transitioned `Build Tauri App (macos-latest)` into active runner execution.
- Verified all code checks locally: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, `npm run build`, and targeted test suites (`repo_db`, `telegram_inbound`, `workspace_lease_manager`) all pass with 0 errors.
