# Specification: 30 Improvements Planning and Execution (Improvements 301–330)

- **Sequence**: 167
- **Feature Directory**: `02-spec/21-app/167-improvements-planning-and-execution/`
- **Plan File**: `.ai-memory/plans/pending/167-improvements-planning-and-execution.md`
- **Subtasks**: `.ai-memory/plans/subtasks/167-improvements-planning-and-execution/`
- **Status**: [APPROVED]
- **Operational Discipline**: Zero testing and zero build checking during implementation. Strict sequential batch execution. Exclusively relative paths.

---

## 1. Executive Summary

This specification outlines the next 30 grounded improvements (Improvements 301 to 330) across five core AGM subsystems:
1. **Proxy Streaming Timeouts, SSE Framing & Payload Guards** (Imp 301–306)
2. **Auto-Switcher Telemetry, Affinity Keys & Throttle Recovery** (Imp 307–312)
3. **Split-SQLite Vacuum Schedulers, Index Health & Transaction Guards** (Imp 313–318)
4. **Multi-Instance Sandboxing, Telemetry Isolation & Environment Pruning** (Imp 319–324)
5. **Desktop React UI & UX Component Modernization** (Imp 325–330)

---

## 2. Detailed Improvement Breakdown

### Batch 1: Proxy Streaming Timeouts, SSE Framing & Payload Guards (Imp 301–306)
- **Imp-301: Request Streaming Timeout Sentinel** (`src-tauri/src/proxy/upstream/streaming_timeout_sentinel.rs`)
  - Detects upstream streaming stalls exceeding inactivity timeouts (e.g., 30s silence between chunks) and terminates dead connections cleanly with graceful cancellation.
- **Imp-302: SSE Event Delimiter Normalizer** (`src-tauri/src/proxy/handlers/common/sse_delimiter_normalizer.rs`)
  - Ensures proper Server-Sent Events double-newline delimiters (`\n\n` vs `\r\n\r\n`) across fragmented packet boundaries.
- **Imp-303: Claude Stream Chunk Aggregator** (`src-tauri/src/proxy/handlers/claude/stream_chunk_aggregator.rs`)
  - Buffers and reassembles partial Claude JSON event fragments into complete `content_block_delta` structures.
- **Imp-304: Gemini Part Envelope Validator** (`src-tauri/src/proxy/handlers/gemini/part_envelope_validator.rs`)
  - Validates Gemini canonical response parts (`candidates[0].content.parts`) to eliminate malformed text/thought wrapper blocks.
- **Imp-305: Upstream Connection Keepalive Configurator** (`src-tauri/src/proxy/upstream/connection_keepalive_config.rs`)
  - Configures TCP keepalive intervals, connection pool idle timeouts (90s), and max idle connections per host.
- **Imp-306: Client Request Body Size Guard** (`src-tauri/src/proxy/handlers/common/request_size_guard.rs`)
  - Inspects incoming payload lengths against per-model size ceilings (e.g. 50MB for multimodal, 10MB for text), returning early 413 Payload Too Large.

### Batch 2: Auto-Switcher Telemetry, Affinity Keys & Throttle Recovery (Imp 307–312)
- **Imp-307: Auto-Switch Event History Pruner** (`src-tauri/src/modules/auto_switcher/event_history_pruner.rs`)
  - Maintains bounded in-memory and SQLite auto-switch history records (capped at 500 events) to prevent memory growth.
- **Imp-308: Multi-Instance Affinity Sticky Key** (`src-tauri/src/modules/auto_switcher/instance_affinity_key.rs`)
  - Generates sticky hashing keys based on client instance ID and project directory to reduce excessive account hopping.
- **Imp-309: Account Throttle Recovery Observer** (`src-tauri/src/modules/auto_switcher/throttle_recovery_observer.rs`)
  - Tracks transient 429 throttle backoffs with exponential backoff half-life decay, gradually restoring candidate scores.
- **Imp-310: Switch Decision Telemetry Logger** (`src-tauri/src/modules/auto_switcher/decision_telemetry_logger.rs`)
  - Emits structured switch telemetry logs with candidate matrix snapshots, winning score differentials, and trigger events.
- **Imp-311: Dynamic Headroom Score Multiplier** (`src-tauri/src/modules/auto_switcher/headroom_score_multiplier.rs`)
  - Calculates non-linear headroom weighting curves favoring accounts with >50% remaining quota over marginally available accounts.
- **Imp-312: Fast-Forward Target Account Resolver** (`src-tauri/src/modules/auto_switcher/fast_forward_target_resolver.rs`)
  - Resolves optimal target account candidates for manual Fast-Forward requests while skipping accounts currently bound to running IDEs.

### Batch 3: Split-SQLite Vacuum Schedulers, Index Health & Transaction Guards (Imp 313–318)
- **Imp-313: Incremental Auto-Vacuum Scheduler** (`src-tauri/src/modules/db_orm/vacuum_scheduler.rs`)
  - Runs periodic `PRAGMA incremental_vacuum(N);` during application idle states to reclaim freed database pages without locking the main thread.
- **Imp-314: Database Index Health Checker** (`src-tauri/src/modules/db_orm/index_health_checker.rs`)
  - Inspects SQLite index fragmentation and validates `EXPLAIN QUERY PLAN` efficiency across hot prompt search paths.
- **Imp-315: Concurrency Deadlock Detector** (`src-tauri/src/modules/db_orm/deadlock_detector.rs`)
  - Monitors SQLite busy handler retry cascades and detects potential lock escalation timeouts between concurrent workers.
- **Imp-316: Split Database Migration Version Stamp** (`src-tauri/src/modules/db_orm/migration_version_stamp.rs`)
  - Writes and validates schema version stamps (`PRAGMA user_version;`) across all split databases during startup migrations.
- **Imp-317: SQLite Page Cache Memory Manager** (`src-tauri/src/modules/db_orm/page_cache_manager.rs`)
  - Dynamically computes negative `PRAGMA cache_size` (Kibibytes) based on total system RAM to optimize query caching without memory pressure.
- **Imp-318: Prompt Audit Table Auto-Archive Sentinel** (`src-tauri/src/modules/db_orm/audit_table_archiver.rs`)
  - Identifies completed prompt records older than the retention threshold and stages them for atomic archival into `task_history.db`.

### Batch 4: Multi-Instance Sandboxing, Telemetry Isolation & Environment Pruning (Imp 319–324)
- **Imp-319: Environment Variable Whitelist Filter** (`src-tauri/src/modules/instance/env_whitelist_filter.rs`)
  - Strips parent process leaking environment variables (`VSCODE_*`, `ELECTRON_RUN_AS_NODE`, `DEBUG`) before spawning sandboxed IDE child instances.
- **Imp-320: Profile Sandboxed Directory Validator** (`src-tauri/src/modules/instance/profile_dir_validator.rs`)
  - Verifies that instance user data directories exist, have correct write permissions, and reside within the allowed profile storage tree.
- **Imp-321: VS Code IPC Socket Cleaner** (`src-tauri/src/modules/instance/ipc_socket_cleaner.rs`)
  - Sweeps stale UNIX domain sockets and Windows named pipes (`vscode-ipc-*`) from previous crashed instance runs.
- **Imp-322: Conscious Process Terminate Fallback Ladder** (`src-tauri/src/modules/instance/terminate_fallback_ladder.rs`)
  - Implements a 3-tier conscious termination ladder (`SIGINT/WM_CLOSE -> SIGTERM/PostMessage -> SIGKILL/TerminateProcess`) with timed settling grace periods.
- **Imp-323: Cloned Profile MachineGuid Reseeder** (`src-tauri/src/modules/instance/cloned_guid_reseeder.rs`)
  - Ensures freshly cloned instance profiles receive unique synthetic MachineGuids and device telemetry fingerprints.
- **Imp-324: Instance State Health Snapshot Exporter** (`src-tauri/src/modules/instance/state_snapshot_exporter.rs`)
  - Exports a sanitized JSON snapshot of all active instance states, allocated ports, and bound accounts for external diagnostics.

### Batch 5: Desktop React UI & UX Component Modernization (Imp 325–330)
- **Imp-325: Segmented Capsule Radio Group** (`src/components/common/SegmentedCapsuleRadio.tsx`)
  - Contiguous segmented pill radio toggle (`rounded-full`, shared border, dark-glass styling) adhering strictly to AGENTS.md UI standards.
- **Imp-326: Status Ping Pulse Badge** (`src/components/common/StatusPingBadge.tsx`)
  - Compact status indicator pill with an animated breathing ping dot (emerald for running, amber for paused, zinc for idle, rose for error).
- **Imp-327: High-Contrast Dark Glass Popover Primitive** (`src/components/common/DarkGlassPopover.tsx`)
  - Accessible floating popover container with backdrop blur, high-contrast borders, click-outside dismissal, and keyboard Escape listener.
- **Imp-328: Debounced Value React Hook** (`src/hooks/useDebouncedValue.ts`)
  - High-performance debounced value hook with immediate cancellation and flush capabilities for fast search inputs and filter controls.
- **Imp-329: Expandable Collapsible Card Section** (`src/components/common/ExpandableSection.tsx`)
  - Smooth animated accordion card section with chevron rotation, header actions, and persistent expand/collapse state memory.
- **Imp-330: Copy to Clipboard Icon Action Button** (`src/components/common/CopyActionButton.tsx`)
  - Self-contained copy button with animated icon transition (`Copy -> Check`), visual feedback tooltip, and error handling.

---

## 3. Operational Invariants
1. **Zero Testing & Build Checking**: Strictly prohibited during implementation.
2. **Exclusively GitMap Search**: Ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
3. **Strictly Relative Paths**: No absolute filesystem paths in specs, plans, code comments, or commits.
4. **No Silent Errors**: All fallible operations route to `AppError` or `useErrorStore`.
