---
name: agm-ssh-key-management-parity
description: Specialized skill for managing Antigravity-Manager native SSH key generation, public key authorization, mesh key distribution, ~/.ssh/config auto-management, and GitMap cluster parity in src-tauri/src/modules/ssh_manager.rs and the agm ssh CLI suite.
---

# AGM SSH Key Management & Fleet Parity

Specialized operational guide for the native **SSH Manager Engine** ([`src-tauri/src/modules/ssh_manager.rs`](src-tauri/src/modules/ssh_manager.rs)) and the associated CLI command family in [`src-tauri/src/bin/agm.rs`](src-tauri/src/bin/agm.rs).

Introduced in commit `c2568adf`, this 1,731-line subsystem provides 100% native Rust SSH key lifecycle management, bidirectional authentication repair, 2-phase mesh public key deployment, and managed `~/.ssh/config` injection with full GitMap cluster parity—without requiring external SSH binaries or Python bridges.

---

## 1. Subsystem Architecture

The SSH Manager engine is organized into five functional pillars:

```
+----------------------------------------------------------------------------------------------------+
|                                    Inbound Command Dispatch                                        |
|   CLI: agm ssh <target> | agm ssh deploy-keys | agm ssh fix-auth | agm ssh keys | agm ssh nodes        |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                Native Engine (ssh_manager.rs)                                      |
|  +---------------------------+  +-------------------------------+  +----------------------------+  |
|  |     Keypair Lifecycle     |  |       Authorization Engine    |  |     Mesh Key Deployment    |  |
|  | - ED25519 & RSA gen       |  | - authorized_keys dedup       |  | - 2-phase parallel rollout |  |
|  | - SHA-256 fingerprinting  |  | - Win32 icacls ACL security   |  | - TCP 22 liveness probe    |  |
|  | - Discovery (~/.ssh/*.pub)|  | - Administrators auth keys    |  | - Remote public key ingest |  |
|  +---------------------------+  +-------------------------------+  +----------------------------+  |
|                                 +-------------------------------+                                  |
|                                 |   Config & Envelope Sync      |                                  |
|                                 | - Managed ~/.ssh/config block |                                  |
|                                 | - SshNodesExportEnvelope JSON |                                  |
|                                 +-------------------------------+                                  |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                              Local & Remote Operating Systems                                      |
|    Windows (OpenSSH + icacls)               macOS (chmod 0600)               Linux (chmod 0600)    |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Key Structures and Serde DTOs

### Core Models in [`ssh_manager.rs`](src-tauri/src/modules/ssh_manager.rs#L28-L162)

- **`SshKeyInfo`**: Metadata representing a discovered or generated local SSH key:
  - `name`: File basename (e.g. `id_ed25519`).
  - `path`: Absolute filesystem path to private key.
  - `pub_path`: Absolute path to `.pub` file.
  - `key_type`: Key algorithm (`"ed25519"`, `"rsa"`, `"ecdsa"`).
  - `fingerprint`: SHA-256 fingerprint string (`SHA256:...`).
  - `comment`: Key comment (e.g. `user@host` or email).
  - `is_default`: Boolean indicating whether key matches standard default identity (`id_ed25519` or `id_rsa`).
- **`SshNodeExportItem`**: Lightweight node entry for portable JSON cluster manifests:
  - `alias`, `host`, `port`, `username`, `os_type`, `auth_method`, `key_path`, `is_active`.
- **`SshConnectionRecord`**: Complete cluster connection entity with encrypted credentials and runtime metadata.
- **`SshNodesExportEnvelope`**: Universal two-tier JSON envelope:
  - `attributes`: `{ type: "agm/ssh-nodes-export", version: "1.0", generated_at, node_count, checksum }`.
  - `data`: `Vec<SshNodeExportItem>`.
- **`MeshDeploySummary`**: Statistical summary of 2-phase mesh key distribution:
  - `total_nodes`, `reachable_nodes`, `keys_collected`, `keys_deployed`, `failures`, `details`.

---

## 3. Core Engine Functions & Implementation

### 3.1 Keypair Lifecycle Management

```rust
// Discover all available public keys in ~/.ssh
pub fn discover_local_ssh_keys() -> Result<Vec<SshKeyInfo>>;

// Ensure a default key exists; generates id_ed25519 if none found
pub fn ensure_default_ssh_key() -> Result<SshKeyInfo>;

// Create new keypair (ED25519 default, RSA fallback) via ssh-keygen
pub fn create_ssh_key(name: &str, key_type: &str, comment: &str) -> Result<SshKeyInfo>;
```

1. **Fingerprint Calculation**: Executes `ssh-keygen -lf <pub_path>` and parses output tokens `[bits, fingerprint, comment, type]`.
2. **Deterministic Defaults**: Checks for `id_ed25519.pub`, `id_rsa.pub`, `id_ecdsa.pub` in priority order.

### 3.2 Authorization Engine & Windows ACL Hardening

```rust
// Validate public key format and length
pub fn validate_ssh_public_key(pub_key: &str) -> Result<()>;

// Install public key into local authorized_keys with deduplication and OS permission hardening
pub fn install_authorized_key_local(pub_key: &str) -> Result<()>;
```

1. **Public Key Validation**: Enforces standard OpenSSH prefixes (`ssh-ed25519`, `ssh-rsa`, `ecdsa-sha2-*`) and verifies that the Base64 key blob contains $\ge 32$ characters.
2. **Deduplication**: Compares the Base64 blob (2nd token) against all existing entries in `authorized_keys`. Duplicate keys are skipped idempotently.
3. **Windows `administrators_authorized_keys` & `icacls` Enforcement**:
   - On Windows, OpenSSH for Administrator users requires public keys in `%ProgramData%\ssh\administrators_authorized_keys`.
   - AGM installs to both `~/.ssh/authorized_keys` and `%ProgramData%\ssh\administrators_authorized_keys`.
   - Enforces strict ACLs using Windows `icacls.exe`:
     ```cmd
     icacls administrators_authorized_keys /inheritance:r /grant SYSTEM:(F) BUILTIN\Administrators:(F)
     ```
   - On macOS/Linux, enforces standard POSIX `chmod 0600`.

### 3.3 Managed `~/.ssh/config` Injection

```rust
pub fn update_ssh_config(sanitize_only: bool) -> Result<()>;
```

1. **Daemon Directive Sanitization**: Automatically intercepts and comments out mispasted server directives (`sshd_config` keywords like `PermitRootLogin`, `UsePAM`, `AuthorizedKeysFile`, `Subsystem`, `MaxAuthTries`, `ClientAliveInterval`) that cause the OpenSSH client to fail on connect.
2. **Managed Block Idempotency**: All AGM entries are enclosed inside:
   ```sshconfig
   # --- BEGIN AGM MANAGED SSH CONFIG ---
   Host github.com
       IdentityFile ~/.ssh/id_ed25519
       IdentitiesOnly yes
       User git

   Host node-alpha
       HostName 192.168.1.101
       Port 22
       User administrator
       IdentityFile ~/.ssh/id_ed25519
       StrictHostKeyChecking accept-new
       ConnectTimeout 5
   # --- END AGM MANAGED SSH CONFIG ---
   ```
   User-defined custom configurations outside this block are 100% preserved.

### 3.4 Parallel 2-Phase Mesh Key Deployment

```rust
pub fn deploy_mesh_keys(target_filter: Option<&str>, dry_run: bool) -> Result<MeshDeploySummary>;
```

1. **Phase 1: Key Discovery & Liveness Probing**:
   - Checks TCP port 22 connectivity for all candidate nodes with a 2,000ms timeout (`check_tcp_liveness`). Unreachable nodes are flagged immediately.
   - Concurrently queries reachable nodes in parallel via `std::thread::scope` to extract their public keys.
2. **Phase 2: Mesh Deployment & Verification**:
   - Aggregates and deduplicates all gathered remote keys alongside local `~/.ssh/*.pub` keys.
   - Pushes the combined public key bundle to every node's `authorized_keys`.
   - Executes a test probe command (`ssh <target> echo AGM_SSH_OK`) to verify bidirectional authorization.

---

## 4. CLI Subcommand Matrix (`agm ssh ...`)

All commands support both human-readable table outputs and `--json` / `-j` machine flags:

| Command | Usage | Description |
|---|---|---|
| **Remote Shell / Exec** | `agm ssh <target> [cmd...]` | Connects to remote node or executes a one-off command. Preferentially delegates to `gitmap ssh` if installed; falls back to native engine. |
| **Port & Password** | `agm ssh <target> -p <port> --password <pwd>` | Overrides connection port or provides one-off password for initial key installation. |
| **Deploy Mesh Keys** | `agm ssh deploy-keys [all] [--dry-run]` | Executes the 2-phase mesh deployment across all registered cluster nodes. |
| **Fix Authentication** | `agm ssh fix-auth <target> [-i <pubkey>]` | Repairs broken remote authorization by appending the local public key via fallback authentication. |
| **Copy Public Key** | `agm ssh copy-id <target> [-i <pubkey>]` | Cross-platform native equivalent to OpenSSH `ssh-copy-id`. |
| **Key List / Inspection** | `agm ssh keys ls [--json]` | Lists all local SSH keys in `~/.ssh` with type, fingerprint, and default flag. |
| **Create Key** | `agm ssh keys create <name> [-t ed25519]` | Generates a new keypair and registers it in managed SSH config. |
| **Show Public Key** | `agm ssh keys cat <name>` | Dumps public key content for manual copying. |
| **Update Config** | `agm ssh keys config [--sanitize]` | Re-evaluates and writes managed `~/.ssh/config` block. |
| **Node List** | `agm ssh nodes ls [--json]` | Displays cluster node fleet, reachability status, and auth type. |
| **Export Nodes** | `agm ssh nodes export-json [out.json]` | Serializes registered cluster nodes into a two-tier JSON envelope. |
| **Import Nodes** | `agm ssh nodes import-json <in.json>` | Imports node fleet from JSON envelope and executes local sync. |

---

## 5. Architectural Invariants & Safety Discipline

1. **Managed Block Boundary Invariant**:
   - Edits to `~/.ssh/config` must NEVER overwrite or mutate configurations outside `# --- BEGIN AGM MANAGED SSH CONFIG ---` and `# --- END AGM MANAGED SSH CONFIG ---`.
2. **Zero Insecure Key Permissions Invariant**:
   - On Windows, all key operations on administrator keys must execute `icacls` to strip inherited permissions and grant full control only to `SYSTEM` and `BUILTIN\Administrators`.
   - On Unix, key files must enforce `0600` and `.ssh/` directory must enforce `0700`.
3. **TCP Liveness Pre-Flight**:
   - Mesh deployment must test port 22 TCP liveness (2,000ms timeout) before attempting OpenSSH handshakes, preventing hung threads.
4. **GitMap Interoperability Fallback**:
   - In `agm.rs`, `forward_to_gitmap_ssh` must first probe for external `gitmap.exe`. If found, forward with exact args; if missing, execute native `ssh_manager.rs` handlers.

---

## 6. Pre-Flight Verification

When modifying `ssh_manager.rs` or `agm ssh` CLI dispatch:
```bash
# 1. Rust formatting check
cd src-tauri && cargo fmt -- --check

# 2. Rust clippy gate
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Test SSH key discovery and public key validation unit tests
cd src-tauri && cargo test modules::ssh_manager
```
