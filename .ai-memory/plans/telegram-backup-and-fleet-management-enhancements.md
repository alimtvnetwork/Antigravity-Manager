# Execution Plan: Telegram Backup & Fleet Management Enhancements

**Slug:** `telegram-backup-and-fleet-management-enhancements`  
**Parent Task:** `Telegram Backup and Fleet Management Enhancements`  
**Status:** Completed (v4.247.0 Released)  
**Lead Orchestrator:** Antigravity AI  

---

## 1. User Request (Verbatim)

```text
Why don't you add a backup prompt option to the Telegram? And when you have the tree view, make it expandable. A tree view needs to have more space on the top level, and indented items need to be on the right-hand side, like a tree view in the terminal. Check for the Gitmap. Check if the installation works. The fleet section gets stuck. Fix it immediately. The fleet section works but is slow; cache it. The names do not match. In the start machine, it shows as main, but it is actually W1. W1 should be the lead because this is the current machine. Check with the Gitmap ``Id`` to find out the IP of the current machine and correct your logic. Correct the code base to match with Gitmap. In the prompt section, try to do the copy to one node for the first time. The copy all image composite PNG does not work; fix it. The send button shows that super base prompt is in QE, which is wrong. Sending works well. Check with send with images. In the filter section on top for the prompt tree view, there should be an option to filter out projects and prompts with images. Auto-hide has a bigger button; fix it. Sending images, sending prompt with images needs testing. Currently, it sends as a file path. In the future, have an endpoint to upload files for a short period. The sync option has a 30-second dropdown; modernize the UI. Test everything and finalize the code. From the Telegram, have the backup, restore, send images from the backup prompt. Fleet naming needs to match. Cannot send prompt from one machine to another in the instance section, in the shared cluster section. Correct this.
```

---

## 2. Architecture & Root Cause Synthesis

### 2.1 Telegram Backup, Restore & Photo Forwarding
- **Inbound Commands**: `/backup` (and aliases `/backpack`, `/brp`) snapshot running prompts and attached media into `backup_prompts.db`. `/restore` (and aliases `/rrp`, `/resend`) restores and re-injects prompts into target or default instances.
- **Inline Keyboards**: Add interactive 1-tap buttons: `[ 📦 Backup Prompts ]`, `[ ♻️ Restore Prompts ]`, `[ 🖼️ Send with Images ]`, `[ 🌲 Expand Tree ]`, `[ ⚡ Active Prompts ]`.
- **Photo Forwarding**: Download attached Telegram photos to `~/.antigravity_tools/saved_images/`, index in `media_vault`, and forward as `![attached-image](<path_or_url>)` in injected prompt text.

### 2.2 Fleet Performance, Identity & Cross-Machine Prompt Dispatch
- **Performance & Caching**: Implement synchronous `localStorage` caching with 30s TTL envelope (`{ timestamp: number, machines: FleetMachineInfo[] }`) in `useFleetMachines.ts`. Render immediately on mount without full-screen loading freezes, and enrich via SWR.
- **Machine Identity Primacy**: Resolve local IP `192.168.1.3` as `w1` (local Lead node) with `is_local: true` and `is_lead: true`. Match remote node `192.168.1.20` as `main`. Ensure `sortFleetMachines` unconditionally places `w1` (lead) at index 0.
- **Cross-Machine Prompt Dispatching**: Connect `SendFleetPromptModal.tsx` in `FleetMachinesTable.tsx` / `FleetMachineRow.tsx` to `send_remote_ssh_prompt`. Sanitize remote CLI arguments (`agm prompt ...` or `gitmap agy ...`) over SSH so cross-machine prompt delivery works seamlessly across cluster nodes.

### 2.3 Prompt Tree Terminal Hierarchy, Image Filter & False Queue Removal
- **Expandable Terminal Tree**: Provide top-level project breathing room (`my-2.5`), right-hand child turn indentation (`ml-5 pl-3 border-l-2`), terminal monospace branch connector glyphs (`├──`, `└──`), and 1-click Expand/Collapse All toggle (`isAllExpanded`).
- **Header Controls & Sizing**: Add `ImageIcon` image filter toggle in `ModalHeader.tsx` connected to `imageFilterEmitter`. Modernize 30s auto-sync interval into a dark-glass popover capsule with checkmarks. Resize Auto-Hide button in `TreePanel.tsx` to a sleek compact pill matching surrounding controls.
- **False `QUEUED` Status Elimination**: In `tree_node_conv.rs`, `transcript.rs`, and `tree_collect.rs`, only declare a turn queued if genuine instance liveness and 600s TTL checks pass, preventing idle or completed prompts from displaying false amber `QUEUED` notices.
- **Multi-Image Composite Clipboard**: Verified Win32 OLE `CF_DIB`, `CF_BITMAP`, `PNG`, and `CF_HDROP` clipboard engine for Microsoft OneNote and Antigravity IDE compatibility.
- **Future Upload Endpoint Architecture**: Detailed architecture design for ephemeral file uploads with sliding TTL.

---

## 3. Disjoint Subtask Breakdown

- **Subtask-01 (Telegram Inbound - Backup, Restore & Media)**:
  - `src-tauri/src/modules/telegram_inbound/commands.rs`
  - `src-tauri/src/modules/telegram_inbound/daemon.rs`
  - `src-tauri/src/modules/telegram_inbound/processing.rs`
  - `src-tauri/src/modules/telegram_inbound/injection.rs`
- **Subtask-02 (Fleet & SSH - Machine Identity, Caching & Cross-Machine Dispatch)**:
  - `src/components/instances/fleet/useFleetMachines.ts`
  - `src/components/instances/fleet/FleetMachinesTable.tsx`
  - `src/components/instances/fleet/FleetMachineRow.tsx`
  - `src/components/instances/fleet/SendFleetPromptModal.tsx`
  - `src-tauri/src/modules/ssh_manager/exec.rs`
  - `src-tauri/src/modules/supabase_sync/query.rs`
- **Subtask-03 (Prompt Tree UI & Status - Terminal Glyphs, Filters & Queue Fix)**:
  - `src/components/instances/PromptTreeView/ModalHeader.tsx`
  - `src/components/instances/PromptTreeView/TreePanel.tsx`
  - `src/components/instances/PromptTreeView/ConversationNode.tsx`
  - `src/components/instances/PromptTreeView/ProjectNode.tsx`
  - `src-tauri/src/modules/repo_db/tree_node_conv.rs`
  - `src-tauri/src/modules/repo_db/transcript.rs`
  - `src-tauri/src/modules/repo_db/tree_collect.rs`
  - `src-tauri/src/modules/repo_db/tree_phases.rs`
- **Subtask-04 (Verification, Pre-flight Linting & Release Ceremony)**:
  - Transient sandbox testing (`agm instances test-create/clean`).
  - Pre-flight quality gates (`cargo fmt`, `cargo clippy`, `npm run build`).
  - Atomic version bump, changelog sync with `@aukgit` attribution, tag, push, and CI/CD monitoring.
