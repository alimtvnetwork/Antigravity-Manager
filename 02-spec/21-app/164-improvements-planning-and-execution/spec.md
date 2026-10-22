# Specification: 30 Improvements Planning and Execution (Improvements 211–240)

- **Task**: Improvements Planning and Execution
- **Sequence**: 164
- **Slug**: `164-improvements-planning-and-execution`
- **Spec Directory**: `02-spec/21-app/164-improvements-planning-and-execution/`
- **Plan Path**: `.ai-memory/plans/pending/164-improvements-planning-and-execution.md`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/164-improvements-planning-and-execution/`

---

## 1. Executive Summary

This specification establishes the roadmap, architectural contracts, and step-by-step execution tasks for 30 concrete improvements (Improvements 211 to 240) across Antigravity-Manager. Following strict operational directives, all tasks are implemented sequentially across five targeted batches without intermediate builds or test execution.

---

## 2. Improvements Catalog & Structural Architecture

### Batch 1: Reverse Proxy Streaming Protocol Resilience & SSE Formatting (Imp 211–216)
- **Imp-211: SSE Comment & Event Stream Stripper** (`src-tauri/src/proxy/handlers/common/sse_comment_filter.rs`)
  Filters out internal telemetry and diagnostic ping comments from downstream client streams while preserving required SSE data events.
- **Imp-212: Upstream HTTP/2 Stream Window Recovery** (`src-tauri/src/proxy/upstream/h2_window_recovery.rs`)
  Monitors HTTP/2 flow-control window exhaustion and gracefully falls back to HTTP/1.1 chunked transfer when upstream window stalls.
- **Imp-213: Stream Frame Size Clamping Sentinel** (`src-tauri/src/proxy/handlers/common/frame_clamper.rs`)
  Ensures SSE chunks and message payloads stay within safe buffer boundaries (e.g. 64KB max per frame), chunking oversized tokens safely.
- **Imp-214: Multi-Model Upstream Timeout Matrix** (`src-tauri/src/proxy/upstream/timeout_matrix.rs`)
  Provides model-specific streaming connection and TTFT (time-to-first-token) timeout configuration (e.g. 180s for Claude 3.7 Thinking, 60s for Flash).
- **Imp-215: Downstream SSE Framing Validator** (`src-tauri/src/proxy/handlers/common/sse_framing_validator.rs`)
  Validates that every emitted SSE block adheres to the W3C EventSource standard (`data: ...\n\n`) preventing parser halts in web clients.
- **Imp-216: Upstream Endpoint Health Decay Scorer** (`src-tauri/src/proxy/upstream/health_decay.rs`)
  Calculates decaying health scores (0.0 to 1.0) for upstream endpoints based on recent latency percentiles and error rates.

### Batch 2: Auto-Switcher Multi-Metric Quota & Headroom Scoring (Imp 217–222)
- **Imp-217: Quota Period Reset Phase Detector** (`src-tauri/src/modules/auto_switcher/reset_phase.rs`)
  Categorizes quota reset status into 4 distinct lifecycle phases (`FRESH`, `ACTIVE`, `HEADROOM_STRETCH`, `EXHAUSTED`).
- **Imp-218: Consecutive Rotation Frequency Brake** (`src-tauri/src/modules/auto_switcher/rotation_brake.rs`)
  Prevents accounts from being switched more than N times in an hour (e.g. max 5 switches/hr), guarding against pathological bouncing.
- **Imp-219: Multi-Instance Binding Affinity Scorer** (`src-tauri/src/modules/auto_switcher/binding_affinity.rs`)
  Calculates sticky affinity scores for accounts previously bound to an instance, reducing unnecessary credential writes when quota is equal.
- **Imp-220: Rolling 5-Hour vs Weekly Quota Gap Calculator** (`src-tauri/src/modules/auto_switcher/quota_gap.rs`)
  Quantifies disparity between 5h rolling window and 7-day weekly quota to select accounts with balanced headroom across both dimensions.
- **Imp-221: Account Switch History Ring Buffer** (`src-tauri/src/modules/auto_switcher/switch_history_ring.rs`)
  Maintains in-memory circular ring buffer of the last 50 switch events with nano-second resolution timestamps and transition metadata.
- **Imp-222: Candidate Selection Tie-Breaker Randomizer** (`src-tauri/src/modules/auto_switcher/tie_breaker.rs`)
  Introduces deterministic pseudo-random jitter for candidate accounts with identical composite scores, distributing load evenly.

### Batch 3: Split-SQLite Maintenance, WAL Compaction & Deadlock Prevention (Imp 223–228)
- **Imp-223: Database File Lock Contention Detector** (`src-tauri/src/modules/db_orm/lock_detector.rs`)
  Tests for exclusive file locks without blocking, logging early diagnostic warnings before queries encounter `SQLITE_BUSY`.
- **Imp-224: PRAGMA Compile-Options Introspection Scanner** (`src-tauri/src/modules/db_orm/compile_options.rs`)
  Inspects SQLite compile-time feature flags (`ENABLE_FTS5`, `ENABLE_JSON1`, `THREADSAFE`) verifying required extensions at runtime.
- **Imp-225: Transaction Rollback Guard Utility** (`src-tauri/src/modules/db_orm/rollback_guard.rs`)
  Provides an RAII transaction rollback guard that safely cancels pending transactions if panics or errors occur before explicit commit.
- **Imp-226: Split-DB Page Allocation Fragmentation Estimator** (`src-tauri/src/modules/db_orm/fragmentation.rs`)
  Estimates database B-Tree fragmentation ratio based on freelist count and page count, advising when vacuum is beneficial.
- **Imp-227: Database Migration Checksum Verifier** (`src-tauri/src/modules/db_orm/migration_checksum.rs`)
  Verifies cryptographic SHA-256 checksums of applied schema migrations against canonical DDL models to detect manual tampering.
- **Imp-228: Connection Busy Handler Configurator** (`src-tauri/src/modules/db_orm/busy_handler.rs`)
  Configures customized busy handler callbacks with progressive sleeping intervals for high-contention split database writes.

### Batch 4: Multi-Instance Profile State, Settings & Session Guards (Imp 229–234)
- **Imp-229: Profile Extension State Sanitizer** (`src-tauri/src/modules/instance/extension_sanitizer.rs`)
  Purges stale extension runtime locks and temporary socket files from cloned instance extension directories.
- **Imp-230: Multi-Profile Port Collision Matrix** (`src-tauri/src/modules/instance/port_matrix.rs`)
  Maintains an in-memory allocation matrix of assigned CDP ports across active instances to prevent conflicting binding attempts.
- **Imp-231: Cloned Profile Machine Identity Reseeder** (`src-tauri/src/modules/instance/identity_reseeder.rs`)
  Re-seeds `storage.json` telemetry identities with unique machine and device UUIDs following instance cloning.
- **Imp-232: Workspace Storage Hash Path Resolver** (`src-tauri/src/modules/instance/workspace_hash.rs`)
  Computes deterministic FNV-1a directory hashes matching Antigravity IDE's internal `workspaceStorage/<hash>` naming convention.
- **Imp-233: Instance Health Check Probe Runner** (`src-tauri/src/modules/instance/health_probe.rs`)
  Performs lightweight HTTP loopback checks against `http://127.0.0.1:<cdp_port>/json/version` to verify instance readiness.
- **Imp-234: Instance State Persistence Snapshotter** (`src-tauri/src/modules/instance/state_snapshot.rs`)
  Creates lightweight JSON state snapshots of instance configs, bound accounts, and PID records for crash recovery.

### Batch 5: Desktop React UI & UX Component Modernization (Imp 235–240)
- **Imp-235: Dark-Glass Segmented Pill Group Primitive** (`src/components/common/SegmentedPillGroup.tsx`)
  Accessible dark-glass segmented pill button group with seamless keyboard navigation and animated active indicator.
- **Imp-236: Model Icon Badge Component** (`src/components/common/ModelIconBadge.tsx`)
  Compact badge rendering distinctive model family icons and colors (Gemini, Claude, GPT, DeepSeek) with high contrast.
- **Imp-237: Byte Size Formatter Utility** (`src/utils/byteFormatter.ts`)
  Formats raw byte counts into human-readable strings (`12.4 KB`, `1.8 MB`, `2.4 GB`) with configurable precision.
- **Imp-238: Compact Loading Spinner Pill** (`src/components/common/LoadingPill.tsx`)
  Monospace loading pill with subtle pulse animation, customizable label, and dark-glass styling.
- **Imp-239: Toast Notification Store Wrapper** (`src/hooks/useToastNotification.ts`)
  Lightweight React hook providing typed toast triggers (`success`, `error`, `warning`, `info`) with auto-dismissal.
- **Imp-240: Accessible Keyboard Shortcut Tag** (`src/components/common/KbdShortcut.tsx`)
  Compact keyboard shortcut badge rendering platform-aware keys (`Cmd` / `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`).

---

## 3. Non-Negotiable Operational Constraints
1. **Zero Testing & Build Checking**: No `cargo test`, `cargo build`, `npm test`, or `npm run build` during execution.
2. **Search Primacy**: Strict ban on raw `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`. All search performed via GitMap.
3. **Relative Paths**: All references use repository-relative Git paths.
4. **No Silent Errors**: Fallible operations propagate through `AppError` or `useErrorStore`.
