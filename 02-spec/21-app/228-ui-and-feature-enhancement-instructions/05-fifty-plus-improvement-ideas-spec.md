# Specification 228: Architectural Catalog — 50+ System, Database, Proxy, Prompt & UI Improvements

- **Specification Slug**: `05-fifty-plus-improvement-ideas-spec`
- **Document Path**: `02-spec/21-app/228-ui-and-feature-enhancement-instructions/05-fifty-plus-improvement-ideas-spec.md`
- **Master Plan**: `.ai-memory/plans/pending/228-ui-and-feature-enhancement-instructions.md`
- **Subtask Plan**: `.ai-memory/plans/subtasks/228-ui-and-feature-enhancement-instructions/03-subtask-dispatch-and-chat-spec.md`
- **Related Specs**:
  - `02-spec/21-app/228-ui-and-feature-enhancement-instructions/01-architecture-spec.md`
  - `02-spec/21-app/228-ui-and-feature-enhancement-instructions/02-component-spec.md`
  - `02-spec/21-app/228-ui-and-feature-enhancement-instructions/03-dispatch-and-verification-spec.md`
  - `02-spec/21-app/228-ui-and-feature-enhancement-instructions/04-chat-section-architecture-spec.md`
- **Target Release Version**: Continuous Improvement Backlog
- **Author**: Spec Author 03
- **Status**: ARCHITECTURAL CATALOG & ROADMAP

---

## 1. Executive Summary & Catalog Methodology

This document establishes a grounded, actionable catalog of 50 comprehensive system improvements across Antigravity Manager. Each improvement is deeply anchored in the existing codebase architecture (`src-tauri/src/modules/`, `src/components/`, `src/pages/`, `src/stores/`, SQLite database schemas, and protocol pipeline stages).

The catalog is organized into five core functional categories containing exactly 10 improvements each:
1. **Instances Architecture & Lifecycle Management** (Items 01–10)
2. **Prompts Architecture, FIFO Queue & Tree View** (Items 11–20)
3. **Speed Database, SQLite Engine & Persistence** (Items 21–30)
4. **Proxy Gateway, Pipeline Engine & Protocol Adapters** (Items 31–40)
5. **Settings, Theming, Accessibility & UI/UX** (Items 41–50)

Each entry provides the Problem Statement, Grounded Architectural Solution, Target Files, and Expected Impact.

---

## 2. Category 1: Instances Architecture & Lifecycle Management (Items 01–10)

### 01. Default Instance Cloning Invariant with Deep Configuration Merging
- **Problem**: Creating a new instance previously defaulted to a blank profile, losing existing extensions, workspace storage (`workspaceStorage`), and customized keybindings from the primary installation.
- **Solution**: Enforce default cloning from the `"Default"` instance in `src/pages/instances/useInstancePage.ts` via `copy_instance_with_options`, applying deep-merge JSON synchronization on `settings.json` while isolating instance IDs and ports.
- **Target Files**: `src/pages/instances/CreateEditDialogs.tsx`, `src-tauri/src/modules/instance.rs`.
- **Impact**: Zero configuration loss when spinning up ephemeral instances for testing.

### 02. Dynamic Port Allocation & Conflict Prevention Engine
- **Problem**: Fixed CDP debugging ports (9222, 9223) cause startup collisions when multiple instances are launched concurrently or when external Chromium instances are open.
- **Solution**: Implement an atomic TCP port probe (`find_available_port(range: 9222..9300)`) in `src-tauri/src/modules/ide_process.rs` before launching the IDE process with `--remote-debugging-port`.
- **Target Files**: `src-tauri/src/modules/ide_process.rs`, `src-tauri/src/modules/instance.rs`.
- **Impact**: Eliminates `ERR_PORT_IN_USE` startup crashes across multi-instance setups.

### 03. CDP Heap Snapshot & Memory Leak Sentinel
- **Problem**: Long-running Antigravity IDE instances accumulate large object graphs in memory during heavy subagent runs, eventually leading to out-of-memory (OOM) renderer crashes.
- **Solution**: Introduce a background sentinel in Rust that polls CDP `Performance.getMetrics` every 60 seconds; triggers garbage collection via `HeapProfiler.collectGarbage` when JS heap exceeds 2.5 GB.
- **Target Files**: `src-tauri/src/modules/ide_cdp.rs`, `src-tauri/src/modules/instance_monitor.rs`.
- **Impact**: Prevents unmonitored IDE renderer crashes during multi-hour autonomous tasks.

### 04. Multi-Root Workspace Discovery & Path Aliasing
- **Problem**: When instances open multi-root VS Code workspaces (`.code-workspace`), prompt tree views fail to associate relative paths correctly across differing root folders.
- **Solution**: Author a workspace parser in `src/utils/workspace.ts` that detects multi-root folder manifests and resolves relative paths against the active root folder context.
- **Target Files**: `src/components/instances/PromptTreeView/types.ts`, `src/utils/workspace.ts`.
- **Impact**: Accurate file link resolution across complex mono-repo structures.

### 05. Dynamic PID Watchdog & Clean Process Termination Ladder
- **Problem**: Terminating an instance sometimes leaves orphaned child processes (`language_server`, `rg.exe`, `git.exe`) consuming CPU and locking files.
- **Solution**: Implement a tree-based process termination ladder in `src-tauri/src/modules/process_killer.rs` that walks child PIDs using Win32 `Toolhelp32Snapshot` / Unix `pgrep` and cleanly sends `SIGTERM` before `SIGKILL`.
- **Target Files**: `src-tauri/src/modules/process_killer.rs`, `src-tauri/src/modules/instance.rs`.
- **Impact**: Zero orphan process leaks on instance teardown.

### 06. Cross-Instance Prompt Tree Migration & Handover
- **Problem**: Operators cannot transfer an active conversation or prompt queue from an exhausted instance to a freshly provisioned instance without manual copy-paste.
- **Solution**: Add `migrate_conversation_to_instance(conv_id, source_inst, target_inst)` in `src-tauri/src/modules/repo_db/` that transactionally migrates active prompts, backups, and metadata.
- **Target Files**: `src-tauri/src/modules/repo_db/mod.rs`, `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`.
- **Impact**: Seamless zero-downtime workload rebalancing across instance profiles.

### 07. Real-Time Per-Instance Resource Telemetry (CPU / RAM / Disk)
- **Problem**: The instance card displays only status badges (Running/Stopped) without insight into resource consumption or throttling bottlenecks.
- **Solution**: Expose `get_instance_resource_usage(instance_id)` returning CPU%, RSS memory, and workspace disk size, visualized via miniature sparkline charts inside `src/pages/instances/InstanceCard.tsx`.
- **Target Files**: `src-tauri/src/modules/sysinfo.rs`, `src/pages/instances/InstanceCard.tsx`.
- **Impact**: Operators instantly spot runaway subagents or frozen instances.

### 08. Sandbox Isolation Profiles for Unsafe / Untrusted Workspaces
- **Problem**: Running agentic tasks on untrusted open-source repositories carries risks of malicious prompt injections or environment variable exfiltration.
- **Solution**: Introduce a "Sandbox Profile" flag that runs instances with restricted filesystem permissions, isolated temp folders, and environment variable scrubs (`clean_env`).
- **Target Files**: `src-tauri/src/modules/instance.rs`, `src/pages/instances/CreateEditDialogs.tsx`.
- **Impact**: Defense-in-depth security when inspecting third-party code.

### 09. Hot-Reload Extension Synchronization Across Cloned Profiles
- **Problem**: Installing a new extension in the default profile does not automatically propagate to already cloned secondary instances.
- **Solution**: Provide a "Sync Extensions" action in `InstanceCard.tsx` that diffs `extensions.json` across profiles and creates NTFS symlinks or performs fast directory synchronization.
- **Target Files**: `src-tauri/src/modules/instance_sync.rs`, `src/pages/instances/InstanceCard.tsx`.
- **Impact**: Eliminates redundant manual extension installations across multiple profiles.

### 10. Instance Lifecycle Webhooks & Event Broadcasting
- **Problem**: External tools, automated test runners, or Slack/Telegram bots cannot react to instance state changes (e.g., instance crash, quota reached).
- **Solution**: Implement an outbound webhook dispatcher in `src-tauri/src/modules/webhooks.rs` supporting configurable POST endpoints for `instance:started`, `instance:stopped`, and `instance:crashed`.
- **Target Files**: `src-tauri/src/modules/webhooks.rs`, `src/pages/Settings/tabs/GeneralTab.tsx`.
- **Impact**: Enables integration into enterprise CI/CD and observability pipelines.

---

## 3. Category 2: Prompts Architecture, FIFO Queue & Tree View (Items 11–20)

### 11. Strict Monotonic FIFO Queue with Multi-Tier Anti-Starvation Scheduling
- **Problem**: Concurrent prompt injection requests from Telegram, CLI, and GUI can result in race conditions or priority inversion in SQLite.
- **Solution**: Enforce strict monotonic sequencing using `AUTOINCREMENT` integer IDs and `ORDER BY created_at ASC, id ASC` across all queue queries in `src-tauri/src/modules/repo_db/queue.rs`.
- **Target Files**: `src-tauri/src/modules/repo_db/queue.rs`, `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`.
- **Impact**: Absolute FIFO guarantee without queue starvation or turn inversion.

### 12. 4-Tier Origin Hierarchy Visualizer with Branch Glyphs
- **Problem**: In deeply nested conversation trees with multiple subagents, it is difficult to determine which subagent spawned which task.
- **Solution**: Implement a tree-line renderer in `src/components/instances/PromptTreeView/TreePanel.tsx` using branch glyphs (`↳`, `└──`) and the 4 origin tier badges (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`).
- **Target Files**: `src/components/instances/PromptTreeView/TreePanel.tsx`, `src/components/instances/PromptTreeView/tierClassification.ts`.
- **Impact**: Immediate, intuitive comprehension of multi-agent execution hierarchies.

### 13. Visual Prompt Diff Viewer Between Revisions & Rerun Turns
- **Problem**: When reviewing rerun nudges or edited prompts, operators have no visual indicator of what changed compared to previous turns.
- **Solution**: Integrate a fast, zero-dependency token diff engine in `src/components/instances/PromptTreeView/markdown.tsx` highlighting additions (green) and deletions (red) between conversation turns.
- **Target Files**: `src/components/instances/PromptTreeView/PromptInstructionView.tsx`, `src/utils/diff.ts`.
- **Impact**: Rapid audit of prompt mutations and instruction refinement.

### 14. Automatic Markdown Table & Code Block Formatter
- **Problem**: Agent responses containing unaligned markdown tables or unclosed code fences cause visual artifacts and layout breakage in the prompt view.
- **Solution**: Introduce a sanitizing pre-processor in `src/components/instances/PromptTreeView/markdown.tsx` that balances code fences and normalizes markdown table column widths.
- **Target Files**: `src/components/instances/PromptTreeView/markdown.tsx`.
- **Impact**: Clean, pristine rendering of tabular and code-heavy assistant responses.

### 15. Interactive Attachment Thumbnail Gallery with Drag-Drop Reordering
- **Problem**: Attached images currently render as plain text links or simple list items without visual thumbnails or order manipulation.
- **Solution**: Build an interactive thumbnail strip in `src/components/instances/PromptTreeView/DetailHeader.tsx` displaying thumbnail previews with drag-and-drop reordering that updates `### Attachments:` in real time.
- **Target Files**: `src/components/instances/PromptTreeView/DetailHeader.tsx`, `src/components/instances/PromptTreeView/PromptImagePreviewModal.tsx`.
- **Impact**: Superior visual context management when composing multimodal prompts.

### 16. Prompt Checkpoint & Rollback State Machine
- **Problem**: If an agent executes an erroneous command or produces corrupt files, rolling back the conversation requires manually clearing transcript files.
- **Solution**: Author `checkpoint_conversation_state` in `src-tauri/src/modules/repo_db/` that creates a snapshot of the prompt tree and provides a one-click "Rollback to Here" action in `ConversationContextMenu.tsx`.
- **Target Files**: `src-tauri/src/modules/repo_db/conversation.rs`, `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`.
- **Impact**: Effortless undo and branch exploration during complex coding tasks.

### 17. Real-Time Token Budget Burndown Estimator
- **Problem**: Operators have no visibility into token consumption trends across a multi-turn conversation until upstream rate limits or quota caps are hit.
- **Solution**: Compute and display a token burndown graph in `src/components/instances/PromptTreeView/DetailTabs.tsx` showing prompt vs completion tokens per turn with warning thresholds at 80% and 95%.
- **Target Files**: `src/components/instances/PromptTreeView/DetailTabs.tsx`, `src/components/instances/PromptTreeView/types.ts`.
- **Impact**: Proactive quota governance preventing mid-task rate limit interruptions.

### 18. Prompt Execution Telemetry & Performance Metric Exporter (CSV/JSONL)
- **Problem**: Benchmarking agent performance and duration across differing models or prompt templates requires laborious manual log parsing.
- **Solution**: Add an "Export Telemetry" action in `ImportExportMenu.tsx` generating structured CSV/JSONL records capturing turn latency, tool call frequency, token counts, and error occurrences.
- **Target Files**: `src/components/instances/PromptTreeView/ImportExportMenu.tsx`, `src/components/instances/PromptTreeView/clipboard.ts`.
- **Impact**: Empirical data for evaluating agent velocity and model efficacy.

### 19. Automated Exponential Backoff Retry Ladder for Transient IDE Ingestion Failures
- **Problem**: When the IDE is momentarily busy compiling or running heavy tests, prompt injection calls can fail transiently with timeout errors.
- **Solution**: Wrap `sendPromptNow` and `enqueuePrompt` with a 3-stage retry ladder (200ms $\to$ 800ms $\to$ 2400ms) with jitter, logging each attempt before reporting failure.
- **Target Files**: `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`.
- **Impact**: Dramatically reduced prompt drop rates during high CPU load.

### 20. Centralized Cross-Project Prompt Template & Suffix Repository
- **Problem**: Custom verification templates and suffix prompts saved in one repository or instance are not shared across other workspace projects.
- **Solution**: Move template persistence from project-scoped storage to the global SQLite database (`app.db`), enabling universal access to shared prompt libraries across all workspaces.
- **Target Files**: `src-tauri/src/modules/repo_db/templates.rs`, `src/components/instances/PromptTreeView/SuffixTemplateModal.tsx`.
- **Impact**: Universal reuse of organizational verification standards and prompts.

---

## 4. Category 3: Speed Database, SQLite Engine & Persistence (Items 21–30)

### 21. WAL-Mode Checkpoint Optimization & Dynamic PRAGMA Auto-Tuning
- **Problem**: Under heavy concurrent writes from auto-switchers and prompt loggers, SQLite write-ahead log (`.db-wal`) files grow excessively, degrading query speed.
- **Solution**: Apply optimized PRAGMAs on database connection initialization:
  `PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA wal_autocheckpoint = 1000; PRAGMA busy_timeout = 5000;`.
- **Target Files**: `src-tauri/src/modules/db.rs`, `src-tauri/src/modules/repo_db/connection.rs`.
- **Impact**: 4x faster concurrent write throughput with bounded WAL file sizes.

### 22. Automated Vacuum & Page Defragmentation Scheduler During Idle Windows
- **Problem**: Frequent prompt insertions, updates, and deletions cause SQLite database fragmentation, increasing disk footprints over time.
- **Solution**: Implement an idle scheduler in `src-tauri/src/modules/db_maintenance.rs` that detects 10 minutes of complete system inactivity and runs `PRAGMA incremental_vacuum;` safely.
- **Target Files**: `src-tauri/src/modules/db_maintenance.rs`.
- **Impact**: Maintains minimal disk footprint without impacting active IDE operations.

### 23. Split-DB Migration Versioning Engine (`db_version` Table Schema)
- **Problem**: Database schema migrations across the three-tier split SQLite architecture (`app.db`, `installation.db`, `repo_prompts.db`) lack atomic migration ledger tracking.
- **Solution**: Author a unified migration runner that queries `CREATE TABLE IF NOT EXISTS db_version (version INTEGER PRIMARY KEY, applied_at TEXT)` and executes migrations inside explicit ACID transactions.
- **Target Files**: `src-tauri/src/modules/db_migrations.rs`.
- **Impact**: Bulletproof database schema upgrades without risk of partial migration corruption.

### 24. SQLite Full-Text Search (FTS5) Index on Prompt Content & Transcript Full Text
- **Problem**: Searching for specific instructions or keywords across thousands of prompt history turns currently requires slow `LIKE '%keyword%'` table scans.
- **Solution**: Create an FTS5 virtual table `active_prompts_fts` with triggers on INSERT/UPDATE/DELETE, supporting BM25 ranking and instant sub-10ms keyword searches.
- **Target Files**: `src-tauri/src/modules/repo_db/fts.rs`, `src/components/instances/PromptTreeView/usePromptTreeData.ts`.
- **Impact**: Instantaneous search across millions of historical prompt tokens.

### 25. Database Connection Pool Leak Detection & Auto-Recycling
- **Problem**: Unclosed SQLite transaction handles during unexpected thread panics can cause database lock timeouts (`database is locked: code 5`).
- **Solution**: Implement an RAII connection wrapper with lifetime tracking in Rust; logs warnings if a connection is held longer than 3,000ms and automatically recycles stale handles.
- **Target Files**: `src-tauri/src/modules/db_pool.rs`.
- **Impact**: Total eradication of persistent `database is locked` deadlocks.

### 26. Read-Only Database Replica Connection Pool for High-Frequency UI Telemetry
- **Problem**: UI telemetry polling (active turn status, queue count) shares write locks with background queue inserters, causing occasional UI stutters.
- **Solution**: Open separate read-only connection handles (`sqlite3_open_v2` with `SQLITE_OPEN_READONLY`) dedicated exclusively to frontend queries.
- **Target Files**: `src-tauri/src/modules/repo_db/mod.rs`.
- **Impact**: Zero contention between UI rendering queries and background database writes.

### 27. Composite Index Optimization on `active_prompts`
- **Problem**: Queue retrieval queries filtering by instance and status execute sequential scans when datasets exceed 10,000 records.
- **Solution**: Create composite indexes:
  `CREATE INDEX IF NOT EXISTS idx_prompts_inst_status_created ON active_prompts (instance_id, status, created_at ASC, id ASC);`.
- **Target Files**: `src-tauri/src/modules/repo_db/schema.rs`.
- **Impact**: Guaranteed $O(\log N)$ queue fetch times regardless of database scale.

### 28. Crash-Resilient ACID Prompt State Transactions with Savepoints
- **Problem**: If an instance crashes midway through dispatching a multi-step prompt sequence, the database can remain in a partially dispatched status.
- **Solution**: Wrap multi-prompt dispatch operations in SQLite `SAVEPOINT` transactions; automatically rolls back to the initial state if any step fails.
- **Target Files**: `src-tauri/src/modules/repo_db/queue.rs`.
- **Impact**: Absolute transactional integrity: prompt turns are either fully dispatched or cleanly restored.

### 29. Automated Rolling Database Backup Generator with SHA-256 Checksum Verification
- **Problem**: Power outages or abnormal OS shutdowns can corrupt the active SQLite database file.
- **Solution**: Implement a background backup generator using SQLite Online Backup API (`sqlite3_backup_step`); generates timestamped `.bak` files with SHA-256 integrity checksums daily.
- **Target Files**: `src-tauri/src/modules/db_backup.rs`.
- **Impact**: Effortless zero-data-loss recovery in disaster scenarios.

### 30. Orphan Conversation Record & Temporary Image Garbage Collector
- **Problem**: Deleted instances leave orphan records in `repo_prompts.db` and abandoned screenshot files in the cache directory.
- **Solution**: Provide an automated cleanup routine that scans for dangling foreign keys and unreferenced `.png` files, safely reclaiming disk space.
- **Target Files**: `src-tauri/src/modules/cleanup.rs`, `src/pages/Settings/tabs/GeneralTab.tsx`.
- **Impact**: Prevents unbounded disk cache growth over months of continuous development.

---

## 5. Category 4: Proxy Gateway, Pipeline Engine & Protocol Adapters (Items 31–40)

### 31. Universal Thinking Budget Backfilling Across OpenAI, Claude, and Gemini
- **Problem**: Different upstream protocols handle reasoning tokens differently (Gemini uses `thinkingConfig.thinkingBudget`, Anthropic uses `thinking.budget_tokens`, OpenAI uses `reasoning_effort`).
- **Solution**: Centralize thinking parameter normalization inside the unified AGM pipeline stage (`src-tauri/src/proxy/pipeline/thinking.rs`), backfilling appropriate defaults across all 4 protocol adapters.
- **Target Files**: `src-tauri/src/proxy/pipeline/thinking.rs`, `src-tauri/src/proxy/adapters/`.
- **Impact**: Seamless switching between reasoning models without payload rejection.

### 32. Prefix Stability Cache for Upstream Prompt KV-Caching Optimization
- **Problem**: Dynamic timestamps or randomized IDs placed early in system prompts invalidate upstream provider KV prompt caching, resulting in higher latency and doubled token costs.
- **Solution**: Restructure system prompt assembly in the pipeline stage to guarantee absolute prefix stability, pushing all dynamic timestamps and turn counters to the terminal payload block.
- **Target Files**: `src-tauri/src/proxy/pipeline/prefix_stability.rs`.
- **Impact**: Up to 50% lower upstream token billing and 3x faster time-to-first-token via KV caching.

### 33. Dynamic Rate-Limit Backpressure Handler with Adaptive Retry-After Parsing
- **Problem**: Upstream HTTP 429 errors currently cause immediate request abortion rather than graceful backpressure pausing.
- **Solution**: Implement an intelligent backpressure queue in the proxy gateway that parses `Retry-After` headers and pauses outbound worker threads until the rate limit window resets.
- **Target Files**: `src-tauri/src/proxy/rate_limit.rs`.
- **Impact**: Smooth handling of burst traffic without dropped prompts or failed subagent turns.

### 34. Multimodal Base64 Image Payload Normalizer & Protocol Divergence Sanitizer
- **Problem**: Some protocols require `data:image/png;base64,...` URL schemas, while others expect raw binary base64 and explicit MIME type fields.
- **Solution**: Author a universal image adapter in `src-tauri/src/proxy/adapters/multimodal.rs` that parses any inbound image structure and transforms it into the exact format expected by the target provider.
- **Target Files**: `src-tauri/src/proxy/adapters/multimodal.rs`.
- **Impact**: Universal compatibility for screenshot and image attachments across all models.

### 35. Streaming SSE Chunk Multiplexer with Zero-Copy Buffer Pools
- **Problem**: String allocations and JSON re-serialization on every Server-Sent Event (SSE) chunk introduce CPU overhead during high-speed streaming.
- **Solution**: Utilize `bytes::BytesMut` and zero-copy string slice parsing in `src-tauri/src/proxy/sse_stream.rs`, streaming deltas straight to the client socket.
- **Target Files**: `src-tauri/src/proxy/sse_stream.rs`.
- **Impact**: 60% lower CPU utilization and ultra-responsive real-time token rendering.

### 36. Dynamic Upstream Failover & Circuit Breaker Across Multiple Gateway Routes
- **Problem**: If an upstream AI provider experiences an outage, ongoing agent tasks fail completely.
- **Solution**: Implement a circuit breaker in `src-tauri/src/proxy/circuit_breaker.rs` that detects 3 consecutive upstream HTTP 5xx errors and automatically routes subsequent requests to a configured fallback model.
- **Target Files**: `src-tauri/src/proxy/circuit_breaker.rs`, `src/pages/Settings/tabs/ProxyTab.tsx`.
- **Impact**: Uninterrupted agent operation even during major cloud provider outages.

### 37. Request & Response Header Sanitizer for Sensitive Credentials & Tokens
- **Problem**: Upstream proxy error logs risk capturing authorization headers or private API tokens in plaintext.
- **Solution**: Enforce strict header sanitization in `src-tauri/src/proxy/sanitizer.rs`, masking all `Authorization`, `x-api-key`, and `cookie` headers before writing to debug logs.
- **Target Files**: `src-tauri/src/proxy/sanitizer.rs`.
- **Impact**: Zero credential leakage across application logs and error diagnostics.

### 38. Gemini 2.0 Thinking Protocol Budget Filtering & Schema Adapter
- **Problem**: Google Gemini 2.0 Flash models reject payloads containing legacy `maxOutputTokens` when `thinkingConfig` is enabled with specific budget values.
- **Solution**: Author a specialized Gemini adapter in the pipeline that filters conflicting parameters and ensures exact compliance with Gemini 2.0 schema specifications.
- **Target Files**: `src-tauri/src/proxy/adapters/gemini.rs`.
- **Impact**: Flawless execution on next-generation Google Gemini reasoning models.

### 39. Mock Upstream Adapter for Offline Integration Testing & Transient Sandboxing
- **Problem**: Running automated end-to-end integration tests requires live API keys and generates network costs.
- **Solution**: Build a deterministic mock adapter (`MockProviderAdapter`) in Rust that returns pre-recorded SSE streams and tool execution fixtures without internet access.
- **Target Files**: `src-tauri/src/proxy/adapters/mock.rs`, `src-tauri/src/bin/agm/test_runner.rs`.
- **Impact**: Fast, zero-cost, 100% reproducible offline CI/CD test execution.

### 40. Upstream Latency Histogram & Prometheus-Compatible Metrics Endpoint
- **Problem**: Operators cannot diagnose whether task slowness originates in local IDE processing or remote AI model latency.
- **Solution**: Expose an internal metrics route (`/metrics`) tracking latency percentiles (p50, p95, p99), time-to-first-token (TTFT), and token generation rates per model.
- **Target Files**: `src-tauri/src/proxy/metrics.rs`.
- **Impact**: Professional observability and bottleneck identification.

---

## 6. Category 5: Settings, Theming, Accessibility & UI/UX Experience (Items 41–50)

### 41. WCAG AAA Contrast Ratio Compliance Across Dark and Light Theming Tokens
- **Problem**: Certain muted text tokens (`text-slate-400` on white, or `text-slate-600` on dark glass) fail WCAG 2.1 AA/AAA contrast guidelines.
- **Solution**: Audit and update color tokens in `src/components/common/selectedState.ts` and `src/App.css` to guarantee minimum 7:1 contrast for all text and interactive glyphs.
- **Target Files**: `src/components/common/selectedState.ts`, `src/App.css`.
- **Impact**: Exceptional readability and accessibility for all developers in any lighting environment.

### 42. Interactive Keyboard Shortcut Customization Matrix with Conflict Detection
- **Problem**: Global hotkeys (e.g., `N` for Send, `Q` for Enqueue, `R` for Rerun) are hardcoded, causing collisions with international keyboards or custom IDE shortcuts.
- **Solution**: Author a shortcut customization panel in `src/pages/Settings/tabs/KeybindingsTab.tsx` with live key-capture listeners and real-time collision detection.
- **Target Files**: `src/pages/Settings/tabs/KeybindingsTab.tsx`, `src/utils/keyboard.ts`.
- **Impact**: Complete ergonomic flexibility for power users.

### 43. Segmented Capsule Geometry Standardization Across All Navigation Tabs
- **Problem**: Sub-tabs in Settings and Prompt Tree View occasionally mix rectangular cards with pill capsules, creating visual inconsistency.
- **Solution**: Enforce strict house capsule styling (`rounded-full`, shared border, subtle divider lines, dark-glass backdrop) across every tab bar in the application.
- **Target Files**: `src/components/navbar/NavMenu.tsx`, `src/components/instances/PromptTreeView/DetailTabs.tsx`.
- **Impact**: Cohesive, world-class desktop visual aesthetics.

### 44. Inline Form Validation Engine with Contextual Dark-Glass Callouts
- **Problem**: Form errors (e.g. invalid instance name, duplicate port) previously triggered abrupt alert dialogs or were silently ignored.
- **Solution**: Implement an inline validation engine using subtle dark-glass callouts anchored directly beneath offending inputs with instant feedback on change.
- **Target Files**: `src/pages/instances/CreateEditDialogs.tsx`, `src/components/common/FormValidationHint.tsx`.
- **Impact**: Non-intrusive, user-friendly error guidance without modal disruption.

### 45. Universal 24-Hour Time Formatting Presets with Optional Seconds & Timezone Display
- **Problem**: Different locales format dates inconsistently, mixing AM/PM and differing separators.
- **Solution**: Implement universal `formatTime24` and `formatDateTime24` in `src/utils/date.ts`, offering user toggles for showing/hiding seconds and timezone indicators.
- **Target Files**: `src/utils/date.ts`, `src/pages/Settings/tabs/GeneralTab.tsx`.
- **Impact**: Consistent, military/technical 24-hour time notation across all logs and tables.

### 46. Granular Privacy & Telemetry Opt-In/Opt-Out Switches
- **Problem**: Enterprise environments require strict isolation guarantees preventing any telemetry, crash reports, or update checks from pinging external servers.
- **Solution**: Add explicit, independent toggle switches in Settings: "Automatic Update Checks", "Crash Report Sync", and "Usage Analytics", honoring offline air-gapped environments.
- **Target Files**: `src/pages/Settings/tabs/GeneralTab.tsx`, `src-tauri/src/modules/telemetry.rs`.
- **Impact**: Full compliance with enterprise security and air-gapped policies.

### 47. Comprehensive Settings Import/Export as Encrypted JSON Bundle
- **Problem**: Migrating AGM configuration, custom prompts, and account profiles to a new workstation requires tedious manual re-entry.
- **Solution**: Provide an "Export Settings Bundle" action that encrypts configuration settings, custom templates, and credentials into an AES-GCM password-protected JSON bundle.
- **Target Files**: `src-tauri/src/modules/config_bundle.rs`, `src/pages/Settings/tabs/GeneralTab.tsx`.
- **Impact**: Instant, secure migration across developer machines.

### 48. Visual Live Theme Builder with Real-Time CSS Variable Previews
- **Problem**: Developers who want personalized color schemes currently have to manually edit source CSS files.
- **Solution**: Create a visual theme builder in Settings where users can adjust accent hues, border radii, and glass opacity with real-time DOM CSS variable reflection.
- **Target Files**: `src/pages/Settings/tabs/ThemeTab.tsx`, `src/utils/theme.ts`.
- **Impact**: High personalization and brand alignment for individual developers.

### 49. Eco / Low-Power Rendering Mode for Reduced Animation Overhead on Laptops
- **Problem**: Continuous CSS animations, glow effects, and 60 FPS canvas updates can accelerate battery drain on mobile laptops.
- **Solution**: Introduce an "Eco Mode" setting that detects battery power status and disables backdrop-blur filters, ambient glow pulses, and reduces animation framerates.
- **Target Files**: `src/App.tsx`, `src/stores/systemStore.ts`.
- **Impact**: Extended laptop battery runtime during intensive mobile coding sessions.

### 50. Headless CLI Parity Engine for 100% GUI Configuration Parity via `agm config`
- **Problem**: Certain GUI configurations (such as suffix templates or auto-switch thresholds) cannot be queried or updated via the CLI on headless Linux servers.
- **Solution**: Expand `agm config` CLI subcommands (`agm config get <key>`, `agm config set <key> <val>`, `agm config list`) to map 1:1 onto all underlying GUI settings.
- **Target Files**: `src-tauri/src/bin/agm/config_cmd.rs`, `src-tauri/src/modules/config.rs`.
- **Impact**: Complete operational parity between desktop GUI and headless CLI environments.
