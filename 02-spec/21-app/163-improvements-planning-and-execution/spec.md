# Specification: 30 Improvements Planning and Execution (Improvements 181–210)

- **Task**: Improvements Planning and Execution
- **Sequence**: 163
- **Slug**: `163-improvements-planning-and-execution`
- **Spec Directory**: `02-spec/21-app/163-improvements-planning-and-execution/`
- **Plan Path**: `.ai-memory/plans/pending/163-improvements-planning-and-execution.md`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/163-improvements-planning-and-execution/`

---

## 1. Executive Summary

This specification establishes the roadmap, architectural contracts, and step-by-step execution tasks for 30 concrete improvements (Improvements 181 to 210) across Antigravity-Manager. Following strict operational directives, all tasks are implemented sequentially across five targeted batches without intermediate builds or test execution.

---

## 2. Improvements Catalog & Structural Architecture

### Batch 1: Reverse Proxy Token Metering, Rate Limiting & Sliding Windows (Imp 181–186)
- **Imp-181: Sliding Window Token Rate Limiter** (`src-tauri/src/proxy/rate_limit/sliding_window.rs`)
  Calculates per-minute and per-hour request and token volume sliding windows, enforcing adaptive throttling before upstream API saturation.
- **Imp-182: Stream SSE Heartbeat Generator** (`src-tauri/src/proxy/handlers/common/sse_heartbeat.rs`)
  Emits periodic non-disruptive SSE keepalive comments (`: ping - keepalive\n\n`) during extended model thinking cycles to prevent gateway idle timeouts.
- **Imp-183: Token Bucket Rate Limiter** (`src-tauri/src/proxy/rate_limit/token_bucket.rs`)
  Implements a thread-safe token bucket algorithm with configurable refill rate and burst capacity for outbound upstream requests.
- **Imp-184: Model Context Length Overflow Guard** (`src-tauri/src/proxy/handlers/common/context_guard.rs`)
  Validates estimated input token volume against known model context limits (e.g. 1M for Gemini, 200k for Claude), rejecting unprocessable payloads early.
- **Imp-185: Upstream Endpoint Round-Robin Health Balancer** (`src-tauri/src/proxy/upstream/round_robin.rs`)
  Maintains round-robin distribution across healthy upstream endpoints with circuit-breaker penalty backoff on persistent 5xx errors.
- **Imp-186: Client Disconnect Stream Abort Sentinel** (`src-tauri/src/proxy/handlers/common/abort_sentinel.rs`)
  Monitors downstream HTTP client connection drops, immediately terminating the corresponding upstream generative AI stream to save quota.

### Batch 2: Auto-Switcher Cooldown, Quota Velocity & Machine Leases (Imp 187–192)
- **Imp-187: Multi-Node Machine Lease Expiration Sweep** (`src-tauri/src/modules/auto_switcher/lease_sweeper.rs`)
  Sweeps distributed workspace leases and marks expired leases (TTL > 3600s or heartbeat age > 300s) as reclaimable for local auto-rotation.
- **Imp-188: Account Cooldown Exponential Decay** (`src-tauri/src/modules/auto_switcher/cooldown_decay.rs`)
  Computes gradual penalty decay for accounts in temporary cooldown, transitioning them smoothly back to candidate eligibility as cooldown expires.
- **Imp-189: Headroom Velocity Trend Forecaster** (`src-tauri/src/modules/auto_switcher/headroom_trend.rs`)
  Analyzes multi-point headroom trends to detect accelerating quota drain before an account hits critical exhaustion.
- **Imp-190: Candidate Score Rounding & Precision Normalizer** (`src-tauri/src/modules/auto_switcher/score_normalizer.rs`)
  Normalizes composite candidate scores into deterministic 3-decimal fixed precision (`0.000` to `1000.000`) for stable scoreboard rendering.
- **Imp-191: Auto-Switch Minimum Run Duration Guard** (`src-tauri/src/modules/auto_switcher/min_run_guard.rs`)
  Enforces a minimum active run duration (e.g. 60 seconds) before an account can be rotated, preventing rapid thrashing on transient spikes.
- **Imp-192: Candidate Account Sort Key Comparator** (`src-tauri/src/modules/auto_switcher/sort_comparator.rs`)
  Implements deterministic multi-criteria sorting comparator: Score DESC, Quota Percent DESC, Headroom Hours DESC, Email ASC.

### Batch 3: Split-SQLite Foreign Key & Constraint Integrity (Imp 193–198)
- **Imp-193: Split-DB Foreign Key Constraint Validator** (`src-tauri/src/modules/db_orm/foreign_keys.rs`)
  Validates `PRAGMA foreign_key_check` across split databases on demand, returning table-level orphan record diagnostics.
- **Imp-194: Database Busy Timeout Retry Ladder** (`src-tauri/src/modules/db_orm/busy_retry.rs`)
  Wraps contentious SQLite write queries with an exponential backoff retry ladder (10ms, 50ms, 200ms, 500ms) to resolve transient write contention.
- **Imp-195: SQLite Auto-Vacuum Incremental Stepper** (`src-tauri/src/modules/db_orm/vacuum_stepper.rs`)
  Executes bounded `PRAGMA incremental_vacuum(N)` in chunks of 100 pages, reclaiming freelist pages incrementally without locking databases.
- **Imp-196: Table Row Count & Size Profiler** (`src-tauri/src/modules/db_orm/table_profiler.rs`)
  Profiles user table row counts and estimated storage footprint across `errors.db`, `repo_prompts.db`, `prompt-backups.db`, and `task-history.db`.
- **Imp-197: Split-DB Schema Version Drift Detector** (`src-tauri/src/modules/db_orm/version_drift.rs`)
  Detects version mismatch between compiled `TARGET_SCHEMA_VERSION` and `SchemaVersion` records in local SQLite files.
- **Imp-198: Safe Query Read-Only Sandbox Guard** (`src-tauri/src/modules/db_orm/read_sandbox.rs`)
  Enforces read-only AST safety verification preventing schema modifications (`DROP`, `ALTER`, `TRUNCATE`, `INSERT`) in diagnostic explorer queries.

### Batch 4: Multi-Instance Profile State, Locks & Keyrings (Imp 199–204)
- **Imp-199: Profile Stale Lockfile Reclaimer** (`src-tauri/src/modules/instance/lock_reclaimer.rs`)
  Safely detects and removes orphaned `.antigravity-session` and `code.lock` files when the owning process is no longer running.
- **Imp-200: Instance Launch Arg Normalizer** (`src-tauri/src/modules/instance/arg_normalizer.rs`)
  Standardizes required Electron / Chromium launch flags (`--user-data-dir`, `--remote-debugging-port=0`, `--password-store=basic`).
- **Imp-201: Active Project Folder URI Formatter** (`src-tauri/src/modules/instance/project_uri.rs`)
  Normalizes file system paths into canonical `file:///...` URIs for `workspaceStorage/<hash>/workspace.json` cross-platform seeding.
- **Imp-202: Instance Process Vitality Scanner** (`src-tauri/src/modules/instance/vitality_scanner.rs`)
  Performs conscious Win32 / POSIX PID verification, distinguishing live Antigravity IDE host processes from dead PIDs.
- **Imp-203: Telemetry Session UUID Rotator** (`src-tauri/src/modules/instance/session_uuid.rs`)
  Generates fresh random UUIDv4 identifiers for `telemetry.sessionId` and uppercase format for `telemetry.sqmId` during profile initialization.
- **Imp-204: Cloned Profile Recent Paths Sanitizer** (`src-tauri/src/modules/instance/path_sanitizer.rs`)
  Sanitizes `openedPathsList` in `storage.json` to filter out non-existent directories from cloned instance workspaces.

### Batch 5: Desktop React UI & UX Component Modernization (Imp 205–210)
- **Imp-205: Dark-Glass Dropdown Menu Primitive** (`src/components/common/DropdownMenu.tsx`)
  Accessible dark-glass dropdown menu with keyboard navigation, click-outside dismissal, and smooth fade transitions.
- **Imp-206: Monospace Token Counter Pill** (`src/components/common/TokenCounterPill.tsx`)
  Compact badge rendering input/output token counts with color-coded warning thresholds when approaching model limits.
- **Imp-207: Relative Time Elapsed Formatter Utility** (`src/utils/relativeTime.ts`)
  Formats timestamps into compact relative strings (`just now`, `5s ago`, `12m ago`, `3h ago`, `2d ago`) with sub-minute granularity.
- **Imp-208: Accessible Number Input Stepper** (`src/components/common/NumberStepper.tsx`)
  Compact dark-glass number stepper with increment/decrement buttons, min/max clamping, and keyboard arrow support.
- **Imp-209: Responsive Search Input Field** (`src/components/common/SearchInput.tsx`)
  Debounced search input with clear button (`X`), search icon, keyboard shortcut badge (`/`), and dark-glass focus styling.
- **Imp-210: Confirmation Modal Dialog Primitive** (`src/components/common/ConfirmModal.tsx`)
  Standardized dark-glass confirmation dialog with destructive / primary action styling, Escape key dismissal, and backdrop blur.

---

## 3. Non-Negotiable Operational Constraints
1. **Zero Testing & Build Checking**: No `cargo test`, `cargo build`, `npm test`, or `npm run build` during execution.
2. **Search Primacy**: Strict ban on raw `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`. All search performed via GitMap.
3. **Relative Paths**: All references use repository-relative Git paths.
4. **No Silent Errors**: Fallible operations propagate through `AppError` or `useErrorStore`.
