# Plan: 30 Improvements Planning and Execution

## Status
`[COMPLETED]`

## User Request (Verbatim)
```text
The improvements that you have, can you please start working on those improvements? Share the steps, how you are going to follow through, and which improvements you are planning. Let's plan next 30 improvements. First, list out the 30 improvements that you are going to do, break it down to step by step, and then do it and move it to completed state. And also, do not do the testing now. For now, just proceed. Do not do the build checking or anything. Always complete the task and move to the next one for now.
```

## Architecture Roadmap & Subtask Allocation (All Completed)

1. **Subtask-01: Batch 1 Execution (Improvements 01–06: Auto-Switcher & Quota Robustness)**
   - File: `.ai-memory/plans/subtasks/157-improvements-planning-and-execution/01-batch1-auto-switcher-improvements.md`
   - Scope: Proportional headroom scoring normalization, model wildcard hierarchy matching, low-quota alert settling suppression, exclusion string normalization, cooldown fallback never-used priority, and pre-switch live probe 5s timeout guard.
   - Status: `[COMPLETED]`

2. **Subtask-02: Batch 2 Execution (Improvements 07–12: Multi-Instance Sandboxing & Lifecycle)**
   - File: `.ai-memory/plans/subtasks/157-improvements-planning-and-execution/02-batch2-multi-instance-sandboxing.md`
   - Scope: Conscious IDE PID process verification, graceful 3000ms instance stop ladder, cloned profile synthetic UUID generation, keyring and account scrubbing on clone, stale lockfile purging, and port rebind settling delay on switch.
   - Status: `[COMPLETED]`

3. **Subtask-03: Batch 3 Execution (Improvements 13–18: Prompt Management & Tree View)**
   - File: `.ai-memory/plans/subtasks/157-improvements-planning-and-execution/03-batch3-prompt-tree-improvements.md`
   - Scope: 4-tier origin hierarchy badge classifier, branch glyph connectors, interactive truncation callouts, dynamic 2s auto-sync acceleration, rerun prompt 3-tier fallback resolution, and rich clipboard formatting.
   - Status: `[COMPLETED]`

4. **Subtask-04: Batch 4 Execution (Improvements 19–24: Reverse Proxy & Protocol Adapters)**
   - File: `.ai-memory/plans/subtasks/157-improvements-planning-and-execution/04-batch4-proxy-gateway-improvements.md`
   - Scope: Canonical IR envelope struct, OpenAI reasoning content mapper, Claude thinking block adapter, Gemini thinking budget backfilling, TCP no-delay SSE socket optimization, and client abort upstream cancellation.
   - Status: `[COMPLETED]`

5. **Subtask-05: Batch 5 Execution (Improvements 25–30: Split-SQLite, Telemetry & UI Polish)**
   - File: `.ai-memory/plans/subtasks/157-improvements-planning-and-execution/05-batch5-split-sqlite-and-ui-polish.md`
   - Scope: PRAGMA WAL & 64MB cache optimization, strict FIFO queue ordering verification, speed error SQLite telemetry logger, Levenshtein CLI command suggester, borderless transparent inline selects, and scoreboard emerald formula styling.
   - Status: `[COMPLETED]`

---

## 30 Improvements Verification Matrix

| # | Improvement | Target Subsystem | Primary File | Outcome |
| :--- | :--- | :--- | :--- | :--- |
| **01** | Proportional Headroom Score Normalization | Auto-Switcher | `src-tauri/src/modules/auto_switcher/scoring.rs` | Applied `/ 100.0` normalization matching frontend formula. |
| **02** | Target Model Wildcard Hierarchy Matching | Auto-Switcher | `src-tauri/src/modules/auto_switcher/scoring.rs` | Added wildcard patterns for `gemini-*-flash-*`, `gemini-*-pro-*`, `gpt`, `o1`, `o3`. |
| **03** | Low-Quota Alert Settling Suppression | Auto-Switcher | `src-tauri/src/modules/auto_switcher/daemon.rs` | Implemented 10-second post-switch settling suppression. |
| **04** | Exclusion String Normalization | Auto-Switcher | `src-tauri/src/modules/auto_switcher/selection.rs` | Normalized exclusions with `.trim().to_lowercase()` deduplication. |
| **05** | Cooldown Fallback Never-Used Priority | Auto-Switcher | `src-tauri/src/modules/auto_switcher/selection.rs` | Prioritized never-used (`last_used == 0`) accounts first in cooldown sort. |
| **06** | Pre-Switch Live Refresh Timeout Guard | Auto-Switcher | `src-tauri/src/modules/auto_switcher/selection.rs` | Wrapped Google API live quota probe in 5000ms `tokio::time::timeout`. |
| **07** | Conscious IDE PID Process Verification | Instances | `src-tauri/src/modules/instance/lifecycle.rs` | Added `is_conscious_ide_pid(pid)` verifying executable belongs to Antigravity IDE. |
| **08** | Graceful 3000ms Instance Stop Ladder | Instances | `src-tauri/src/modules/instance/close.rs` | Extended graceful exit ladder to 3000ms before force kill. |
| **09** | Cloned Profile Synthetic UUID Generation | Instances | `src-tauri/src/modules/instance/clone.rs` | Generated fresh RFC4122 v4 UUIDs for `storage.json` telemetry & machine ID. |
| **10** | Keyring & Account Scrubbing on Clone | Instances | `src-tauri/src/modules/db.rs` | Scrubbed account, oauth, and auth session rows from cloned `state.vscdb`. |
| **11** | Stale Lock & Dead PID Purging | Instances | `src-tauri/src/modules/instance/registry.rs` | Auto-purged stale lockfiles from data dirs of inactive instances on startup. |
| **12** | Port Rebind Settling Delay on Switch | Instances | `src-tauri/src/modules/instance/switch.rs` | Added 500ms socket settling delay before spawning new IDE instance. |
| **13** | 4-Tier Origin Badge Classifier | Prompt Tree | `src/components/instances/PromptTreeView/treeOrigin.ts` | Created canonical origin classifier for User, Subagent, System, and Tool Output. |
| **14** | Branch Glyph Hierarchy Connectors | Prompt Tree | `src/components/instances/PromptTreeView/ConversationNode.tsx` | Preserved and enforced branch glyphs (`└── ↳`, `├──`). |
| **15** | Interactive Truncation Callout | Prompt Tree | `src/components/instances/PromptTreeView/TruncatedContextCallout.tsx` | Integrated dark-glass omission banner across preview & markdown renderers. |
| **16** | Dynamic 2s Auto-Sync Acceleration | Prompt Tree | `src/components/instances/PromptTreeView/usePromptTreeData.ts` | Accelerated poll interval from 30s to 2s when active prompts are running. |
| **17** | Rerun Prompt 3-Tier Fallback | Prompt Tree | `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | 3-tier fallback: turn summary -> root prompt summary/title -> ledger. |
| **18** | Dual-MIME Rich Clipboard Formatter | Prompt Tree | `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | Implemented `buildStructuredTurnsMarkdown` multi-turn formatter. |
| **19** | Canonical IR Envelope Struct | Gateway Proxy | `src-tauri/src/proxy/canonical.rs` | Defined protocol-agnostic `CanonicalRequestEnvelope` and `CanonicalMessage`. |
| **20** | OpenAI Reasoning Content Mapper | Gateway Proxy | `src-tauri/src/proxy/handlers/openai/chat_conversion.rs` | Verified reasoning_content extraction into reasoning summary blocks. |
| **21** | Claude Thinking Block Adapter | Gateway Proxy | `src-tauri/src/proxy/handlers/claude/thinking.rs` | Extracted and normalized Claude thinking budget tokens. |
| **22** | Gemini Thinking Budget Backfilling | Gateway Proxy | `src-tauri/src/proxy/pipeline/inbound/thinking.rs` | Backfilled default 2048 thinking budget for thinking models when omitted. |
| **23** | TCP No-Delay SSE Socket Optimization | Gateway Proxy | `src-tauri/src/proxy/server/bind.rs` | Applied `socket.set_nodelay(true)` on single and dual-stack listeners. |
| **24** | Client Abort Upstream Cancellation | Gateway Proxy | `src-tauri/src/proxy/server/axum_server.rs` | Verified cancellation propagation upon downstream connection abort. |
| **25** | PRAGMA WAL & Cache Optimization | Split-SQLite | `src-tauri/src/modules/db_orm/connection.rs` | Applied `PRAGMA cache_size = -64000;` (64MB memory page cache). |
| **26** | Strict FIFO Queue Ordering | Split-SQLite | `src-tauri/src/modules/repo_db/restore.rs` | Inverted LIFO order replaced with strict `ORDER BY created_at ASC, id ASC`. |
| **27** | Speed Error SQLite Telemetry Logger | Telemetry | `src-tauri/src/modules/speed_errors.rs` | Created asynchronous diagnostic telemetry recorder for codes E1001–E9001. |
| **28** | Levenshtein CLI Command Suggester | Telemetry | `src-tauri/src/modules/failed_commands.rs` | Created dynamic Levenshtein distance $\le 2$ command suggester. |
| **29** | Borderless Transparent Inline Selects | Desktop UI | `src/components/navbar/NavMenu.tsx` | Enforced borderless transparent styling in segmented capsules. |
| **30** | Scoreboard Emerald Formula Styling | Desktop UI | `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx` | High-contrast emerald mono formula box on dark card container. |
