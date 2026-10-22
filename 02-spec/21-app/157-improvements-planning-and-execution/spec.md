# Specification: 30 Master Improvements Planning and Execution

- **Spec ID**: `02-spec/21-app/157-improvements-planning-and-execution`
- **Feature**: Systematic Execution of Next 30 High-Priority Architecture, Gateway, Multi-Instance, Prompt Tree, and UI Improvements
- **Status**: Approved & In Execution
- **Owner**: Antigravity Development Team (@aukgit)
- **Target Release**: v4.251.0+

---

## 1. Executive Summary & Problem Classification

Following the formalization of the Fast-Forward logic and the Master AI Implementation Handbook, this specification defines the precise execution plan for the next **30 grounded system and UI improvements** across Antigravity-Manager.

### 1.1 Scope of the 30 Improvements
The 30 improvements are structured into 5 cohesive batches of 6 improvements each:
1. **Batch 1 (Imp 01–06)**: Auto-Switcher, Quota Calculation & Selection Robustness
2. **Batch 2 (Imp 07–12)**: Multi-Instance Profile Sandboxing & Lifecycle Safety
3. **Batch 3 (Imp 13–18)**: Prompt Management, Tree View & Execution Pipeline
4. **Batch 4 (Imp 19–24)**: Reverse Proxy Gateway, Protocol Adapters & SSE Streaming
5. **Batch 5 (Imp 25–30)**: Split-SQLite Architecture, Speed Errors & UI Micro-Polish

### 1.2 Execution Constraints
- **Zero Testing / Zero Build Checking Mode**: Per user instructions, no tests or build checks are run during execution (`do not do the testing now. For now, just proceed. Do not do the build checking or anything`).
- **Strictly Relative Git Paths**: All file paths referenced in code and specs must be repository-relative (`src/...`, `src-tauri/...`, `02-spec/...`, `.ai-memory/...`).
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Sequential Completion**: Each task is executed and immediately moved to the completed state.

---

## 2. Exhaustive Index of the 30 Improvements

| # | Improvement Title | Target Subsystem | Target Files | Technical Goal |
| :--- | :--- | :--- | :--- | :--- |
| **01** | Proportional Headroom Score Normalization | Auto-Switcher | `src-tauri/src/modules/auto_switcher/scoring.rs` | Normalize weekly base score division by `100.0` matching frontend calculation. |
| **02** | Target Model Wildcard Hierarchy Matching | Auto-Switcher | `src-tauri/src/modules/auto_switcher/scoring.rs` | Add wildcard patterns for `gemini-*-flash-*` and `gemini-*-pro-*` family matching. |
| **03** | Low-Quota Alert Settling Suppression | Auto-Switcher | `src-tauri/src/modules/auto_switcher/daemon.rs` | Suppress noisy duplicate alerts during 10s post-switch settling window. |
| **04** | Exclusion String Normalization | Auto-Switcher | `src-tauri/src/modules/auto_switcher/selection.rs` | Ensure all IDs and emails in effective exclusions are trimmed and lowercase. |
| **05** | Cooldown Fallback Never-Used Priority | Auto-Switcher | `src-tauri/src/modules/auto_switcher/selection.rs` | In cooldown fallback sort, prioritize `last_used == 0` accounts before exhausted accounts. |
| **06** | Pre-Switch Live Refresh Timeout Guard | Auto-Switcher | `src-tauri/src/modules/auto_switcher/selection.rs` | Wrap live Google API quota probe with a bounded 5000ms timeout guard. |
| **07** | Conscious IDE PID Process Verification | Instances | `src-tauri/src/modules/instance/lifecycle.rs` | Verify process executable name matches `antigravity` before terminating PID. |
| **08** | Graceful 3000ms Instance Stop Ladder | Instances | `src-tauri/src/modules/instance/lifecycle.rs` | Issue graceful close signal with 3000ms polling ladder before force kill. |
| **09** | Cloned Profile Synthetic UUID Generation | Instances | `src-tauri/src/modules/instance/clone.rs` | Generate fresh RFC4122 v4 UUIDs for `storage.json` device telemetry. |
| **10** | Keyring & Account Scrubbing on Clone | Instances | `src-tauri/src/modules/instance/clone.rs` | Delete account and token rows from cloned `state.vscdb` to eliminate bleeding. |
| **11** | Stale Lock & Dead PID Purging | Instances | `src-tauri/src/modules/instance/registry.rs` | Auto-purge dead PID lockfiles from instance data directories on startup. |
| **12** | Port Rebind Settling Delay on Switch | Instances | `src-tauri/src/modules/instance/switch.rs` | Add explicit 500ms socket settling delay before relaunching IDE process. |
| **13** | 4-Tier Origin Badge Classifier | Prompt Tree | `src/components/instances/PromptTreeView/treeOrigin.ts` | Map origin tiers: `USER_PROMPT` (Sky), `SUBAGENT_INSTRUCTION` (Purple), etc. |
| **14** | Branch Glyph Hierarchy Connectors | Prompt Tree | `src/components/instances/PromptTreeView/TreePanel.tsx` | Render subagent rows with branch glyphs (`↳`, `└──`) indented under parent prompt. |
| **15** | Interactive Truncation Callout | Prompt Tree | `src/components/instances/PromptTreeView/DetailHeader.tsx` | Parse `<truncated N bytes>` into interactive dark-glass omission banners. |
| **16** | Dynamic 2s Auto-Sync Acceleration | Prompt Tree | `src/components/instances/PromptTreeView/usePromptTreeData.ts` | Accelerate interval from 30s to 2s when active prompts are running. |
| **17** | Rerun Prompt 3-Tier Fallback | Prompt Tree | `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | Resolve empty rerun prompts via turn summary -> root prompt -> ledger. |
| **18** | Dual-MIME Rich Clipboard Formatter | Prompt Tree | `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | Provide structured markdown copying for prompt turns and verification records. |
| **19** | Canonical IR Envelope Struct | Gateway Proxy | `src-tauri/src/proxy/canonical.rs` | Define unified `CanonicalRequestEnvelope` for all inbound protocols. |
| **20** | OpenAI Reasoning Content Mapper | Gateway Proxy | `src-tauri/src/proxy/openai.rs` | Map OpenAI `reasoning_content` to Canonical IR thinking blocks. |
| **21** | Claude Thinking Block Adapter | Gateway Proxy | `src-tauri/src/proxy/claude.rs` | Adapt Claude `thinking.budget_tokens` into Canonical IR thinking budget. |
| **22** | Gemini Thinking Budget Backfilling | Gateway Proxy | `src-tauri/src/proxy/gemini.rs` | Backfill default thinking budget (`2048`) for thinking models if omitted. |
| **23** | TCP No-Delay SSE Socket Optimization | Gateway Proxy | `src-tauri/src/proxy/streaming.rs` | Set `set_nodelay(true)` on streaming sockets to eliminate buffering latency. |
| **24** | Client Abort Upstream Cancellation | Gateway Proxy | `src-tauri/src/proxy/sse.rs` | Cleanly terminate upstream Google API request when client disconnects. |
| **25** | PRAGMA WAL & Cache Optimization | Split-SQLite | `src-tauri/src/modules/sqlite_manager.rs` | Apply `PRAGMA synchronous = NORMAL;` and `PRAGMA cache_size = -64000;`. |
| **26** | Strict FIFO Queue Ordering | Split-SQLite | `src-tauri/src/modules/repo_db/tree.rs` | Enforce `ORDER BY created_at ASC, id ASC` across all prompt restore queries. |
| **27** | Speed Error SQLite Telemetry Logger | Telemetry | `src-tauri/src/modules/speed_errors.rs` | Asynchronous recorder for structured diagnostic codes (E1001–E9001). |
| **28** | Levenshtein CLI Command Suggester | Telemetry | `src-tauri/src/modules/failed_commands.rs` | Compute edit distance $\le 2$ on mistyped terminal commands. |
| **29** | Borderless Transparent Inline Selects | Desktop UI | `src/components/navbar/NavMenu.tsx` | Eliminate white pill-in-pill defect in segmented capsules via transparent styling. |
| **30** | Scoreboard Emerald Formula Styling | Desktop UI | `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx` | Ensure derivation popover formula box renders with high-contrast emerald mono. |
