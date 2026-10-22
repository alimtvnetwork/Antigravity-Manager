# Specification: Next 30 Architecture, Gateway & UI Improvements (31–60)

## 1. Executive Summary

This document specifies the execution plan for the **next 30 system, gateway, database, and UI improvements (Improvements 31–60)** across Antigravity-Manager, following the Master AI Implementation Handbook.

### 1.1 Architecture Domains & Subtasks
1. **Batch 1 (Imp 31–36)**: Telegram Inbound Control & Remote Dispatch Latency Optimization
2. **Batch 2 (Imp 37–42)**: Supabase Distributed Leases & Multi-Node Cluster Arbitration
3. **Batch 3 (Imp 43–48)**: Split-SQLite Concurrency, Partitioning & Auto-Vacuum
4. **Batch 4 (Imp 49–54)**: Proxy Security, IP Filters & Upstream Network Resilience
5. **Batch 5 (Imp 55–60)**: UI State Resilience, Accessibility & Error Visibility

### 1.2 Execution Constraints
- **Zero Testing / Zero Build Checking Mode**: Per user instructions, no tests or build checks are run during execution (`do not do the testing now. For now, just proceed. Do not do the build checking or anything`).
- **Strictly Relative Git Paths**: All file paths referenced in code and specs must be repository-relative (`src/...`, `src-tauri/...`, `02-spec/...`, `.ai-memory/...`).
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Sequential Completion**: Each task is executed and immediately moved to the completed state.

---

## 2. Exhaustive Index of Improvements 31–60

| # | Improvement Title | Target Subsystem | Target Files | Technical Goal |
| :--- | :--- | :--- | :--- | :--- |
| **31** | Telegram Startup Sleep Reduction | Telegram | `src-tauri/src/modules/telegram_inbound/daemon.rs` | Reduce daemon startup sleep from 60s to 500ms for immediate inbound polling. |
| **32** | Immediate Callback Spinner Dismissal | Telegram | `src-tauri/src/modules/telegram_inbound/daemon.rs` | Call `answerCallbackQuery` immediately before heavy report generation. |
| **33** | Telegram Instant Prompt Confirmation | Telegram | `src-tauri/src/modules/telegram_inbound/send_flow.rs` | Send instant text confirmation before executing screenshot capture. |
| **34** | Detached Background Proof Capture | Telegram | `src-tauri/src/modules/telegram_inbound/send_flow.rs` | Detach `capture_and_send_proof_shot` into background Tokio task. |
| **35** | Cluster Node Discovery 30s TTL Cache | Telegram | `src-tauri/src/modules/telegram_inbound/cluster.rs` | Cache GitMap cluster node discovery results in memory for 30 seconds. |
| **36** | Telegram Markdown Chunking Buffer | Telegram | `src-tauri/src/modules/telegram_inbound/prompt_history.rs` | Enforce 4000-character chunking guard preserving code blocks. |
| **37** | Workspace Lease Heartbeat Renewal | Supabase | `src-tauri/src/modules/workspace_lease_manager.rs` | Implement periodic lease renewal heartbeat during active instance tasks. |
| **38** | Multi-Node Lease Expiration Guard | Supabase | `src-tauri/src/modules/workspace_lease_manager.rs` | Reject candidate account if held by another active node whose lease has not expired. |
| **39** | Universal URL Endpoint Normalization | Supabase | `src-tauri/src/modules/supabase_client.rs` | Normalize Supabase project URLs stripping trailing slashes and default ports. |
| **40** | Multi-VM Inbound Email Collision Shield | Remote Daemon | `src-tauri/src/modules/email_inbound.rs` | Filter recently switched accounts across VMs within 3600-second window. |
| **41** | Background Node Telemetry Push Guard | Supabase | `src-tauri/src/modules/supabase_sync.rs` | Non-blocking telemetry push with error recording. |
| **42** | Offline Command Queue Fallback Ladder | Remote Daemon | `src-tauri/src/modules/supabase_command_queue.rs` | Gracefully degrade to local execution when Supabase is unreachable. |
| **43** | Dedicated Read-Only Connection Helper | Split-SQLite | `src-tauri/src/modules/db_orm/connection.rs` | Provide `open_split_db_readonly` connection factory with read-only flags. |
| **44** | Incremental Auto-Vacuum Configuration | Split-SQLite | `src-tauri/src/modules/db_orm/connection.rs` | Execute `PRAGMA auto_vacuum = INCREMENTAL;` to prevent database bloat. |
| **45** | Active Prompts FIFO Index Creation | Split-SQLite | `src-tauri/src/modules/repo_db/schema.rs` | Create index `idx_active_prompts_fifo` on `(status, created_at, id)`. |
| **46** | Task History 500-Row Auto-Rotation | Split-SQLite | `src-tauri/src/modules/task_history_db.rs` | Partition historical audit trail when records exceed 500 rows. |
| **47** | Prepared Statement Caching for CRUD | Split-SQLite | `src-tauri/src/modules/repo_db/prompts_crud.rs` | Cache common query statements for rapid prompt status updates. |
| **48** | SQLite Startup Integrity Check Guard | Split-SQLite | `src-tauri/src/modules/db_orm/connection.rs` | Run quick integrity check on boot and log speed error on failure. |
| **49** | Zero-Allocation Case Header Matching | Reverse Proxy | `src-tauri/src/proxy/handlers/common.rs` | Match protocol headers case-insensitively using byte-level ASCII checks. |
| **50** | Upstream HTTP/2 Keep-Alive Optimization | Reverse Proxy | `src-tauri/src/proxy/upstream/client/client_core.rs` | Tune connection pool keep-alive and HTTP/2 settings for upstream clients. |
| **51** | Stream Client Disconnect Abort Guard | Reverse Proxy | `src-tauri/src/proxy/handlers/openai/chat_completions_send.rs` | Terminate upstream streaming pipeline when downstream client aborts. |
| **52** | Proxy Pool Latency Decay Scoring | Reverse Proxy | `src-tauri/src/proxy/proxy_pool.rs` | Apply exponential decay to egress proxy latency scores. |
| **53** | CIDR Range Matching Security Guard | Reverse Proxy | `src-tauri/src/proxy/security.rs` | Validate client IP against CIDR subnets before proxying requests. |
| **54** | Authorization Header Log Scrubbing | Reverse Proxy | `src-tauri/src/proxy/payload_audit/payload.rs` | Mask bearer tokens and sensitive credentials in debug audit payloads. |
| **55** | Frontend TrackWarning Error Routing | Desktop UI | `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | Route caught clipboard and modal errors to `trackWarning`. |
| **56** | Segmented Capsule Borderless Dropdown | Desktop UI | `src/components/instances/PromptTreeView/DetailHeader.tsx` | Ensure inline `<select>` uses borderless transparent styling. |
| **57** | Scoreboard Tooltip Escape Dismissal | Desktop UI | `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx` | Close floating derivation card on `Escape` key press. |
| **58** | Instance PID Monospace Styling | Desktop UI | `src/components/instances/InstanceCard.tsx` | Render process PID and port in high-contrast emerald mono badge. |
| **59** | Prompt Tree Modal Escape Trapping | Desktop UI | `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | Trap Escape key to close modal cleanly without losing draft input. |
| **60** | Keyboard Navigation for Prompt Tree | Desktop UI | `src/components/instances/PromptTreeView/TreePanel.tsx` | Support ArrowUp/ArrowDown navigation across conversation nodes. |
