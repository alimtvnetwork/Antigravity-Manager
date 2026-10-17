# Code DRYness & Guard Clause Inversion — 100-Step Master Plan

## Executive Overview & Architectural Invariants
- **Primary Objective**: Systematically eliminate magic strings, hardcoded constants, duplicated conditional logic, deeply nested `if` statements, and repetitive code across `Antigravity-Manager`.
- **Core Principles**:
  1. **Single Source of Truth**: Every status, action, channel name, and timeout must originate from a dedicated typed constant or enum.
  2. **Guard Clause Precedence (Quick Exit)**: Invert conditions to exit as early as possible, keeping the happy path at the root indentation level.
  3. **Module Bounded Size**: Decompose oversized functions (>15 lines) and oversized files (>150 lines) into focused, composable helpers.
  4. **Dynamic Interpolation**: Never hardcode values in messages that are controlled by constants or runtime parameters.

---

## The 100-Step DRY Roadmap

### Block 1: Status & Enum Constants Centralization (Steps 1–10)
- [ ] **Step 001**: Define `PromptStatus` enum in `src-tauri/src/modules/repo_db/models.rs` (`Running`, `Queued`, `Pending`, `Dispatched`, `BackedUp`, `Completed`, `Failed`).
- [ ] **Step 002**: Replace magic strings `"running"`, `"queued"`, `"pending"`, `"dispatched"` across `prompts_crud.rs` with `PromptStatus` constants.
- [ ] **Step 003**: Replace prompt status literals in `tree.rs` and `backup.rs` with `PromptStatus`.
- [ ] **Step 004**: Define `AccountState` / `AccountTier` enums in `src-tauri/src/modules/account/` (`Active`, `Cooldown`, `Disabled`, `Exhausted`, `Ultra`, `Pro`, `Free`).
- [ ] **Step 005**: Replace raw tier strings (`"ultra"`, `"pro"`, `"free"`) across candidate scoring algorithms with `AccountTier`.
- [ ] **Step 006**: Define `InstanceLifecycleState` enum in `src-tauri/src/modules/instance/` (`Running`, `Stopped`, `Launching`, `Terminating`).
- [ ] **Step 007**: Centralize notification channel identifiers (`Telegram`, `Email`, `Supabase`, `DesktopToast`) in `notification_hub/`.
- [ ] **Step 008**: Replace `"default"` instance and project fallback literals with `pub const DEFAULT_PROJECT_NAME: &str = "default";`.
- [ ] **Step 009**: Unify Telegram command prefixes and callback action verbs (`"cmd:"`, `"proj:"`, `"inst:"`) in `telegram_inbound/constants.rs`.
- [ ] **Step 010**: Centralize CLI subcommand string constants across `src-tauri/src/bin/agm/` and `src-tauri/src/modules/cli/`.

### Block 2: Timeouts, Intervals & Rate Limits Centralization (Steps 11–20)
- [ ] **Step 011**: Define `SystemIntervals` in a central module (`MIN_BROADCAST_INTERVAL_SECS = 30`, `DEFAULT_HEARTBEAT_SECS = 30`).
- [ ] **Step 012**: Reconcile all background worker polling intervals in `daemon.rs` and `scheduler.rs` to use `SystemIntervals`.
- [ ] **Step 013**: Extract duplicate suppression window constants (`DUP_SWITCH_SUPPRESSION_SECS = 30`) across notification channels.
- [ ] **Step 014**: Consolidate HTTP upstream timeouts (`CONNECT_TIMEOUT_SECS = 10`, `REQUEST_TIMEOUT_SECS = 60`) in proxy client.
- [ ] **Step 015**: Centralize TCP probe timeouts and retry ladders (`TCP_PROBE_INITIAL_MS = 500`, `TCP_PROBE_MAX_MS = 1500`) in fleet sync.
- [ ] **Step 016**: Extract WebSocket / SSE keepalive ping intervals into shared proxy configuration constants.
- [ ] **Step 017**: Unify SQLite busy timeout durations (`SQLITE_BUSY_TIMEOUT_MS = 5000`) across all split-DB connection pools.
- [ ] **Step 018**: Standardize debouncing intervals for window resize, file watchers, and settings auto-save.
- [ ] **Step 019**: Extract process termination grace periods (`SIGTERM_WAIT_MS = 3000`, `KILL_FORCE_WAIT_MS = 1000`) in `process/`.
- [ ] **Step 020**: Harmonize frontend query refresh intervals and polling ticks in Zustand stores.

### Block 3: Guard Inversion & Quick Exit in `repo_db` (Steps 21–30)
- [ ] **Step 021**: Invert nested `if` in `start_prompt_goal_heartbeat` (`goal_heartbeat.rs`) using guard clauses for workspace validation.
- [ ] **Step 022**: Flatten nested loops and conditions in `inspect_prompt_goal_status` with `let ... else { return ... }` pattern.
- [ ] **Step 023**: Refactor `ensure_prompt_goals_running_for_instance` to quick-exit on missing registry or empty workspaces.
- [ ] **Step 024**: Flatten nested `if let Ok(...)` blocks in `run_goal_worker_loop` to reduce indentation from 5 levels to 1 level.
- [ ] **Step 025**: Rewrite `tree.rs` workspace directory scanning with early continues for non-existent or hidden paths.
- [ ] **Step 026**: Invert error handling in `prompts_crud.rs::save_or_requeue_prompt` to return early on validation failures.
- [ ] **Step 027**: Simplify `backup.rs` prompt backup queries using guard clauses for un-namespaced instance IDs.
- [ ] **Step 028**: Extract repetitive transcript JSON parsing logic in `tree.rs` into a standalone helper function.
- [ ] **Step 029**: Encapsulate project status calculation in `tree.rs` into a pure evaluation helper.
- [ ] **Step 030**: Replace nested match branches in SQLite row deserialization with combinator functions (`map`, `and_then`).

### Block 4: Guard Inversion & Quick Exit in `telegram_inbound` (Steps 31–40)
- [ ] **Step 031**: Flatten nested `if` structure in `send_flow.rs::handle_callback` using an early-return dispatcher.
- [ ] **Step 032**: Invert conditions in `send_flow.rs::machine_entries` to filter out non-matching nodes early.
- [ ] **Step 033**: Rewrite `destination_locked_reply` with early returns for missing instances or inactive workspaces.
- [ ] **Step 034**: Flatten nested token parsing in `injection.rs` using structured token iterator helpers.
- [ ] **Step 035**: Invert permission and auth check in `processing.rs` to reject unauthorized Telegram chat IDs immediately.
- [ ] **Step 036**: Flatten `open_shot.rs` window screenshot capture with guard clauses on OS window handles.
- [ ] **Step 037**: Encapsulate multi-step machine label matching in `node_label.rs` into early-exit boolean predicates.
- [ ] **Step 038**: Simplify Telegram message chunking (4000-char limit) into a recursive or clean iterator pipeline.
- [ ] **Step 039**: Extract Telegram markdown/HTML entity sanitization into a single DRY utility function.
- [ ] **Step 040**: Invert command queue insertion checks in `cluster.rs` with early returns for offline nodes.

### Block 5: Guard Inversion & Quick Exit in `instance` & `process` (Steps 41–50)
- [ ] **Step 041**: Flatten `launch.rs::launch_instance_inner_with_extra_workspaces` by inverting process existence checks.
- [ ] **Step 042**: Replace deep nested `if` chains in `pid_scan.rs` with iterator filters and early exits.
- [ ] **Step 043**: Invert conditions in `close_instance_verified` to return immediately if the target process is already dead.
- [ ] **Step 044**: Simplify Windows Job Object breakaway fallback ladder with a sequential loop over fallback strategies.
- [ ] **Step 045**: Flatten credential injection logic in `credentials.rs` by checking SQLite file locks up front.
- [ ] **Step 046**: Invert workspace storage directory validation to return early on invalid or missing paths.
- [ ] **Step 047**: Flatten `ensure_instance_running_smart` to quick-exit when cached PID vitality is confirmed.
- [ ] **Step 048**: Extract process command line token extraction into a reusable helper with early exit on empty args.
- [ ] **Step 049**: Invert registry file read/write locks to handle poison errors immediately without deep indentation.
- [ ] **Step 050**: Encapsulate environment variable overrides into a single clean map-transformation helper.

### Block 6: Proxy Adapters & Protocol Normalization DRYness (Steps 51–60)
- [ ] **Step 051**: Deduplicate request header forwarding logic across OpenAI, Claude, and Gemini proxy adapters.
- [ ] **Step 052**: Extract streaming SSE chunk parsing and delimiter handling into a shared `sse_codec.rs`.
- [ ] **Step 053**: Unify thinking block injection and thought tag extraction across upstream protocol mappers.
- [ ] **Step 054**: Flatten nested JSON schema validation in `apply_patch_preflight` with early returns.
- [ ] **Step 055**: Centralize model name aliases and wildcard patterns (`gemini-*-flash-*`, `claude-*`) in a routing table.
- [ ] **Step 056**: Invert rate-limit detection and token replenishment checks in proxy middleware.
- [ ] **Step 057**: Extract upstream fallback retry logic into a generic retry policy executor.
- [ ] **Step 058**: Deduplicate signature cache lookup and L1/L2 cache hydration logic.
- [ ] **Step 059**: Invert payload size checks to reject oversized bodies immediately before buffer allocation.
- [ ] **Step 060**: Standardize proxy error JSON responses into a single canonical error constructor.

### Block 7: Split-SQLite Architecture & Query Builder DRYness (Steps 61–70)
- [ ] **Step 061**: Consolidate database path resolution logic across all 6 split SQLite databases.
- [ ] **Step 062**: Extract standard PRAGMA initialization (`WAL`, `busy_timeout=5000`, `foreign_keys=ON`) into a shared helper.
- [ ] **Step 063**: Deduplicate table migration runner logic across `task_history_db`, `training_vault`, and `repo_db`.
- [ ] **Step 064**: Invert connection pool acquisition checks with early error propagation.
- [ ] **Step 065**: Create a reusable generic query pagination helper (`paginate<T>(query, limit, offset)`).
- [ ] **Step 066**: Replace repeated `ORDER BY created_at ASC, id ASC` boilerplate with a FIFO query trait or macro.
- [ ] **Step 067**: Standardize SQLite transaction wrapper with automatic rollback on error.
- [ ] **Step 068**: Encapsulate database lock recovery and stale lockfile cleanup into a unified utility.
- [ ] **Step 069**: Deduplicate JSON field serialization / deserialization in SQLite query adapters.
- [ ] **Step 070**: Consolidate database backup and vacuuming routines into a unified maintenance engine.

### Block 8: Frontend UI Component & Layout DRYness (Steps 71–80)
- [ ] **Step 071**: Extract dark-glass segmented pill capsule styling (`rounded-full`, shared border, dark glass) into a shared CSS class / component.
- [ ] **Step 072**: Deduplicate transparent borderless `<select>` styling across toolbar capsules.
- [ ] **Step 073**: Unify context menu positioning, coordinate clamping, and click-outside dismissal in a `useContextMenu` hook.
- [ ] **Step 074**: Consolidate clipboard copy operations with toast feedback into a single `copyToClipboardWithFeedback` helper.
- [ ] **Step 075**: Invert conditions in `PromptTreeViewModal.tsx` rendering to exit early on loading or empty state.
- [ ] **Step 076**: Deduplicate modal backdrop, ESC key listener, and focus trapping across all dialogs.
- [ ] **Step 077**: Extract quota progress bar styling and checkpoint indicators into a shared atomic primitive.
- [ ] **Step 078**: Harmonize table row hover inversion styles across Accounts Table and Instances Table.
- [ ] **Step 079**: Centralize theme color token constants (`primary`, `accent`, `progress-green`) in Tailwind and JS constants.
- [ ] **Step 080**: Consolidate keyboard shortcut bindings (`F2`, `Enter`, `Escape`) into a declarative hook.

### Block 9: CLI Commands & Envelopes DRYness (Steps 81–90)
- [ ] **Step 081**: Unify CLI JSON envelope construction (`{ "status": "success", "data": ... }`) in `agm/common.rs`.
- [ ] **Step 082**: Extract target instance and project resolution ladder across CLI prompt commands.
- [ ] **Step 083**: Deduplicate sequence token formatting (`[AGM:P001 | GM:#1]`) into a shared formatter.
- [ ] **Step 084**: Invert CLI argument parsing in `prompt_cmds.rs` with early exit on missing required flags.
- [ ] **Step 085**: Consolidate CLI table rendering and colorized status glyphs into a single output helper.
- [ ] **Step 086**: Deduplicate remote SSH command execution wrappers in `ssh_cmds.rs`.
- [ ] **Step 087**: Extract updater download progress display and console configuration into a shared module.
- [ ] **Step 088**: Centralize exit codes (`0: Success`, `1: Error`, `2: Refusal/Self-Preservation`) into an enum.
- [ ] **Step 089**: Invert command prerequisite checks (e.g. GitMap binary presence, SSH keys) with early error returns.
- [ ] **Step 090**: Standardize CLI interactive confirmation prompts (`[y/N]`) into a reusable input helper.

### Block 10: Code Modularization & Line Count Reduction (Steps 91–100)
- [ ] **Step 091**: Decompose `src-tauri/src/modules/repo_db/goal_heartbeat.rs` (467 lines) into config, worker, and inspection modules (<= 150 lines each).
- [ ] **Step 092**: Decompose `src-tauri/src/modules/repo_db/tree.rs` into tree scan, caching, and model modules.
- [ ] **Step 093**: Decompose `src-tauri/src/modules/telegram_inbound/send_flow.rs` into wizard stages, keyboard builders, and dispatchers.
- [ ] **Step 094**: Decompose `src-tauri/src/modules/notification_hub/channels.rs` into dedicated email, telegram, and supabase channels.
- [ ] **Step 095**: Decompose `src-tauri/src/modules/instance/launch.rs` into command builder, process spawner, and post-launch verifier.
- [ ] **Step 096**: Split oversized frontend component `DetailHeader.tsx` into action capsules, tab bar, and navigation controls.
- [ ] **Step 097**: Split `PromptTreeViewModal.tsx` into sidebar tree container and details pane.
- [ ] **Step 098**: Audit all functions exceeding 15 lines across refactored modules and extract focused sub-helpers.
- [ ] **Step 099**: Verify repository-wide DRY compliance, zero compiler warnings, and clean formatting.
- [ ] **Step 100**: Author summary audit report and mark all 100 steps verified.
