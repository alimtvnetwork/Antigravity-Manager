# Specification: 30 Improvements Planning and Execution (Improvements 151–180)

- **Sequence ID**: `162-improvements-planning-and-execution`
- **Specification Path**: `02-spec/21-app/162-improvements-planning-and-execution/spec.md`
- **Plan Path**: `.ai-memory/plans/pending/162-improvements-planning-and-execution.md`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/162-improvements-planning-and-execution/`
- **Scope**: Next 30 concrete improvements (Improvements 151–180) across Multi-Instance Profile Sandbox Isolation, Reverse Proxy Stream Resilience, Split-SQLite Database Maintenance, Auto-Switcher Candidate Selection, and Desktop React UI Modernization.
- **Discipline**: Zero testing and zero build checking mode during execution. Sequential completion with immediate state transitions.

---

## 1. Executive Summary & Strategic Intent
This specification details the architecture, file boundaries, and implementation steps for Improvements 151 to 180 in the Antigravity-Manager ecosystem. Following the completion of Improvements 01–150 in sequence folders 157, 158, 159, 160, and 161, this batch focuses on profile cloning isolation, stream boundary normalizers, SQLite WAL watchdog checkpoints, candidate tier multipliers, and reusable accessible UI primitives.

---

## 2. 30 Improvements Architecture Breakdown

### Batch 1: Multi-Instance Profile Sandbox Isolation & Keyring Security (Imp 151–156)
- **Imp-151: Multi-Instance Local Storage Scrubber** (`src-tauri/src/modules/instance/storage_scrubber.rs`)
  Deep-sanitizes `storage.json` across cloned instances, stripping previous machine telemetry tokens, installation UUIDs, and service credentials.
- **Imp-152: Ephemeral Sandbox Profile Purger** (`src-tauri/src/modules/instance/sandbox_purger.rs`)
  Removes stale temporary sandbox test folders (`.antigravity-sandbox-*`, `.test-profile-*`) with conscious PID and locked file checks.
- **Imp-153: Keyring Mock Mode Arguments Injector** (`src-tauri/src/modules/instance/keyring_flags.rs`)
  Injects OS keyring isolation flags (`--password-store=basic`, `--use-mock-keychain`) into headless or sandboxed IDE spawn invocations.
- **Imp-154: Instance Workspace Continuity Detector** (`src-tauri/src/modules/instance/workspace_continuity.rs`)
  Validates that `workspaceStorage` paths remain linked to the target repository folder during profile clone operations.
- **Imp-155: Cross-Instance Port Collision Detector** (`src-tauri/src/modules/instance/port_detector.rs`)
  Scans and identifies active CDP ports (`9222`, `9223`..) to allocate next free non-conflicting debugging port for launched profiles.
- **Imp-156: Profile Clone Integrity Checksum** (`src-tauri/src/modules/instance/clone_checksum.rs`)
  Verifies source and destination byte size parity for critical configuration files (`settings.json`, `keybindings.json`).

### Batch 2: Reverse Proxy Stream Resilience & Protocol Adapters (Imp 157–162)
- **Imp-157: Claude SSE Content Block Merger** (`src-tauri/src/proxy/handlers/claude/block_merger.rs`)
  Merges split Claude content blocks into unified text streams while preserving thinking block boundaries.
- **Imp-158: Upstream HTTP/2 Connection Pool Tuner** (`src-tauri/src/proxy/upstream/h2_tuner.rs`)
  Optimizes HTTP/2 multiplexing parameters (max concurrent streams = 64, adaptive window size = 1MB) for upstream Google APIs.
- **Imp-159: Stream Chunk Boundary Normalizer** (`src-tauri/src/proxy/handlers/common/boundary_normalizer.rs`)
  Splits and normalizes arbitrarily fragmented byte chunks into complete `data: {...}\n\n` SSE event boundaries.
- **Imp-160: Gemini System Instruction Sanitizer** (`src-tauri/src/proxy/handlers/gemini/system_sanitizer.rs`)
  Strips prohibited safety bypass strings and trims redundant whitespace in incoming system instructions.
- **Imp-161: Proxy Request Header Fingerprint Anonymizer** (`src-tauri/src/proxy/handlers/common/header_anonymizer.rs`)
  Strips sensitive client identification headers (`x-forwarded-for`, `user-agent` spoofing) prior to upstream forwarding.
- **Imp-162: Upstream Retry Delay Exponential Jitter** (`src-tauri/src/proxy/upstream/retry_jitter.rs`)
  Calculates exponential backoff delay with decorrelated jitter for rate-limited (HTTP 429) requests.

### Batch 3: Split-SQLite Database Maintenance & Schema Verification (Imp 163–168)
- **Imp-163: Split-DB Schema Migration Verification Gate** (`src-tauri/src/modules/db_orm/migration_gate.rs`)
  Inspects table column definitions against expected Diesel schemas, verifying all required columns exist prior to app boot.
- **Imp-164: SQLite WAL File Size Watchdog** (`src-tauri/src/modules/db_orm/wal_watchdog.rs`)
  Monitors `<dbname>.db-wal` byte size; triggers auto-checkpoint (`PRAGMA wal_checkpoint(TRUNCATE)`) if size exceeds 16MB.
- **Imp-165: SQLite Index Health Diagnostic Scanner** (`src-tauri/src/modules/db_orm/index_health.rs`)
  Runs `PRAGMA integrity_check` and `PRAGMA quick_check` on demand, returning structured health diagnostics.
- **Imp-166: Database Connection Pool Concurrency Limiter** (`src-tauri/src/modules/db_orm/pool_limiter.rs`)
  Enforces maximum concurrent read connections (max = 8) and serialized single-writer discipline per split database.
- **Imp-167: Auto-Increment Highwater Mark Inspector** (`src-tauri/src/modules/db_orm/highwater.rs`)
  Inspects `sqlite_sequence` to alert when table autoincrement IDs approach 2 billion limit.
- **Imp-168: Database Temp Store Memory Monitor** (`src-tauri/src/modules/db_orm/temp_store.rs`)
  Verifies `PRAGMA temp_store` memory allocation status and monitors transient file spillover.

### Batch 4: Auto-Switcher Multi-Account Candidate Selection (Imp 169–174)
- **Imp-169: Candidate Account Tier Weight Normalizer** (`src-tauri/src/modules/auto_switcher/tier_weights.rs`)
  Calculates normalized tier multipliers (Ultra = 4.0, Pro = 2.0, Free = 1.0) with user-configurable overrides.
- **Imp-170: Headroom Cycle Reset Boundary Inspector** (`src-tauri/src/modules/auto_switcher/reset_boundary.rs`)
  Detects when a weekly quota cycle has reset within the last 5 minutes, granting fresh account boost scores.
- **Imp-171: Sibling Machine Collision Exclusion Filter** (`src-tauri/src/modules/auto_switcher/sibling_filter.rs`)
  Excludes accounts actively leased by sibling machines in the multi-machine matrix from local selection candidate lists.
- **Imp-172: Account Quota Change Velocity Forecaster** (`src-tauri/src/modules/auto_switcher/quota_forecaster.rs`)
  Projects estimated time until account reaches low-quota threshold based on recent consumption slope.
- **Imp-173: Account Switch Reason Audit Code Formatter** (`src-tauri/src/modules/auto_switcher/reason_formatter.rs`)
  Formats machine-readable switch reasons (`LOW_QUOTA_PRIMARY`, `COOLDOWN_FALLBACK`, `MANUAL_OVERRIDE`, `PROACTIVE_STARTUP`).
- **Imp-174: Switch Settling Guard State Validator** (`src-tauri/src/modules/auto_switcher/settling_guard.rs`)
  Checks whether target instance is within the 10-second post-switch settling window, suppressing duplicate rotation triggers.

### Batch 5: Desktop React UI & UX Component Modernization (Imp 175–180)
- **Imp-175: Dark-Glass Accordion Panel Primitive** (`src/components/common/AccordionPanel.tsx`)
  Accessible dark-glass collapsible accordion component with keyboard navigation and smooth CSS grid transitions.
- **Imp-176: Dynamic Table Column Auto-Sizer** (`src/utils/tableAutoSizer.ts`)
  Calculates optimal column width distributions based on viewport width and cell content length.
- **Imp-177: High-Contrast Status Pill Tag Component** (`src/components/common/StatusPill.tsx`)
  Standardized micro-pill component rendering semantic colors (emerald, sky, amber, rose, zinc) with high-contrast text.
- **Imp-178: Accessible Tooltip Portal Wrapper** (`src/components/common/PortalTooltip.tsx`)
  Floating tooltip wrapper using React Portals with screen boundary collision detection and dark-glass styling.
- **Imp-179: Live Log Stream Monospace Auto-Scroller** (`src/hooks/useAutoScroll.ts`)
  React hook maintaining smart auto-scroll anchored to bottom unless user manually scrolls up to inspect previous logs.
- **Imp-180: Copyable Text Monospace Chip** (`src/components/common/CopyableChip.tsx`)
  Compact monospace pill with one-click copy to clipboard, visual checkmark feedback, and hover highlight.

---

## 3. Non-Negotiable Operational Constraints
1. **Zero Testing & Build Checking**: No `cargo test`, `cargo build`, `npm test`, or `npm run build` during execution.
2. **Search Primacy**: Strict ban on raw `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`. All search performed via GitMap.
3. **Relative Paths**: All references use repository-relative Git paths.
4. **No Silent Errors**: Fallible operations propagate through `AppError` or `useErrorStore`.
