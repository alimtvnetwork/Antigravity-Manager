# Specification: 30 Improvements Planning and Execution (Improvements 391–420)

- **Sequence**: 170
- **Feature Directory**: `02-spec/21-app/170-improvements-planning-and-execution/`
- **Plan File**: `.ai-memory/plans/pending/170-improvements-planning-and-execution.md`
- **Subtasks**: `.ai-memory/plans/subtasks/170-improvements-planning-and-execution/`
- **Status**: [APPROVED]
- **Operational Discipline**: Zero testing and zero build checking during implementation. Strict sequential batch execution. Exclusively relative paths.

---

## 1. Executive Summary

This specification defines the 170th sequence of 30 grounded improvements (Improvements 391 to 420) spanning five core Antigravity Manager subsystems:
1. **Proxy SSE Framing, Abort Sentinel & Payload Sanitization** (Imp 391–396)
2. **Auto-Switcher Cooldown, Quota & Rotation Guards** (Imp 397–402)
3. **Split-SQLite Maintenance, WAL Recovery & Forensics** (Imp 403–408)
4. **Multi-Instance Sandboxing, Sockets & Environment** (Imp 409–414)
5. **Desktop React UI & UX Component Modernization** (Imp 415–420)

All tasks adhere to the strict house rules: no silent errors, protocol-agnostic pipeline primacy, strictly relative repository paths, and zero testing/build checks during active execution.

---

## 2. Detailed Improvement Breakdown

### Batch 1: Proxy SSE Framing, Abort Sentinel & Payload Sanitization (Imp 391–396)
- **Imp-391: Upstream Connection Idle Reaper** (`src-tauri/src/proxy/upstream/idle_connection_reaper.rs`)
  - Identifies and terminates zombie/idle upstream HTTP/2 and TCP connections that exceeded the idle timeout (60s) to prevent socket leaks.
- **Imp-392: SSE Boundary Delimiter Normalizer** (`src-tauri/src/proxy/handlers/common/sse_boundary_normalizer.rs`)
  - Standardizes irregular SSE delimiters (`\r\n\r\n`, trailing solitary `\n`) into RFC-compliant double newline `\n\n` event boundaries.
- **Imp-393: Stream Early Abort Sentinel** (`src-tauri/src/proxy/handlers/common/stream_early_abort_sentinel.rs`)
  - Detects client-side TCP reset (RST) and disconnect events mid-stream, immediately propagating cancellation upstream to cease token consumption.
- **Imp-394: Request Header Sanitization Filter** (`src-tauri/src/proxy/handlers/common/request_header_sanitizer.rs`)
  - Strips hop-by-hop, internal host, and sensitive credential headers (`x-forwarded-for`, `cookie`, `proxy-authorization`) before routing upstream.
- **Imp-395: Claude Tool Use Stream Frame Assembler** (`src-tauri/src/proxy/handlers/claude/tool_use_stream_assembler.rs`)
  - Reconstructs fragmented `content_block_start` and `content_block_delta` JSON chunks for Claude tool uses into complete function arguments.
- **Imp-396: Upstream Route Latency Exponential Smoothing** (`src-tauri/src/proxy/upstream/latency_exponential_smoother.rs`)
  - Applies exponential moving average (EMA with alpha = 0.2) to smooth jittery route latency measurements for stable upstream gateway routing.

### Batch 2: Auto-Switcher Cooldown, Quota & Rotation Guards (Imp 397–402)
- **Imp-397: Sliding Window Quota Velocity Guard** (`src-tauri/src/modules/auto_switcher/sliding_velocity_guard.rs`)
  - Calculates rolling 5-minute quota consumption velocity with burst dampening to prevent switching to accounts with impending rate limits.
- **Imp-398: Consecutive Failure Cooldown Multiplier** (`src-tauri/src/modules/auto_switcher/consecutive_failure_multiplier.rs`)
  - Applies progressive geometric backoff multipliers (1x, 2x, 4x, 8x) to cooldown timers when an account fails multiple consecutive rotations.
- **Imp-399: Multi-Instance Binding Conflict Detector** (`src-tauri/src/modules/auto_switcher/binding_conflict_detector.rs`)
  - Checks across active instance profile records to detect if two separate IDE instances are concurrently bound to the same target account.
- **Imp-400: Candidate Rotation Cooldown Sweeper** (`src-tauri/src/modules/auto_switcher/candidate_cooldown_sweeper.rs`)
  - Sweeps expired account cooldown records and returns refreshed candidate lists ready for immediate rotation scoring.
- **Imp-401: Account Tier Priority Weight Normalizer** (`src-tauri/src/modules/auto_switcher/tier_priority_normalizer.rs`)
  - Normalizes account tier weights (Ultra, Pro, Free, Sandbox) into calibrated scoring multipliers to preserve priority ordering.
- **Imp-402: Instance Relaunch Readiness Evaluator** (`src-tauri/src/modules/auto_switcher/relaunch_readiness_evaluator.rs`)
  - Evaluates whether an instance that underwent account switching has fully settled (process alive, socket open, storage clean) before dispatching prompts.

### Batch 3: Split-SQLite Maintenance, WAL Recovery & Forensics (Imp 403–408)
- **Imp-403: SQLite Connection Pool Starvation Guard** (`src-tauri/src/modules/db_orm/pool_starvation_guard.rs`)
  - Tracks checkout durations for SQLite connections, warning and recovering connections held longer than the 5-second deadline.
- **Imp-404: Split Database Free Page Stepper** (`src-tauri/src/modules/db_orm/free_page_stepper.rs`)
  - Inspects `PRAGMA freelist_count` and executes stepped incremental vacuums (`PRAGMA incremental_vacuum(N)`) to reclaim disk space gradually.
- **Imp-405: Index Fragmentation Analyzer** (`src-tauri/src/modules/db_orm/index_fragmentation_analyzer.rs`)
  - Runs targeted index density checks on `active_prompts` and `task_history` tables to detect when `REINDEX` is required.
- **Imp-406: Table Column Default Value Validator** (`src-tauri/src/modules/db_orm/column_default_validator.rs`)
  - Inspects `PRAGMA table_info` across all split databases, verifying default values exist on nullable timestamp and status columns.
- **Imp-407: Transaction Commit Latency Observer** (`src-tauri/src/modules/db_orm/commit_latency_observer.rs`)
  - Measures commit durations across split SQLite transactions, alerting when WAL synchronization exceeds 50ms thresholds.
- **Imp-408: Database File Header Checksum Guard** (`src-tauri/src/modules/db_orm/header_checksum_guard.rs`)
  - Validates SQLite 3 database header bytes, page size offsets, and write version counters before establishing new connection pools.

### Batch 4: Multi-Instance Sandboxing, Sockets & Environment (Imp 409–414)
- **Imp-409: Sandboxed Environment Variable Allowlist** (`src-tauri/src/modules/instance/env_allowlist_filter.rs`)
  - Filters environment variables passed to spawned IDE instances, stripping parent proxy tokens and developer secrets.
- **Imp-410: Stale Unix/Named Pipe Socket Cleaner** (`src-tauri/src/modules/instance/pipe_socket_cleaner.rs`)
  - Inspects instance runtimes for orphaned named pipes and domain socket files (`antigravity-*.sock`), removing them safely before launch.
- **Imp-411: Instance State Snapshot Rollback** (`src-tauri/src/modules/instance/state_snapshot_rollback.rs`)
  - Creates pre-switch state snapshots of instance configuration files and enables 1-step rollback if launch or account binding fails.
- **Imp-412: Cloned Profile Telemetry Salt Generator** (`src-tauri/src/modules/instance/cloned_telemetry_salt.rs`)
  - Computes and salts distinct anonymous telemetry hash seeds for duplicated instance workspaces to guarantee clean analytics separation.
- **Imp-413: Child Process Termination Guard** (`src-tauri/src/modules/instance/child_termination_guard.rs`)
  - Provides a 3-stage escalating kill ladder (SIGINT -> SIGTERM -> SIGKILL / TerminateProcess) for stubborn IDE child processes.
- **Imp-414: Instance Workspace Directory Validator** (`src-tauri/src/modules/instance/workspace_dir_validator.rs`)
  - Validates that target workspace directories exist, have write permissions, and contain valid project metadata before launching.

### Batch 5: Desktop React UI & UX Component Modernization (Imp 415–420)
- **Imp-415: Dark Glass Popover Card** (`src/components/common/DarkGlassPopoverCard.tsx`)
  - Floating dark-glass popover card component with portal rendering, backdrop blur, arrow positioning, and automatic boundary clamping.
- **Imp-416: Inline Copy Chip** (`src/components/common/InlineCopyChip.tsx`)
  - Compact code/text pill with click-to-copy, hover tooltip, and checkmark feedback, ideal for IDs, hashes, and port numbers.
- **Imp-417: Click Outside Listener Hook** (`src/hooks/useClickOutsideListener.ts`)
  - Custom hook to dismiss popovers, menus, and flyouts when clicking outside target elements with multi-ref support.
- **Imp-418: Quota Health Status Capsule** (`src/components/common/QuotaHealthStatusCapsule.tsx`)
  - Visual health status pill showing quota percentage with dynamic color gradients (green >50%, amber 15-50%, rose <15%) and pulsing state.
- **Imp-419: Byte Size Formatter Utility** (`src/utils/formatByteSize.ts`)
  - High-precision zero-dependency byte formatting helper (`B`, `KB`, `MB`, `GB`) with decimal alignment and localized output.
- **Imp-420: Expandable Code Block Banner** (`src/components/common/ExpandableCodeBlockBanner.tsx`)
  - Interactive dark-glass code/JSON viewer with collapse/expand toggle, line numbering, and copy-all action.

---

## 3. Implementation Verification Criteria

1. **Strictly Disjoint Modules**: All new source files reside in their designated architectural subdirectories.
2. **Re-Export Facade Integrity**: Submodules are cleanly re-exported in parent facades.
3. **Zero Errors Swallowed**: All fallible calls return contextual `AppError` or use `trackWarning`.
4. **Zero-Test / Zero-Build Execution**: Immediate sequential implementation without test or build pauses.
