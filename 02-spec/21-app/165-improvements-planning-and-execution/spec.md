# Specification: 30 Improvements Planning and Execution (Improvements 241–270)

- **Task**: Improvements Planning and Execution
- **Sequence**: 165
- **Slug**: `165-improvements-planning-and-execution`
- **Spec Directory**: `02-spec/21-app/165-improvements-planning-and-execution/`
- **Plan Path**: `.ai-memory/plans/pending/165-improvements-planning-and-execution.md`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/165-improvements-planning-and-execution/`

---

## 1. Executive Summary

This specification establishes the architectural contracts, structural definitions, and implementation guidelines for 30 concrete improvements (Improvements 241 to 270) across Antigravity-Manager. Following non-negotiable operational guidelines, all 30 improvements are structured across five targeted batches, implemented sequentially with zero intermediate testing or build checks.

---

## 2. Improvements Catalog & Technical Contracts

### Batch 1: Reverse Proxy Protocol Translation & IR Payload Normalization (Imp 241–246)
- **Imp-241: Canonical Role Token Mapper** (`src-tauri/src/proxy/handlers/common/canonical_role_mapper.rs`)
  Normalizes protocol-divergent role tokens (`developer`, `model`, `assistant`, `user`, `system`, `tool`) into canonical IR representations (`User`, `Model`, `System`, `Tool`) with robust fallback.
- **Imp-242: Upstream Dynamic DNS Resolver & Cache** (`src-tauri/src/proxy/upstream/dynamic_dns_resolver.rs`)
  Provides an in-memory DNS cache and fallback IP resolver for upstream AI endpoints (`generativelanguage.googleapis.com`, `api.anthropic.com`) to mitigate transient NXDOMAIN errors.
- **Imp-243: Deep JSON Payload Sanitizer** (`src-tauri/src/proxy/handlers/common/json_payload_sanitizer.rs`)
  Sanitizes incoming JSON payloads by stripping unescaped ASCII control characters and trailing commas before passing them to upstream adapters.
- **Imp-244: Upstream Circuit Breaker Statistics Recorder** (`src-tauri/src/proxy/upstream/circuit_breaker_stats.rs`)
  Maintains moving-window circuit breaker state (consecutive failures, half-open probe counts, backoff duration) for upstream AI endpoints.
- **Imp-245: UTF-8 Stream Boundary Guard** (`src-tauri/src/proxy/handlers/common/stream_boundary_guard.rs`)
  Guards SSE streaming buffers against splitting multibyte UTF-8 codepoints across chunk boundaries, holding trailing incomplete bytes until the next frame.
- **Imp-246: TLS Client Connector Configuration Helper** (`src-tauri/src/proxy/upstream/tls_fingerprint_config.rs`)
  Configures native client TLS parameters, ALPN protocol negotiation (`h2`, `http/1.1`), and cipher preferences for upstream connections.

### Batch 2: Auto-Switcher Adaptive Quota Scoring & Recovery Metrics (Imp 247–252)
- **Imp-247: Quota Velocity Exponential Moving Average Smoother** (`src-tauri/src/modules/auto_switcher/velocity_smoother.rs`)
  Applies exponential smoothing (EMA) to account token consumption velocity to avoid premature throttling from short-lived prompt bursts.
- **Imp-248: Multi-Tier Account Failover Ladder** (`src-tauri/src/modules/auto_switcher/failover_ladder.rs`)
  Provides structured fallback progression (`Primary -> Secondary -> Sandbox -> EmergencyQuota`) when top-ranked candidate accounts are depleted.
- **Imp-249: Hourly Switch Entropy & Oscillation Guard** (`src-tauri/src/modules/auto_switcher/hourly_entropy_guard.rs`)
  Calculates normalized Shannon entropy over recent account transitions to detect and halt pathological rapid switching loops.
- **Imp-250: Per-Instance Account Cooldown Interval Map** (`src-tauri/src/modules/auto_switcher/instance_cooldown_map.rs`)
  Thread-safe in-memory map tracking active account cooldown timers independently per instance profile.
- **Imp-251: Comprehensive Account Readiness Evaluator** (`src-tauri/src/modules/auto_switcher/readiness_evaluator.rs`)
  Composite readiness checker verifying OAuth token validity, distributed cluster lease expiration, and 4h rolling quota headroom.
- **Imp-252: Decorrelated Jitter Backoff Calculator** (`src-tauri/src/modules/auto_switcher/jittered_backoff.rs`)
  Calculates randomized exponential backoff with decorrelated jitter to prevent thundering-herd reconnect storms upon quota reset.

### Batch 3: Split-SQLite Concurrency, Indexing & Pragma Tuners (Imp 253–258)
- **Imp-253: PRAGMA Cache-Size Memory Tuner** (`src-tauri/src/modules/db_orm/cache_size_tuner.rs`)
  Calculates optimal negative KiB PRAGMA `cache_size` values based on database file size and available RAM.
- **Imp-254: Query Plan Index Coverage Inspector** (`src-tauri/src/modules/db_orm/index_usage_inspector.rs`)
  Parses `EXPLAIN QUERY PLAN` output to detect unindexed table scans on prompt history and task tables.
- **Imp-255: Explicit Write-Ahead Log Flusher** (`src-tauri/src/modules/db_orm/write_ahead_flush.rs`)
  Coordinates passive and restart WAL checkpoints to guarantee dirty page flushing before instance shutdown or account switch.
- **Imp-256: Freelist Threshold Vacuum Optimizer** (`src-tauri/src/modules/db_orm/vacuum_optimizer.rs`)
  Evaluates database freelist page count and triggers incremental vacuum only when reclaimable space exceeds configured thresholds.
- **Imp-257: Transaction Latency & Health Logger** (`src-tauri/src/modules/db_orm/transaction_logger.rs`)
  Diagnostic transaction latency timer issuing warning telemetry for slow database operations (>50ms).
- **Imp-258: Split-Database Connection Pool Metrics Collector** (`src-tauri/src/modules/db_orm/connection_pool_metrics.rs`)
  Tracks real-time connection pool state (active leases, queue depth, peak usage) across split databases.

### Batch 4: Multi-Instance Sandboxing, Process Lifecycle & Workspace Storage (Imp 259–264)
- **Imp-259: Orphan Socket & Named Pipe Sweeper** (`src-tauri/src/modules/instance/orphan_socket_sweeper.rs`)
  Cleans up stale Unix domain sockets, temporary pipe locks, and orphaned debugger handles in instance sandbox folders.
- **Imp-260: Instance Rapid Crash-Loop Quarantine Guard** (`src-tauri/src/modules/instance/crash_loop_guard.rs`)
  Detects instances crashing within 5 seconds of launch and places the profile in temporary quarantine to avoid process churn.
- **Imp-261: Cloned Profile Telemetry Storage Cleaner** (`src-tauri/src/modules/instance/telemetry_storage_cleaner.rs`)
  Cleans machine GUIDs, device fingerprints, and telemetry sessions from cloned `storage.json` files.
- **Imp-262: IDE Window Position & Monitor Restorer** (`src-tauri/src/modules/instance/window_state_restorer.rs`)
  Saves and restores window coordinates, monitor bindings, and maximization states across instance relaunches.
- **Imp-263: Launch Environment Variable Sanitizer** (`src-tauri/src/modules/instance/launch_env_sanitizer.rs`)
  Sanitizes child process environment variables, stripping leaking parent variables (`NODE_OPTIONS`, `VSCODE_PID`, etc.).
- **Imp-264: Profile Clone Cryptographic Manifest Generator** (`src-tauri/src/modules/instance/profile_clone_manifest.rs`)
  Generates and checks SHA-256 manifests of essential configuration files during instance profile duplication.

### Batch 5: Desktop React UI & UX Component Modernization (Imp 265–270)
- **Imp-265: Status Badge Component Primitive** (`src/components/common/StatusBadge.tsx`)
  Versatile dark-glass status badge supporting `running`, `queued`, `idle`, `error`, and `success` with animated pulse indicators.
- **Imp-266: Micro Copy-to-Clipboard Button** (`src/components/common/CopyIconButton.tsx`)
  Compact icon button providing one-click copy with animated checkmark transition and fallback toast notification.
- **Imp-267: High-Precision Duration Formatter Utility** (`src/utils/durationFormatter.ts`)
  Formats millisecond durations into compact strings (`12ms`, `3.4s`, `2m 15s`, `4h 12m`) with configurable precision.
- **Imp-268: Accessible Dark-Glass Accordion Section** (`src/components/common/AccordionSection.tsx`)
  Accessible collapsible panel with ARIA attributes, keyboard support, and dark-glass styling.
- **Imp-269: Ergonomic Debounced Callback Hook** (`src/hooks/useDebouncedCallback.ts`)
  React hook providing debounced function execution with configurable leading/trailing execution flags and cleanup.
- **Imp-270: Inline Monospace Code Badge** (`src/components/common/InlineCodeBadge.tsx`)
  Compact monospace badge for rendering code symbols, file paths, and model names with optional click-to-copy.

---

## 3. Operational Invariants
1. **Zero Testing & Build Checking**: Strictly prohibited during implementation.
2. **Exclusively GitMap Search**: Ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
3. **Strictly Relative Paths**: No absolute filesystem paths in specs, plans, code comments, or commits.
4. **No Silent Errors**: All fallible operations route to `AppError` or `useErrorStore`.
