# Subtask 04: Frontend Telegram Command Chips, CSV/Balance Quick Actions & SSH/GitMap Fleet UI

## 1. Objective
Implement the enhanced Telegram Settings UI (`TelegramSection.tsx` in both EmailNotifications and SupabaseSync settings) with click-to-copy command chips and quick telemetry actions, create the new SSH & GitMap Fleet Control section (`SshGitmapSection.tsx`), and wire it to the backend Tauri IPC commands (`src-tauri/src/commands/ssh.rs`) via `src/services/sshService.ts`.

## 2. Target Files (Relative Paths)
- `src-tauri/src/commands/ssh.rs` (Create / implement Tauri IPC commands for SSH, RSA encryption, GitMap detection/install, cluster balances, remote prompt dispatch, and CSV export)
- `src-tauri/src/commands/mod.rs` (Export `pub mod ssh; pub use ssh::*;`)
- `src-tauri/src/lib.rs` (Register `ssh` commands in `tauri::generate_handler![...]`)
- `src/services/sshService.ts` (Create frontend service wrapping all 10 Tauri IPC commands with `useErrorStore` integration)
- `src/components/settings/EmailNotifications/SshGitmapSection.tsx` (Create SSH & GitMap Fleet Control component)
- `src/components/settings/EmailNotifications/TelegramSection.tsx` (Enhance with click-to-copy command chips & quick actions)
- `src/components/settings/SupabaseSync/TelegramSection.tsx` (Enhance with click-to-copy command chips & quick actions)
- `src/components/settings/EmailNotificationSettings.tsx` (Mount `<SshGitmapSection />`)
- `src/components/settings/SupabaseSyncSettings.tsx` (Mount `<SshGitmapSection />`)
- `src/pages/Settings.tsx` (Ensure `/settings/ssh` route/tab navigation scrolls to or opens the SSH & GitMap section)

## 3. Implementation Requirements

### 3.1 Enhanced Telegram Settings (`TelegramSection.tsx`)
1. **Interactive Click-to-Copy Command Chips**:
   - Render clickable command pills for Telegram commands:
     - `/help`
     - `/balance`
     - `/accounts --csv`
     - `/instances --csv`
     - `/export csv`
     - `/prompt <machine>:<instance> <text>`
     - `/send`
     - `/ssh nodes`
     - `/gitmap cluster status`
   - Render clickable command pills for CLI equivalents:
     - `agm accounts --csv`
     - `agm instances --csv`
     - `agm ssh balance`
     - `agm prompt --node <alias> --instance <inst> "<text>"`
   - Clicking any chip copies the command via `copyToClipboard` and triggers `showToast`.
2. **Telegram Quick Action Buttons**:
   - Add quick action buttons in `TelegramSection.tsx`:
     - **Balance Report to Telegram**: Triggers cluster/account balance summary push to Telegram.
     - **Export CSV to Telegram**: Calls `export_system_data_csv` with `sendToTelegram: true` and copies/downloads CSV.
     - **Test Multi-Machine Prompting**: Allows testing `/prompt <machine>:<instance> <text>` dispatch directly from the UI.

### 3.2 SSH & GitMap Fleet Control Section (`SshGitmapSection.tsx`)
1. **GitMap Installation Status Banner**:
   - Queries `get_ssh_gitmap_status` on mount.
   - Shows green badge when `gitmap` is installed (`gitmap_installed: true`).
   - When not installed, displays a copyable install command AND an **"Open PowerShell & Install GitMap"** button wired to `install_gitmap_powershell`.
2. **SSH Node Manager Form & Table (RSA Password Encryption)**:
   - Form fields: Machine Alias, IP / Host, Port (default `22`), Username, SSH Key Path, and First-Time Password.
   - Submitting the form invokes `save_ssh_node_rsa`, which immediately encrypts the password using RSA so plaintext is never stored on disk, then clears the password input state.
   - Node table lists all configured SSH nodes with per-row actions: **Test SSH Connection**, **Check Balances**, **Send Prompt**, and **Delete** (`delete_ssh_node`).
3. **Fleet Action Toolbar & Inspectors**:
   - **"Import SSH from GitMap"** button calling `import_ssh_nodes_from_gitmap`.
   - **"Deploy Encrypted Credentials to Nodes"** button calling `deploy_encrypted_ssh_bundle`.
   - **"Check Multi-Machine Instances & Account Balances"** button calling `get_cluster_instances_and_balances` and rendering a table showing which instance on which machine is using which account along with `4h` and `weekly` credits.
   - **"Send Prompt to Remote Machine/Instance"** controls calling `send_remote_ssh_prompt`.

## 4. Acceptance Criteria
- [ ] All 10 Tauri IPC commands (`get_ssh_gitmap_status`, `install_gitmap_powershell`, `list_ssh_nodes`, `save_ssh_node_rsa`, `delete_ssh_node`, `import_ssh_nodes_from_gitmap`, `deploy_encrypted_ssh_bundle`, `get_cluster_instances_and_balances`, `send_remote_ssh_prompt`, `export_system_data_csv`) are registered and callable from the frontend.
- [ ] Clicking any Telegram or CLI command chip in `TelegramSection.tsx` copies the command to clipboard and shows a toast notification.
- [ ] `SshGitmapSection.tsx` renders GitMap installation status, RSA-backed SSH node form/table, multi-machine instance & balance matrix, and remote prompt dispatcher without any silent errors.
- [ ] `npm run build` and `cd src-tauri && cargo clippy --all-targets --all-features` pass with zero errors.
