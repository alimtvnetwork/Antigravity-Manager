# Specification: 30 Improvements Planning and Execution (Improvements 331–360)

- **Sequence**: 168
- **Feature Directory**: `02-spec/21-app/168-improvements-planning-and-execution/`
- **Plan File**: `.ai-memory/plans/pending/168-improvements-planning-and-execution.md`
- **Subtasks**: `.ai-memory/plans/subtasks/168-improvements-planning-and-execution/`
- **Status**: [APPROVED]
- **Operational Discipline**: Zero testing and zero build checking during implementation. Strict sequential batch execution. Exclusively relative paths.

---

## 1. Executive Summary

This specification outlines the next 30 grounded improvements (Improvements 331 to 360) across five core AGM subsystems:
1. **Proxy Circuit Breakers & Bounding** (Imp 331–336)
2. **Auto-Switcher Jitter & Dampeners** (Imp 337–342)
3. **Split-SQLite Recovery & Diagnostics** (Imp 343–348)
4. **Multi-Instance Ports & Crash Loop** (Imp 349–354)
5. **Desktop React UI & UX Component Modernization** (Imp 355–360)

---

## 2. Detailed Improvement Breakdown

### Batch 1: Proxy Circuit Breakers & Bounding (Imp 331–336)
- **Imp-331: Upstream Circuit Breaker Trip Counter** (`src-tauri/src/proxy/upstream/circuit_breaker_counter.rs`)
  - Tracks consecutive upstream HTTP 5xx errors and connection dropouts, tripping the circuit when threshold (5 failures) is reached within 60s.
- **Imp-332: Adaptive Retry Delay Decelerator** (`src-tauri/src/proxy/upstream/retry_decelerator.rs`)
  - Scales retry delays dynamically based on endpoint error rates and previous trip frequency to prevent slamming failing upstreams.
- **Imp-333: Protocol Role Normalization Validator** (`src-tauri/src/proxy/handlers/common/role_normalization_validator.rs`)
  - Validates role mappings (`system`, `user`, `assistant`, `tool`, `model`) before converting between Gemini Canonical IR, OpenAI, and Claude formats.
- **Imp-334: Stream Frame Length Bounding Clamper** (`src-tauri/src/proxy/handlers/common/frame_length_clamper.rs`)
  - Clamps oversized SSE chunk frames exceeding maximum client buffer limits (1MB per event), splitting them into sequential chunks.
- **Imp-335: OpenAI Tool Call Delta Assembler** (`src-tauri/src/proxy/handlers/openai/tool_call_assembler.rs`)
  - Aggregates streaming `tool_calls` delta fragments (`index`, `id`, `name`, `arguments`) into unified JSON call structures.
- **Imp-336: Proxy Route Latency Weight Calculator** (`src-tauri/src/proxy/upstream/route_latency_calculator.rs`)
  - Computes weighted routing scores based on exponential moving average (EMA) latencies of upstream endpoints.

### Batch 2: Auto-Switcher Jitter & Dampeners (Imp 337–342)
- **Imp-337: Jittered Quota Velocity Estimator** (`src-tauri/src/modules/auto_switcher/jittered_velocity_estimator.rs`)
  - Injects micro-jitter into consumption velocity calculations to prevent simultaneous multi-instance synchronized rotation spikes.
- **Imp-338: Rapid Account Bounce Dampener** (`src-tauri/src/modules/auto_switcher/bounce_dampener.rs`)
  - Suppresses rapid back-and-forth switching between two high-scoring candidate accounts within short time windows (<60s).
- **Imp-339: Candidate Quota Window Align Checker** (`src-tauri/src/modules/auto_switcher/quota_window_align.rs`)
  - Evaluates remaining time to 5-hour rolling quota window reset, prioritizing accounts whose reset occurs sooner.
- **Imp-340: Profile Switch Settling Watchdog** (`src-tauri/src/modules/auto_switcher/switch_settling_watchdog.rs`)
  - Enforces and monitors the mandatory 10-second settling period after IDE account switching before injecting queued prompts.
- **Imp-341: Multi-Model Headroom Composite Scorer** (`src-tauri/src/modules/auto_switcher/multimodel_composite_scorer.rs`)
  - Computes combined multi-model headroom scores across Gemini 2.0 Flash, Claude 3.5 Sonnet, and GPT-4o for versatile account ranking.
- **Imp-342: Stale Binding Sweeper Sentinel** (`src-tauri/src/modules/auto_switcher/stale_binding_sweeper.rs`)
  - Periodically audits active instance-account bindings, releasing locks on accounts whose parent process has closed or disappeared.

### Batch 3: Split-SQLite Recovery & Diagnostics (Imp 343–348)
- **Imp-343: WAL Log Recovery Checkpoint Sentinel** (`src-tauri/src/modules/db_orm/wal_recovery_sentinel.rs`)
  - Performs emergency passive and truncate checkpoints on split databases during startup to recover interrupted WAL writes.
- **Imp-344: Query Execution Profile Exporter** (`src-tauri/src/modules/db_orm/query_profile_exporter.rs`)
  - Collects slow query execution profiles (>100ms) with parameterized SQL and duration metrics for diagnostic auditing.
- **Imp-345: Schema Column Nullability Validator** (`src-tauri/src/modules/db_orm/column_nullability_validator.rs`)
  - Inspects `PRAGMA table_info;` to verify NOT NULL constraints on critical foreign key and timestamp columns.
- **Imp-346: Split Database Health Probe Aggregator** (`src-tauri/src/modules/db_orm/health_probe_aggregator.rs`)
  - Probes all 4 split databases (`repo_prompts.db`, `errors.db`, `task_history.db`, `prompt_backups.db`) and returns aggregated health status.
- **Imp-347: Transaction Rollback Telemetry Reporter** (`src-tauri/src/modules/db_orm/rollback_telemetry.rs`)
  - Hooks transaction rollback events, logging database name, error reason, and stack callsite into `errors.db`.
- **Imp-348: Database Backup Snapshot Verifier** (`src-tauri/src/modules/db_orm/backup_snapshot_verifier.rs`)
  - Verifies SQLite header magic bytes (`SQLite format 3\0`) and page integrity checksums before committing database backups.

### Batch 4: Multi-Instance Ports & Crash Loop (Imp 349–354)
- **Imp-349: Sandboxed Keychain File Guard** (`src-tauri/src/modules/instance/keychain_file_guard.rs`)
  - Guarantees `antigravity-keyring-unavailable` marker files exist and have non-zero size before every instance process launch.
- **Imp-350: Port Collision Immediate Evictor** (`src-tauri/src/modules/instance/port_collision_evictor.rs`)
  - Detects if an allocated debugging port is already occupied by a rogue non-instance process and assigns an alternate port immediately.
- **Imp-351: Crash Loop Exponential Backoff Guard** (`src-tauri/src/modules/instance/crash_loop_backoff.rs`)
  - Tracks rapid successive process crashes (<10s lifespan), applying escalating delays (2s, 4s, 8s, 16s) before permitting relaunch.
- **Imp-352: Cloned Instance Storage Checksum Guard** (`src-tauri/src/modules/instance/storage_checksum_guard.rs`)
  - Verifies that cloned `storage.json` files parse into valid JSON and contain non-colliding machine UUIDs before launch.
- **Imp-353: Child Process Health Heartbeat Monitor** (`src-tauri/src/modules/instance/child_health_monitor.rs`)
  - Periodically verifies that child IDE process trees retain active main thread message pump responses without freezing.
- **Imp-354: Instance Workspace Hash Collision Sentinel** (`src-tauri/src/modules/instance/workspace_hash_sentinel.rs`)
  - Checks for workspace folder hash collisions across active instance registrations, ensuring each instance manages unique project paths.

### Batch 5: Desktop React UI & UX Component Modernization (Imp 355–360)
- **Imp-355: Compact Pill Dropdown Select Primitive** (`src/components/common/CompactPillSelect.tsx`)
  - Inline borderless `<select>` capsule with dark-glass styling (`bg-transparent border-0 focus:ring-0`) eliminating nested white pill visual bugs.
- **Imp-356: Animated Number Counter Display** (`src/components/common/AnimatedCounter.tsx`)
  - Smooth easing counter animation component for token totals, request counts, and score percentage metrics.
- **Imp-357: Micro Notification Toast Component** (`src/components/common/MicroToast.tsx`)
  - Compact, non-intrusive notification toast supporting success, warning, error, and info states with auto-dismiss timers.
- **Imp-358: Resizable Split Pane Divider Hook** (`src/hooks/useResizableDivider.ts`)
  - High-performance drag-resize hook with min/max bounds, mouse/touch listeners, and localStorage persistence for tree panels.
- **Imp-359: Keyboard Shortcut Legend Modal Card** (`src/components/common/ShortcutLegendCard.tsx`)
  - Dark-glass modal card rendering active keyboard shortcuts (`Cmd+K`, `Esc`, `Tab`, `N`) with kbd badges and descriptions.
- **Imp-360: Status Indicator Color Token Helper** (`src/utils/statusColorTokens.ts`)
  - Centralized color token helper mapping statuses and origin tiers to Tailwind dark-glass badge classes and SVG icons.

---

## 3. Operational Invariants
1. **Zero Testing & Build Checking**: Strictly prohibited during implementation.
2. **Exclusively GitMap Search**: Ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
3. **Strictly Relative Paths**: No absolute filesystem paths in specs, plans, code comments, or commits.
4. **No Silent Errors**: All fallible operations route to `AppError` or `useErrorStore`.
