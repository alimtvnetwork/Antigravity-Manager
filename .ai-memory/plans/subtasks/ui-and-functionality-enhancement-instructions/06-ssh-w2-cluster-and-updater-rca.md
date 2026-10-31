# Subtask 06: SSH Node W2 Parity, Shared Cluster & Updater RCA

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Task-06`  
**Status:** `READY`

## 1. Objectives & Scope
- SSH & Shared Cluster:
  - Implement 10s TTL liveness cache in `src-tauri/src/modules/ssh_manager/exec.rs` so offline nodes like W2 fast-fail in <10ms without blocking UI polling.
  - Update `gitmap-ssh-nodes.json` metadata for W2 (`os: "windows"`).
  - Add modern OpenSSH flags (`-o StrictHostKeyChecking=accept-new`, `-o PubkeyAcceptedKeyTypes=+ssh-rsa`, `-o ConnectTimeout=3`).
- Updater Pinned Version & Version Detection:
  - In `src-tauri/src/modules/delegate_updater/args.rs` and `src-tauri/src/bin/agm/update_cmds.rs`: strip leading URL prefixes from version tokens (e.g. `https://github.com/.../releases/tag/v4.226.3` -> `4.226.3`).
  - In `install.ps1`: query live process PE header (`FileVersionInfo.ProductVersion`) before checking disk directories or falling back to stale Windows Registry keys.

## 2. Disjoint File Ownership
- `Owned Files:` `src-tauri/src/modules/ssh_manager/exec.rs`, `gitmap-ssh-nodes.json`, `src-tauri/src/modules/delegate_updater/args.rs`, `src-tauri/src/bin/agm/update_cmds.rs`, `install.ps1`

## 3. Verification Evidence Contract
- AGM SSH node check for W2 returns cached offline state in <10ms.
- Pinned version URL resolves normalized tag `4.226.3`.
