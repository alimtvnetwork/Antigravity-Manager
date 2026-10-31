# Antigravity-Manager: Detailed File-by-File AI Implementation Blueprints

- **Spec Location**: `02-spec/21-app/156-fast-forward-logic-and-improvement-detailing/ai-journeys.md`
- **Parent Spec**: `02-spec/21-app/156-fast-forward-logic-and-improvement-detailing/spec.md`
- **Status**: Production Blueprint Ready for Autonomous Agent Dispatch
- **Target Architecture**: Antigravity-Manager Native Rust Gateway + React/Tauri Modern Desktop UI

---

## Blueprint Index by Subsystem

1. [Subsystem 1: Auto-Switcher & Quota Intelligence Engine](#subsystem-1-auto-switcher--quota-intelligence-engine)
2. [Subsystem 2: Multi-Instance Profile Sandboxing & Lifecycle](#subsystem-2-multi-instance-profile-sandboxing--lifecycle)
3. [Subsystem 3: Prompt Management, Tree View & Execution Pipeline](#subsystem-3-prompt-management-tree-view--execution-pipeline)
4. [Subsystem 4: Reverse Proxy Gateway & 4-Protocol Adapters](#subsystem-4-reverse-proxy-gateway--4-protocol-adapters)
5. [Subsystem 5: Supabase Fleet Synchronization & Remote Daemons](#subsystem-5-supabase-fleet-synchronization--remote-daemons)
6. [Subsystem 6: Split-SQLite Database Architecture & REST Gateway](#subsystem-6-split-sqlite-database-architecture--rest-gateway)
7. [Subsystem 7: React Desktop UI, Design Tokens & Dark-Glass Themes](#subsystem-7-react-desktop-ui-design-tokens--dark-glass-themes)
8. [Subsystem 8: Speed Error Telemetry & Autonomous Self-Healing](#subsystem-8-speed-error-telemetry--autonomous-self-healing)
9. [Subsystem 9: Coding Guideline Modernization & Complexity Elimination](#subsystem-9-coding-guideline-modernization--complexity-elimination)

---

## Subsystem 1: Auto-Switcher & Quota Intelligence Engine

### Blueprint 1.1: Quota Invariant & Dual-Key Exclusion Enforcer
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/scoring.rs`
  - `src-tauri/src/modules/auto_switcher/selection.rs`
  - `src-tauri/src/modules/auto_switcher/manual.rs`
  - `src-tauri/src/modules/auto_switcher/instances.rs`
  - `src/services/instanceService/scoring.ts`
  - `src/stores/useInstanceStore/smart.ts`
- **Impact & Blast Radius**:
  - Eliminates self-rotations where Fast-Forward switches an instance into the exact account it is currently using.
  - Eliminates candidate pollution where accounts with depleted/reduced 4-hour quotas ($<100\%$, e.g. 63%, 98%) are chosen over fresh 100% accounts.
  - Guarantees zero cross-instance account collisions across running siblings on the host machine.
- **AI Implementation Journey**:
  1. *Step 1 (Backend Primary Score Gate)*: Open `src-tauri/src/modules/auto_switcher/scoring.rs`. In `score_candidate_account`, verify `q_4h = calculate_4h_window_quota(acc, target_model).unwrap_or(0.0)`. If `q_4h < 100.0`, return `0.0` unconditionally without bypassing via `is_period_finished`.
  2. *Step 2 (Backend False 100% Override Removal)*: In `src-tauri/src/modules/auto_switcher/selection.rs`, locate candidate discovery loops (Steps 1 & 2). Ensure `quota` is strictly computed via `calculate_candidate_quota` and never set to `100.0` simply because `is_period_finished` is true. Candidate pool admission requires `quota >= 100.0` and `score > 0.0`.
  3. *Step 3 (Dual-Key In-Use Exclusion)*: In `src-tauri/src/modules/auto_switcher/instances.rs`, update `get_active_in_use_account_ids()` to inspect both `inst.bound_account_id` and `inst.bound_email`. Look up `account::get_account_by_email` to append UUIDs and push raw emails to exclusion vectors.
  4. *Step 4 (Target Instance Self-Exclusion)*: In `src-tauri/src/modules/auto_switcher/manual.rs` and `selection.rs`, inspect `current_instance_id`. Add both `bound_account_id` and `bound_email` of the target instance to `effective_exclusions`.
  5. *Step 5 (Frontend Scoring Parity)*: In `src/services/instanceService/scoring.ts`, ensure `calculateMultiplicativeScore` returns `{ score: 0, activeFactor: 0, ... }` whenever `fourHourQuotaPercent < 100 || activeFactor === 0`. In `rankSmartCandidates`, filter `primaryEligible` strictly with `q4h >= 100`.
  6. *Step 6 (Frontend Email Matching)*: In `src/stores/useInstanceStore/smart.ts`, resolve `curAccount` by both ID and email. Ensure `activeInUseAccountIds` collects both ID and email across all running instances.
- **Code Anchors**:
  - `scoring.rs: score_candidate_account(acc: &Account, target_model: &str, now_sec: i64) -> f64`
  - `selection.rs: select_candidate_profiles(current_instance_id: &str, ...)`
  - `instances.rs: get_active_in_use_account_ids() -> Vec<String>`
  - `scoring.ts: calculateMultiplicativeScore(acc: Account, ...)`
  - `smart.ts: smartRotateProfileAccount(targetInstanceId?: string)`

---

### Blueprint 1.2: Adaptive High-Frequency Quota Poller with Sliding Jitter
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/daemon.rs`
  - `src-tauri/src/modules/auto_switcher/status.rs`
  - `src/stores/useAccountStore.ts`
- **Impact & Blast Radius**:
  - Reduces Google Quota API calls by 40% during idle periods while providing 5-second responsiveness when quota drops below caution threshold ($\le 20\%$).
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/auto_switcher/daemon.rs`. Locate the main daemon loop `run_auto_profile_switcher_loop`.
  2. *Step 2*: Inspect current quota state across active accounts. If any active account has $Q_{4\text{h}} \le 20\%$, dynamically scale polling interval to `caution_interval_secs` (default 30s). If $Q_{4\text{h}} \le 10\%$, scale to `critical_interval_secs` (default 15s).
  3. *Step 3*: Apply a $\pm 10\%$ pseudo-random jitter (`rand::thread_rng().gen_range(0.9..1.1)`) to interval calculation to prevent API call synchronization storms across multi-machine clusters.
  4. *Step 4*: In `status.rs`, export the dynamic interval state in `DaemonStatus` so the frontend UI displays the current polling cadence.
- **Code Anchors**:
  - `daemon.rs: run_auto_profile_switcher_loop()`
  - `daemon.rs: determine_dynamic_sleep_interval(...)`
  - `status.rs: get_daemon_status() -> Result<DaemonStatus, String>`

---

## Subsystem 2: Multi-Instance Profile Sandboxing & Lifecycle

### Blueprint 2.1: Conscious Process Termination & Port Rebind Safety Gate
- **Target Files**:
  - `src-tauri/src/modules/instance/lifecycle.rs`
  - `src-tauri/src/modules/instance/switch.rs`
  - `src-tauri/src/commands/instance.rs`
- **Impact & Blast Radius**:
  - Prevents IDE crash on restart caused by orphaned node processes holding lockfiles on `state.vscdb` or workspace storage.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/instance/lifecycle.rs`. In `stop_instance_process(pid: u32)`, implement conscious PID tree verification using `sysinfo` to verify the process executable matches Antigravity IDE (`antigravity.exe` / `Antigravity.app`).
  2. *Step 2*: Before issuing `SIGKILL` / `TerminateProcess`, issue a graceful `WM_CLOSE` (on Windows) or `SIGTERM` (on Unix) with a 3000ms bounded timeout.
  3. *Step 3*: If process fails to terminate within 3000ms, escalate to force kill.
  4. *Step 4*: In `switch.rs: execute_profile_switch`, ensure the SQLite database handles for `state.vscdb` are explicitly closed and file locks released before new credentials or tokens are written.
- **Code Anchors**:
  - `lifecycle.rs: stop_instance_process(pid: u32, data_dir: &str)`
  - `switch.rs: perform_profile_switch(instance_id: &str, target_account: &Account)`

---

### Blueprint 2.2: Profile Cloning with Deep Settings Merge & Keyring Bypass
- **Target Files**:
  - `src-tauri/src/modules/instance/clone.rs`
  - `src-tauri/src/modules/instance/settings_sync.rs`
  - `src/components/instances/InstanceCloneModal.tsx`
- **Impact & Blast Radius**:
  - Enables instant 1-click duplication of configured IDE workspaces, keybindings, and extensions while preserving complete account isolation and preventing OS keyring credential bleeding.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/instance/clone.rs`. In `clone_instance_profile(source_id: &str, target_name: &str, options: CloneOptions)`:
  2. *Step 2*: Copy `User/settings.json`, `User/keybindings.json`, and extension manifests from source instance data directory to target data directory.
  3. *Step 3*: Explicitly sanitize and scrub `state.vscdb` in target data directory: delete table rows matching `antigravity.account`, `antigravity.token`, and `telemetry.machineId`.
  4. *Step 4*: Inject synthetic `storage.json` with a newly generated synthetic `telemetry.machineId` and `telemetry.devDeviceId` to ensure anti-abuse device isolation.
  5. *Step 5*: In `InstanceCloneModal.tsx`, provide toggle pills for `Copy Extensions`, `Copy Workspace Settings`, and `Auto-Generate Synthetic Fingerprint`.
- **Code Anchors**:
  - `clone.rs: clone_instance_profile(...)`
  - `settings_sync.rs: deep_merge_settings(source: &Path, target: &Path)`
  - `InstanceCloneModal.tsx: handleCloneSubmit()`

---

## Subsystem 3: Prompt Management, Tree View & Execution Pipeline

### Blueprint 3.1: Live Prompt Tree 30s Auto-Sync with Dynamic Task Acceleration
- **Target Files**:
  - `src/components/instances/PromptTreeView/usePromptTreeData.ts`
  - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
  - `src-tauri/src/modules/repo_db/tree.rs`
- **Impact & Blast Radius**:
  - Solves user reported issue where running prompts did not sync automatically inside open tree view.
  - Bypasses stale backend SQLite cache so prompt status updates reflect within 1–2 seconds during active prompt execution.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src/components/instances/PromptTreeView/usePromptTreeData.ts`.
  2. *Step 2*: In the periodic interval hook `useEffect`, detect whether any node in `treeNodes` has `status === 'RUNNING' || status === 'PENDING'`.
  3. *Step 3*: When active prompts are detected, temporarily accelerate sync interval from 30s to 3s. When all prompts reach terminal state (`COMPLETED` or `FAILED`), throttle back to 30s.
  4. *Step 4*: In `loadTree(silent, isForce)`, ensure interval calls pass `isForce: true` to bypass the backend 30s SQLite tree cache.
  5. *Step 5*: In `src-tauri/src/modules/repo_db/tree.rs`, verify `get_prompt_tree_cached` invalidates when `force == true` or when new prompts are inserted into `repo_prompts.db`.
- **Code Anchors**:
  - `usePromptTreeData.ts: useEffect(() => { ... safeIntervalMs ... }, [treeNodes])`
  - `tree.rs: get_prompt_tree_cached(repo_path: &str, force: bool)`

---

### Blueprint 3.2: Prompt Rerun Multi-Iteration Pacing & Preemption Isolation
- **Target Files**:
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
  - `src/components/instances/PromptTreeView/verificationTemplates.ts`
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
- **Impact & Blast Radius**:
  - Prevents multi-round prompt rerun iterations (`rerunCount > 1`) from prematurely canceling prior iterations through immediate preemption collisions.
  - Ensures verification templates ("Is it done?", "Is it released?") retain original prompt context without empty delimiter blocks.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src/components/instances/PromptTreeView/verificationTemplates.ts`. In `buildRerunPrompt`, check if `promptOrContext` is empty. If empty, omit `<<<ORIGINAL_TASK_PROMPT_START>>>` delimiters. If custom nudge message is provided, format as clean markdown callout.
  2. *Step 2*: Open `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`. In `dispatchRerunMessages`:
  3. *Step 3*: Between rerun iterations (`round < totalReruns`), insert an explicit asynchronous pause: `await new Promise(r => setTimeout(r, 1000));`.
  4. *Step 4*: In `handleRerunConversation`, resolve `originalPrompt` using 3-tier fallback (turn summary -> root prompt history -> task ledger).
  5. *Step 5*: Ensure Send (Preempt) and Enqueue (FIFO) buttons remain visually decoupled and independently dispatchable.
- **Code Anchors**:
  - `verificationTemplates.ts: buildRerunPrompt(promptOrContext, round, total)`
  - `usePromptTreePromptActions.ts: dispatchRerunMessages(...)`
  - `send_flow.rs: dispatch_send_prompt(...)`

---

## Subsystem 4: Reverse Proxy Gateway & 4-Protocol Adapters

### Blueprint 4.1: Canonical IR Protocol Normalizer & Stream Fragment Alignment
- **Target Files**:
  - `src-tauri/src/proxy/canonical.rs`
  - `src-tauri/src/proxy/openai.rs`
  - `src-tauri/src/proxy/claude.rs`
  - `src-tauri/src/proxy/gemini.rs`
  - `src-tauri/src/proxy/streaming.rs`
- **Impact & Blast Radius**:
  - Ensures unified context handling, thinking budget backfilling, and tool call payload transformation across OpenAI, Claude, and Gemini protocols into Antigravity IR format.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/proxy/canonical.rs`. Define `CanonicalRequestEnvelope` with normalized fields: `model`, `system_instructions`, `messages`, `thinking_budget`, `tools`, and `temperature`.
  2. *Step 2*: In `openai.rs`, transform OpenAI chat completion payload (`messages: [{role, content}]`) into `CanonicalRequestEnvelope`. Convert assistant reasoning tokens (`reasoning_content`) to Antigravity `thought` parts.
  3. *Step 3*: In `claude.rs`, transform Claude Messages API (`system`, `messages: [{role, content: [{type, text}]}]`) into `CanonicalRequestEnvelope`. Map Claude thinking blocks (`type: "thinking"`) into Antigravity IR thinking chunks.
  4. *Step 4*: In `gemini.rs`, adapt Antigravity IR directly to Google Gemini upstream endpoints (`v1internal:streamGenerateContent`). Backfill default thinking budget (`budget: 2048`) if omitted by client.
  5. *Step 5*: In `streaming.rs`, ensure SSE chunks maintain strict line breaks (`\ndata: ...\n\n`) and flush immediately without TCP buffering delay.
- **Code Anchors**:
  - `canonical.rs: struct CanonicalRequestEnvelope`
  - `openai.rs: transform_openai_to_canonical(...)`
  - `claude.rs: transform_claude_to_canonical(...)`
  - `gemini.rs: format_gemini_upstream_request(...)`
  - `streaming.rs: forward_sse_stream(...)`

---

## Subsystem 5: Supabase Fleet Synchronization & Remote Daemons

### Blueprint 5.1: Telegram Sub-Second Dispatch with Async Visual Proof Capture
- **Target Files**:
  - `src-tauri/src/modules/telegram_inbound/daemon.rs`
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
  - `src-tauri/src/modules/telegram_inbound/cluster.rs`
- **Impact & Blast Radius**:
  - Eliminates 3–12 second Telegram response delays, providing instant `<100ms` feedback when users dispatch commands or inspect prompts via Telegram bot.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/telegram_inbound/daemon.rs`. Ensure startup delay is bounded to $\le 500\text{ms}$ (`tokio::time::sleep(Duration::from_millis(500))`).
  2. *Step 2*: When receiving callback queries, immediately invoke `answerCallbackQuery` to dismiss the button loading spinner before processing heavy operations.
  3. *Step 3*: Open `src-tauri/src/modules/telegram_inbound/send_flow.rs`. In `dispatch_send_prompt`:
  4. *Step 4*: Send immediate confirmation message (`✅ Prompt dispatched to IDE!`) to user via Telegram API.
  5. *Step 5*: Detach screenshot capture (`capture_and_send_proof_shot`) into an asynchronous background task (`tauri::async_runtime::spawn(...)`). The bot completes the command response instantly while screenshot uploads concurrently.
  6. *Step 6*: In `cluster.rs`, cache GitMap cluster node discovery results for 30s using `std::sync::RwLock` to avoid spawning sequential subprocesses on every command.
- **Code Anchors**:
  - `daemon.rs: start_telegram_inbound_daemon(...)`
  - `send_flow.rs: dispatch_send_prompt(...)`
  - `cluster.rs: query_gitmap_cluster_nodes_cached(...)`

---

## Subsystem 6: Split-SQLite Database Architecture & REST Gateway

### Blueprint 6.1: High-Concurrency WAL Architecture & Schema Migrations
- **Target Files**:
  - `src-tauri/src/modules/sqlite_manager.rs`
  - `src-tauri/src/modules/repo_db/mod.rs`
  - `src-tauri/src/modules/task_history_db.rs`
  - `src-tauri/src/modules/rest_gateway/routes.rs`
- **Impact & Blast Radius**:
  - Eliminates `database is locked` SQLite contention errors during high-frequency prompt logging and multi-instance switches.
  - Provides REST API access for external scripts and headless environments.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/sqlite_manager.rs`. In database connection factory `open_sqlite_connection(path: &Path)`:
  2. *Step 2*: Execute PRAGMA statements on connection creation:
     - `PRAGMA journal_mode = WAL;`
     - `PRAGMA synchronous = NORMAL;`
     - `PRAGMA busy_timeout = 5000;`
     - `PRAGMA cache_size = -64000;` (64MB memory cache)
  3. *Step 3*: Enforce strict FIFO ordering across queries (`ORDER BY created_at ASC, id ASC`) in prompt queues and backup tables.
  4. *Step 4*: In `rest_gateway/routes.rs`, register HTTP endpoints for `/api/instances`, `/api/accounts`, and `/api/prompts/queue`, returning standardized JSON envelopes: `{ "success": true, "data": ..., "timestamp": ... }`.
- **Code Anchors**:
  - `sqlite_manager.rs: open_sqlite_connection(path: &Path) -> Result<Connection, AppError>`
  - `repo_db/mod.rs: list_active_prompts(...)`
  - `rest_gateway/routes.rs: register_rest_routes(...)`

---

## Subsystem 7: React Desktop UI, Design Tokens & Dark-Glass Themes

### Blueprint 7.1: Dark-Glass Design System & Scoreboard Tooltip Contrast
- **Target Files**:
  - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
  - `src/components/instances/PromptTreeView/TreePanel.tsx`
  - `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx`
  - `src/components/navbar/NavMenu.tsx`
  - `src/components/common/ThemeManager.tsx`
- **Impact & Blast Radius**:
  - Eliminates stark white container regressions caused by arbitrary Tailwind opacity syntax.
  - Ensures candidate scoring derivation popovers and navigation capsules look sleek, professional, and accessible.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`.
  2. *Step 2*: Replace all raw arbitrary CSS variable opacity modifiers (e.g. `dark:bg-[var(--app-bg)]/95`) with valid dark-glass utility combinations: `dark:bg-[#0c0817]/95 backdrop-blur-2xl border border-slate-200 dark:border-purple-900/40`.
  3. *Step 3*: Open `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx`. In the portaled tooltip container, apply `bg-slate-900/95 dark:bg-slate-900/95 backdrop-blur-md border border-slate-700/60 shadow-2xl text-slate-100`.
  4. *Step 4*: Set formula calculation box to `bg-slate-950/80 border border-slate-800 text-emerald-300 font-mono text-xs p-2 rounded`.
  5. *Step 5*: Open `src/components/navbar/NavMenu.tsx`. Wrap adjacent navigation buttons into a contiguous segmented pill capsule (`rounded-full bg-gray-100 dark:bg-slate-900/90 border border-gray-200/60 dark:border-slate-800 p-0.5 shadow-xs divide-x`).
  6. *Step 6*: In `src/components/common/ThemeManager.tsx`, define fallback RGB variables (`--app-bg-rgb`, `--app-surface-rgb`) to guard against future theme variable invalidations.
- **Code Anchors**:
  - `PromptTreeViewModal.tsx: lines 800-920 (modal shell & left panel)`
  - `AlgorithmScoreCell.tsx: lines 80-180 (createPortal tooltip)`
  - `NavMenu.tsx: lines 110-220 (segmented pill capsule)`
  - `ThemeManager.tsx: registerThemeVariables(...)`

---

## Subsystem 8: Speed Error Telemetry & Autonomous Self-Healing

### Blueprint 8.1: SQLite Speed Error Diagnostics (errors.db) & Levenshtein Telemetry
- **Target Files**:
  - `src-tauri/src/modules/speed_errors.rs`
  - `src-tauri/src/modules/failed_commands.rs`
  - `src-tauri/src/commands/diagnostics.rs`
  - `src-tauri/src/bin/agm.rs`
- **Impact & Blast Radius**:
  - Provides sub-millisecond recording of runtime faults (E1001–E9001) and gives CLI users automatic command suggestions on mistyped terminal inputs.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/speed_errors.rs`. Verify `errors.db` schema: `(id INTEGER PRIMARY KEY, error_code TEXT, module TEXT, message TEXT, context_json TEXT, created_at INTEGER)`.
  2. *Step 2*: Ensure every error record routes through `record_speed_error(code, module, message, context)` asynchronously without blocking the calling thread.
  3. *Step 3*: Open `src-tauri/src/modules/failed_commands.rs`. In `suggest_command(input: &str, candidates: &[&str])`, compute Levenshtein distance: return closest candidate if edit distance $\le 2$.
  4. *Step 4*: In `agm.rs`, implement `agm errors fix`: query unresolved diagnostic codes and execute automated self-healing procedures (pruning dead locks, refreshing stale tokens, syncing DB schemas).
- **Code Anchors**:
  - `speed_errors.rs: record_speed_error(...)`
  - `failed_commands.rs: suggest_command(...)`
  - `agm.rs: cmd_errors(...)`

---

## Subsystem 9: Coding Guideline Modernization & Complexity Elimination

### Blueprint 9.1: Guard Inversion, Boolean Simplification & AppError Propagation
- **Target Files**:
  - `src-tauri/src/error.rs`
  - `src-tauri/src/modules/auto_switcher/quota.rs`
  - `src-tauri/src/modules/repo_db/transcript.rs`
  - `src/stores/error-store.ts`
- **Impact & Blast Radius**:
  - Eliminates 200+ coding guideline violations cataloged in `agm-coding-guideline-audit-200-cases.md`.
  - Replaces nested `if-else` cascades with flat guard clauses, eliminates silent error discarding (`let _ = ...`), and enforces affirmative naming conventions.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/error.rs`. Verify comprehensive `AppError` enum variants: `Io`, `Json`, `Sqlite`, `Tauri`, `Network`, `Config`, `Process`, and `Validation`.
  2. *Step 2*: Audit `src-tauri/src/modules/repo_db/transcript.rs`. Replace all bare `let _ = fs::write(...)` with `fs::write(...).map_err(|e| AppError::Io(format!("...: {e}")))?;` or `crate::error::record_ignored(...)` with mandatory `// Justification:` comment.
  3. *Step 3*: Audit `src-tauri/src/modules/auto_switcher/quota.rs`. Decompose complex multi-clause boolean conditions (`if !a && (!b || c)`) into discrete, named affirmative variables (`let is_target_model = ...; let has_available_bucket = ...;`).
  4. *Step 4*: Open `src/stores/error-store.ts`. Ensure all frontend catch blocks route through `captureError` (user-facing modal) or `trackWarning` (telemetry history), eliminating bare `catch {}` blocks.
- **Code Anchors**:
  - `error.rs: enum AppError`
  - `error.rs: pub fn record_ignored<T, E: std::fmt::Display>(res: Result<T, E>, op_name: &str)`
  - `error-store.ts: trackWarning(err: unknown, context?: ErrorContext)`
