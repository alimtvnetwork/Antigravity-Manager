# Component Specification: Enhanced Telegram UI, GitMap Integration & SSH Fleet Control

## 1. Executive Summary & Architectural Scope

This component specification defines the frontend React components, state management, Tauri IPC command contracts, and RSA credential workflows required to deliver:
1. **Enhanced Telegram Settings UI** (`src/components/settings/EmailNotifications/TelegramSection.tsx` and `src/components/settings/SupabaseSync/TelegramSection.tsx`):
   - Interactive click-to-copy command chips/buttons for Telegram chat commands (`/help`, `/balance`, `/accounts --csv`, `/instances --csv`, `/export csv`, `/prompt <machine>:<instance> <text>`, `/send`, `/ssh nodes`, `/gitmap cluster status`) and terminal CLI equivalents (`agm accounts --csv`, `agm instances --csv`, `agm ssh balance`, `agm prompt --node <alias> --instance <inst> "<text>"`).
   - One-click quick action buttons in the Telegram Settings UI to trigger a live **Balance Report to Telegram**, **Export CSV to Telegram**, or **Test Multi-Machine Prompting**.
2. **SSH & GitMap Fleet Control Settings Section** (`src/components/settings/EmailNotifications/SshGitmapSection.tsx` mounted in Settings under `src/components/settings/EmailNotificationSettings.tsx` and `src/components/settings/SupabaseSyncSettings.tsx`, and accessible via `/settings/ssh`, `/settings/email`, or `/settings/supabase`):
   - GitMap installation status banner that detects whether `gitmap` is installed on the host; when missing, renders a copyable install command alongside an **"Open PowerShell & Install GitMap"** action button (`install_gitmap_powershell` / `install_gitmap_via_powershell`).
   - SSH Node Manager form and interactive inventory table capturing **IP / Host**, **Port**, **Machine Alias**, **Username**, **SSH Key Path**, and a **First-Time Password** field that immediately encrypts the password via RSA algorithm (`rsa_encrypt_ssh_password` / `save_ssh_node_rsa`) so plaintext is never persisted to disk.
   - Fleet & per-node action controls: **"Import SSH from GitMap"** (`import_ssh_nodes_from_gitmap` / `import_ssh_from_gitmap`), **"Deploy Encrypted Credentials to Nodes"** (`deploy_encrypted_ssh_bundle` / `deploy_encrypted_ssh_to_nodes`), **"Test SSH Connection"**, **"Check Multi-Machine Instances & Account Balances"** (`get_cluster_instances_and_balances`), and **"Send Prompt to Remote Machine/Instance"** (`send_remote_ssh_prompt`).
3. **Tauri IPC Command Layer** (`src-tauri/src/commands/ssh.rs` registered in `src-tauri/src/commands/mod.rs` and `src-tauri/src/lib.rs`).

---

## 2. Enhanced Telegram Settings UI Architecture

### 2.1 Target Files
- `src/components/settings/EmailNotifications/TelegramSection.tsx`
- `src/components/settings/SupabaseSync/TelegramSection.tsx`
- `src/components/settings/EmailNotifications/useTelegramHandlers.ts`
- `src/services/telegramService.ts`

### 2.2 Interactive Click-to-Copy Command Matrix
Replace static `<code>` text blocks in both `TelegramSection.tsx` views with interactive pill badges (`rounded-full`, dark-glass border styling, hover highlight, copy icon indicator) that invoke `copyToClipboard(command)` and trigger `showToast('Copied: ' + command, 'info')`.

#### Telegram Chat Command Chips
| Command Chip Label | Copied Payload | Description |
| :--- | :--- | :--- |
| `/help` | `/help` | Full categorized command reference with copyable examples |
| `/balance` | `/balance` | Multi-machine & local account quota balance summary (4h & weekly) |
| `/accounts --csv` | `/accounts --csv` | Export all accounts and dual-window balances as CSV attachment |
| `/instances --csv` | `/instances --csv` | Export all local & SSH fleet instances and bound accounts as CSV |
| `/export csv` | `/export csv` | Full system telemetry CSV bundle (accounts, instances, nodes) |
| `/prompt <machine>:<instance> <text>` | `/prompt vm-1:default Continue current task and run tests` | Target specific machine & instance via colon syntax |
| `/send` | `/send C001 --node vm-1 --instance default "Check status"` | Direct prompt dispatch with explicit node & instance flags |
| `/ssh nodes` | `/ssh nodes` | List configured SSH cluster nodes and connectivity status |
| `/gitmap cluster status` | `/gitmap cluster status` | Query GitMap multi-machine cluster topology and sync status |

#### Terminal CLI Equivalent Chips
| CLI Chip Label | Copied Payload | Description |
| :--- | :--- | :--- |
| `agm accounts --csv` | `agm accounts --csv` | Print or save account quota CSV table in terminal |
| `agm instances --csv` | `agm instances --csv` | Print or save instance & bound account CSV table |
| `agm ssh balance` | `agm ssh balance` | Poll all SSH nodes for active instances and 4h/weekly credits |
| `agm prompt --node <alias> --instance <inst> "<text>"` | `agm prompt --node vm-1 --instance default "Verify build"` | CLI remote SSH & instance prompt injection |

### 2.3 Telegram Quick Action Toolbar
Located inside `TelegramSection.tsx` directly above the Save/Test action bar, wrapped in a contiguous segmented pill capsule or clean action row:
1. **Send Balance Report to Telegram**:
   - Invokes `export_system_data_csv` / Telegram balance dispatch or `send_remote_ssh_prompt` / bot report trigger to push the live cluster balance table to the configured `allowed_chat_id`.
2. **Export CSV to Telegram**:
   - Invokes `export_system_data_csv({ target: 'all', sendToTelegram: true })` to generate the accounts + instances CSV and transmit it to the configured Telegram chat (and/or copy CSV to clipboard).
3. **Test Multi-Machine Prompting**:
   - Opens an inline quick-prompt bar (or pre-fills target `<machine>:<instance>` selector + prompt input) allowing the user to test dispatching a prompt to a selected machine/instance and echoing a receipt to Telegram.

---

## 3. SSH & GitMap Fleet Control Component (`SshGitmapSection.tsx`)

### 3.1 Target Files
- `src/components/settings/EmailNotifications/SshGitmapSection.tsx` (New shared component)
- `src/services/sshService.ts` (New TypeScript IPC service wrapper)
- `src/components/settings/EmailNotificationSettings.tsx` (Mounts `<SshGitmapSection />`)
- `src/components/settings/SupabaseSyncSettings.tsx` (Mounts `<SshGitmapSection />`)
- `src/pages/Settings.tsx` (Supports `/settings/ssh` tab/scroll target alongside `/settings/email` and `/settings/supabase`)

### 3.2 Component Hierarchy & Layout
```
+--------------------------------------------------------------------------------------------------+
| SshGitmapSection.tsx                                                                             |
|                                                                                                  |
| 1. Header & Fleet Action Segmented Capsule                                                       |
|    - Title: "SSH & GitMap Multi-Machine Fleet Control"                                           |
|    - Segmented Pill Actions:                                                                     |
|      [Import SSH from GitMap] | [Deploy Encrypted Credentials] | [Check Cluster Balances]        |
|                                                                                                  |
| 2. GitMap Installation Status Banner                                                             |
|    - Installed State: Emerald pill badge "GitMap vX.Y.Z Installed" + binary path                 |
|    - Missing State: Amber warning banner with:                                                   |
|      * Copyable command chip: `npm install -g @gitmap/cli` / PowerShell bootstrap one-liner      |
|      * Action button: [Open PowerShell & Install GitMap] -> calls `install_gitmap_powershell`    |
|                                                                                                  |
| 3. SSH Node Manager Form (Add / Update Node with Immediate RSA Password Encryption)              |
|    - Inputs: Machine Alias (`vm-1`), IP / Host (`192.168.1.50`), Port (`22`),                    |
|              Username (`ubuntu`), SSH Key Path (`~/.ssh/id_ed25519`),                            |
|              First-Time Password (masked password input -> encrypted via RSA before disk write)  |
|    - Security Callout: "Password is RSA-encrypted immediately; plaintext is never stored."       |
|    - Submit Button: [Save & Encrypt SSH Node] -> calls `save_ssh_node_rsa`                       |
|                                                                                                  |
| 4. Registered SSH Nodes Table                                                                    |
|    - Columns: Alias | Host:Port | User | Auth / RSA Status | OS | Actions                        |
|    - Row Actions: [Test SSH] [Check Balances] [Send Prompt] [Delete]                             |
|                                                                                                  |
| 5. Multi-Machine Instances & Account Balances Inspector Panel                                    |
|    - Displays per-machine, per-instance table:                                                   |
|      Machine Alias | Instance Name/ID | PID / State | Bound Account Email | 4h Quota | Weekly    |
|    - Includes [Export CSV] button (`export_system_data_csv`)                                     |
|                                                                                                  |
| 6. Remote Machine / Instance Prompt Dispatcher Bar                                               |
|    - Target Node Selector (`local` or any SSH `alias`)                                           |
|    - Target Instance Selector (`default`, `#1`, or custom instance name)                         |
|    - Prompt Text Input + [Send Prompt to Remote Machine/Instance] (`send_remote_ssh_prompt`)     |
+--------------------------------------------------------------------------------------------------+
```

### 3.3 TypeScript Data Interfaces (`src/services/sshService.ts`)

```typescript
export interface SshGitmapStatus {
  gitmap_installed: boolean;
  gitmap_version: string | null;
  gitmap_path: string | null;
  install_command: string;
  ssh_config_path: string;
  connections_json_path: string;
  rsa_public_key_pem: string | null;
  total_nodes: number;
}

export interface SshNodeRecord {
  alias: string;
  ip_address: string;
  port: number;
  username: string;
  os: string;
  auth_method: string;
  key_path: string;
  has_encrypted_password: boolean;
  encrypted_password_preview?: string;
}

export interface SaveSshNodeRsaPayload {
  alias: string;
  ip_address: string;
  port: number;
  username: string;
  os?: string;
  key_path?: string;
  plaintext_password?: string | null;
}

export interface ClusterInstanceBalanceItem {
  machine_alias: string;
  ip_address: string;
  is_local: boolean;
  online: boolean;
  instance_id: string;
  instance_name: string;
  running: boolean;
  pid: number | null;
  bound_email: string | null;
  tier: string | null;
  quota_4h_pct: number | null;
  quota_weekly_pct: number | null;
  last_updated: string;
  error?: string | null;
}

export interface RemotePromptDispatchPayload {
  node_alias: string;
  instance_target: string;
  prompt_text: string;
  conversation_id?: string | null;
}

export interface RemotePromptDispatchResult {
  node_alias: string;
  instance_target: string;
  success: boolean;
  command_executed: string;
  stdout: string;
  stderr: string;
  duration_ms: number;
}
```

---

## 4. Tauri IPC Contract (`src-tauri/src/commands/ssh.rs`)

All commands in `src-tauri/src/commands/ssh.rs` return `AppResult<T>` (`Result<T, crate::error::AppError>`), routing any fallible operation through `AppError` with descriptive context in compliance with the project's **No Silent Errors** rule.

### 4.1 Registered Tauri Commands
Registered in `src-tauri/src/commands/mod.rs` (`pub mod ssh; pub use ssh::*;`) and mounted inside `tauri::generate_handler![...]` in `src-tauri/src/lib.rs`:

1. `get_ssh_gitmap_status() -> AppResult<SshGitmapStatus>`
   - Checks whether `gitmap` is available in `PATH` or standard global npm directories (`gitmap --version`).
   - Resolves the managed SSH config path (`~/.ssh/config`), `ssh_connections.json`, and local RSA public key fingerprint/PEM status.
   - Returns `install_command` (e.g., `npm install -g @gitmap/cli` or PowerShell installer script command) and total registered SSH nodes.

2. `install_gitmap_powershell() -> AppResult<String>` (also aliased / Invokable from UI button `"Open PowerShell & Install GitMap"`)
   - Spawns an interactive or visible PowerShell window (`powershell.exe -NoExit -ExecutionPolicy Bypass -Command "npm install -g @gitmap/cli; gitmap --version"` on Windows, or platform terminal fallback on macOS/Linux) using `spawn_with_job_breakaway_fallback` / safe process spawner.
   - Returns confirmation string describing the launched installer process.

3. `list_ssh_nodes() -> AppResult<Vec<SshNodeRecord>>`
   - Reads managed SSH nodes and connection records from `crate::modules::ssh_manager` (`load_managed_ssh_nodes` + `load_connections_store`).
   - Merges host/port/key metadata with `has_encrypted_password` status without ever exposing raw credentials.

4. `save_ssh_node_rsa(payload: SaveSshNodeRsaPayload) -> AppResult<SshNodeRecord>`
   - Validates `alias`, `ip_address`, `port`, and `username`.
   - If `plaintext_password` is provided and non-empty, immediately encrypts it in memory using the local RSA public key (`rsa_encrypt_ssh_password` helper in `ssh_manager::keys` / `ssh_manager::connections`) and zeroes/drops the plaintext buffer before writing the base64 ciphertext (`rsa:v1:<base64>`) to `ssh_connections.json` and updating `~/.ssh/config`.
   - Never writes plaintext passwords to disk or logs.

5. `delete_ssh_node(alias: String) -> AppResult<bool>`
   - Removes the specified `alias` from both the managed block in `~/.ssh/config` and `ssh_connections.json`.

6. `import_ssh_nodes_from_gitmap() -> AppResult<NodeSyncStats>`
   - Discovers GitMap's SSH node store (`~/.gitmap/ssh_nodes.json`, `~/.gitmap/connections.json`, or `gitmap ssh export --json`) and synchronizes nodes into AGM's `ssh_manager` via `sync_nodes_to_store`.

7. `deploy_encrypted_ssh_bundle(target_aliases: Option<Vec<String>>) -> AppResult<MeshDeploySummary>`
   - Packages the encrypted `ssh_connections.json` bundle and authorized public keys, deploying them across the selected (or all) remote SSH nodes via `ssh_manager::mesh` / `ssh_manager::bundles` so every machine in the fleet can communicate passwordlessly.

8. `get_cluster_instances_and_balances() -> AppResult<Vec<ClusterInstanceBalanceItem>>`
   - Collects local instances and bound account dual-window quotas (`4h` and `weekly`) first.
   - Concurrently queries each registered remote SSH node (`agm instances --json` / `agm accounts --json` over SSH `exec_on_node` with bounded timeout) and aggregates a unified multi-machine instance + account balance matrix.

9. `send_remote_ssh_prompt(payload: RemotePromptDispatchPayload) -> AppResult<RemotePromptDispatchResult>`
   - If `node_alias` is `"local"` or matches the current hostname, dispatches directly to the local instance via AGM's prompt dispatcher.
   - Otherwise, executes `agm prompt --instance <instance_target> "<escaped_prompt>"` on the target SSH node via `ssh_manager::exec::exec_on_node`, returning structured stdout, stderr, exit status, and duration.

10. `export_system_data_csv(kind: String, send_to_telegram: Option<bool>) -> AppResult<String>`
    - Generates RFC 4180 compliant CSV output for `"accounts"`, `"instances"`, `"cluster"`, or `"all"`.
    - When `send_to_telegram == Some(true)`, sends the generated CSV report to the active Telegram chat via `telegram_inbound` and returns the CSV string to the frontend for clipboard copy or file download.

---

## 5. Error Handling & UI Design Invariants

1. **No Silent Errors**:
   - Every frontend `invoke(...)` call inside `src/services/sshService.ts`, `TelegramSection.tsx`, and `SshGitmapSection.tsx` routes errors to `useErrorStore.getState().captureError(err, { source, triggerAction })` or `trackWarning(...)`.
   - Every Rust command in `src-tauri/src/commands/ssh.rs` propagates errors via `AppError` with clear context strings.
2. **Minimalist & Contextual Segmented UI**:
   - Adjacent toolbar actions in `SshGitmapSection.tsx` and `TelegramSection.tsx` use contiguous segmented pill capsules (`rounded-full`, shared subtle border, divider lines) or cohesive dark-glass cards matching AGM's design language.
   - Inline `<select>` dropdowns inside segmented capsules use `bg-transparent border-0 focus:ring-0 focus:outline-none`.
3. **Zero Plaintext Password Persistence**:
   - First-time SSH passwords entered in the UI are encrypted via RSA immediately upon submission and cleared from React state right after `save_ssh_node_rsa` completes.
