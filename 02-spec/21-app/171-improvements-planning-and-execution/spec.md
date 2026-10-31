# Specification: 30 Improvements Planning and Execution (Improvements 421–450)

- **Sequence**: 171
- **Feature Directory**: `02-spec/21-app/171-improvements-planning-and-execution/`
- **Plan File**: `.ai-memory/plans/pending/171-improvements-planning-and-execution.md`
- **Subtasks**: `.ai-memory/plans/subtasks/171-improvements-planning-and-execution/`
- **Status**: [APPROVED]
- **Operational Discipline**: Zero testing and zero build checking during implementation. Strict sequential batch execution. Exclusively relative paths.

---

## 1. Executive Summary

This specification defines the 171st sequence of 30 grounded improvements (Improvements 421 to 450) spanning five core Antigravity Manager subsystems:
1. **Proxy Upstream TLS, HTTP/2 Flow Control & SSE Framing** (Imp 421–426)
2. **Auto-Switcher Variance Scoring & Cooldown Telemetry** (Imp 427–432)
3. **Split-SQLite WAL Stepper, Profiling & Schema Integrity** (Imp 433–438)
4. **Multi-Instance Sandboxed Keychain, PID Reclaim & Session Cleanup** (Imp 439–444)
5. **Desktop React UI Modernization, Accessibility & Primitives** (Imp 445–450)

All tasks adhere to the strict house rules: no silent errors, protocol-agnostic pipeline primacy, strictly relative repository paths, and zero testing/build checks during active execution.

---

## 2. Detailed Improvement Breakdown

### Batch 1: Proxy Upstream TLS, HTTP/2 Flow Control & SSE Framing (Imp 421–426)
- **Imp-421: TLS SNI Hostname Sanitizer & IDN Normalizer** (`src-tauri/src/proxy/upstream/tls_sni_sanitizer.rs`)
  - Sanitizes and normalizes upstream TLS Server Name Indication (SNI) hostnames, stripping port numbers, path fragments, and converting internationalized domain names (IDN/Punycode).
- **Imp-422: HTTP/2 Window Size & Concurrency Flow Controller** (`src-tauri/src/proxy/upstream/http2_flow_controller.rs`)
  - Configures adaptive HTTP/2 connection window size (64KB to 16MB) and concurrent stream thresholds to prevent upstream flow-control buffer stalls.
- **Imp-423: SSE Chunk Coalescer & Incomplete Frame Buffer** (`src-tauri/src/proxy/handlers/common/sse_chunk_coalescer.rs`)
  - Buffers and coalesces fragmented SSE byte slices, ensuring downstream clients receive intact `data: {...}\n\n` event boundaries without split UTF-8 runes.
- **Imp-424: Request Content-Length & Body Size Bounds Validator** (`src-tauri/src/proxy/handlers/common/request_content_length_validator.rs`)
  - Validates `Content-Length` headers against actual stream body sizes, intercepting payload size spoofing and oversized requests (>32MB).
- **Imp-425: Claude Message Content & Multimodal Block Flattener** (`src-tauri/src/proxy/handlers/claude/message_content_flattener.rs`)
  - Normalizes heterogeneous Claude message contents (strings, nested text blocks, image payloads) into canonical linear content representations.
- **Imp-426: Weighted Round Robin with Randomized Jitter Load Balancer** (`src-tauri/src/proxy/upstream/round_robin_jitter.rs`)
  - Implements smooth weighted round-robin upstream routing augmented with proportional jitter to eliminate traffic synchronization across identical weight nodes.

### Batch 2: Auto-Switcher Variance Scoring & Cooldown Telemetry (Imp 427–432)
- **Imp-427: Candidate Score Variance & Outlier Dampener** (`src-tauri/src/modules/auto_switcher/score_variance_guard.rs`)
  - Analyzes candidate score distribution across account pools and dampens transient metric spikes to prevent switch flapping.
- **Imp-428: Stale Profile Lease Reclaim Watchdog** (`src-tauri/src/modules/auto_switcher/lease_reclaim_watchdog.rs`)
  - Sweeps and forcefully reclaims orphaned workspace leases held by terminated or non-responsive IDE instances past the TTL threshold.
- **Imp-429: Cooldown Event & Rotation Telemetry Reporter** (`src-tauri/src/modules/auto_switcher/cooldown_telemetry_reporter.rs`)
  - Emits structured telemetry records capturing cooldown reasons, durations, and account transitions for audit and fleet sync.
- **Imp-430: Account Priority Tier Fallback Ladder** (`src-tauri/src/modules/auto_switcher/priority_tier_ladder.rs`)
  - Provides a deterministic fallback ladder (Ultra -> Pro -> Sandbox -> Free) when primary tier accounts enter active cooldown.
- **Imp-431: Candidate Exclusion Registry & Manual Bypass Cache** (`src-tauri/src/modules/auto_switcher/candidate_exclusion_registry.rs`)
  - Maintains in-memory thread-safe registries of temporarily excluded accounts (manual bypass, user disabled, quota exhausted).
- **Imp-432: Account Switch Settling Barrier & State Prober** (`src-tauri/src/modules/auto_switcher/switch_settling_barrier.rs`)
  - Coordinates a non-blocking settling delay barrier with progressive health probes before enabling prompt injection on switched profiles.

### Batch 3: Split-SQLite WAL Stepper, Profiling & Schema Integrity (Imp 433–438)
- **Imp-433: WAL Stepped Incremental Checkpointer** (`src-tauri/src/modules/db_orm/wal_stepped_checkpoint.rs`)
  - Executes incremental stepped WAL checkpoints (`PASSIVE` -> `RESTART` -> `TRUNCATE`) based on WAL file size and active reader count.
- **Imp-434: SQLite Lock Contention & Busy Wait Profiler** (`src-tauri/src/modules/db_orm/lock_contention_profiler.rs`)
  - Measures lock contention events and busy-handler execution durations across split SQLite databases, emitting high-latency warnings.
- **Imp-435: Column Type Affinity & Constraint Validator** (`src-tauri/src/modules/db_orm/column_type_affinity_validator.rs`)
  - Verifies SQLite column affinity definitions and primary key autoincrement constraints across split databases during startup.
- **Imp-436: Query Explain Query Plan Cost Estimator** (`src-tauri/src/modules/db_orm/query_cost_estimator.rs`)
  - Inspects `EXPLAIN QUERY PLAN` output for table scans (`SCAN TABLE`) and flags missing index references on frequently queried columns.
- **Imp-437: Temporary Database & Journal File Cleanup Sentinel** (`src-tauri/src/modules/db_orm/temp_db_cleanup_sentinel.rs`)
  - Sweeps orphaned temporary SQLite journal files (`-journal`, `-wal.tmp`) left behind by interrupted process crashes.
- **Imp-438: Split Database Backup Snapshot Manifest Generator** (`src-tauri/src/modules/db_orm/database_snapshot_manifest.rs`)
  - Generates atomic metadata manifests (file sizes, page counts, SHA-256 hashes) for multi-database point-in-time backup sets.

### Batch 4: Multi-Instance Sandboxed Keychain, PID Reclaim & Session Cleanup (Imp 439–444)
- **Imp-439: Platform-Agnostic Sandboxed Keychain Path Resolver** (`src-tauri/src/modules/instance/keychain_path_resolver.rs`)
  - Resolves sandboxed mock keyring file paths and mock credential store flags across Windows, Linux, and macOS runtimes.
- **Imp-440: Instance PID Reclaim Barrier & Liveness Verifier** (`src-tauri/src/modules/instance/pid_reclaim_barrier.rs`)
  - Verifies that a target instance process has truly terminated before releasing saved PID locks, preventing zombie PID re-use.
- **Imp-441: Volatile Session & Cache Directory Cleanup Worker** (`src-tauri/src/modules/instance/session_cleanup_worker.rs`)
  - Cleans temporary session caches, crash dump directories, and GPU cache folders within sandboxed instance profile roots.
- **Imp-442: Instance Profile Clone Integrity Verifier** (`src-tauri/src/modules/instance/profile_clone_verifier.rs`)
  - Verifies the byte-for-byte and structural integrity of cloned instance profile directories before activating them in the registry.
- **Imp-443: Dynamic Port Lease Allocation Tracker** (`src-tauri/src/modules/instance/port_lease_tracker.rs`)
  - Tracks active and reserved TCP/CDP debugging ports across multi-instance sandboxes to prevent binding collisions.
- **Imp-444: Workspace Storage State & Window Layout Sanitizer** (`src-tauri/src/modules/instance/workspace_storage_sanitizer.rs`)
  - Sanitizes stale workspace storage state files and window coordinate locks to prevent multi-instance window position clumping.

### Batch 5: Desktop React UI Modernization, Accessibility & Primitives (Imp 445–450)
- **Imp-445: Contiguous Segmented Pill Toggle Switch** (`src/components/common/SegmentedPillSwitch.tsx`)
  - Contiguous dark-glass segmented switch component with animated sliding active pill highlight and keyboard navigation support.
- **Imp-446: Top-Level Modal Portal Layer** (`src/components/common/ModalPortalLayer.tsx`)
  - Top-level modal backdrop portal container with z-index elevation (`z-[99999]`), focus containment, and click-outside dismissal.
- **Imp-447: Accessible Keyboard-Only Focus Ring Detection Hook** (`src/hooks/useKeyboardFocusRing.ts`)
  - React hook detecting Tab/keyboard navigation to conditionally render high-contrast focus rings while suppressing mouse click rings.
- **Imp-448: Pulsing Multi-State Status Dot Indicator** (`src/components/common/StatusDotIndicator.tsx`)
  - Compact status dot with configurable pinging animation, color tokens (green, amber, rose, slate, sky), and accessible ARIA attributes.
- **Imp-449: Compact Metric Number Abbreviator Utility** (`src/utils/numberAbbreviator.ts`)
  - Formats large numbers into readable compact metric units (`1.2K`, `45.6M`, `2.1B`) with decimal precision and boundary safety.
- **Imp-450: Contiguous Dark-Glass Compact Action Toolbar Capsule** (`src/components/common/CompactActionBar.tsx`)
  - Contiguous segmented pill capsule grouping action buttons with borderless styling, micro-dividers, and dark-glass aesthetics.

---

## 3. Implementation Verification Criteria
- All 30 improvements implemented cleanly in their respective Rust modules and React components.
- Zero testing and zero build checking executed during implementation as instructed.
- All code changes strictly utilize repository-relative paths and adhere to error handling conventions (`AppError` in Rust, `trackWarning`/`captureError` in TS).
