# Specification: 30 Improvements Planning and Execution (Improvements 271–300)

- **Task**: Improvements Planning and Execution
- **Sequence**: 166
- **Slug**: `166-improvements-planning-and-execution`
- **Spec Directory**: `02-spec/21-app/166-improvements-planning-and-execution/`
- **Plan Path**: `.ai-memory/plans/pending/166-improvements-planning-and-execution.md`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/166-improvements-planning-and-execution/`

---

## 1. Executive Summary

This specification establishes the architectural contracts, structural definitions, and implementation guidelines for 30 concrete improvements (Improvements 271 to 300) across Antigravity-Manager. Following strict operational guidelines, all 30 improvements are structured across five targeted batches, implemented sequentially with zero intermediate testing or build checks.

---

## 2. Improvements Catalog & Technical Contracts

### Batch 1: Reverse Proxy Header Sanitization, Upstream Fallback & Response Assembly (Imp 271–276)
- **Imp-271: Header Allowlist Filter** (`src-tauri/src/proxy/handlers/common/header_allowlist_filter.rs`)
  Filters forwarded request headers against an allowlist, preventing credential leakage or internal header contamination upstream.
- **Imp-272: Upstream Non-Streaming Response Chunk Assembler** (`src-tauri/src/proxy/upstream/response_chunk_assembler.rs`)
  Reassembles fragmented non-streaming HTTP upstream response bodies with size-capping guards to prevent memory exhaustion.
- **Imp-273: User-Agent Header Normalizer** (`src-tauri/src/proxy/handlers/common/user_agent_normalizer.rs`)
  Normalizes client User-Agent headers into standard Google Cloud Code / Antigravity signatures required by edge endpoints.
- **Imp-274: Upstream Endpoint Weight Matrix** (`src-tauri/src/proxy/upstream/endpoint_weight_matrix.rs`)
  Dynamically assigns weighted priority scores to upstream endpoints based on stability, latency, and recent error rates.
- **Imp-275: Compression & Content-Encoding Guard** (`src-tauri/src/proxy/handlers/common/compression_guard.rs`)
  Validates Content-Encoding headers (`gzip`, `br`, `deflate`) and provides decompression fallback when upstream compresses unexpectedly.
- **Imp-276: Upstream Transport Metrics Collector** (`src-tauri/src/proxy/upstream/transport_metrics_collector.rs`)
  Records low-level transport latency milestones (TCP connect, TLS handshake, TTFB) across upstream endpoints.

### Batch 2: Auto-Switcher Score Serialization, Cooldown Watchdog & Headroom Ratios (Imp 277–282)
- **Imp-277: Candidate Score JSON Snapshotter** (`src-tauri/src/modules/auto_switcher/score_snapshot.rs`)
  Captures candidate account ranking breakdowns into structured JSON snapshots for UI scoreboard inspection and diagnostic logging.
- **Imp-278: Account Cooldown Expiration Watchdog** (`src-tauri/src/modules/auto_switcher/cooldown_watchdog.rs`)
  Maintains an asynchronous watchdog timer triggering candidate re-evaluation as soon as a cooling account becomes eligible again.
- **Imp-279: Weekly Quota-to-Time Headroom Ratio Calculator** (`src-tauri/src/modules/auto_switcher/weekly_headroom_ratio.rs`)
  Calculates normalized quota-to-time ratio ($Q_{weekly} / T_{remaining}$) to favor accounts with disproportionately high weekly quota relative to elapsed time.
- **Imp-280: Target Model Quota Filter** (`src-tauri/src/modules/auto_switcher/model_quota_filter.rs`)
  Ensures candidate accounts have specific model quota headroom before auto-switching when targeting models like Gemini 2.0 or Claude.
- **Imp-281: Switch Audit Reason Sanitizer** (`src-tauri/src/modules/auto_switcher/switch_reason_sanitizer.rs`)
  Scrubs access tokens, full email strings, and sensitive IDs from switch audit records before writing to SQLite or emitting UI events.
- **Imp-282: Candidate Account List Deduplicator** (`src-tauri/src/modules/auto_switcher/candidate_dedup.rs`)
  Eliminates duplicate candidate entries resulting from multiple binding references, retaining the record with highest composite score.

### Batch 3: Split-SQLite Schema Integrity, Foreign Key Watchdogs & Checkpoints (Imp 283–288)
- **Imp-283: SQLite Schema Integrity Gate** (`src-tauri/src/modules/db_orm/schema_integrity_gate.rs`)
  Validates table existence, column nullability, and primary key constraints across split databases during boot.
- **Imp-284: Foreign Key Watchdog Inspector** (`src-tauri/src/modules/db_orm/foreign_key_watchdog.rs`)
  Periodically executes `PRAGMA foreign_key_check;` to identify and alert on orphaned records in prompt and task history tables.
- **Imp-285: Dynamic WAL Checkpoint Scheduler** (`src-tauri/src/modules/db_orm/checkpoint_scheduler.rs`)
  Adapts WAL checkpoint cadence based on write velocity, scheduling passive checkpoints during high load and full checkpoints during quiet periods.
- **Imp-286: Platform Memory-Mapped I/O Tuner** (`src-tauri/src/modules/db_orm/mmap_tuner.rs`)
  Sets optimal `PRAGMA mmap_size` based on 32-bit vs 64-bit platform address space limits to maximize read throughput.
- **Imp-287: Query Execution Timeout Guard** (`src-tauri/src/modules/db_orm/query_timeout_guard.rs`)
  Hooks into SQLite progress callbacks to abort long-running queries that exceed execution time limits.
- **Imp-288: Split Database File Size Watchdog** (`src-tauri/src/modules/db_orm/db_size_watchdog.rs`)
  Tracks on-disk size for all split database files (`repo_prompts.db`, `errors.db`, `task_history.db`) to trigger proactive retention pruning.

### Batch 4: Multi-Instance Keyring Bypass, Process Health & Workspace Storage (Imp 289–294)
- **Imp-289: Mock Keyring Marker Writer** (`src-tauri/src/modules/instance/mock_keyring_writer.rs`)
  Writes mock keychain flags and `--password-store=basic` markers into instance data directories to prevent OS keyring prompt popups.
- **Imp-290: Lightweight Process Vitality Probe** (`src-tauri/src/modules/instance/process_vitality_probe.rs`)
  Verifies PID liveness using targeted OS kernel APIs (`OpenProcess` on Windows, `kill(pid, 0)` on Unix) without heavy sysinfo scans.
- **Imp-291: Workspace Storage Migrator & Re-anchor** (`src-tauri/src/modules/instance/workspace_storage_migrator.rs`)
  Re-maps and preserves IDE project workspace state and session caches when cloning instances across profile directories.
- **Imp-292: Abandoned Debugging Port Reclaimer** (`src-tauri/src/modules/instance/port_reclaim_sentinel.rs`)
  Reclaims assigned debugging ports in the port allocation matrix when an instance terminates ungracefully.
- **Imp-293: Modular Instance Launch Argv Builder** (`src-tauri/src/modules/instance/launch_argv_builder.rs`)
  Constructs clean, validated argument lists for spawning child IDE instances with sandbox isolation flags and workspace paths.
- **Imp-294: Singleton Session Lockfile Reclaimer** (`src-tauri/src/modules/instance/session_file_reclaimer.rs`)
  Removes lingering Chromium and VS Code singleton lockfiles prior to launch to prevent startup blockades.

### Batch 5: Desktop React UI & UX Component Modernization (Imp 295–300)
- **Imp-295: Dark-Glass Progress Bar Capsule** (`src/components/common/ProgressBarCapsule.tsx`)
  Reusable progress capsule supporting milestone checkpoints, threshold color transitions (emerald, amber, rose), and smooth width animations.
- **Imp-296: Portaled Non-Clipping Tooltip** (`src/components/common/TooltipPortal.tsx`)
  Renders floating tooltips via `createPortal` with coordinate calculation to guarantee visibility across modals and tables.
- **Imp-297: High-Efficiency Number & Token Formatter** (`src/utils/numberFormatter.ts`)
  Formats token counts, byte tallies, and request totals with compact SI prefixes (`1.2k`, `45.8M`, `3.1B`) and configurable decimal precision.
- **Imp-298: Search & Filter Toolbar Primitive** (`src/components/common/SearchFilterBar.tsx`)
  Accessible search bar combining clear button, filter trigger, and platform-aware keyboard shortcut (`Cmd+K` / `Ctrl+K`).
- **Imp-299: Multi-Target Click Outside Hook** (`src/hooks/useClickOutside.ts`)
  React hook detecting clicks and touch events outside a referenced element with multi-ref support to dismiss dropdowns and modals.
- **Imp-300: Standard Empty State Placeholder Card** (`src/components/common/EmptyStateCard.tsx`)
  Clean empty state placeholder providing visual feedback and primary call-to-action when lists or search results are empty.

---

## 3. Operational Invariants
1. **Zero Testing & Build Checking**: Strictly prohibited during implementation.
2. **Exclusively GitMap Search**: Ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
3. **Strictly Relative Paths**: No absolute filesystem paths in specs, plans, code comments, or commits.
4. **No Silent Errors**: All fallible operations route to `AppError` or `useErrorStore`.
