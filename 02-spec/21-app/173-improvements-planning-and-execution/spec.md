# Specification: 30 Improvements Planning and Execution (Improvements 481–510)

- **Sequence**: 173
- **Feature Directory**: `02-spec/21-app/173-improvements-planning-and-execution/`
- **Plan File**: `.ai-memory/plans/pending/173-improvements-planning-and-execution.md`
- **Subtasks**: `.ai-memory/plans/subtasks/173-improvements-planning-and-execution/`
- **Status**: [APPROVED]
- **Operational Discipline**: Zero testing and zero build checking during implementation. Strict sequential batch execution. Exclusively relative paths.

---

## 1. Executive Summary

This specification defines the 173rd sequence of 30 grounded improvements (Improvements 481 to 510) spanning five core Antigravity Manager subsystems:
1. **Proxy SSE Framing, Chunk Buffering & Upstream Health** (Imp 481–486)
2. **Auto-Switcher Adaptive Dampeners & Quota Boundary Synchronization** (Imp 487–492)
3. **Split-SQLite Maintenance, WAL Checkpoint Schedulers & Schema Gates** (Imp 493–498)
4. **Multi-Instance Sandboxed Keychain, Environment & Lifecycle** (Imp 499–504)
5. **Desktop React UI Modernization, Feedback & Component Primitives** (Imp 505–510)

All tasks adhere to the strict house rules: no silent errors, protocol-agnostic pipeline primacy, strictly relative repository paths, and zero testing/build checks during active execution.

---

## 2. Detailed Improvement Breakdown

### Batch 1: Proxy SSE Framing, Chunk Buffering & Upstream Health (Imp 481–486)
- **Imp-481: Upstream TCP Keep-Alive Probe Configuration** (`src-tauri/src/proxy/upstream/tcp_keepalive_probe.rs`)
  - Configures platform-level TCP keepalive probes (idle 30s, interval 10s, probe count 3) to detect silent upstream socket drops.
- **Imp-482: SSE Stream Preamble Buffer Padder** (`src-tauri/src/proxy/handlers/common/sse_stream_padder.rs`)
  - Generates initial SSE comment byte buffers (`: padding to bypass intermediate proxy buffers\n\n`) to ensure immediate client streaming.
- **Imp-483: Request Content-Encoding Negotiator** (`src-tauri/src/proxy/handlers/common/request_content_encoding_negotiator.rs`)
  - Negotiates Content-Encoding compatibility between downstream client capabilities and upstream endpoint headers.
- **Imp-484: Upstream Stream Drain Cancellation Guard** (`src-tauri/src/proxy/handlers/common/stream_drain_guard.rs`)
  - Consumes residual HTTP body bytes upon downstream client abort, returning upstream connections to the pool cleanly.
- **Imp-485: Claude Message Alternating Role Prefix Sanitizer** (`src-tauri/src/proxy/handlers/claude/role_prefix_sanitizer.rs`)
  - Validates and sanitizes message role alternation (user/assistant) in Claude payloads, inserting empty padding turns when needed.
- **Imp-486: Multi-Endpoint Dynamic Route Balancer** (`src-tauri/src/proxy/upstream/dynamic_route_balancer.rs`)
  - Dynamically adjusts upstream route weights based on rolling error-rate scores and response latencies.

### Batch 2: Auto-Switcher Adaptive Dampeners & Quota Boundary Synchronization (Imp 487–492)
- **Imp-487: 5-Hour Rolling Quota Boundary Synchronizer** (`src-tauri/src/modules/auto_switcher/quota_boundary_synchronizer.rs`)
  - Aligns rotation evaluation timers with rolling 5-hour quota reset boundaries to prevent switching right before an account recovers.
- **Imp-488: Consecutive Switch Velocity Throttle** (`src-tauri/src/modules/auto_switcher/consecutive_switch_throttle.rs`)
  - Enforces a hard throttle limiting instance account switches to at most 3 operations within any 60-second sliding window.
- **Imp-489: LRU Account-Instance Binding History Cache** (`src-tauri/src/modules/auto_switcher/binding_history_cache.rs`)
  - Maintains in-memory LRU tracking of recent account-to-instance bindings to preserve profile continuity and affinity.
- **Imp-490: Idle Candidate Score Time-Decay Curve** (`src-tauri/src/modules/auto_switcher/candidate_score_decay.rs`)
  - Applies a progressive time-decay curve to candidate scores for accounts that remain idle, balancing load across peers.
- **Imp-491: High-Speed Split-DB Switch Audit Logger** (`src-tauri/src/modules/auto_switcher/switch_audit_logger.rs`)
  - Logs structured switch execution audits directly into `task_history.db` with rationale codes and settling latencies.
- **Imp-492: Active Instance Readiness Prober** (`src-tauri/src/modules/auto_switcher/active_instance_prober.rs`)
  - Performs non-blocking HTTP and CDP health probes to confirm an IDE instance is ready to receive prompt injections.

### Batch 3: Split-SQLite Maintenance, WAL Checkpoint Schedulers & Schema Gates (Imp 493–498)
- **Imp-493: Periodic WAL Truncate Checkpoint Scheduler** (`src-tauri/src/modules/db_orm/wal_truncate_scheduler.rs`)
  - Triggers periodic `PRAGMA wal_checkpoint(TRUNCATE)` when the WAL file page count exceeds safety thresholds (500 pages).
- **Imp-494: Connection Pool Checkout Rate Limiter** (`src-tauri/src/modules/db_orm/connection_checkout_limiter.rs`)
  - Limits concurrent connection checkouts across background tasks to protect user-facing interactive queries.
- **Imp-495: Split Database Schema User Version Gate** (`src-tauri/src/modules/db_orm/schema_version_checker.rs`)
  - Compares `PRAGMA user_version` against compiled application migration numbers before initializing SQLite connection pools.
- **Imp-496: Table Index Cardinality Analyzer** (`src-tauri/src/modules/db_orm/index_cardinality_analyzer.rs`)
  - Analyzes column distinct value counts to identify low-selectivity indexes that cause query optimizer regressions.
- **Imp-497: Temporary Storage Memory Guard** (`src-tauri/src/modules/db_orm/temp_store_memory_guard.rs`)
  - Enforces `PRAGMA temp_store = MEMORY;` across split database connections to keep transient tables in RAM.
- **Imp-498: Table Page Space Allocation Counter** (`src-tauri/src/modules/db_orm/table_page_counter.rs`)
  - Inspects page allocations per table across split databases, detecting unexpected table bloat before it impacts performance.

### Batch 4: Multi-Instance Sandboxed Keychain, Environment & Lifecycle (Imp 499–504)
- **Imp-499: Environment Variable Host Token Scrubber** (`src-tauri/src/modules/instance/env_token_scrubber.rs`)
  - Scrubs sensitive host environment variables (`GITHUB_TOKEN`, `OPENAI_API_KEY`) from environment blocks passed to instances.
- **Imp-500: Stale Socket Lockfile Recovery Sentinel** (`src-tauri/src/modules/instance/socket_lock_recovery.rs`)
  - Detects and cleans abandoned socket lockfiles (`antigravity-*.lock`) left behind by interrupted process shutdowns.
- **Imp-501: Cloned Profile Telemetry Isolation Verifier** (`src-tauri/src/modules/instance/profile_telemetry_isolation.rs`)
  - Verifies that `storage.json` within cloned instance directories contains unique, non-colliding telemetry salts and machine GUIDs.
- **Imp-502: Multi-Stage Process Spawn Ladder** (`src-tauri/src/modules/instance/process_spawn_ladder.rs`)
  - Implements a resilient 3-stage spawn ladder: direct target binary -> detected base binary -> Windows Shell dispatch (`cmd.exe /c start`).
- **Imp-503: Non-Blocking Port Availability Guard** (`src-tauri/src/modules/instance/port_allocation_guard.rs`)
  - Performs non-blocking TCP socket bind checks on candidate remote debugging ports before assigning them to instance profiles.
- **Imp-504: Instance Workspace Subdirectory Guard** (`src-tauri/src/modules/instance/workspace_directory_guard.rs`)
  - Verifies and auto-creates required instance directory hierarchies (`User`, `logs`, `workspaceStorage`) before launch.

### Batch 5: Desktop React UI Modernization, Feedback & Component Primitives (Imp 505–510)
- **Imp-505: Expandable Dark-Glass Accordion Section Card** (`src/components/common/AccordionSectionCard.tsx`)
  - Collapsible card component with smooth chevron rotation, subtitle, badge support, and dark-glass aesthetics.
- **Imp-506: Accessible Hover Tooltip Bubble** (`src/components/common/TooltipBubble.tsx`)
  - Lightweight dark-glass tooltip container with boundary-clamping positioning and keyboard focus visibility.
- **Imp-507: Type-Safe LocalStorage Persistent State Hook** (`src/hooks/usePersistentState.ts`)
  - React hook synchronizing component state with browser localStorage with schema serialization and SSR safety.
- **Imp-508: Contiguous Segmented Tab Group** (`src/components/common/SegmentedTabGroup.tsx`)
  - Contiguous segmented pill tabs with animated sliding indicator, micro-dividers, and keyboard navigation.
- **Imp-509: Middle-Ellipsis String Truncation Utility** (`src/utils/stringTruncator.ts`)
  - Formats lengthy file paths, URLs, and hash identifiers with readable middle-ellipsis (`abc...xyz`).
- **Imp-510: Metric Comparison Delta Badge** (`src/components/common/MetricComparisonBadge.tsx`)
  - Visual delta pill displaying positive/negative performance changes with directional arrow icons and color ramps.

---

## 3. Implementation Verification Criteria
- All 30 improvements implemented cleanly in their respective Rust modules and React components.
- Zero testing and zero build checking executed during implementation as instructed.
- All code changes strictly utilize repository-relative paths and adhere to error handling conventions (`AppError` in Rust, `trackWarning`/`captureError` in TS).
