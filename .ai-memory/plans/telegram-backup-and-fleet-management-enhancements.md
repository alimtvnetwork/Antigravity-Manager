# Master Plan: Telegram Backup and Fleet Management Enhancements

**Slug**: `telegram-backup-and-fleet-management-enhancements`  
**Parent Task**: Telegram Backup, Fleet Identity, Prompt Tree Liveness, UI/UX Polish & SSH Cluster Dispatch  
**Status**: IN_PROGRESS  
**Lead Orchestrator**: Antigravity Lead Agent  
**Created**: 2026-10-20  

---

## 1. User Request (Verbatim)

```text
# Telegram Backup and Fleet Management Enhancements: high priority instruction, non-negotiable task

Why don't you add a backup prompt option to the Telegram? And when you have the tree view, make it expandable. A tree view needs to have more space on the top level, and indented items need to be on the right-hand side, like a tree view in the terminal. Check for the Gitmap. Check if the installation works. The fleet section gets stuck. Fix it immediately. The fleet section works but is slow; cache it. The names do not match. In the start machine, it shows as main, but it is actually W1. W1 should be the lead because this is the current machine. Check with the Gitmap ``Id`` to find out the IP of the current machine and correct your logic. Correct the code base to match with Gitmap. In the prompt section, try to do the copy to one node for the first time. The copy all image composite PNG does not work; fix it. The send button shows that super base prompt is in QE, which is wrong. Sending works well. Check with send with images. In the filter section on top for the prompt tree view, there should be an option to filter out projects and prompts with images. Auto-hide has a bigger button; fix it. Sending images, sending prompt with images needs testing. Currently, it sends as a file path. In the future, have an endpoint to upload files for a short period. The sync option has a 30-second dropdown; modernize the UI. Test everything and finalize the code. From the Telegram, have the backup, restore, send images from the backup prompt. Fleet naming needs to match. Cannot send prompt from one machine to another in the instance section, in the shared cluster section. Correct this.
```

---

## 2. Architectural Pillars & System Invariants

### Pillar 1: Strategic Intent & Systems Reasoning
1. **Fleet Machine Identity & Cluster-IP Primacy**: Reconcile local host `192.168.1.3` to alias `w1` (as lead / current machine) matching GitMap node registry, preventing collision with `main` (`192.168.1.20`) and restoring the authentic 6-node cluster topology.
2. **High-Speed Fleet Caching**: Add 30-second in-memory TTL caching on `get_cluster_instances_and_balances` and decouple remote balance probes via SWR to completely eliminate UI freezes.
3. **SSH Cluster Reset**: Reload valid node definitions from `gitmap-ssh-nodes.json` preserving passwords and private key paths, while invalidating liveness caches.
4. **Cross-Machine Prompt Dispatch**: Encode remote prompt payloads as Base64 strings across SSH commands, preserving newlines, special characters, and image markdown tags without shell quote corruption.
5. **Prompt Tree View Liveness & Hierarchy**: Resolve default instance ID parsing in `observe.rs` and `liveness.rs`, synchronize conversation renames to parent state and `state.vscdb`, add generous top-level spacing with rightward terminal tree glyphs (`├──`, `└──`), add image filter toggle in the header, resize the auto-hide button into a sleek segmented capsule, and reposition the Details button with its green health pulse indicator (`#1af18d`).
6. **Telegram Inbound Prompt Backup**: Wire `/backup`, `/backup_prompt`, `/backup_running` into the inbound Telegram bot with multimodal image reporting and instance resolution.

### Pillar 2: Explicit Scope Boundaries
- **In-Scope**:
  - `src-tauri/src/modules/ssh_manager/` (`exec.rs`, `sync.rs`, `connections.rs`)
  - `src-tauri/src/modules/supabase_sync/` (`query.rs`, `node.rs`)
  - `src-tauri/src/modules/email_watcher/` (`detection.rs`)
  - `src-tauri/src/modules/repo_db/` (`liveness.rs`, `gemini_dirs.rs`, `sequences.rs`)
  - `src-tauri/src/modules/instance/` (`observe.rs`)
  - `src-tauri/src/modules/telegram_inbound/` (`processing.rs`, `commands.rs`, `injection.rs`)
  - `src-tauri/src/commands/` (`cmd_files.rs`, `cmd_running_up_next.rs`)
  - `src/components/instances/fleet/` (`useFleetMachines.ts`, `FleetMachinesTable.tsx`, `FleetMachineRow.tsx`)
  - `src/components/instances/PromptTreeView/` (`TreePanel.tsx`, `ProjectNode.tsx`, `ConversationNode.tsx`, `DetailTabs.tsx`, `ModalHeader.tsx`, `clipboard.ts`, `usePromptTreeData.ts`)
  - `src/components/settings/EmailNotifications/SshGitmapSection.tsx`
  - `src/pages/instances/InstancesHeader.tsx`
- **Excluded**:
  - Modifying core proxy adapters or external authentication services.
  - Full-suite unit testing / heavy cargo builds during routine edits.

### Pillar 3: Non-Negotiable Constraints & Negative Guardrails
- TOTAL BAN on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- TOTAL BAN on worker git commands (`git add`, `git commit`, `git push`, etc.).
- Strictly relative Git paths only (`src/...`, `src-tauri/...`, `02-spec/...`).
- Final atomic GitMap commit with hyphen format: `gitmap cpb "<module> - <summary>"`.

---

## 3. Subtask Decomposition (3 Waves, A = 3)

| Subtask ID | Focus Area | Primary Subagent | Owned Files | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Subtask-01** | Fleet Identity, 30s Caching, Cluster Reset & Cross-Machine Dispatch | Worker 01 | `src-tauri/src/modules/ssh_manager/`<br>`src-tauri/src/modules/supabase_sync/`<br>`src/components/instances/fleet/` | IN_PROGRESS |
| **Subtask-02** | Prompt Tree Liveness, Expandable Hierarchy, Rename Sync & Header Controls | Worker 02 | `src-tauri/src/modules/repo_db/`<br>`src-tauri/src/modules/instance/`<br>`src/components/instances/PromptTreeView/` | IN_PROGRESS |
| **Subtask-03** | Telegram Backup, Multi-Image Composite PNG, SSH Icons & Start All | Worker 03 | `src-tauri/src/modules/telegram_inbound/`<br>`src-tauri/src/commands/cmd_files.rs`<br>`src/components/settings/`<br>`src/pages/instances/` | IN_PROGRESS |

---

## 4. Verification Evidence Contract
1. `gitmap cluster nodes --json` and `useFleetMachines` render all 6 nodes with `w1` (`192.168.1.3`) as active lead node.
2. `get_cluster_instances_and_balances` resolves in <1ms on cache hit.
3. Default instance running prompts report `RUNNING` status without false `IDLE` demotion.
4. Conversation renaming in sidebar immediately reflects in header title and `state.vscdb`.
5. Telegram `/backup` snapshots active prompts and returns rich HTML confirmation.
6. Pre-flight checks pass: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, `npm run build`.
