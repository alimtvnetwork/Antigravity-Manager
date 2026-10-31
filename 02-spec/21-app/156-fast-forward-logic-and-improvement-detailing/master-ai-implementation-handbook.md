# Master AI Implementation Handbook: System, UI, and Architecture Blueprints

- **Document Location**: `02-spec/21-app/156-fast-forward-logic-and-improvement-detailing/master-ai-implementation-handbook.md`
- **Specification ID**: `02-spec/21-app/156-fast-forward-logic-and-improvement-detailing`
- **Target Audience**: Autonomous AI Coding Agents, Swarm Workers, Core Maintainers (@aukgit)
- **Status**: Ready for Autonomous Agent Dispatch & Execution
- **System**: Antigravity-Manager (AGM) High-Performance Gateway & React/Tauri Desktop

---

## Handbook Overview & Execution Directives for AI Agents

When an autonomous AI agent is dispatched to execute any plan or improvement in this repository, it MUST adhere to the following non-negotiable operational invariants:

1. **Strictly Relative Git Paths**: Never output, hardcode, or record absolute filesystem paths (e.g. `C:\...`, `D:\...`, `/home/...`). Every path reference must be repository-relative (`02-spec/...`, `src/...`, `src-tauri/...`, `.ai-memory/...`).
2. **Worker Git Ban**: Subagents must NEVER execute raw git commands (`git add`, `git commit`, `git push`, `git checkout`). Only the primary orchestrator commits at completion via GitMap (`gitmap cpb "<module> - <summary>"`).
3. **No Build / No Heavy Test During Execution**: Subagents must NEVER run `npm run build` or `cargo build`. Rely on focused unit tests (`npm test -- <filter>`, targeted `cargo test`) and the CI pipeline.
4. **No Silent Errors Rule**: Every fallible operation must route errors into `AppError` (Rust) or `useErrorStore` (TypeScript). Never use bare `let _ = ...`, `.ok()`, or empty `catch {}` blocks. Best-effort operations must use `record_ignored(result, "op_name")` with a mandatory `// Justification:` comment.
5. **Search Primacy**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, and `Select-String`. Always use GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`).

---

## Table of Contents

1. [Module 1: Auto-Switcher, Quota Calculation & Candidate Scoring Engine](#module-1-auto-switcher-quota-calculation--candidate-scoring-engine)
2. [Module 2: Multi-Instance Profile Isolation & Sandboxing](#module-2-multi-instance-profile-isolation--sandboxing)
3. [Module 3: Prompt Pipeline, Tree View & Execution Engine](#module-3-prompt-pipeline-tree-view--execution-engine)
4. [Module 4: Reverse Proxy Gateway, Protocol Adapters & Multimodal Streaming](#module-4-reverse-proxy-gateway-protocol-adapters--multimodal-streaming)
5. [Module 5: Supabase Fleet Synchronization, SSH Mesh & Inbound Control](#module-5-supabase-fleet-synchronization-ssh-mesh--inbound-control)
6. [Module 6: Split-SQLite Architecture & REST Gateway](#module-6-split-sqlite-architecture--rest-gateway)
7. [Module 7: React Desktop UI, Design Tokens & Dark-Glass Themes](#module-7-react-desktop-ui-design-tokens--dark-glass-themes)
8. [Module 8: DB REST Data Browser & Admin UI](#module-8-db-rest-data-browser--admin-ui)
9. [Module 9: Speed Error Telemetry & Autonomous Self-Healing](#module-9-speed-error-telemetry--autonomous-self-healing)
10. [Module 10: Coding Guideline Modernization & Complexity Elimination](#module-10-coding-guideline-modernization--complexity-elimination)

---

## Module 1: Auto-Switcher, Quota Calculation & Candidate Scoring Engine

### 1.1 Strict 100% 4-Hour Quota Primacy & Fallback Engine
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/scoring.rs`
  - `src-tauri/src/modules/auto_switcher/selection.rs`
  - `src/services/instanceService/scoring.ts`
  - `src/stores/useInstanceStore/smart.ts`
- **Architectural Context & Problem**:
  Accounts with partially consumed 4-hour quotas (e.g. 63%, 98%) were erroneously scoring full points and being selected over pristine 100% accounts because historical model reset timestamps evaluated to $\le \text{now}$, triggering a false `is_period_finished` 100% override.
- **Core Invariant**:
  - In primary scoring, an account with $Q_{4\text{h}} < 100.0\%$ MUST evaluate to `0.0`.
  - Pass 2 (Proportional Fallback) is activated IF AND ONLY IF zero accounts with $100\%$ quota exist in the candidate pool:
    $$\text{Score}_{\text{fallback}} = \left(\frac{Q_{4\text{h}}}{100.0}\right) \times \text{Score}_{\text{weekly}}$$
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/auto_switcher/scoring.rs`. In `score_candidate_account`, calculate `q_4h = calculate_4h_window_quota(acc, target_model).unwrap_or(0.0)`. Enforce `if q_4h < 100.0 { return 0.0; }`.
  2. *Step 2*: Open `src-tauri/src/modules/auto_switcher/selection.rs`. Remove synthetic `let quota = if is_period_finished { 100.0 } else { ... };` in candidate discovery loops. Candidate admission strictly requires `quota >= 100.0 && score > 0.0`.
  3. *Step 3*: Open `src/services/instanceService/scoring.ts`. In `calculateMultiplicativeScore`, return `score: 0` if `fourHourQuotaPercent < 100 || activeFactor === 0`.
  4. *Step 4*: In `rankSmartCandidates`, filter `primaryEligible` strictly with `q4h >= 100`. Only evaluate fallback pass if `primaryEligible.length === 0`.
- **Code Anchors**:
  - `scoring.rs`: `pub fn score_candidate_account(acc: &Account, target_model: &str, now_sec: i64) -> f64`
  - `selection.rs`: `pub fn select_candidate_profiles(current_instance_id: &str, ...)`
  - `scoring.ts`: `export function calculateMultiplicativeScore(...)`
- **Verification**: Run `npm test -- instanceService`. Ensure all 7 candidate ranking and primacy tests pass with 0 failures.

---

### 1.2 Dual-Key In-Use Account Collision Eliminator
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/instances.rs`
  - `src-tauri/src/modules/auto_switcher/manual.rs`
  - `src-tauri/src/modules/auto_switcher/selection.rs`
  - `src/stores/useInstanceStore/smart.ts`
- **Architectural Context & Problem**:
  Instances bound via `bound_email` without an explicit `bound_account_id` were omitted from in-use exclusion lists, causing Fast-Forward to select an account already running on another instance or rotate the instance into itself.
- **Core Invariant**:
  Exclusion matching must always be dual-key: internal account UUID (`acc.id`) AND normalized email (`acc.email.toLowerCase()`). Target instance self-binding must always be excluded from rotation targets.
- **AI Implementation Journey**:
  1. *Step 1*: In `src-tauri/src/modules/auto_switcher/instances.rs`, update `get_active_in_use_account_ids()` to inspect both `inst.bound_account_id` and `inst.bound_email`. Use `account::get_account_by_email` to resolve UUIDs for email-only bindings and push both to the in-use vector.
  2. *Step 2*: In `src-tauri/src/modules/auto_switcher/manual.rs`, add target instance's `bound_email` and resolved ID to `excluded`. Add all running siblings' `bound_email` and resolved IDs to `excluded`.
  3. *Step 3*: In `src-tauri/src/modules/auto_switcher/selection.rs`, add target instance's account (by ID, email, or default account) to `effective_exclusions`.
  4. *Step 4*: In `src/stores/useInstanceStore/smart.ts`, collect both `bound_account_id` and `bound_email` across running instances into `activeInUseAccountIds`. Include current instance account by ID and email.
- **Code Anchors**:
  - `instances.rs`: `pub fn get_active_in_use_account_ids() -> Vec<String>`
  - `manual.rs`: `pub async fn trigger_manual_rotation_for_instance(...)`
  - `smart.ts`: `smartRotateProfileAccount: async (targetInstanceId?: string) => { ... }`
- **Verification**: In test suite, assert that passing an email into in-use exclusion list sets `activeFactor = 0` and `score = 0`.

---

### 1.3 Adaptive Dynamic Quota Poller with Jitter
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/daemon.rs`
  - `src-tauri/src/modules/auto_switcher/status.rs`
- **Architectural Context & Problem**:
  Static 60-second polling intervals either poll too aggressively during prolonged idle periods (exhausting Google API rate limits) or react too slowly when an account drops below the 20% critical threshold.
- **Core Invariant**:
  Polling cadence must automatically scale: 60s when healthy ($> 50\%$), 30s when caution ($\le 20\%$), 15s when critical ($\le 10\%$), with $\pm 10\%$ jitter to avoid fleet sync collisions.
- **AI Implementation Journey**:
  1. *Step 1*: In `daemon.rs`, define `calculate_dynamic_sleep_interval(accounts: &[Account], cfg: &SwitcherConfig) -> Duration`.
  2. *Step 2*: Inspect lowest 4-hour quota among actively used accounts.
  3. *Step 3*: Apply linear scale: if `min_q <= 10.0`, interval is 15s; if `min_q <= 20.0`, interval is 30s; otherwise interval is 60s.
  4. *Step 4*: Multiply interval by random jitter factor `0.9 + (rand::random::<f64>() * 0.2)`.
  5. *Step 5*: In `status.rs`, expose `current_poll_interval_secs` in `DaemonStatus` struct.
- **Code Anchors**:
  - `daemon.rs`: `pub async fn run_auto_profile_switcher_loop()`
  - `status.rs`: `pub struct DaemonStatus`
- **Verification**: Run `cargo test daemon`; verify interval dynamically scales based on mocked account quotas.

---

## Module 2: Multi-Instance Profile Isolation & Sandboxing

### 2.1 Conscious Process Termination & Port Rebind Safety Gate
- **Target Files**:
  - `src-tauri/src/modules/instance/lifecycle.rs`
  - `src-tauri/src/modules/instance/switch.rs`
- **Architectural Context & Problem**:
  Killing an instance by PID without verifying process executable ownership risks killing unrelated OS processes or leaving child node processes holding file locks on `state.vscdb`.
- **Core Invariant**:
  Before killing a process, verify executable name matches `antigravity.exe` / `Antigravity`. Issue graceful termination (`WM_CLOSE` / `SIGTERM`) with 3000ms timeout before force kill (`TerminateProcess` / `SIGKILL`). Release all SQLite locks before writing new profile state.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/instance/lifecycle.rs`. In `stop_instance_process(pid: u32, data_dir: &str)`:
  2. *Step 2*: Use `sysinfo` to inspect process by PID. Verify `exe.file_name()` matches expected IDE binary. If mismatch, log critical security warning and abort.
  3. *Step 3*: Issue graceful termination signal. Wait up to 3000ms with 200ms sleep loop checking `process.is_alive()`.
  4. *Step 4*: If process is still alive after 3000ms, issue force kill.
  5. *Step 5*: In `switch.rs: perform_profile_switch`, ensure connection pools to `state.vscdb` are closed, wait 500ms settling time, then write new tokens and configuration.
- **Code Anchors**:
  - `lifecycle.rs`: `pub fn stop_instance_process(pid: u32, data_dir: &str) -> Result<(), AppError>`
  - `switch.rs`: `pub async fn perform_profile_switch(instance_id: &str, target_account: &Account)`
- **Verification**: Run `scripts/test-instance-isolation.ps1`; ensure stopping an instance leaves zero orphaned lockfiles.

---

### 2.2 Profile Cloning with Keyring Bypass & Synthetic Device Fingerprints
- **Target Files**:
  - `src-tauri/src/modules/instance/clone.rs`
  - `src-tauri/src/modules/instance/settings_sync.rs`
  - `src/components/instances/InstanceCloneModal.tsx`
- **Architectural Context & Problem**:
  Copying an instance directory duplicates the source account's OAuth session tokens and hardware machine IDs, causing token bleeding and anti-abuse telemetry conflicts on Google servers.
- **Core Invariant**:
  Cloned instance data directories must copy extensions and UI settings, but MUST scrub `state.vscdb` account rows and regenerate synthetic `storage.json` telemetry IDs (`telemetry.machineId`, `telemetry.devDeviceId`, `telemetry.macMachineId`).
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/instance/clone.rs`. In `clone_instance_profile`:
  2. *Step 2*: Copy directory tree `User/settings.json`, `User/keybindings.json`, `User/snippets/`, and extensions.
  3. *Step 3*: Open cloned `state.vscdb` using SQLite. Execute:
     `DELETE FROM ItemTable WHERE key IN ('antigravity.account', 'antigravity.token', 'telemetry.machineId');`
  4. *Step 4*: Generate synthetic UUIDs for `storage.json`. Write:
     ```json
     {
       "telemetry.machineId": "<uuid_v4_sha256>",
       "telemetry.devDeviceId": "<uuid_v4>",
       "telemetry.macMachineId": "<uuid_v4>"
     }
     ```
  5. *Step 5*: In `InstanceCloneModal.tsx`, provide options checkboxes for settings sync.
- **Code Anchors**:
  - `clone.rs`: `pub fn clone_instance_profile(source_id: &str, target_name: &str, options: CloneOptions) -> Result<InstanceConfig, AppError>`
  - `settings_sync.rs`: `pub fn scrub_cloned_storage(target_dir: &Path) -> Result<(), AppError>`
- **Verification**: Inspect cloned instance directory; verify `storage.json` contains unique UUIDs distinct from parent instance.

---

## Module 3: Prompt Pipeline, Tree View & Execution Engine

### 3.1 Live Prompt Tree 30s Auto-Sync with Task Acceleration
- **Target Files**:
  - `src/components/instances/PromptTreeView/usePromptTreeData.ts`
  - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
  - `src-tauri/src/modules/repo_db/tree.rs`
- **Architectural Context & Problem**:
  When a prompt was running in the IDE, an already-open Prompt Tree View did not reflect status updates automatically because backend cached tree queries for up to 120s and interval polling passed `force: false`.
- **Core Invariant**:
  - Periodic polling in `usePromptTreeData.ts` must call `loadTree(false, true)` with `force: true` to bypass SQLite cache.
  - When active tasks are running (`RUNNING` or `PENDING`), sync interval dynamically accelerates to 2–3s.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src/components/instances/PromptTreeView/usePromptTreeData.ts`.
  2. *Step 2*: In the periodic interval hook `useEffect`, detect if any prompt node has `status === 'RUNNING' || status === 'PENDING'`.
  3. *Step 3*: If active tasks exist, set interval to 2000ms. If idle, set interval to 30000ms.
  4. *Step 4*: Call `loadTree(false, true)` on timer fire so `force: true` is passed to backend IPC.
  5. *Step 5*: Open `src-tauri/src/modules/repo_db/tree.rs`. Ensure `get_prompt_tree_cached` checks `if force { return compute_tree(); }`.
- **Code Anchors**:
  - `usePromptTreeData.ts`: `const activeIntervalMs = hasRunningPrompts ? 2000 : 30000;`
  - `tree.rs`: `pub fn get_prompt_tree_cached(repo_path: &str, force: bool) -> Result<PromptTreeResponse, AppError>`
- **Verification**: Trigger a running prompt; observe tree view node status badge transitions to `RUNNING` within 2 seconds without reopening modal.

---

### 3.2 Prompt Rerun Multi-Iteration Pacing & FIFO Enqueue Decoupling
- **Target Files**:
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
  - `src/components/instances/PromptTreeView/verificationTemplates.ts`
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
- **Architectural Context & Problem**:
  Multi-round prompt reruns (`rerunCount > 1`) fired simultaneously without pacing, causing the second iteration to preemptively abort the first iteration. Send and Enqueue buttons were tightly coupled in a brittle dropdown.
- **Core Invariant**:
  - Multi-iteration reruns must include an explicit asynchronous pause (`1000ms`) between iterations.
  - Send (Preempt: cancel running task then execute) and Enqueue (FIFO: queue after current task) must be distinct, visible actions in the header capsule.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src/components/instances/PromptTreeView/verificationTemplates.ts`. In `buildRerunPrompt`, check if task prompt is present. If empty, omit delimiter tags. Format custom nudge cleanly.
  2. *Step 2*: Open `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`. In `dispatchRerunMessages`:
  3. *Step 3*: Insert `await new Promise(r => setTimeout(r, 1000));` between rounds when `round < totalReruns`.
  4. *Step 4*: Open `src/components/instances/PromptTreeView/DetailHeader.tsx`. Maintain contiguous segmented capsule: `[ Send (Preempt) | Enqueue (FIFO) | v (Chevron) ]`.
- **Code Anchors**:
  - `usePromptTreePromptActions.ts`: `async function dispatchRerunMessages(...)`
  - `DetailHeader.tsx`: `<div className="flex items-center rounded-lg bg-slate-800 ...">`
- **Verification**: Dispatch 3-round rerun; verify in `repo_prompts.db` that all 3 iterations complete sequentially with 1s pauses.

---

## Module 4: Reverse Proxy Gateway, Protocol Adapters & Multimodal Streaming

### 4.1 Canonical IR Envelope Normalizer
- **Target Files**:
  - `src-tauri/src/proxy/canonical.rs`
  - `src-tauri/src/proxy/openai.rs`
  - `src-tauri/src/proxy/claude.rs`
  - `src-tauri/src/proxy/gemini.rs`
- **Architectural Context & Problem**:
  Protocol divergences among OpenAI, Claude, and Gemini (such as thinking blocks, system prompt formats, and tool calling conventions) were causing brittle ad-hoc patches in downstream routes.
- **Core Invariant**:
  All 4 inbound protocols must convert into an intermediate `CanonicalRequestEnvelope` before entering the gateway pipeline. The pipeline processes thinking budgets, token buckets, and prompt sanitization on the IR envelope before outputting Gemini wire format.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/proxy/canonical.rs`. Define `CanonicalRequestEnvelope`:
     ```rust
     pub struct CanonicalRequestEnvelope {
         pub model: String,
         pub system_instruction: Option<String>,
         pub messages: Vec<CanonicalMessage>,
         pub thinking_budget: Option<u32>,
         pub temperature: Option<f32>,
         pub tools: Vec<CanonicalTool>,
     }
     ```
  2. *Step 2*: In `openai.rs`, implement `From<OpenAIChatRequest> for CanonicalRequestEnvelope`. Map `developer`/`system` roles to `system_instruction`. Map `reasoning_content` to thinking parts.
  3. *Step 3*: In `claude.rs`, implement `From<ClaudeMessagesRequest> for CanonicalRequestEnvelope`. Map `thinking: { type: "enabled", budget_tokens }` to `thinking_budget`.
  4. *Step 4*: In `gemini.rs`, implement `From<CanonicalRequestEnvelope> for GeminiGenerateContentRequest`. Backfill default thinking budget (`2048`) if model supports thinking and budget is None.
- **Code Anchors**:
  - `canonical.rs`: `impl CanonicalRequestEnvelope`
  - `openai.rs`: `pub fn transform_openai_payload(...)`
  - `claude.rs`: `pub fn transform_claude_payload(...)`
  - `gemini.rs`: `pub fn format_gemini_upstream(...)`
- **Verification**: Send test curl requests using OpenAI schema and Claude schema; verify both generate valid upstream Gemini protobuf payloads with correct thinking budgets.

---

### 4.2 SSE Streaming Stream Fragment Flusher & Backpressure Guard
- **Target Files**:
  - `src-tauri/src/proxy/streaming.rs`
  - `src-tauri/src/proxy/sse.rs`
- **Architectural Context & Problem**:
  Downstream IDE clients experienced jerky token delivery and buffering delays caused by TCP Nagle buffering and fragmented SSE line termination.
- **Core Invariant**:
  SSE stream chunks must be explicitly delimited with `data: <json>\n\n`, flushed immediately on the network transport without buffering delay, and handle client disconnects gracefully.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/proxy/streaming.rs`.
  2. *Step 2*: In the response streamer, wrap the inner stream with `tokio_util::io::ReaderStream` and configure TCP socket with `set_nodelay(true)`.
  3. *Step 3*: Ensure every SSE event chunk ends with double newline `\n\n`.
  4. *Step 4*: In `sse.rs`, implement client cancellation watcher: if client terminates connection, cleanly abort the upstream Google API request to conserve token quota.
- **Code Anchors**:
  - `streaming.rs`: `pub async fn forward_sse_response(...)`
  - `sse.rs`: `pub fn format_sse_chunk(event: &str, data: &str) -> Bytes`
- **Verification**: Run streaming benchmark curl; verify TTFT (Time-to-First-Token) is $< 250\text{ms}$ with smooth continuous token stream.

---

## Module 5: Supabase Fleet Synchronization, SSH Mesh & Inbound Control

### 5.1 Telegram Sub-Second Dispatch with Background Proof Capture
- **Target Files**:
  - `src-tauri/src/modules/telegram_inbound/daemon.rs`
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
  - `src-tauri/src/modules/telegram_inbound/cluster.rs`
- **Architectural Context & Problem**:
  Telegram prompt inspection and dispatch took 3–12 seconds because screenshot capture (`capture_and_send_proof_shot`) was awaited synchronously before responding to the user, and cluster node lookups spawned sequential CLI subprocesses.
- **Core Invariant**:
  - Telegram bot commands must reply immediately with textual confirmation ($< 100\text{ms}$).
  - Proof screenshot capture must run asynchronously in a detached background task.
  - GitMap cluster node discovery must be cached in memory with a 30-second TTL.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/telegram_inbound/daemon.rs`. Ensure long-polling startup delay is bounded to 500ms (`tokio::time::sleep(Duration::from_millis(500))`).
  2. *Step 2*: When receiving callback queries, immediately invoke `answerCallbackQuery` to dismiss the loading spinner.
  3. *Step 3*: Open `src-tauri/src/modules/telegram_inbound/send_flow.rs`. In `dispatch_send_prompt`:
  4. *Step 4*: Issue prompt injection to IDE. Send immediate Telegram text response: `✅ Prompt dispatched to IDE!`.
  5. *Step 5*: Detach screenshot capture:
     ```rust
     tauri::async_runtime::spawn(async move {
         super::open_shot::capture_and_send_proof_shot(&bot, chat_id, &args).await;
     });
     ```
  6. *Step 6*: In `cluster.rs`, implement `query_gitmap_cluster_nodes_cached()` backed by `tokio::sync::RwLock` and 30-second timestamp check.
- **Code Anchors**:
  - `daemon.rs`: `pub async fn start_telegram_inbound_daemon(...)`
  - `send_flow.rs`: `pub async fn dispatch_send_prompt(...)`
  - `cluster.rs`: `pub async fn query_gitmap_cluster_nodes_cached() -> Vec<ClusterNode>`
- **Verification**: Dispatch prompt via Telegram bot `/send`; verify text confirmation arrives within 150ms followed by background screenshot attachment.

---

### 5.2 Supabase Distributed Workspace Leases & Multi-Node Arbitration
- **Target Files**:
  - `src-tauri/src/modules/workspace_lease_manager.rs`
  - `src-tauri/src/modules/supabase_sync/probe.rs`
- **Architectural Context & Problem**:
  Across multi-machine fleet environments (e.g. W1, W2, Mac), multiple machines were occasionally leasing the same Google account simultaneously, causing quota exhaustion and rate limit collisions.
- **Core Invariant**:
  - Account leases must be recorded in Supabase `workspace_leases` table with heartbeat timestamps (`leased_at`, `expires_at`).
  - An account leased by Machine A cannot be selected by Machine B until lease expiration ($> 3600\text{s}$) or explicit release.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/workspace_lease_manager.rs`.
  2. *Step 2*: In `acquire_workspace_lease(account_id: &str, machine_id: &str)`:
  3. *Step 3*: Query Supabase `workspace_leases` where `account_id = $1 AND expires_at > now()`. If active lease owned by another machine exists, abort acquisition and return `Err("Account leased by another node")`.
  4. *Step 4*: Insert or update lease row with `expires_at = now() + interval '60 minutes'`.
  5. *Step 5*: In background task, send periodic heartbeat every 5 minutes to renew active leases.
- **Code Anchors**:
  - `workspace_lease_manager.rs`: `pub async fn acquire_workspace_lease(...)`
  - `workspace_lease_manager.rs`: `pub async fn release_workspace_lease(...)`
- **Verification**: Simulate lease acquisition from synthetic machine ID; verify sibling node rejects account in candidate selection.

---

## Module 6: Split-SQLite Architecture & REST Gateway

### 6.1 WAL Concurrency Engine & Strict FIFO Ordering
- **Target Files**:
  - `src-tauri/src/modules/sqlite_manager.rs`
  - `src-tauri/src/modules/repo_db/mod.rs`
  - `src-tauri/src/modules/backup_prompts_db.rs`
- **Architectural Context & Problem**:
  Concurrent database reads and writes during prompt queuing and switching caused intermittent `database is locked` errors. Some queue queries accidentally used inverted LIFO ordering (`ORDER BY updated_at DESC`).
- **Core Invariant**:
  - All SQLite database connections must enforce PRAGMA WAL mode, synchronous NORMAL, busy timeout 5000ms, and 64MB cache size.
  - All prompt restore, dispatch, and queue processing queries MUST enforce strict FIFO ordering: `ORDER BY created_at ASC, id ASC`.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/sqlite_manager.rs`. In `open_connection(path: &Path) -> Result<Connection, AppError>`:
  2. *Step 2*: Execute PRAGMAs:
     ```sql
     PRAGMA journal_mode = WAL;
     PRAGMA synchronous = NORMAL;
     PRAGMA busy_timeout = 5000;
     PRAGMA cache_size = -64000;
     ```
  3. *Step 3*: Audit `repo_db/mod.rs` and `backup_prompts_db.rs`. Replace any instances of `ORDER BY updated_at DESC` or `ORDER BY id DESC` in queue queries with `ORDER BY created_at ASC, id ASC`.
- **Code Anchors**:
  - `sqlite_manager.rs`: `pub fn open_sqlite_connection(...) -> Result<Connection, AppError>`
  - `repo_db/mod.rs`: `pub fn list_pending_queue_prompts(...)`
- **Verification**: Run concurrent write stress test (100 parallel prompt inserts); verify zero database lock errors and strict FIFO queue retrieval.

---

### 6.2 Standalone REST API Gateway for Headless & CLI Parity
- **Target Files**:
  - `src-tauri/src/modules/rest_gateway/mod.rs`
  - `src-tauri/src/modules/rest_gateway/routes.rs`
  - `src-tauri/src/commands/mod.rs`
- **Architectural Context & Problem**:
  Headless server instances and terminal scripts lacked access to Tauri IPC commands, preventing remote automation of account switching and prompt queuing.
- **Core Invariant**:
  The REST gateway must expose authenticated HTTP endpoints on configured port (`localhost:18188`) matching all Tauri IPC capabilities, returning standardized JSON envelopes:
  `{ "success": boolean, "data": any, "error": string | null, "timestamp": number }`.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/rest_gateway/routes.rs`.
  2. *Step 2*: Register REST routes using Axum / Warp / Tokio:
     - `GET /api/instances`: List all configured instances and statuses.
     - `POST /api/instances/switch`: Switch instance profile account.
     - `GET /api/accounts`: List accounts and quota statuses.
     - `POST /api/prompts/send`: Immediate prompt injection.
     - `POST /api/prompts/enqueue`: FIFO prompt queueing.
  3. *Step 3*: Add HMAC-SHA256 bearer token authentication middleware.
  4. *Step 4*: In `commands/mod.rs`, ensure underlying business logic functions are shared between Tauri command handlers and REST route handlers.
- **Code Anchors**:
  - `rest_gateway/mod.rs`: `pub async fn start_rest_gateway(...)`
  - `routes.rs`: `pub fn api_router() -> Router`
- **Verification**: Send curl request to `http://127.0.0.1:18188/api/instances`; verify 200 OK with valid JSON envelope.

---

## Module 7: React Desktop UI, Design Tokens & Dark-Glass Themes

### 7.1 Dark-Glass Design System & Invalidation Safeguard
- **Target Files**:
  - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
  - `src/components/instances/PromptTreeView/TreePanel.tsx`
  - `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx`
  - `src/components/navbar/NavMenu.tsx`
  - `src/components/common/ThemeManager.tsx`
- **Architectural Context & Problem**:
  Tailwind arbitrary opacity modifiers (e.g. `dark:bg-[var(--app-bg)]/95`) broke during theme switching because CSS variables contained non-RGB hex values, triggering fallback to stark white backgrounds.
- **Core Invariant**:
  - All dark-glass panels must use validated Tailwind combinations: `dark:bg-[#0c0817]/95 backdrop-blur-2xl border border-slate-200 dark:border-purple-900/40`.
  - Scoreboard candidate tooltips must use high-contrast dark card containers (`bg-slate-900/95`) with emerald formula boxes and clear text labels.
  - Header icons must render inside contiguous segmented capsules (`rounded-full bg-slate-900/90 divide-x`).
- **AI Implementation Journey**:
  1. *Step 1*: Open `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`. Replace broken opacity modifiers on modal container, left tree panel, splitter, and detail views with solid dark-glass classes.
  2. *Step 2*: Open `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx`. In the portaled popover container, apply `bg-slate-900/95 backdrop-blur-md border border-slate-700/60 shadow-2xl text-slate-100`.
  3. *Step 3*: Format formula derivation box with `bg-slate-950/80 border border-slate-800 text-emerald-300 font-mono text-xs p-2.5 rounded`.
  4. *Step 4*: Open `src/components/navbar/NavMenu.tsx`. Wrap adjacent icon buttons into contiguous segmented pill capsule with subtle dividers and dark hover highlights.
  5. *Step 5*: Open `src/components/common/ThemeManager.tsx`. Define RGB fallbacks for `--app-bg-rgb` and `--app-surface-rgb`.
- **Code Anchors**:
  - `PromptTreeViewModal.tsx`: lines 800-920
  - `AlgorithmScoreCell.tsx`: lines 80-180
  - `NavMenu.tsx`: lines 110-220
- **Verification**: Inspect Prompt Tree and Scoreboard in both dark and light modes; verify zero stark white containers and complete text legibility.

---

### 7.2 MiniView Floating Desktop Widget with Live Quota Pill
- **Target Files**:
  - `src/components/miniview/MiniView.tsx`
  - `src/components/miniview/useMiniViewState.ts`
  - `src-tauri/src/commands/window.rs`
- **Architectural Context & Problem**:
  Users running IDE tasks full-screen needed an unobtrusive, always-on-top micro widget to monitor active account quota, prompt status, and trigger 1-click Fast-Forward without opening the full AGM window.
- **Core Invariant**:
  MiniView must be an ultra-lightweight borderless window (height $\le 48\text{px}$), draggable, always-on-top, displaying active instance name, quota progress pill, and fast-forward button.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src/components/miniview/MiniView.tsx`.
  2. *Step 2*: Structure layout: `flex items-center space-x-2 px-3 py-1.5 rounded-full bg-slate-950/90 border border-slate-800 backdrop-blur-xl shadow-2xl text-xs text-white`.
  3. *Step 3*: Display active account email (masked), current model 4h quota percentage, and colored indicator dot (Green $\ge 50\%$, Amber $\ge 20\%$, Red $< 20\%$).
  4. *Step 4*: Add fast-forward icon button (`FastForward` from `lucide-react`) invoking `smartRotateProfileAccount()`.
  5. *Step 5*: Add double-click handler on title bar region to restore main application window.
- **Code Anchors**:
  - `MiniView.tsx`: `export function MiniView()`
  - `window.rs`: `pub fn toggle_miniview_mode(app: AppHandle, enable: bool)`
- **Verification**: Toggle MiniView; verify compact widget floats above other windows and fast-forward button rotates account smoothly.

---

## Module 8: DB REST Data Browser & Admin UI

### 8.1 Split-DB REST Data Browser & SQL Sandbox
- **Target Files**:
  - `src/components/databrowser/DataBrowserModal.tsx`
  - `src/components/databrowser/TableGrid.tsx`
  - `src/components/databrowser/SqlSandbox.tsx`
  - `src-tauri/src/modules/rest_gateway/db_routes.rs`
- **Architectural Context & Problem**:
  Developers and power users had no visual way to inspect SQLite tables across the 3 split databases (`repo_prompts.db`, `task_index.db`, `errors.db`) or query records without external database tools.
- **Core Invariant**:
  Data Browser must provide read-only visual inspection, pagination, search filtering, and safe read-only SQL query execution across split databases with query timeout guards.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/rest_gateway/db_routes.rs`.
  2. *Step 2*: Implement safe query handler: inspect SQL string, reject any mutating statements (`INSERT`, `UPDATE`, `DELETE`, `DROP`, `ALTER`). Enforce `LIMIT 500`.
  3. *Step 3*: Return table schema, columns, and paginated rows in standardized format.
  4. *Step 4*: Open `src/components/databrowser/TableGrid.tsx`. Render virtualized table using `@tanstack/react-virtual` with sticky headers and column resizing.
  5. *Step 5*: Open `src/components/databrowser/SqlSandbox.tsx`. Provide code editor textarea with execute button and execution duration timer (`ms`).
- **Code Anchors**:
  - `db_routes.rs`: `pub async fn execute_readonly_query(...)`
  - `DataBrowserModal.tsx`: `export function DataBrowserModal(...)`
- **Verification**: Run `SELECT * FROM prompt_history LIMIT 10` in SQL sandbox; verify results render in virtualized grid within 50ms.

---

## Module 9: Speed Error Telemetry & Autonomous Self-Healing

### 9.1 SQLite Speed Error Telemetry (`errors.db`) & Autonomous Fixer
- **Target Files**:
  - `src-tauri/src/modules/speed_errors.rs`
  - `src-tauri/src/modules/failed_commands.rs`
  - `src-tauri/src/bin/agm.rs`
- **Architectural Context & Problem**:
  Runtime faults (such as E1001 proxy failures, E9001 switch collisions, or E5001 token expirations) were logged to flat text files without structured codes, making automated programmatic remediation difficult.
- **Core Invariant**:
  - Errors must be recorded in dedicated `errors.db` with structured diagnostic error codes (E1001–E9001), subsystem labels, stack traces, and JSON context.
  - CLI command `agm errors fix` must parse active faults and trigger targeted self-healing scripts.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/modules/speed_errors.rs`. Verify `errors.db` schema:
     `CREATE TABLE IF NOT EXISTS error_records (id INTEGER PRIMARY KEY, error_code TEXT, module TEXT, message TEXT, context_json TEXT, resolved INTEGER, created_at INTEGER);`
  2. *Step 2*: Implement `record_speed_error(code, module, message, context)` asynchronously using unblocking mpsc channel.
  3. *Step 3*: Open `src-tauri/src/modules/failed_commands.rs`. In `suggest_command`, compute Levenshtein distance against known CLI command verbs; suggest closest match if distance $\le 2$.
  4. *Step 4*: In `agm.rs: cmd_errors`:
     - `agm errors ls`: List recent unresolved errors.
     - `agm errors fix`: For E9001 (collision), purge stale leases. For E5001 (auth), refresh OAuth tokens. For E2001 (SQLite lock), checkpoint WAL.
- **Code Anchors**:
  - `speed_errors.rs`: `pub fn record_speed_error(...)`
  - `agm.rs`: `pub fn cmd_errors(args: &Args) -> Result<(), String>`
- **Verification**: Trigger a mock diagnostic error; run `agm errors ls` and verify record appears with full JSON context.

---

## Module 10: Coding Guideline Modernization & Complexity Elimination

### 10.1 AppError Propagation & Boolean Guard Inversion
- **Target Files**:
  - `src-tauri/src/error.rs`
  - `src-tauri/src/modules/auto_switcher/quota.rs`
  - `src-tauri/src/modules/repo_db/transcript.rs`
  - `src/stores/error-store.ts`
- **Architectural Context & Problem**:
  Codebase audits identified 215 coding guideline violations, including bare `let _ = fs::write(...)` (swallowing errors), nested `if-else` pyramids with 4+ indent levels, and complex compound negative booleans (`if !a && (!b || c)`).
- **Core Invariant**:
  - Every fallible operation must propagate via `Result<T, AppError>`. Swallowing errors is strictly forbidden.
  - Complex boolean conditions must be decomposed into discrete affirmative boolean variables with `is_` or `has_` prefixes.
  - Nested conditionals must be flattened into top-level guard clauses.
- **AI Implementation Journey**:
  1. *Step 1*: Open `src-tauri/src/error.rs`. Ensure `enum AppError` contains standard variants: `Io`, `Json`, `Sqlite`, `Tauri`, `Network`, `Config`, `Process`, `Validation`.
  2. *Step 2*: Open `src-tauri/src/modules/repo_db/transcript.rs`. Audit all file write calls. Replace:
     `let _ = fs::write(&path, data);`
     with:
     `fs::write(&path, data).map_err(|e| AppError::Io(format!("Failed to write transcript: {e}")))?;`
  3. *Step 3*: Open `src-tauri/src/modules/auto_switcher/quota.rs`. Audit nested `if` chains in bucket evaluation. Flatten using early returns and named affirmative booleans:
     ```rust
     let is_target_model = is_model_matching_target(&m.name, target_model);
     if !is_target_model {
         continue;
     }
     let is_within_window = (0.0..=1.0).contains(&b.remaining_fraction);
     if !is_within_window {
         continue;
     }
     ```
  4. *Step 4*: Open `src/stores/error-store.ts`. Ensure frontend errors route through `useErrorStore.getState().trackWarning(e, { source: '...', triggerAction: '...' })`.
- **Code Anchors**:
  - `error.rs`: `pub enum AppError`
  - `quota.rs`: `pub fn evaluate_account_period_status(...)`
  - `transcript.rs`: `pub fn save_transcript_record(...)`
- **Verification**: Run `cd src-tauri && cargo clippy --all-targets --all-features`; verify zero compiler warnings or dead code alerts.

---

## Master Implementation Matrix (200 Improvements Index)

| ID | Title / Feature | Subsystem | Target Files | Invariant / Expected State |
| :--- | :--- | :--- | :--- | :--- |
| **001** | Strict 100% 4h Quota Gate | Auto-Switcher | `scoring.rs`, `selection.rs`, `scoring.ts` | Any account with $Q_{4\text{h}} < 100\%$ strictly scores `0.0` in primary candidate pool. |
| **002** | Dual-Key In-Use Exclusion | Auto-Switcher | `instances.rs`, `manual.rs`, `smart.ts` | Target instance and running siblings strictly excluded by both ID and email. |
| **003** | Adaptive Poller with Jitter | Auto-Switcher | `daemon.rs`, `status.rs` | Polling cadence scales dynamically (15s–60s) with $\pm 10\%$ anti-collision jitter. |
| **004** | Conscious PID Termination | Instances | `lifecycle.rs`, `switch.rs` | Verifies binary name before kill; 3000ms graceful shutdown timeout before SIGKILL. |
| **005** | Synthetic Device Fingerprints | Instances | `clone.rs`, `settings_sync.rs` | Generates unique `storage.json` UUIDs on instance clone to prevent telemetry bleeding. |
| **006** | Prompt Tree 30s Auto-Sync | Prompt Tree | `usePromptTreeData.ts`, `tree.rs` | Dynamic 2s interval on active tasks; periodic timer uses `force: true` cache bypass. |
| **007** | Rerun Multi-Round Pacing | Prompt Tree | `usePromptTreePromptActions.ts` | Inserts 1000ms async pause between rerun iterations to prevent preemption loops. |
| **008** | Canonical IR Envelope | Proxy | `canonical.rs`, `openai.rs`, `claude.rs` | All 4 inbound protocols normalize to Canonical IR before downstream adaptation. |
| **009** | Telegram Sub-Second Dispatch | Telegram | `daemon.rs`, `send_flow.rs`, `cluster.rs` | Detaches proof shot to background task; responds to user within 150ms. |
| **010** | WAL Concurrency Mode | SQLite | `sqlite_manager.rs`, `repo_db/mod.rs` | PRAGMA WAL mode, 5000ms busy timeout, 64MB cache; strict FIFO ordering. |
| **011** | Dark-Glass Design System | React UI | `PromptTreeViewModal.tsx`, `TreePanel.tsx` | Validated Tailwind dark-glass utility combinations, zero stark white fallbacks. |
| **012** | Scoreboard Popover Contrast | React UI | `AlgorithmScoreCell.tsx` | High-contrast dark popover container (`bg-slate-900/95`) with emerald formula box. |
| **013** | MiniView Floating Widget | React UI | `MiniView.tsx`, `useMiniViewState.ts` | Draggable borderless always-on-top micro widget with 1-click fast-forward button. |
| **014** | DB REST Data Browser | Admin UI | `DataBrowserModal.tsx`, `TableGrid.tsx` | Read-only visual data grid and SQL sandbox across split databases. |
| **015** | SQLite Speed Error Telemetry | Telemetry | `speed_errors.rs`, `errors.db`, `agm.rs` | Sub-millisecond error telemetry with diagnostic codes and automated `agm errors fix`. |
| **016** | AppError Propagation Gate | Architecture | `error.rs`, `transcript.rs`, `quota.rs` | Total ban on bare `let _ = ...`; explicit `AppError` and guard clause inversion. |
| ... | *See complete 200 item audit* | *All Modules* | *`02-spec/21-app/156-.../ai-journeys.md`* | *Full blueprints indexed across all 10 architectural modules.* |
