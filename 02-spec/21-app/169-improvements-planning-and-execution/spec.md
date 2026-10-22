# Specification: 30 Improvements Planning and Execution (Improvements 361–390)

- **Sequence**: 169
- **Feature Directory**: `02-spec/21-app/169-improvements-planning-and-execution/`
- **Plan File**: `.ai-memory/plans/pending/169-improvements-planning-and-execution.md`
- **Subtasks**: `.ai-memory/plans/subtasks/169-improvements-planning-and-execution/`
- **Status**: [APPROVED]
- **Operational Discipline**: Zero testing and zero build checking during implementation. Strict sequential batch execution. Exclusively relative paths.

---

## 1. Executive Summary

This specification defines the 169th sequence of 30 grounded improvements (Improvements 361 to 390) spanning five core Antigravity Manager subsystems:
1. **Proxy Rate Limiting, Framing & Heartbeat** (Imp 361–366)
2. **Auto-Switcher Adaptive Quota & Headroom** (Imp 367–372)
3. **Split-SQLite Concurrency & Diagnostics** (Imp 373–378)
4. **Multi-Instance Sandboxing & Lifecycle** (Imp 379–384)
5. **Desktop React UI & UX Component Modernization** (Imp 385–390)

All tasks adhere to the strict house rules: no silent errors, protocol-agnostic pipeline primacy, strictly relative repository paths, and zero testing/build checks during active execution.

---

## 2. Detailed Improvement Breakdown

### Batch 1: Proxy Rate Limiting, Framing & Heartbeat (Imp 361–366)
- **Imp-361: Stream Burst Rate Limiter** (`src-tauri/src/proxy/rate_limit/stream_burst_limiter.rs`)
  - Implements a token-bucket rate limiter tuned for streaming chunks with configurable burst size and token refill rate to prevent downstream network saturation.
- **Imp-362: SSE Comment Heartbeat Ping** (`src-tauri/src/proxy/handlers/common/sse_ping_heartbeat.rs`)
  - Generates periodic SSE comment frames (`: keepalive <timestamp>\n\n`) when reasoning/thinking delays exceed the ping threshold, preventing TCP connection drops.
- **Imp-363: Client Compression Negotiator** (`src-tauri/src/proxy/handlers/common/compression_negotiator.rs`)
  - Evaluates client `Accept-Encoding` against upstream response encodings, deciding whether streaming body should be passed raw or compressed.
- **Imp-364: Tool Call Response Formatter** (`src-tauri/src/proxy/handlers/common/tool_response_formatter.rs`)
  - Normalizes tool call execution outputs into unified Canonical IR responses across Claude, OpenAI, and Gemini protocols.
- **Imp-365: Upstream Endpoint Health Decay** (`src-tauri/src/proxy/upstream/endpoint_health_decay.rs`)
  - Implements half-life decay of upstream error weights so recovering endpoints regain traffic gradually without thunder-herd effects.
- **Imp-366: Claude Stop Reason Normalizer** (`src-tauri/src/proxy/handlers/claude/stop_reason_normalizer.rs`)
  - Maps Anthropic Claude finish reasons (`end_turn`, `max_tokens`, `stop_sequence`, `tool_use`) to Canonical IR and OpenAI-compatible finish reasons.

### Batch 2: Auto-Switcher Adaptive Quota & Headroom (Imp 367–372)
- **Imp-367: Dynamic Quota Floor Guard** (`src-tauri/src/modules/auto_switcher/dynamic_quota_floor.rs`)
  - Calculates dynamic quota safety margins based on the target model's context window and average prompt size before allowing account selection.
- **Imp-368: Entropy Protected Candidate Shuffler** (`src-tauri/src/modules/auto_switcher/entropy_candidate_shuffler.rs`)
  - Uses crypto-seeded pseudorandom shuffling for candidate accounts with tied scores to ensure even distribution and prevent affinity bias.
- **Imp-369: Quota Consumption Acceleration Detector** (`src-tauri/src/modules/auto_switcher/quota_acceleration_detector.rs`)
  - Computes the rate of change of quota velocity (second derivative) to detect runaway prompt consumption loops early.
- **Imp-370: Machine Lease Pre-Release Coordinator** (`src-tauri/src/modules/auto_switcher/lease_prerelease_coordinator.rs`)
  - Handles orderly lease hand-off and pre-release before host sleep, hibernation, or application shutdown.
- **Imp-371: Cooldown Recovery Curve Calculator** (`src-tauri/src/modules/auto_switcher/cooldown_recovery_curve.rs`)
  - Calculates smooth non-linear sigmoid recovery curves for accounts emerging from rate-limit cooldown.
- **Imp-372: Account Session Affinity Tracker** (`src-tauri/src/modules/auto_switcher/session_affinity_tracker.rs`)
  - Maintains conversation-to-account stickiness windows to prevent unnecessary switches in the middle of active multi-turn interactions.

### Batch 3: Split-SQLite Concurrency & Diagnostics (Imp 373–378)
- **Imp-373: SQLite Lock Escalation Guard** (`src-tauri/src/modules/db_orm/lock_escalation_guard.rs`)
  - Intercepts read-to-write transaction escalation, applying bounded exponential backoff to prevent SQLITE_BUSY deadlocks in WAL mode.
- **Imp-374: Table Row Size Profiler** (`src-tauri/src/modules/db_orm/table_row_size_profiler.rs`)
  - Scans and samples row byte sizes in `repo_prompts.db` and `prompt_backups.db` to identify oversized prompt payloads.
- **Imp-375: Split Database Orphan Checker** (`src-tauri/src/modules/db_orm/split_db_orphan_checker.rs`)
  - Validates logical cross-database foreign keys between task history entries and conversation prompt records.
- **Imp-376: SQLite Page Cache Sizing Adjuster** (`src-tauri/src/modules/db_orm/page_cache_adjuster.rs`)
  - Evaluates system memory and dynamically tunes `PRAGMA cache_size` to optimize memory footprint vs query performance.
- **Imp-377: Split Database Pragma Diagnostic Reporter** (`src-tauri/src/modules/db_orm/pragma_diagnostic_reporter.rs`)
  - Inspects and compiles comprehensive PRAGMA configuration reports (`wal_autocheckpoint`, `synchronous`, `cache_size`, `busy_timeout`).
- **Imp-378: Stale WAL File Recovery Sentinel** (`src-tauri/src/modules/db_orm/stale_wal_cleanup_sentinel.rs`)
  - Scans for dangling `-wal` and `-shm` files on startup after abnormal app termination and safely runs WAL checkpoint recovery.

### Batch 4: Multi-Instance Sandboxing & Lifecycle (Imp 379–384)
- **Imp-379: Project Path Canonical Normalizer** (`src-tauri/src/modules/instance/project_path_canonical_normalizer.rs`)
  - Resolves Windows 8.3 short paths, mixed delimiters, and trailing slashes to guarantee consistent repository path indexing.
- **Imp-380: Keyring Mock Marker Writer** (`src-tauri/src/modules/instance/keyring_mock_writer.rs`)
  - Atomically writes and verifies the OS keyring bypass markers and mock flags in sandboxed instance user data directories.
- **Imp-381: Instance Session Trace UUID** (`src-tauri/src/modules/instance/session_trace_uuid.rs`)
  - Generates cryptographically secure, instance-tagged session trace tokens (`agm-inst-<id>-<uuid>`) for unified telemetry across log streams.
- **Imp-382: Instance Child Process Tree Enumerator** (`src-tauri/src/modules/instance/child_process_tree_enumerator.rs`)
  - Recursively discovers all child processes spawned by an Antigravity IDE instance (language servers, git workers) before safe termination.
- **Imp-383: Storage JSON Migration Guard** (`src-tauri/src/modules/instance/storage_migration_guard.rs`)
  - Validates `storage.json` integrity across IDE updates, backfilling missing telemetry keys and stripping corrupted state nodes.
- **Imp-384: Remote Debugging Port Allocator** (`src-tauri/src/modules/instance/remote_debugging_port_allocator.rs`)
  - Scans and allocates free TCP ports for CDP remote debugging with socket bind verification and race-condition resistance.

### Batch 5: Desktop React UI & UX Component Modernization (Imp 385–390)
- **Imp-385: Origin Tier Badge Pill** (`src/components/common/OriginTierBadgePill.tsx`)
  - Renders the 4-tier prompt origin classifications (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`) with icon, label, and high-contrast color styling.
- **Imp-386: Segmented Tab Group** (`src/components/common/SegmentedTabGroup.tsx`)
  - Seamless pill segmented tab bar component with smooth indicator slide, dark-glass borderless styling, and ARIA keyboard support.
- **Imp-387: Escape Dialog Dismissal Hook** (`src/hooks/useEscapeDialogDismissal.ts`)
  - Custom hook providing reliable Escape key modal dismissal, stopping propagation and tracking active modal stacks.
- **Imp-388: Truncated Text Tooltip Capsule** (`src/components/common/TruncatedTextTooltip.tsx`)
  - Truncated text wrapper with interactive hover popover and 1-click copy action, avoiding layout distortion in tables and cards.
- **Imp-389: Micro Time Ago Formatter** (`src/utils/microTimeAgo.ts`)
  - Compact humanized time difference helper (`now`, `12s`, `4m`, `2h`, `1d`) with sub-second precision.
- **Imp-390: Micro Action Button Group** (`src/components/common/MicroActionButtonGroup.tsx`)
  - Reusable contiguous segmented pill capsule for table and card row actions (`Copy`, `Rerun`, `Delete`) with shared borders and hover states.

---

## 3. Implementation Verification Criteria

1. **Strictly Disjoint Modules**: All new source files reside in their designated architectural subdirectories.
2. **Re-Export Facade Integrity**: Submodules are cleanly re-exported in parent facades.
3. **Zero Errors Swallowed**: All fallible calls return contextual `AppError` or use `trackWarning`.
4. **Zero-Test / Zero-Build Execution**: Immediate sequential implementation without test or build pauses.
