# Subtask 03: GitMap Multi-Machine SSH Control, RSA-2048 Password Encryption & CLI Parity

> **Subtask Path:** `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/03-gitmap-ssh-rsa-and-cli.md`  
> **Architecture Spec:** `02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/01-architecture-spec.md`  

---

## 1. Objective

Implement RSA-2048 asymmetric envelope password encryption (`ag_rsa_v1_`) for SSH credentials so passwords are never stored in plaintext on disk, integrate GitMap multi-machine/multi-instance SSH inventory import and remote execution (`agm ssh import-gitmap`, `agm ssh balance`, `agm ssh status`, `agm ssh prompt`, `agm ssh switch`, `agm ssh ff`), provide `--csv` / `--json` CLI output parity, and add `gitmap` binary detection with an automated PowerShell / shell installation helper (`agm ssh install-gitmap` and Tauri command `install_gitmap_cli`).

---

## 2. Owned Target Files (Strictly Disjoint)

1. `src-tauri/src/utils/crypto.rs`
2. `src-tauri/src/modules/ssh_manager/types.rs`
3. `src-tauri/src/modules/ssh_manager/keys.rs`
4. `src-tauri/src/modules/ssh_manager/connections.rs`
5. `src-tauri/src/modules/ssh_manager/sync.rs`
6. `src-tauri/src/modules/ssh_manager/exec.rs`
7. `src-tauri/src/modules/ssh_manager/bundles.rs`
8. `src-tauri/src/modules/ssh_manager/tests.rs`
9. `src-tauri/src/commands/fleet.rs`
10. `src-tauri/src/bin/agm/ssh_cmds.rs`
11. `src-tauri/src/bin/agm/ssh_handlers.rs`
12. `src-tauri/src/bin/agm/quota_cmds.rs`
13. `src-tauri/src/bin/agm/main.rs`
14. `src-tauri/src/lib.rs`

---

## 3. Detailed Implementation Requirements

### 3.1 RSA-2048 Envelope Password Encryption (`crypto.rs`, `keys.rs`, `connections.rs`)

- **`src-tauri/src/modules/ssh_manager/keys.rs`:**
  1. Implement `pub fn get_rsa_vault_key_paths() -> Result<(PathBuf, PathBuf), String>` returning `~/.agm/ssh/rsa_vault_key` and `~/.agm/ssh/rsa_vault_key.pub`.
  2. Implement `pub fn ensure_rsa_vault_keypair() -> Result<(Vec<u8>, Vec<u8>), String>`:
     - Loads existing PEM files if present.
     - Otherwise generates a 2048-bit RSA keypair using `boring2::rsa::Rsa::generate(2048)` (`private_key_to_pem` and `public_key_to_pem_pkcs1` / `public_key_to_pem`), writes them to `~/.agm/ssh/rsa_vault_key` and `~/.agm/ssh/rsa_vault_key.pub` (setting `0o600` permissions on Unix), and returns `(priv_pem, pub_pem)`.
- **`src-tauri/src/utils/crypto.rs`:**
  1. Define `pub const ENCRYPTED_RSA_V1_PREFIX: &str = "ag_rsa_v1_";`.
  2. Implement `pub fn encrypt_password_rsa(password: &str) -> Result<String, String>`:
     - If `password.is_empty()` or already starts with `ENCRYPTED_RSA_V1_PREFIX`, return `Ok(password.to_string())`.
     - Obtain public key PEM from `crate::modules::ssh_manager::ensure_rsa_vault_keypair()`.
     - Generate a random 32-byte AES-256 key and 12-byte GCM nonce via `OsRng`.
     - Encrypt `password.as_bytes()` using `Aes256Gcm`.
     - Encrypt the 32-byte AES key with `boring2::rsa::Rsa::public_key_from_pem` using `boring2::rsa::Padding::PKCS1_OAEP` (or `Padding::PKCS1`).
     - Format as `ag_rsa_v1_<b64_rsa_key>.<b64_nonce>.<b64_ciphertext>`.
  3. Implement `pub fn decrypt_password_rsa(encrypted: &str) -> Result<String, String>`:
     - Strip `ag_rsa_v1_`, split into `<b64_rsa_key>.<b64_nonce>.<b64_ciphertext>`, decrypt the 32-byte AES key using the RSA private key PEM from `ensure_rsa_vault_keypair()`, and decrypt the ciphertext with `Aes256Gcm`.
  4. Update `decrypt_string` and `deserialize_password` in `src-tauri/src/utils/crypto.rs` so `ag_rsa_v1_` is transparently decrypted alongside `ag_enc_v2_` and `ag_enc_`.
- **`src-tauri/src/modules/ssh_manager/connections.rs`:**
  - In `save_ssh_connections` and `decode_connections_from_json` / `upsert_ssh_connection_password`, ensure any non-empty `encrypted_password` that is plaintext (does not start with `ag_rsa_v1_` or `ag_enc_`) is immediately encrypted via `crate::utils::crypto::encrypt_password_rsa` before persisting to disk.
  - Provide `pub fn set_node_password_rsa(alias: &str, raw_password: &str) -> Result<SshConnectionRecord, String>` for first-time SSH password login encryption.

### 3.2 GitMap SSH Node Import & Multi-Machine Control (`sync.rs`, `exec.rs`, `ssh_cmds.rs`)

- **`src-tauri/src/modules/ssh_manager/sync.rs`:**
  1. Implement `pub fn import_nodes_from_gitmap() -> Result<(String, NodeSyncStats), String>`:
     - First attempts running `gitmap ssh nodes export-json` (or reading `~/.gitmap/gitmap-ssh-nodes.json` and candidate seed paths).
     - Also parses any `Host` entries in `~/.ssh/config` outside the managed block so custom SSH hosts are discovered.
     - Encrypts any unencrypted passwords using `encrypt_password_rsa`, syncs connections into `ssh_nodes.json`, and updates `~/.ssh/config`.
- **`src-tauri/src/modules/ssh_manager/exec.rs`:**
  1. Implement `pub fn check_gitmap_installed_status() -> (bool, Option<String>)` by probing `gitmap --version`.
  2. Implement `pub fn install_gitmap_via_shell(interactive_powershell: bool) -> Result<String, String>` using `crate::utils::process::spawn_with_job_breakaway_fallback` (or `Command::new("powershell")` on Windows / `sh` on Unix) to install or update `gitmap` safely.
  3. Implement multi-machine SSH helpers:
     - `pub fn exec_remote_agm_command(target: &str, agm_args: &[&str]) -> Result<Vec<(String, bool, String)>, String>`
- **`src-tauri/src/commands/fleet.rs` & `src-tauri/src/lib.rs`:**
  1. Add `#[tauri::command] pub async fn install_gitmap_cli() -> AppResult<FleetDeployResult>` and `#[tauri::command] pub async fn import_gitmap_ssh_nodes() -> AppResult<serde_json::Value>` and register them in `src-tauri/src/lib.rs` if not already exposed.

### 3.3 AGM CLI Expansion (`ssh_cmds.rs`, `ssh_handlers.rs`, `quota_cmds.rs`, `main.rs`)

- **`src-tauri/src/bin/agm/ssh_cmds.rs` & `ssh_handlers.rs`:**
  1. Support `agm ssh import-gitmap` (alias `agm ssh sync-gitmap`).
  2. Support `agm ssh install-gitmap` to trigger `ssh_manager::install_gitmap_via_shell(true)`.
  3. Support `agm ssh set-password <alias> <password>` to encrypt and store a node's SSH password using RSA-2048 (`set_node_password_rsa`).
  4. Support `agm ssh balance [target] [--csv|--json]`, `agm ssh status [target] [--csv|--json]`, `agm ssh prompt <node> <instance> "<prompt>"`, `agm ssh switch <node> <instance> [email]`, and `agm ssh ff [node]`.
  5. When `gitmap` is not installed and a user runs an SSH fleet command, print a clear suggestion banner showing `agm ssh install-gitmap`.
- **`src-tauri/src/bin/agm/quota_cmds.rs` & `main.rs`:**
  - Add `agm balance [--csv|--json]` (aliases `agm credits`, `agm quota`) outputting per-instance active account, 4h & weekly quota percentages, and credit headroom in table, CSV (`--csv` / `--format csv`), or JSON (`--json` / `--format json`) format.

---

## 4. Acceptance Criteria

- [ ] `encrypt_password_rsa` and `decrypt_password_rsa` in `src-tauri/src/utils/crypto.rs` encrypt/decrypt SSH passwords using a 2048-bit RSA keypair stored at `~/.agm/ssh/rsa_vault_key` (`.pub`) with prefix `ag_rsa_v1_`.
- [ ] `save_ssh_connections` never writes plaintext passwords to disk; any raw password is automatically converted to `ag_rsa_v1_...`.
- [ ] `agm ssh import-gitmap`, `agm ssh balance`, `agm ssh status`, `agm ssh prompt`, `agm ssh install-gitmap`, and `agm balance --csv|--json` work end-to-end.
- [ ] Unit tests in `src-tauri/src/utils/crypto.rs` and `src-tauri/src/modules/ssh_manager/tests.rs` pass.
