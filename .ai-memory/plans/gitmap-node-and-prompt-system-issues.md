# Master Execution Plan: gitmap-node-and-prompt-system-issues

## Strategic Goal
Resolve critical architectural, usability, and stability issues across four key subsystems in Antigravity-Manager:
1. **GitMap Nodes & SSH Cluster Management**:
   - Repair node aliasing (GitHub/GitMap cluster integration).
   - Fix instance and account metadata discovery so nodes display bound accounts and real instance counts instead of dummy `(unbound)` placeholders.
   - Fix SSH connection testing and password/key authentication pipelines.
2. **Prompt Tree & UX System**:
   - Fix unresponsive or miswired action buttons: Send, Backup (direct JSON file save), Restore (direct JSON file open), Reset (cache purge with visual feedback), and Refresh (spinning indicator and tree reload).
   - Eliminate screen glitches and flickering caused by destructive skeleton swapping during background refreshes and auto-selection flapping.
   - Enforce top-priority ordering for all `RUNNING` and `QUEUED`/`ENQUEUED` prompts with explicit amber/emerald badges.
   - Declutter Prompt Preview header: extract duplicate tokens (`User Prompt` text, `default-copy-5597` instance name) into the existing `Details` modal.
   - Fix Import/Export dropdown z-index stacking context bug where underlying FIFO banners and buttons bleed through.
3. **Supabase 5-Minute Error Circuit Breaker & Polling Dampening**:
   - Introduce an atomic 5-minute (300s) cooldown circuit breaker in backend Supabase synchronization to prevent retry storms and stack trace emissions on connection failures.
   - Remove self-sabotaging `invalidate_prompt_tree_cache()` calls from `detection.rs:254` to stop CLM (Conversation Liveness Module) query spam.
   - Implement 30-second TTL caching for account listing (`list_accounts`) in the Rust backend to prevent continuous reading of 46 JSON account files on every UI poll.
   - Relax aggressive frontend polling: Navbar instance selector (4s -> event-driven + 60s), Instances page (15s -> 60s/300s), and Fleet machines (15s -> 300s).
4. **Telegram Prompt Transmission & Machine Resolution**:
   - Remove the faulty `!is_current_machine(machine)` callback drop guard in `send_flow.rs:572`.
   - Fix remote node command execution syntax to properly target remote instances without invalid flags or non-existent subcommands.
   - Ensure the UI Send button executes Chrome DevTools Protocol (CDP) prompt submission into running IDE windows rather than leaving prompts stranded in queued status.

---

## Architecture & Subtask Decomposition

### Subtask 01: GitMap Nodes, SSH Cluster & Telegram Prompt Dispatch
- **Files Owned**:
  - `src-tauri/src/modules/ssh_manager/exec.rs`
  - `src-tauri/src/modules/ssh_manager/connections.rs`
  - `src-tauri/src/modules/ssh_manager/types.rs`
  - `src-tauri/src/commands/ssh.rs`
  - `src/components/settings/EmailNotifications/SshGitmapSection.tsx`
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
  - `src-tauri/src/modules/telegram_inbound/injection.rs`
- **Deliverables**:
  - Fix remote node instance and account query fallback (use GitMap or appropriate inspection).
  - Ensure SSH connection handshake handles key path passing (`-i <pubkey>`) and returns actionable errors.
  - Fix Telegram callback dropping and command syntax. Ensure UI Send button dispatches via CDP when IDE is running.

### Subtask 02: Prompt Tree Actions, Ordering, Preview Header & Z-Index Stacking
- **Files Owned**:
  - `src/components/instances/PromptTreeView/ModalHeader.tsx`
  - `src/components/instances/PromptTreeView/ImportExportMenu.tsx`
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/instances/PromptTreeView/TreePanel.tsx`
  - `src/components/instances/PromptTreeView/conversationGuards.ts`
  - `src/components/instances/PromptTreeView/ProjectNode.tsx`
  - `src/components/instances/PromptTreeView/tierClassification.ts`
  - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
  - `src/components/instances/PromptTreeView/usePromptTreeCoreActions.ts`
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- **Deliverables**:
  - Fix `ModalHeader.tsx` stacking context by adding `relative z-40` so `ImportExportMenu` renders cleanly above `DetailHeader` (`z-30`) and FIFO banners.
  - Wire direct 1-click Backup and Restore in `PromptTreeViewModal.tsx` to `coreActions.handleBackup()` and `coreActions.handleRestore()`.
  - Fix Refresh spinner state (`isLoading || isAutoSyncing || isRefreshing`) and Reset feedback banner.
  - Remove destructive skeleton pulsing during background refreshes in `TreePanel.tsx` and `PromptTreeViewModal.tsx`.
  - Enforce 3-tier sorting in `conversationGuards.ts`, `ProjectNode.tsx`, and `tierClassification.ts` (Running -> Queued -> Timestamp Descending) with case-insensitive status matching.
  - Move redundant `User Prompt` text and instance name from `DetailHeader.tsx` into `DetailsModal.tsx`.

### Subtask 03: Supabase Circuit Breaker, Polling Cadence & CLM Dampening
- **Files Owned**:
  - `src-tauri/src/modules/supabase_sync/types.rs`
  - `src-tauri/src/modules/supabase_sync/fleet.rs`
  - `src-tauri/src/modules/supabase_sync/query.rs`
  - `src-tauri/src/modules/workspace_lease_manager/release.rs`
  - `src-tauri/src/commands/supabase.rs`
  - `src-tauri/src/modules/repo_db/detection.rs`
  - `src-tauri/src/modules/account/crud.rs`
  - `src/components/instances/fleet/useFleetMachines.ts`
  - `src/components/navbar/instanceSelector/useInstanceSelector.ts`
  - `src/pages/instances/useInstancePage.ts`
  - `src/stores/useAccountStore.ts`
- **Deliverables**:
  - Implement 300s atomic Supabase cooldown circuit breaker on network error.
  - Remove circular cache invalidations from `detection.rs:254` to allow `get_project_conversation_tree_cached` to serve cached trees.
  - Add 30-second TTL cache for `list_accounts()` in `crud.rs`.
  - Dampen frontend polling timers across navbar, fleet, and instance page.

---

## Verification & Pre-flight Protocol
- **Gate 1**: `cd src-tauri && cargo fmt -- --check` [PASSED]
- **Gate 2**: `cd src-tauri && cargo clippy --all-targets --all-features` [PASSED]
- **Gate 3**: `npm run build` [PASSED]
- **Gate 4**: Minor version bump release ceremony (`npm run bump minor` -> `v4.233.0`), changelog synchronization (`@aukgit` attribution), and GitMap atomic commit (`gitmap cpf "Release - v4.233.0 GitMap Node and Prompt System Issues with Instance Management Sync"`). [COMPLETED]

## Status: COMPLETED (Verified 2026-10-19)

