# Specification: 30 Improvements Planning and Execution (Improvements 511–540)

- **Task**: Improvements Planning and Execution
- **Slug**: `174-improvements-planning-and-execution`
- **Sequence**: 174
- **Date**: 2026-10-22
- **Author**: Lead Orchestrator
- **Constraint Policy**: Zero testing, zero build checking, sequential execution, strictly relative repository paths.

---

## 1. Executive Summary & Architectural Intent

This specification defines the 174th cycle of codebase improvements across Antigravity-Manager, delivering 30 distinct enhancements (Improvements 511 through 540) partitioned across 5 critical subsystems:
1. **Reverse Proxy SSE Protocol, Chunk Multiplexing & Connection Keepalive (Imp 511–516)**: SSE frame splitting, exponential backoff matrix, request gzip sniffing, TLS ALPN negotiation, Claude turn validation, and DNS failover.
2. **Auto-Switcher Candidate Scoring, Velocity Dampening & Quota Alignment (Imp 517–522)**: Quota reset epoch tracking, consecutive switch cooldowns, account staleness detection, percentile score normalization, 10-second settling delays, and instance readiness barriers.
3. **Split-SQLite Maintenance, WAL Checkpointing & Lock Recovery (Imp 523–528)**: Passive non-blocking WAL checkpoints, dynamic busy timeout scaling, migration journals, table column statistics, serializable read-uncommitted guards, and page freelist analyzers.
4. **Multi-Instance Sandboxed Profile Isolation & Process Watchdogs (Imp 529–534)**: Sandboxed PATH normalizers, stale PID reapers, SHA-256 profile state hashers, binary execution probes, remote debugging port range guards, and directory permission checkers.
5. **Desktop React UI Modernization, Accessibility & Feedback Primitives (Imp 535–540)**: Collapsible detail rows, floating notification toast cards, sliding window metrics hooks, compact number steppers, human-readable duration formatters, and glowing health pulse status indicators.

---

## 2. Exhaustive 30-Improvement Inventory

### Batch 1: Proxy SSE Protocol, Chunk Multiplexing & Connection Keepalive (Imp 511–516)
- **Imp-511: Raw SSE Stream Frame Splitter** (`src-tauri/src/proxy/handlers/common/sse_stream_splitter.rs`)
  - Splits coalesced raw SSE network frames on double newline `\n\n` boundaries while preserving partial chunk tails across incoming buffers.
- **Imp-512: Upstream Connection Exponential Backoff Matrix** (`src-tauri/src/proxy/upstream/connection_backoff_matrix.rs`)
  - Computes exponential backoff intervals with jitter matrix for transient upstream network disconnects and HTTP 503/502 faults.
- **Imp-513: Request Gzip Compression Sniffer** (`src-tauri/src/proxy/handlers/common/request_gzip_sniffer.rs`)
  - Detects and validates gzip/deflate magic bytes in raw HTTP request payloads before invoking decompressors.
- **Imp-514: Upstream TLS ALPN Protocol Negotiator** (`src-tauri/src/proxy/upstream/tls_alpn_negotiator.rs`)
  - Configures TLS ALPN preferences (h2, http/1.1) and enforces protocol fallback ladders during upstream socket handshakes.
- **Imp-515: Claude Message Turn Structure Validator** (`src-tauri/src/proxy/handlers/claude/message_turn_validator.rs`)
  - Validates Claude messages payload structure, ensuring non-empty role arrays and stripping orphaned trailing whitespace blocks.
- **Imp-516: In-Memory DNS Failover Resolver** (`src-tauri/src/proxy/upstream/dns_failover_resolver.rs`)
  - Provides in-memory cached DNS resolution with dual-stack IPv4/IPv6 fallback for upstream hosts.

### Batch 2: Auto-Switcher Candidate Scoring, Velocity Dampening & Quota Alignment (Imp 517–522)
- **Imp-517: Quota Reset Epoch Replenishment Tracker** (`src-tauri/src/modules/auto_switcher/quota_reset_epoch_tracker.rs`)
  - Records historical quota reset timestamps and computes rolling replenishment intervals per account.
- **Imp-518: Consecutive Switch Cooldown Penalty Ladder** (`src-tauri/src/modules/auto_switcher/consecutive_switch_cooldown.rs`)
  - Tracks consecutive switches per instance profile and enforces progressive cooldown penalties to suppress rapid cycling.
- **Imp-519: Inactive Account Staleness Detector** (`src-tauri/src/modules/auto_switcher/account_staleness_detector.rs`)
  - Detects stale or inactive accounts based on last successful turn timestamps and flags them for quota re-probing.
- **Imp-520: Candidate Score Percentile Normalizer** (`src-tauri/src/modules/auto_switcher/score_percentile_normalizer.rs`)
  - Normalizes candidate algorithm scores into percentile bands (0–1000) for consistent scoreboard visualization.
- **Imp-521: Post-Switch Settling Delay Guard** (`src-tauri/src/modules/auto_switcher/switch_settling_timer.rs`)
  - Enforces an async non-blocking 10-second settling delay following credential switches before allowing prompt injection.
- **Imp-522: Sibling Instance Switch Readiness Barrier** (`src-tauri/src/modules/auto_switcher/instance_readiness_barrier.rs`)
  - Coordinates multi-instance switch readiness barriers, preventing concurrent rotations on sibling profiles.

### Batch 3: Split-SQLite Maintenance, WAL Checkpointing & Lock Recovery (Imp 523–528)
- **Imp-523: Passive WAL Checkpoint Scheduler** (`src-tauri/src/modules/db_orm/wal_passive_checkpoint_scheduler.rs`)
  - Executes passive non-blocking `PRAGMA wal_checkpoint(PASSIVE)` during idle periods to shrink log page counts without reader contention.
- **Imp-524: Dynamic Busy Timeout Scaler** (`src-tauri/src/modules/db_orm/connection_busy_timeout_scaler.rs`)
  - Scales SQLite `busy_timeout` dynamically according to active connection count and read/write load.
- **Imp-525: Schema Migration Append-Only Journal** (`src-tauri/src/modules/db_orm/schema_migration_journal.rs`)
  - Maintains an append-only migration journal in SQLite tracking schema transition history and applied timestamps.
- **Imp-526: Split Table Column Statistics Collector** (`src-tauri/src/modules/db_orm/table_column_stats_collector.rs`)
  - Gathers row counts and approximate null-ratio statistics across split database tables for query tuning.
- **Imp-527: Serializable Read-Uncommitted Guard** (`src-tauri/src/modules/db_orm/read_uncommitted_guard.rs`)
  - Configures `PRAGMA read_uncommitted = 0;` on transaction boundaries to ensure strict serializable isolation.
- **Imp-528: Database Page Freelist Reclaim Analyzer** (`src-tauri/src/modules/db_orm/page_freelist_analyzer.rs`)
  - Measures `PRAGMA freelist_count;` and determines whether `incremental_vacuum` should reclaim unused pages.

### Batch 4: Multi-Instance Sandboxed Profile Isolation & Process Watchdogs (Imp 529–534)
- **Imp-529: Sandbox Toolchain PATH Normalizer** (`src-tauri/src/modules/instance/env_path_normalizer.rs`)
  - Normalizes system `PATH` entries for child IDE instances, ensuring sandbox tools and GitMap take precedence.
- **Imp-530: Stale Instance PID Process Reaper** (`src-tauri/src/modules/instance/stale_pid_reaper.rs`)
  - Inspects tracked instance PIDs against OS process lists, marking orphaned instance records as stopped cleanly.
- **Imp-531: Instance Profile Configuration SHA-256 Hasher** (`src-tauri/src/modules/instance/profile_state_hasher.rs`)
  - Computes SHA-256 fingerprint hashes of instance profile configuration states to detect external edits.
- **Imp-532: IDE Binary Architecture and Execution Probe** (`src-tauri/src/modules/instance/binary_execution_probe.rs`)
  - Probes candidate IDE executables for launchability, verifying binary architecture and execution permissions.
- **Imp-533: Remote Debugging Port Lease Guard** (`src-tauri/src/modules/instance/remote_port_guard.rs`)
  - Reserves and validates high-range remote debugging ports (9222–9299), tracking port leases across instances.
- **Imp-534: Instance Directory Permission Pre-Flight Guard** (`src-tauri/src/modules/instance/directory_permission_guard.rs`)
  - Checks read/write permissions across instance data directories before initiating process launch.

### Batch 5: Desktop React UI Modernization, Accessibility & Feedback Primitives (Imp 535–540)
- **Imp-535: Accessible Collapsible Detail Row** (`src/components/common/CollapsibleDetailRow.tsx`)
  - Expandable table/card detail row with smooth height transition and accessible keyboard controls.
- **Imp-536: Dark-Glass Floating Toast Card** (`src/components/common/FloatingNotificationCard.tsx`)
  - Dark-glass toast card with progress bar, dismiss button, and icon-coded severity styling.
- **Imp-537: Sliding Window Telemetry Metrics Hook** (`src/hooks/useSlidingWindowMetrics.ts`)
  - React hook maintaining a fixed-size time-window buffer of latency/throughput metrics for dashboard sparklines.
- **Imp-538: Segmented Number Stepper Input** (`src/components/common/CompactNumberStepper.tsx`)
  - Accessible segmented number stepper (`[-] [value] [+]`) with keyboard arrow increments and bounds clamping.
- **Imp-539: Human-Readable Duration Formatter** (`src/utils/timeDurationFormatter.ts`)
  - Formats milliseconds into human-readable compact time units (`12ms`, `1.4s`, `3m 22s`, `4h 10m`).
- **Imp-540: Glowing Health Pulse Status Indicator** (`src/components/common/HealthPulseIndicator.tsx`)
  - Animated heartbeat/pulse status dot with glowing ring effects and WCAG-compliant status labels.

---

## 3. Implementation Verification & Invariant Alignment
- **Strictly Zero Tests / Zero Builds**: No build or test execution during implementation as mandated by user request.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors**: All fallible operations must propagate errors into `AppError` in Rust or `useErrorStore` in TypeScript.
