# Component Specification: SSH Node Connectivity, GitMap Parity & Shared Cluster Restoration

> **Spec ID:** `02-component-spec`  
> **Parent Plan:** `ui-and-functionality-enhancement-instructions`  
> **Status:** APPROVED & ARCHITECTED  
> **Scope:** SSH Node Connectivity, Go In-Process SSH Analysis, GitMap Delegation & Fallback, Automated Key Deployment (`fix-auth`), Shared Cluster Hybrid Aggregation, Non-Supabase Fleet Visibility  

---

## 1. Executive Summary

This component specification defines the architectural and implementation blueprints for two critical subsystems in Antigravity-Manager (AGM):
1. **SSH Node Connectivity & GitMap Parity Engine**: Eliminates remote node connection failures (e.g. node `w2`) by bridging AGM's native OpenSSH execution with GitMap's battle-tested Go in-process SSH client (`golang.org/x/crypto/ssh`), utilizing GitMap's encrypted password vault (`gitmap.db`), fallback cluster credentials (`rtyrty123@` for Windows nodes), automated key deployment (`gitmap ssh fix-auth`), and execution delegation.
2. **Shared Cluster Section Restoration**: Fixes the broken Shared Cluster view in the Instances screen by transforming `src-tauri/src/modules/supabase_sync/query.rs` and `src/components/instances/fleet/useFleetMachines.ts` into a hybrid cluster aggregator that seamlessly merges Supabase cloud sync with local SSH fleet records (`ssh_nodes.json`, `gitmap.db`, `04-ssh-nodes.json`), guaranteeing full operational visibility even when Supabase sync is disabled.

---

## 2. GitMap SSH Architecture & Password Vault Analysis

### 2.1 GitMap In-Process Go SSH Engine

In GitMap (`github.com/alimtvnetwork/gitmap-v28`), SSH connectivity is implemented directly inside the Go runtime using `golang.org/x/crypto/ssh` rather than relying solely on the external system `ssh` binary.

```
+--------------------------------------------------------------------------------------------------+
|                                    GitMap SSH Dial Engine                                        |
|                          (cli/cmdssh/ssh_dial_fallback.go)                                       |
+--------------------------------------------------------------------------------------------------+
                                                  |
         +----------------------------------------+---------------------------------------+
         |                                        |                                       |
         v                                        v                                       v
+------------------+                    +--------------------+                 +--------------------+
| Tier 1: Key Auth |                    | Tier 2: Def Keys   |                 | Tier 3: Password   |
| tryDialWith-     | --[Fail / Missing]-> | tryDialWith-       | -[Fail / None]-> | tryDialWith-       |
| SpecifiedKey()   |                    | DefaultKeys()      |                 | PasswordCandidate()|
+------------------+                    +--------------------+                 +--------------------+
                                                                                          |
                                                +-----------------------------------------+
                                                |
                                                v
                               +----------------------------------+
                               |     Password Resolution Order    |
                               | 1. c.EncryptedPassword (AES/RSA) |
                               | 2. gitmap.db host password       |
                               | 3. ResolveFallbackCredentials()  |
                               |    (06-vmpass.json / vmpass.json)|
                               +----------------------------------+
```

### 2.2 Password Vault Resolution Ladder

When connecting to a remote node, GitMap evaluates credentials in the following strict order:
1. **Specified Private Key (`c.KeyPath`)**: Checks if key exists on disk and initiates public key handshake.
2. **Default Key Scanning (`findAllUserSSHKeys`)**: Scans `~/.ssh/id_ed25519`, `~/.ssh/id_rsa`, and `~/.ssh/id_ecdsa`.
3. **Encrypted Host Credential (`c.EncryptedPassword`)**: Stored in `gitmap.db` (`hosts` / `ssh_connections` tables), encrypted with RSA-OAEP or AES-GCM.
4. **Cluster Fallback Vault (`ResolveFallbackCredentials`)**:
   - Ingests `repo-secrets/01-gitmap/06-vmpass.json` (or `~/.gitmap/vmpass.json`, `D:\work\repo-secrets\vmpass.json`).
   - Decrypts salted rotation cipher strings:
     * **Windows Target (`Administrator` / `admin`)**: `winPass` = `salt:9f8b2c4e:UypnMmpIN3BMLg==` $\rightarrow$ Plaintext: `rtyrty123@`.
     * **Linux / Ubuntu Target (`a`)**: `linuxPass` = `salt:4e8a1d2f:PQ==` $\rightarrow$ Plaintext: `a`.
   - Uses `secrets.ConnectWithPassword(c.IPAddress, c.Username, plain)` via Go `ssh.ClientConfig` with `ssh.Password(plain)`.

### 2.3 Automated Public Key Deployment (`gitmap ssh fix-auth`)

GitMap provides native automated public key injection (`cli/cmdssh/ssh_auth_key_deploy.go`). It dials the target using password authentication (either vaulted or fallback) and executes a hardened platform-specific deployment script:

- **Windows Targets (`buildWindowsAuthKeyScript`)**:
  1. Creates `~/.ssh/authorized_keys` and appends the public key if not present.
  2. Sets Windows ACLs via `icacls $userKeys /inheritance:r /grant "$($env:USERNAME):F" /grant 'SYSTEM:F'`.
  3. Detects if running with administrative rights. If administrator:
     - Creates `%ProgramData%\ssh\administrators_authorized_keys`.
     - Appends the public key.
     - Hardens ACLs: `icacls $adminKeys /inheritance:r /grant 'Administrators:F' /grant 'SYSTEM:F'`.
  4. Inspects Windows OpenSSH Service (`sshd`): sets `StartupType Automatic` and executes `Start-Service sshd`.
- **Linux / macOS Targets (`buildUnixAuthKeyScript`)**:
  ```bash
  mkdir -p ~/.ssh && chmod 700 ~/.ssh && touch ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys && (grep -qF '<pubkey>' ~/.ssh/authorized_keys || echo '<pubkey>' >> ~/.ssh/authorized_keys)
  ```

---

## 3. AGM SSH Manager Subsystem Enhancements

### 3.1 The Root Cause of AGM Connection Failures to Node `w2`

In `src-tauri/src/modules/ssh_manager/exec.rs`:
- AGM executes standard OpenSSH CLI (`ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o ConnectTimeout=6 ...`).
- If node `w2` does not yet have AGM's local public key (`id_ed25519.pub` or `id_rsa.pub`) installed in its `authorized_keys` or `administrators_authorized_keys`, OpenSSH in `BatchMode=yes` terminates with exit code 255 (`Permission denied (publickey,password)`).
- AGM does not prompt for passwords, does not read the vaulted `rtyrty123@` credential, and does not automatically delegate to GitMap when native `ssh` fails.

### 3.2 GitMap Delegation & Fallback in `exec_ssh_command`

Modify `src-tauri/src/modules/ssh_manager/exec.rs` to implement a multi-stage execution ladder:

```rust
// Logical Flow in exec_ssh_command() for each node:
// Stage 1: TCP Port 22 Liveness Probe (timeout 1500ms)
// Stage 2: Native OpenSSH CLI (BatchMode=yes)
// Stage 3: If Native SSH fails with auth error (exit code 255):
//          a. Check if GitMap CLI is available (resolve_gitmap_bin())
//          b. Run: gitmap ssh exec <target> "<command>"
//          c. If successful, capture stdout/stderr and return success
// Stage 4: If still unauthenticated, execute auto-repair via GitMap fix-auth:
//          a. Run: gitmap ssh fix-auth <target>
//          b. If fix-auth succeeds, re-execute Stage 2 (Native OpenSSH)
```

```
+--------------------------------------------------------------------------------------------------+
|                                AGM SSH Execution Fallback Ladder                                 |
+--------------------------------------------------------------------------------------------------+
                                                  |
                                                  v
                                     [TCP 22 Liveness Check]
                                                  |
                         +------------------------+------------------------+
                         | (Open)                                          | (Closed)
                         v                                                 v
             [Stage 1: Native OpenSSH]                            [Exit 255: Unreachable]
                         |
           +-------------+-------------+
           | (Exit 0)                  | (Exit != 0 / Auth Failure)
           v                           v
       [Success]          [Stage 2: GitMap In-Process Exec]
                          (gitmap ssh exec <target> "<cmd>")
                                       |
                         +-------------+-------------+
                         | (Exit 0)                  | (Exit != 0)
                         v                           v
                     [Success]           [Stage 3: Auto-Fix Auth]
                                         (gitmap ssh fix-auth <target>)
                                                     |
                                       +-------------+-------------+
                                       | (Repaired)                | (Failed)
                                       v                           v
                           [Retry Native OpenSSH]          [Return Detailed Error]
```

### 3.3 Enhanced Candidate Seed Node Resolution in `paths.rs`

Expand `src-tauri/src/modules/ssh_manager/paths.rs` to include all standardized GitMap secrets and JSON formats:

```rust
pub(crate) fn candidate_seed_node_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("gitmap-ssh-nodes.json"),
        PathBuf::from("04-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/gitmap-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/04-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/01-gitmap/gitmap-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/01-gitmap/04-ssh-nodes.json"),
        PathBuf::from("../../repo-secrets/gitmap-ssh-nodes.json"),
        PathBuf::from("../../repo-secrets/01-gitmap/04-ssh-nodes.json"),
    ];
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".gitmap").join("gitmap-ssh-nodes.json"));
        paths.push(home.join(".gitmap").join("04-ssh-nodes.json"));
        paths.push(home.join(".gitmap").join("ssh-nodes.json"));
        paths.push(home.join("gitmap-ssh-nodes.json"));
    }
    // Also include local appdata gitmap directory if present
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local_app_data)
            .join("gitmap-cli")
            .join("data")
            .join("gitmap-ssh-nodes.json");
        paths.push(p);
    }
    paths
}
```

### 3.4 Tauri Command Update: `test_ssh_node_connection`

In `src-tauri/src/commands/ssh.rs`:
- Update `test_ssh_node_connection(id_or_alias: String)`:
  1. Executes `exec_ssh_command` with fallback enabled.
  2. If the connection fails with authentication rejected, it invokes `ssh_manager::fix_node_auth_via_gitmap(&clean)` and retries the test command `echo AGM_SSH_OK`.
  3. Returns `SshExecNodeResult` with accurate telemetry (`duration_ms`, `exit_code`, `stdout`).

---

## 4. Shared Cluster Section Restoration

### 4.1 Root Cause Analysis of Broken Shared Cluster View

1. **Frontend Hard-Abort in `useFleetMachines.ts`**:
   ```typescript
   // src/components/instances/fleet/useFleetMachines.ts
   const syncActive = Boolean(config?.is_sync_enabled);
   setIsSyncEnabled(syncActive);

   if (!syncActive) {
       setMachines([]); // <-- Empties machines unconditionally!
       return;
   }
   ```
2. **Backend Hard-Abort in `query_fleet_machines()`**:
   ```rust
   // src-tauri/src/modules/supabase_sync/query.rs
   if !config.is_sync_enabled {
       return Ok(build_local_fallback_machine()); // <-- Returns ONLY local fallback!
   }
   ```
   In `useFleetMachines.ts`, local machines are filtered out via `m.node_id !== effectiveLocalId`. When only the local fallback is returned, the filtered array is completely empty `[]`.
3. **UI Gate in `FleetMachinesTable.tsx`**:
   ```tsx
   if (!isLoading && !isSyncEnabled) {
       return <FleetDisabledState />;
   }
   ```
   This prevents users without Supabase from ever seeing their local cluster nodes, even when SSH nodes are registered in `gitmap.db` or `ssh_nodes.json`.

### 4.2 Backend Solution: Hybrid Multi-Source Cluster Aggregation

Modify `src-tauri/src/modules/supabase_sync/query.rs` to aggregate across multiple sources:
1. **Source A: Supabase Cloud Sync** (when `is_sync_enabled == true` and endpoints are configured).
2. **Source B: Registered SSH / GitMap Fleet Nodes** (via `ssh_manager::load_ssh_connections()`).
3. **Source C: Local Instance Registry** (via `instance::load_registry()`).

#### Aggregation & Deduplication Logic
- When Supabase sync is disabled (`!config.is_sync_enabled`):
  1. Build local machine entry (`build_local_fallback_machine()`).
  2. For each node in `ssh_manager::load_ssh_connections()`:
     - Check TCP liveness on configured port (default 22).
     - Construct a `FleetMachineInfo` entity:
       * `node_id`: `format!("ssh_{}", conn.alias)`.
       * `node_alias`: `conn.alias`.
       * `alias`: `conn.alias`.
       * `ip_address`: `conn.ip_address`.
       * `is_online`: boolean from TCP probe.
       * `status`: if online `"reachable"` else `"offline"`.
       * `source`: `"ssh-mesh"`.
       * `os_info`: `Some(conn.os)`.
       * `in_flight_prompts_count`: `0`.
       * `total_instances`: `1`.
       * `active_instances`: vector containing default remote instance.
       * `bound_emails`: `vec![format!("{}@{}", conn.username, conn.alias)]`.
       * `is_local`: `false`.
  3. Return aggregated machines.
- When Supabase sync is enabled:
  1. Query Supabase `nodes`, `instance_profiles`, and `workspace_leases`.
  2. Query `ssh_manager::load_ssh_connections()`.
  3. Merge SSH nodes into the machine list if their `ip_address` or `alias` is not already present from Supabase.
  4. Tag entries with `source: "supabase"` or `source: "ssh-mesh"`.

### 4.3 Frontend Solution: Non-Blocking Cluster Hook & UI State

#### Changes in `src/components/instances/fleet/useFleetMachines.ts`
1. Remove `if (!syncActive) { setMachines([]); return; }`.
2. Always execute `fetchFleetMachinesApi()`.
3. Filter out `effectiveLocalId` while preserving remote SSH and Supabase machines.
4. Provide `hasClusterNodes: machines.length > 0` in state.

#### Changes in `src/components/instances/FleetMachinesTable.tsx`
1. Replace strict `!isSyncEnabled` gate:
   ```tsx
   // Only show disabled state if NEITHER Supabase sync IS enabled NOR any SSH/cluster nodes exist
   if (!isLoading && !isSyncEnabled && machines.length === 0) {
       return <FleetDisabledState />;
   }
   ```
2. Update header badge and description:
   - When `isSyncEnabled == true`: "Real-time multi-machine cluster state synced via Supabase & SSH Mesh".
   - When `isSyncEnabled == false && machines.length > 0`: "Registered SSH Fleet Nodes (Local Mesh Mode)".
3. Add a source badge in `FleetMachineRow.tsx`:
   - `supabase`: Sky blue pill (`Supabase Cloud`).
   - `ssh-mesh`: Purple pill (`SSH Mesh / GitMap`).

---

## 5. Component Contracts & Data Models

### 5.1 Updated Rust Types (`src-tauri/src/modules/supabase_sync/models.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetMachineInfo {
    pub node_id: String,
    pub node_alias: String,
    pub alias: String,
    pub os_info: Option<String>,
    pub ip_address: String,
    pub is_online: bool,
    pub last_heartbeat_timestamp: i64,
    pub last_heartbeat_at: i64,
    pub uptime_seconds: u64,
    pub status: String,
    pub source: String, // "supabase" | "ssh-mesh" | "local"
    pub in_flight_prompts_count: usize,
    pub total_running_prompts: usize,
    pub total_instances: usize,
    pub active_instances: Vec<FleetInstanceSummary>,
    pub bound_emails: Vec<String>,
    pub active_accounts: Vec<String>,
    pub leases: Vec<FleetLeaseInfo>,
    pub is_local: bool,
    pub ssh_port: Option<u16>,
    pub auth_status: Option<String>, // "authorized" | "auth_failed" | "untested"
}
```

### 5.2 Updated TypeScript Interfaces (`src/components/instances/fleet/fleetTypes.ts`)

```typescript
export interface FleetMachineInfo {
    node_id: string;
    node_alias: string;
    alias: string;
    os_info?: string;
    ip_address: string;
    is_online: boolean;
    last_heartbeat_timestamp: number;
    last_heartbeat_at: number;
    uptime_seconds: number;
    status: string;
    source: 'supabase' | 'ssh-mesh' | 'local';
    in_flight_prompts_count: number;
    total_running_prompts: number;
    total_instances: number;
    active_instances: FleetInstanceSummary[];
    bound_emails: string[];
    active_accounts: string[];
    leases: FleetLeaseInfo[];
    is_local: boolean;
    ssh_port?: number;
    auth_status?: 'authorized' | 'auth_failed' | 'untested';
}
```

---

## 6. Security, Permission & Cross-Platform Invariants

1. **No Hardcoded Passwords in Source Code Invariant**:
   - Plaintext passwords must never be committed to source files.
   - Credentials must be resolved at runtime from encrypted vault files (`06-vmpass.json`, `gitmap.db`) or user prompts.
2. **Strict Host Key Checking Invariant**:
   - All OpenSSH and GitMap operations must enforce `StrictHostKeyChecking=accept-new` to prevent interactive hangs while safeguarding against MITM key alterations.
3. **Windows ACL Hardening Invariant**:
   - Any public key installation to `%ProgramData%\ssh\administrators_authorized_keys` must enforce `icacls` granting full control only to `SYSTEM` and `BUILTIN\Administrators`.
4. **Non-Blocking Background Execution**:
   - All TCP probes and SSH executions must run in `tokio::task::spawn_blocking` or detached thread pools with strict timeouts (1,500ms–2,500ms) to ensure the UI remains fluid.

---

## 7. Verification & Acceptance Criteria

1. **Node `w2` Reachability**:
   - Running `agm ssh test w2` or clicking "Test Connection" in the UI successfully verifies connection using GitMap fallback or automated key repair.
2. **Shared Cluster Visibility**:
   - With Supabase sync turned OFF, navigating to Instances screen shows the "Other Machines / Fleet Nodes" table populated with registered SSH nodes (`w1`, `w2`, `w3`, `w4`, `u1`, `main`).
   - Clicking "Refresh" triggers TCP liveness probes and updates the online/offline badges without error modals.
3. **Pre-flight Quality Gates**:
   - `cargo fmt -- --check` passes with zero violations.
   - `cargo clippy --all-targets --all-features` passes with zero warnings.
   - `npm run build` succeeds without TypeScript type errors.
