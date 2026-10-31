# Specification: 30 Improvements Planning and Execution (Improvements 451–480)

- **Sequence**: 172
- **Feature Directory**: `02-spec/21-app/172-improvements-planning-and-execution/`
- **Plan File**: `.ai-memory/plans/pending/172-improvements-planning-and-execution.md`
- **Subtasks**: `.ai-memory/plans/subtasks/172-improvements-planning-and-execution/`
- **Status**: [APPROVED]
- **Operational Discipline**: Zero testing and zero build checking during implementation. Strict sequential batch execution. Exclusively relative paths.

---

## 1. Executive Summary

This specification defines the 172nd sequence of 30 grounded improvements (Improvements 451 to 480) spanning five core Antigravity Manager subsystems:
1. **Proxy Upstream Resilience, Compression & Chunk Framing** (Imp 451–456)
2. **Auto-Switcher Cooldown, Quota Velocity & Selection Jitter** (Imp 457–462)
3. **Split-SQLite Index Integrity, Page Budget & Query Profiling** (Imp 463–468)
4. **Multi-Instance Sandboxed Config, Port Reclamation & Launch Telemetry** (Imp 469–474)
5. **Desktop React UI Modernization, Feedback & Component Primitives** (Imp 475–480)

All tasks adhere to the strict house rules: no silent errors, protocol-agnostic pipeline primacy, strictly relative repository paths, and zero testing/build checks during active execution.

---

## 2. Detailed Improvement Breakdown

### Batch 1: Proxy Upstream Resilience, Compression & Chunk Framing (Imp 451–456)
- **Imp-451: Stream Brotli Decompression Fallback Filter** (`src-tauri/src/proxy/upstream/brotli_decompressor.rs`)
  - Decompresses `br` encoded upstream responses when client accepts uncompressed payloads, safeguarding stream parser pipelines.
- **Imp-452: Upstream Connection Maximum Age Sentinel** (`src-tauri/src/proxy/upstream/connection_age_sentinel.rs`)
  - Tracks upstream HTTP/2 connection establishment timestamps and triggers smooth connection rotation after 300s to avoid stale NAT bindings.
- **Imp-453: SSE Chunk Stream Boundary Validator** (`src-tauri/src/proxy/handlers/common/chunk_boundary_validator.rs`)
  - Validates streaming chunk transitions and byte boundaries, detecting truncated frame markers and malformed data prefixes.
- **Imp-454: Response Content-Type Header Normalizer** (`src-tauri/src/proxy/handlers/common/response_content_type_matcher.rs`)
  - Matches and normalizes upstream `Content-Type` headers (`text/event-stream; charset=utf-8`, `application/json`) for downstream clients.
- **Imp-455: Claude Stream Custom Stop Sequence Detector** (`src-tauri/src/proxy/handlers/claude/stop_sequence_matcher.rs`)
  - Detects client-specified custom stop sequences in active Claude streaming deltas and safely terminates generation upon match.
- **Imp-456: Adaptive Connection Pool Dynamic Sizer** (`src-tauri/src/proxy/upstream/dynamic_pool_sizer.rs`)
  - Adjusts maximum idle connections per upstream host dynamically (5 to 50) based on real-time request concurrency.

### Batch 2: Auto-Switcher Cooldown, Quota Velocity & Selection Jitter (Imp 457–462)
- **Imp-457: Short-Term Quota Velocity Spike Dampener** (`src-tauri/src/modules/auto_switcher/velocity_spike_dampener.rs`)
  - Dampens transient 1-minute quota consumption velocity spikes to avoid premature account switching during bursty multi-agent runs.
- **Imp-458: Stepped Exponential Cooldown Decay** (`src-tauri/src/modules/auto_switcher/cooldown_decay_stepper.rs`)
  - Reduces cooldown durations step-wise for accounts exhibiting healthy quota resets, enabling faster candidate re-entry.
- **Imp-459: Non-Deterministic Candidate Selection Jitter** (`src-tauri/src/modules/auto_switcher/selection_jitter_allocator.rs`)
  - Injects subtle proportional score jitter (+/-0.5%) into identical candidate scores to eliminate deterministic rotation loops.
- **Imp-460: Multi-Instance Lease Conflict Claim Coordinator** (`src-tauri/src/modules/auto_switcher/instance_lease_claim.rs`)
  - Coordinates profile lease acquisition with atomic reservation checks to prevent concurrent sibling instances from double-booking.
- **Imp-461: Account Model Capability Tier Filter** (`src-tauri/src/modules/auto_switcher/account_tier_filter.rs`)
  - Filters candidate accounts by required model capabilities (e.g. Claude 3.5 Sonnet, Gemini 2.0 Flash) before score evaluation.
- **Imp-462: Post-Switch Request Health Tracker** (`src-tauri/src/modules/auto_switcher/switch_recovery_tracker.rs`)
  - Monitors the first 3 prompt dispatches to a newly switched instance, verifying zero transport errors or quota rejections.

### Batch 3: Split-SQLite Index Integrity, Page Budget & Query Profiling (Imp 463–468)
- **Imp-463: Composite Index Coverage Verifier** (`src-tauri/src/modules/db_orm/index_coverage_verifier.rs`)
  - Audits table indexes against frequent multi-column query patterns in `active_prompts` and `task_history` split databases.
- **Imp-464: Dynamic Page Cache Memory Budget Controller** (`src-tauri/src/modules/db_orm/page_budget_controller.rs`)
  - Calculates optimal SQLite `PRAGMA cache_size` based on system available memory and active split database count.
- **Imp-465: Dynamic Busy Timeout Scaler** (`src-tauri/src/modules/db_orm/busy_timeout_scaler.rs`)
  - Scales `PRAGMA busy_timeout` between 1,000ms and 10,000ms according to recent lock contention history.
- **Imp-466: Foreign Key Action Rule Validator** (`src-tauri/src/modules/db_orm/foreign_key_action_validator.rs`)
  - Verifies `ON DELETE CASCADE` and `ON UPDATE NO ACTION` foreign key rules across table definitions.
- **Imp-467: Query Latency Histogram Collector** (`src-tauri/src/modules/db_orm/query_latency_histogram.rs`)
  - Bucketizes query execution times into fixed latency histograms (<1ms, 1-5ms, 5-20ms, >20ms) for runtime telemetry.
- **Imp-468: WAL Frame Header Integrity Inspector** (`src-tauri/src/modules/db_orm/wal_header_inspector.rs`)
  - Validates low-level 32-byte WAL file headers, salt values, and page checksums before reading WAL frames.

### Batch 4: Multi-Instance Sandboxed Config, Port Reclamation & Launch Telemetry (Imp 469–474)
- **Imp-469: IDE Launch Argv Security Sanitizer** (`src-tauri/src/modules/instance/launch_argv_sanitizer.rs`)
  - Strips dangerous command-line arguments (`--inspect`, `--no-sandbox`, host-specific user data paths) before spawning instances.
- **Imp-470: CDP Remote Port Reclaim Watchdog** (`src-tauri/src/modules/instance/port_reclaim_watchdog.rs`)
  - Detects when terminated IDE processes close their remote debugging ports and immediately marks port leases as available.
- **Imp-471: Profile Structural Fingerprint Hasher** (`src-tauri/src/modules/instance/profile_fingerprint_hasher.rs`)
  - Computes SHA-256 hashes of profile configuration files to identify configuration drift across instance clones.
- **Imp-472: Instance Runtime Crash Frequency Counter** (`src-tauri/src/modules/instance/runtime_crash_counter.rs`)
  - Tracks abnormal crash events per instance profile within a 10-minute sliding window to trigger backoff delay ladders.
- **Imp-473: Window Layout Coordinates Anchor** (`src-tauri/src/modules/instance/window_layout_anchor.rs`)
  - Persists and staggers default window coordinates for multi-instance launches to prevent window stacking.
- **Imp-474: Sandboxed Storage Obsolete File Cleaner** (`src-tauri/src/modules/instance/sandboxed_storage_cleaner.rs`)
  - Cleans obsolete backup stores and temporary database snapshots from instance directories.

### Batch 5: Desktop React UI Modernization, Feedback & Component Primitives (Imp 475–480)
- **Imp-475: Contiguous Dark-Glass Linear Progress Bar** (`src/components/common/LinearProgressBar.tsx`)
  - Smooth progress bar with percentage label, gradient fill, and subtle animated striped effect.
- **Imp-476: Dark-Glass Empty State Placeholder** (`src/components/common/EmptyStatePlaceholder.tsx`)
  - Reusable empty state view with icon, title, description, and primary action button with dark-glass styling.
- **Imp-477: Debounced Side Effect Hook** (`src/hooks/useDebouncedEffect.ts`)
  - React hook executing debounced side-effects with automatic cancellation on dependency change or component unmount.
- **Imp-478: Contiguous Horizontal Badge Group** (`src/components/common/BadgePillGroup.tsx`)
  - Horizontal badge list container with overflow collapse (`+N more`) and click-to-expand flyout.
- **Imp-479: High-Precision Duration Formatter Utility** (`src/utils/durationFormatter.ts`)
  - Formats milliseconds/seconds into compact, localized duration strings (`14s`, `3m 20s`, `2h 10m`).
- **Imp-480: Key-Value Detail Row Component** (`src/components/common/KeyValueDetailRow.tsx`)
  - Structured key-value item row with label, value, optional copy action, and dark-glass borders.

---

## 3. Implementation Verification Criteria
- All 30 improvements implemented cleanly in their respective Rust modules and React components.
- Zero testing and zero build checking executed during implementation as instructed.
- All code changes strictly utilize repository-relative paths and adhere to error handling conventions (`AppError` in Rust, `trackWarning`/`captureError` in TS).
