# Specification: 30 Improvements Planning and Execution (Improvements 121–150)

- **Sequence ID**: `161-improvements-planning-and-execution`
- **Specification Path**: `02-spec/21-app/161-improvements-planning-and-execution/spec.md`
- **Plan Path**: `.ai-memory/plans/pending/161-improvements-planning-and-execution.md`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/161-improvements-planning-and-execution/`
- **Scope**: Next 30 concrete improvements (Improvements 121–150) across Prompt Tree Forensics & Attachment Handling, Auto-Switcher Multi-Cluster Lease Guard, Reverse Proxy Streaming Resilience & Compression, Split-SQLite Maintenance & Integrity Recovery, and Desktop React UI & UX Component Modernization.
- **Discipline**: Zero testing and zero build checking mode during execution. Sequential completion with immediate state transitions.

---

## 1. Executive Summary & Strategic Intent
This specification details the architecture, file boundaries, and implementation steps for Improvements 121 to 150 in the Antigravity-Manager ecosystem. Following the completion of Improvements 01–120 in sequence folders 157, 158, 159, and 160, this batch focuses on prompt media deduplication, multi-cluster machine lease safety, streaming TCP disconnect guards, live SQLite VACUUM backup snapshots, and accessible UI component utilities.

---

## 2. 30 Improvements Architecture Breakdown

### Batch 1: Prompt Tree Advanced Forensics & Rich Media Attachment Handling (Imp 121–126)
- **Imp-121: Multimodal Image Attachment Hash Resolver** (`src-tauri/src/modules/repo_db/attachment_hasher.rs`)
  Computes SHA-256 fingerprint of prompt image blobs/files to eliminate duplicate stored attachments across repeated prompt rounds.
- **Imp-122: Conversation Turn Token Counter Estimator** (`src/utils/turnTokenEstimator.ts`)
  Estimates turn-by-turn prompt token counts (user prompt, thoughts, tool outputs) in the React UI before sending.
- **Imp-123: Prompt Tree Branch ASCII Tree Visualizer** (`src-tauri/src/modules/repo_db/ascii_tree.rs`)
  Generates clean monospace text-based ASCII tree representations (`├──`, `└──`, `│`) of conversation turns and subagents for headless CLI and logs.
- **Imp-124: Subagent Hierarchy Nesting Depth Guard** (`src-tauri/src/modules/repo_db/depth_guard.rs`)
  Prevents runaway recursive subagent chains by validating and capping max nesting depth (max depth = 10).
- **Imp-125: Rich Prompt Template Macro Engine** (`src/utils/promptTemplateMacro.ts`)
  Client-side variable interpolator replacing tokens (`{{workspace}}`, `{{account}}`, `{{branch}}`, `{{timestamp}}`) in rerun and nudge templates.
- **Imp-126: Attachment Filetype Detection & Sanitizer** (`src-tauri/src/modules/repo_db/attachment_mime.rs`)
  Validates image attachments against strict MIME whitelist (image/png, image/jpeg, image/webp, image/gif) and flags corrupt headers.

### Batch 2: Auto-Switcher Multi-Cluster Lease & Profile Guard (Imp 127–132)
- **Imp-127: Machine Lease Expiry Guard** (`src-tauri/src/modules/auto_switcher/lease_guard.rs`)
  Verifies active machine leases, preventing profile stealing when another cluster machine holds a valid lease (`< 300s`).
- **Imp-128: Account Quota History Ring Buffer** (`src-tauri/src/modules/auto_switcher/quota_ring_buffer.rs`)
  Stores circular memory buffer of recent 20 quota samples per account to track rate of drop and detect rapid exhaustion.
- **Imp-129: Switch Event Audit Log Enricher** (`src-tauri/src/modules/auto_switcher/audit_enricher.rs`)
  Enriches switch events with host machine alias, OS platform, trigger reason, and previous/target quota snapshots.
- **Imp-130: Stale Cooldown Cleaner** (`src-tauri/src/modules/auto_switcher/cooldown_cleaner.rs`)
  Periodically purges expired account cooldown timestamps from memory to avoid hash map bloat.
- **Imp-131: Quota Health Score Categorizer** (`src/utils/quotaHealthCategory.ts`)
  Frontend utility classifying accounts into visual health bands (`Pristine: 100%`, `Healthy: 50-99%`, `Caution: 20-49%`, `Depleted: <20%`).
- **Imp-132: Auto-Switch Throttle Brake** (`src-tauri/src/modules/auto_switcher/throttle_brake.rs`)
  Enforces minimum delay (60s) between consecutive auto-switches on the same instance to prevent rapid flip-flopping.

### Batch 3: Reverse Proxy Streaming Resilience & Compression (Imp 133–138)
- **Imp-133: SSE Stream Buffer Flusher** (`src-tauri/src/proxy/handlers/common/stream_flusher.rs`)
  Enforces periodic flush heartbeat comments (`: keepalive\n\n`) on idle SSE connections to bypass proxy gateway timeouts.
- **Imp-134: Dynamic Gzip/Brotli Content Decompressor** (`src-tauri/src/proxy/handlers/common/decompressor.rs`)
  Safely transparently handles upstream compressed responses (gzip, deflate, zstd) during error inspection.
- **Imp-135: Client Disconnect Stream Abort Guard** (`src-tauri/src/proxy/handlers/common/stream_abort.rs`)
  Monitors downstream client TCP drops and immediately cancels upstream Tokio HTTP requests to conserve account quota.
- **Imp-136: Thinking Signature Re-Validator** (`src-tauri/src/proxy/thinking_store/signature_validator.rs`)
  Verifies thought block integrity signatures before injecting into conversation turns, filtering malformed cryptographic hashes.
- **Imp-137: Upstream Endpoint Health Probe Cache** (`src-tauri/src/proxy/upstream/probe_cache.rs`)
  Maintains in-memory TTL cache (60s) of upstream endpoint reachability to bypass dead endpoints without per-request connection delays.
- **Imp-138: Model Alias Translation Dictionary** (`src-tauri/src/proxy/common/model_alias.rs`)
  Canonical mapping dictionary mapping generic model names (`gemini-flash`, `claude-3-5-sonnet`, `gemini-pro`) to exact API identifiers.

### Batch 4: Split-SQLite Maintenance & Integrity Recovery (Imp 139–144)
- **Imp-139: SQLite VACUUM Into Backup Engine** (`src-tauri/src/modules/db_orm/vacuum_backup.rs`)
  Implements non-blocking `VACUUM INTO '<path>'` snapshot creation for live split databases without write locks.
- **Imp-140: Database Page Size & Cache Size Tuner** (`src-tauri/src/modules/db_orm/pragmas.rs`)
  Applies optimized high-performance pragmas (`cache_size = -64000`, `temp_store = MEMORY`, `mmap_size = 268435456`) on connection init.
- **Imp-141: SQLite Table Row Size Forensics** (`src-tauri/src/modules/db_orm/row_size.rs`)
  Analyzes average and maximum row byte sizes per table to identify bloat or large orphaned blobs.
- **Imp-142: Split-DB Table Truncate Cleaner** (`src-tauri/src/modules/db_orm/truncate.rs`)
  Safely truncates table records and resets autoincrement counters in SQLite (`DELETE FROM table; DELETE FROM sqlite_sequence...`).
- **Imp-143: Database File Lock Checker** (`src-tauri/src/modules/db_orm/lock_checker.rs`)
  Tests if a SQLite database file is locked by another external process prior to attempting schema migrations.
- **Imp-144: SQLite JSON Column Extractor Helper** (`src-tauri/src/modules/db_orm/json_extract.rs`)
  Queries and indexes nested JSON fields inside SQLite TEXT columns using `json_extract()` with type conversion.

### Batch 5: Desktop React UI & UX Component Modernization (Imp 145–150)
- **Imp-145: Keyboard Shortcut Tooltip Stacker** (`src/utils/shortcutFormatter.ts`)
  Formats multi-platform keyboard shortcuts (`Ctrl+Shift+P` vs `⌘⇧P` on macOS) with high-contrast badge styling.
- **Imp-146: Time Ago Dynamic Relative Formatter** (`src/utils/relativeTime.ts`)
  Calculates human-readable relative timestamps (`just now`, `45s ago`, `3m ago`, `2h ago`) with auto-refresh intervals.
- **Imp-147: Dark-Glass Segmented Button Capsule Helper** (`src/utils/capsuleStyles.ts`)
  Exports standardized Tailwind classes for contiguous segmented capsules, eliminating nested white border bugs.
- **Imp-148: Safe LocalStorage Session Cache Wrapper** (`src/utils/safeStorage.ts`)
  Safe storage helper handling quota exceeded exceptions, JSON serialization errors, and fallbacks to in-memory store.
- **Imp-149: Virtualized Row Index Range Calculator** (`src/utils/virtualRange.ts`)
  Calculates visible row indices and vertical offsets for high-performance table virtualization.
- **Imp-150: System Diagnostic Health Badge Resolver** (`src/utils/healthBadge.ts`)
  Maps doctor diagnostic check results (Ok, Warning, Critical) to high-contrast badge styles and icons.

---

## 3. Non-Negotiable Operational Constraints
1. **Zero Testing & Build Checking**: No `cargo test`, `cargo build`, `npm test`, or `npm run build` during execution.
2. **Search Primacy**: Strict ban on raw `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`. All search performed via GitMap.
3. **Relative Paths**: All references use repository-relative Git paths.
4. **No Silent Errors**: Fallible operations propagate through `AppError` or `useErrorStore`.
