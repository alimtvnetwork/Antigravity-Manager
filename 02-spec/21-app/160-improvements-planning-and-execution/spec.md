# Specification: 30 Improvements Planning and Execution (Improvements 91–120)

- **Sequence ID**: `160-improvements-planning-and-execution`
- **Specification Path**: `02-spec/21-app/160-improvements-planning-and-execution/spec.md`
- **Plan Path**: `.ai-memory/plans/pending/160-improvements-planning-and-execution.md`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/160-improvements-planning-and-execution/`
- **Scope**: Next 30 concrete improvements (Improvements 91–120) across REST API Workbench, Split-SQLite Data Explorer, Auto-Switcher Adaptive Quota, Reverse Proxy Gateway Analytics, and Desktop React UI Accessibility.
- **Discipline**: Zero testing and zero build checking mode during execution. Sequential completion with immediate state transitions.

---

## 1. Executive Summary & Strategic Intent
This specification details the architecture, file boundaries, and implementation steps for Improvements 91 to 120 in the Antigravity-Manager ecosystem. Following the completion of Improvements 01–90 in sequence folders 157, 158, and 159, this batch focuses on exposing developer tools, database forensics, adaptive quota guards, reverse proxy stream telemetry, and accessible desktop React interactions.

---

## 2. 30 Improvements Architecture Breakdown

### Batch 1: DB REST API Workbench & OpenAPI Documentation Engine (Imp 91–96)
- **Imp-91: OpenAPI Spec Generator** (`src-tauri/src/proxy/server/admin_openapi.rs`)
  Generates a dynamic OpenAPI v3.1 specification JSON describing all proxy routes (`/v1/models`, `/v1/chat/completions`, `/api/prompts`, `/api/instances`, `/api/accounts`, `/api/doctor`).
- **Imp-92: Interactive REST Try-It-Out Endpoint Dispatcher** (`src-tauri/src/proxy/server/admin_rest_runner.rs`)
  Implements `/api/rest/execute` proxy test runner allowing secure internal execution of API requests with timing telemetry (DNS, handshake, TTFB).
- **Imp-93: Multi-Language Snippet Synthesizer** (`src/utils/codeSnippetGenerator.ts`)
  Generates copy-ready code snippets in `cURL`, `Python (requests)`, `TypeScript (fetch)`, `Rust (reqwest)`, and `Go (net/http)` for any API endpoint.
- **Imp-94: Environment Profile Resolver** (`src-tauri/src/proxy/server/admin_env.rs`)
  Exposes active environment configuration (`/api/env/info`) providing gateway host, active proxy port, cloudflared tunnel status, and node identity.
- **Imp-95: Request History & Replay Subsystem** (`src-tauri/src/modules/task_history_db/rest_history.rs`)
  Adds REST API test request logging in SQLite with replay capabilities.
- **Imp-96: Live JSON Schema Parameter Validator** (`src/utils/schemaValidator.ts`)
  Client-side recursive validator ensuring user inputs conform to expected JSON schemas prior to transmission.

### Batch 2: Split-SQLite Data Explorer & Table Forensics (Imp 97–102)
- **Imp-97: Split-DB Schema Inspection Service** (`src-tauri/src/modules/db_orm/schema_inspector.rs`)
  Exposes database introspection querying table names, columns, datatypes, and indices across all decoupled SQLite databases.
- **Imp-98: SQL Safety Execution Guard** (`src-tauri/src/modules/db_orm/safety_guard.rs`)
  Intercepts raw user queries and enforces read-only execution when safety mode is enabled (blocking `DROP`, `DELETE`, `UPDATE`, `ALTER`, `TRUNCATE`).
- **Imp-99: SQLite Table Row Pagination Engine** (`src-tauri/src/modules/db_orm/pagination.rs`)
  Generic paginated query runner with bound cursor offsets (`LIMIT ? OFFSET ?`), total row count calculation, and dynamic sorting.
- **Imp-100: Single-Click Database Snapshot Dumper** (`src-tauri/src/modules/db_orm/dump.rs`)
  Implements portable SQL schema and data export (`export_database_to_sql_dump`) to generate clean backup dumps.
- **Imp-101: Foreign Key Navigation Resolver** (`src-tauri/src/modules/db_orm/fk_resolver.rs`)
  Identifies outbound and inbound foreign key references between tables across split databases.
- **Imp-102: Instant Column Statistics Analyzer** (`src-tauri/src/modules/db_orm/statistics.rs`)
  Computes distribution metrics (row count, non-null count, distinct count, min, max, avg) for any chosen table column.

### Batch 3: Auto-Switcher Adaptive Quota & Scoring Primacy (Imp 103–108)
- **Imp-103: Adaptive Dynamic Quota Poller Interval** (`src-tauri/src/modules/auto_switcher/daemon.rs`)
  Scales polling interval dynamically: 60s when healthy (>50%), 30s when caution (<=20%), 15s when critical (<=10%).
- **Imp-104: Quota Jitter Shield** (`src-tauri/src/modules/auto_switcher/jitter.rs`)
  Introduces ±10% pseudo-randomized jitter to polling loops to prevent synchronization storms across multi-machine clusters.
- **Imp-105: Proportional Fallback Score Calculator** (`src-tauri/src/modules/auto_switcher/scoring.rs`)
  Explicit proportional scoring fallback when zero 100% accounts exist: `(Q_4h / 100.0) * weekly_score`.
- **Imp-106: Cooldown Expiry Countdown Helper** (`src-tauri/src/modules/auto_switcher/cooldown_timer.rs`)
  Calculates exact remaining cooldown duration in seconds and humanized format for switch-restricted accounts.
- **Imp-107: Candidate Score Reason Breakdown Formatter** (`src-tauri/src/modules/auto_switcher/breakdown.rs`)
  Generates structured explanation of score factors (readiness, quota, status, active multiplier, liveness).
- **Imp-108: Dual-Key Sibling Conflict Checker** (`src-tauri/src/modules/auto_switcher/conflict_checker.rs`)
  Detects and reports potential email/ID cross-instance conflicts across all active sandbox profiles.

### Batch 4: Reverse Proxy Gateway & Token Analytics (Imp 109–114)
- **Imp-109: Upstream Latency Benchmark Reporter** (`src-tauri/src/proxy/server/admin_latency.rs`)
  Exposes comparative latency metrics across upstream providers (Google Daily, Google Sandbox, Anthropic Claude).
- **Imp-110: Thinking Budget Clamping Widget Backend** (`src-tauri/src/proxy/server/admin_thinking.rs`)
  Exposes REST endpoint to view and update runtime thinking token limits (0 to 32,768 tokens).
- **Imp-111: Token Cost Estimator Calculator** (`src-tauri/src/proxy/token_stats/calculator.rs`)
  Calculates approximate USD expenditure based on model token rates (Flash vs Pro vs Claude).
- **Imp-112: In-Flight Stream Connection Counter** (`src-tauri/src/proxy/server/admin_streams.rs`)
  Exposes active SSE stream counter and throughput telemetry for live monitoring.
- **Imp-113: Curl Command Generator for Failed Requests** (`src-tauri/src/proxy/handlers/common/curl_gen.rs`)
  Generates reproducible `curl` CLI commands from incoming rejected requests for fast debugging.
- **Imp-114: Upstream Response Status Code Mapper** (`src-tauri/src/proxy/handlers/common/status_mapper.rs`)
  Standardizes upstream HTTP error codes into clean canonical JSON error responses.

### Batch 5: Desktop React UI & UX Accessibility Polish (Imp 115–120)
- **Imp-115: Global Command Palette Registry** (`src/utils/commandPaletteRegistry.ts`)
  Command registry with fuzzy search support for navigation actions, instance controls, and database tools.
- **Imp-116: Modal Focus Trap & Escape Guard Hook** (`src/hooks/useModalFocusTrap.ts`)
  React hook managing DOM focus traps, previous focus restoration, and Escape dismissal for accessible dialogs.
- **Imp-117: Splitter Rail localStorage Persistence** (`src/hooks/usePersistentSplitter.ts`)
  Reusable hook storing and restoring split panel widths across app sessions.
- **Imp-118: Real-time Quota Burn Velocity Meter** (`src/utils/quotaVelocity.ts`)
  Computes hourly quota consumption velocity (%/hr) based on recent prompt timestamps.
- **Imp-119: Formatted Audit Data Exporter** (`src/utils/auditExport.ts`)
  Client-side export helper converting audit log records into clean CSV, TSV, and Markdown tables.
- **Imp-120: Interactive Cloudflared URL Capsule Helper** (`src/utils/tunnelHelpers.ts`)
  Parses and formats Cloudflared tunnel URLs with quick copy and QR code generation helpers.

---

## 3. Non-Negotiable Operational Constraints
1. **Zero Testing & Build Checking**: No `cargo test`, `cargo build`, `npm test`, or `npm run build` during execution.
2. **Search Primacy**: Strict ban on raw `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`. All search performed via GitMap.
3. **Relative Paths**: All references use repository-relative Git paths.
4. **No Silent Errors**: Fallible operations propagate through `AppError` or `useErrorStore`.
