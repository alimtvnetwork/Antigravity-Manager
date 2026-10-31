# Architecture Specification: GitMap Node & Prompt System Issues Remediation

- **Feature / Issue Slug:** `gitmap-node-and-prompt-system-issues`
- **Specification Document:** `02-spec/21-app/gitmap-node-and-prompt-system-issues/01-architecture-spec.md`
- **Subtask Plan:** `.ai-memory/plans/subtasks/gitmap-node-and-prompt-system-issues/01-ssh-nodes-and-telegram.md`
- **Related Spec & Standards:** `02-spec/01-spec-authoring-guide/`, `02-spec/02-coding-guidelines/`, `02-spec/03-error-manage/`
- **Target Subsystems:**
  - `src-tauri/src/modules/ssh_manager/` (SSH fleet, key management, auth repair, balance querying)
  - `src-tauri/src/bin/agm/` (`ssh_cmds.rs`, `ssh_handlers.rs`, `prompt_cmds.rs`)
  - `src-tauri/src/modules/telegram_inbound/` (`send_flow.rs`, `injection.rs`, `cluster.rs`, `node_label.rs`)
  - `src-tauri/src/modules/repo_db/` (`agy.rs`, `prompts_crud.rs`)
  - `src-tauri/src/modules/instance/` (`cdp_onboarding.rs`, `smart_run.rs`)
  - `src/components/instances/PromptTreeView/` (`usePromptTreePromptActions.ts`)

---

## 1. User Request (Verbatim)

```text
You are Spec Author 01 for task gitmap-node-and-prompt-system-issues. You have no prior chat context; this brief is your complete specification.

### Objectives:
Author two comprehensive specification and subtask plan documents:
1. `02-spec/21-app/gitmap-node-and-prompt-system-issues/01-architecture-spec.md`
2. `.ai-memory/plans/subtasks/gitmap-node-and-prompt-system-issues/01-ssh-nodes-and-telegram.md`

### Contents to Include:
- Architectural specifications and implementation details for:
  - GitMap Nodes & SSH Cluster Management: Node naming/aliasing, querying real remote instance and account balances (replacing `agm instances list --json` with GitMap command or proper discovery fallback), passing key paths `-i <pubkey>` in auth repair, and removing hardcoded single-node bypasses.
  - Telegram Prompt Sending & Machine Resolution: Removing the fatal callback drop guard `!is_current_machine(machine)` in `src-tauri/src/modules/telegram_inbound/send_flow.rs:572`, resolving target machines accurately, fixing remote command syntax in `injection.rs`, and ensuring UI Send button triggers Chrome DevTools Protocol (CDP) prompt injection when an IDE window is active.
- Exact file boundaries, interface schemas, error handling, and test verification contracts.

### Boundaries & Crash Prevention:
- You are a subagent with write tools (`write_to_file`, `replace_file_content`).
- You MUST write ONLY to your two assigned files:
  - `02-spec/21-app/gitmap-node-and-prompt-system-issues/01-architecture-spec.md`
  - `.ai-memory/plans/subtasks/gitmap-node-and-prompt-system-issues/01-ssh-nodes-and-telegram.md`
- NEVER write to any other file!
- TOTAL BAN ON RAW SEARCH TOOLS: NEVER run `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `Get-ChildItem -Recurse`, or `findstr`.
- TOTAL BAN ON GIT COMMANDS: NEVER run `git add`, `git commit`, `git status`, `git diff`, etc.
- Strict Relative Git Paths: All paths cited must be relative to the repository root.

When both files are created and complete, send a message to the caller and stop.
```

---

## 2. Executive Summary & Root Cause Analysis

This specification addresses two deeply coupled and critical functional defect clusters across Antigravity-Manager's multi-machine fleet management and prompt execution pipeline:

1. **GitMap Nodes & SSH Cluster Management Vulnerabilities**:
   - **Key Path Forwarding Loss**: When executing `agm ssh fix-auth <target> -i <identity_pubkey>`, the key path `-i <pubkey>` is properly parsed at the CLI top level, but when falling back to GitMap delegation (`ssh_manager::fix_node_auth_via_gitmap(target)`), the parameter is completely discarded. GitMap is executed with bare `gitmap ssh fix-auth <target>`, discarding the user's chosen identity and failing authentication repair.
   - **Hardcoded Single-Node Bypasses**: Heuristic code paths in `src-tauri/src/modules/ssh_manager/mesh.rs` contain hardcoded references (e.g. `w2 via password vault`) and `GITMAP_CLUSTER_NODES` static lists that bypass uniform error recovery, violating the principle of node-agnostic cluster orchestration.
   - **Fabricated Remote Balances & Incomplete Instance Discovery**: In `src-tauri/src/modules/ssh_manager/exec.rs` (`get_cluster_instances_and_balances`), remote node querying invokes bare `agm instances list --json`. If the remote host runs a different binary naming or uses GitMap cluster agents, the query fails or returns instances without balance details. The code subsequently hardcodes `tier: "PRO"`, `quota_4h_pct: 100.0`, and `quota_weekly_pct: 100.0`, fabricating full quotas and leading to catastrophic switch decisions.

2. **Telegram Prompt Sending & Machine Resolution Defects**:
   - **Fatal Callback Drop Guard (`!is_current_machine(machine)`)**: In `src-tauri/src/modules/telegram_inbound/send_flow.rs:572`, the handler `on_instance_picked` contains a hardcoded drop guard:
     ```rust
     if !super::node_label::is_current_machine(machine) {
         tracing::debug!("Dropping send:i:{}|{} callback on node {} — targeted for different machine", machine, instance, local_machine_label());
         return (String::new(), None);
     }
     ```
     Because the Telegram bot coordinator runs on a single node (or local machine), tapping any remote node's instance button causes the bot to silently drop the callback, rendering remote prompt dispatch via Telegram completely non-functional.
   - **Target Machine Misresolution & Colon Syntax Divergence**: Remote node matching between compound `alias|ip`, hostnames, and sequence targets diverges between `send_flow.rs` and `injection.rs`.
   - **Malformed Remote Shell Command Syntax**: In `src-tauri/src/modules/telegram_inbound/injection.rs:295`, remote prompt dispatch generates:
     ```rust
     let remote_cmd = format!("agm prompt {}{} \"{}\"", target_project, inst_flag, prompt_text);
     ```
     AGM CLI does not have a top-level `agm prompt <project> --instance <inst> "<text>"` command (the CLI command is `agm prompt send` or `agm ssh prompt`). Furthermore, raw quotes around `prompt_text` break under remote Win32 PowerShell/Cmd execution whenever prompt text contains quotes, newlines, or shell metacharacters.
   - **UI Send Button Bypassing Active IDE Windows**: When a user clicks the "Send" button in the Prompt Tree View UI, `src-tauri/src/modules/repo_db/agy.rs` (`send_prompt_now_for_instance`) prioritizes spawning a new headless `agy` CLI worker process (`spawn_or_replace_prompt_via_agy`). If the user already has an interactive IDE window open for that instance, spawning `agy` creates an asynchronous, decoupled turn that desynchronizes from the active editor. The UI Send button must trigger Chrome DevTools Protocol (CDP) prompt injection into the active IDE window whenever an IDE window is active.

---

## 3. Subsystem Architecture & Data Flow

```
+----------------------------------------------------------------------------------------------------+
|                                    Inbound Command & Control                                       |
|  1. Telegram Bot (/send, /prompt)                 2. GUI Prompt Tree ("Send" Button)               |
+---------------------------------------------------+------------------------------------------------+
                                                    |
                                                    v
+----------------------------------------------------------------------------------------------------+
|                             Machine Resolution & Routing Layer                                     |
|  - Resolve target: local vs remote GitMap/SSH node (alias, IP, or compound alias|ip)               |
|  - NO fatal callback drops for remote nodes (send_flow.rs:572 removed)                             |
+-------------------------+--------------------------------------------------+-----------------------+
                          |                                                  |
                          v (Local Dispatch)                                 v (Remote Dispatch)
+---------------------------------------------------+  +---------------------------------------------+
|              Local Prompt Execution               |  |          Remote Cluster Dispatch            |
|  1. Active IDE Window Check (DevToolsActivePort)  |  |  1. Sanitized Remote CLI Command Assembly   |
|     - If active: CDP Direct Prompt Injection      |  |     - `agm prompt send ...` or              |
|       (Input.insertText + button click)           |  |       `agm ssh prompt <node> <inst> <text>` |
|     - If inactive: Spawn agy CLI fallback         |  |  2. Transport Ladder:                       |
|  2. Atomic Resume Task Record Seeding             |  |     - GitMap Cluster Exec -> SSH Exec ->    |
|  3. SQLite repo_db Status Tracking                |  |       Supabase Command Queue Fallback       |
+---------------------------------------------------+  +---------------------------------------------+
                          |                                                  |
                          +-------------------------+------------------------+
                                                    |
                                                    v
+----------------------------------------------------------------------------------------------------+
|                                SSH Fleet & Auth Repair Pipeline                                    |
|  - Key Path Forwarding: `fix-auth <target> -i <key>` passes `-i <key>` to native & GitMap handlers |
|  - Balance Ingestion: Query real remote quotas (`agm quota show --json` / `agm accounts list`)    |
|  - Strict Zero Hardcoded Bypasses: Generic credential vault resolution across all fleet nodes     |
+----------------------------------------------------------------------------------------------------+
```

---

## 4. Technical Specifications & File Boundaries

### 4.1 GitMap Nodes & SSH Cluster Management

#### 4.1.1 Key Path Forwarding in Authentication Repair
- **Target Files:**
  - `src-tauri/src/bin/agm/ssh_handlers.rs`
  - `src-tauri/src/modules/ssh_manager/exec.rs`
  - `src-tauri/src/modules/ssh_manager/mesh.rs`
- **Specification:**
  - In `src-tauri/src/modules/ssh_manager/exec.rs`, update `fix_node_auth_via_gitmap`:
    ```rust
    pub fn fix_node_auth_via_gitmap(target: &str, pubkey_path: Option<&str>) -> Result<String, String>
    ```
  - When `pubkey_path` is provided, append `"-i"` and the canonical path to the `gitmap ssh fix-auth` argument list:
    ```rust
    cmd.args(["ssh", "fix-auth", target.trim()]);
    if let Some(pk) = pubkey_path.map(str::trim).filter(|s| !s.is_empty()) {
        cmd.args(["-i", pk]);
    }
    ```
  - In `src-tauri/src/bin/agm/ssh_handlers.rs` (`handle_ssh_fix_auth`), pass the parsed `pubkey_path` directly to `ssh_manager::fix_node_auth_via_gitmap(target, pubkey_path)`.
  - In `src-tauri/src/modules/ssh_manager/mesh.rs` (`deploy_auth_key_to_target`), propagate `pubkey_path_opt` when calling `fix_node_auth_via_gitmap(&node.alias, pubkey_path_opt)`.

#### 4.1.2 Eliminating Hardcoded Single-Node Bypasses
- **Target Files:**
  - `src-tauri/src/modules/ssh_manager/mesh.rs`
  - `src-tauri/src/modules/ssh_manager/exec.rs`
- **Specification:**
  - Remove all hardcoded special-case node bypasses (e.g. `// Check if gitmap fix-auth can reach it anyway (e.g. w2 via password vault)`).
  - Generalize `deploy_auth_key_to_target`: If TCP liveness returns false or standard SSH key injection fails, all nodes must uniformly attempt the GitMap vaulted password repair delegation, regardless of whether the node is named `w2`, `w1`, `main`, or any arbitrary worker ID.
  - Remove static IP bindings from `GITMAP_CLUSTER_NODES` when resolving dynamic cluster nodes. Node descriptors must be loaded dynamically from `load_ssh_connections()` and GitMap cluster exports (`gitmap cluster nodes --json`).

#### 4.1.3 Real Remote Instance & Account Balance Querying
- **Target Files:**
  - `src-tauri/src/modules/ssh_manager/exec.rs`
  - `src-tauri/src/modules/ssh_manager/types.rs`
- **Specification:**
  - In `src-tauri/src/modules/ssh_manager/exec.rs` (`get_cluster_instances_and_balances`), replace the simplistic single command `["agm", "instances", "list", "--json"]` with an adaptive 2-tier remote query strategy:
    1. **Primary Remote Command**: Execute `agm instances list --json` and `agm accounts list --json` (or unified `agm quota show --json`).
    2. **GitMap Discovery Fallback**: If `agm` binary is not found on the remote machine, execute `gitmap cluster exec <target> "agm quota show --json"` or fallback to inspecting remote configuration files.
    3. **Real Balance Parsing**: Parse the JSON output from `agm accounts list --json` or `agm quota show --json`. Extract:
       - Actual subscription tier (`acc.quota.subscription_tier`, e.g. "PRO", "FREE", "ULTRA").
       - Real 5-hour rolling quota (`quota_4h_pct`).
       - Real 7-day weekly quota (`quota_weekly_pct`).
    4. **Forbidden Fallback**: NEVER populate dummy `100.0` quotas without explicitly setting `status_label: "unverified-balance"` or logging a diagnostic warning. If remote quotas cannot be retrieved, mark `quota_4h_pct: 0.0` or report the error in `error_message`.

---

### 4.2 Telegram Prompt Sending & Machine Resolution

#### 4.2.1 Removing Fatal Callback Drop Guard
- **Target Files:**
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
- **Specification:**
  - Line 572 of `src-tauri/src/modules/telegram_inbound/send_flow.rs` contains:
    ```rust
    fn on_instance_picked(chat_id: i64, machine: &str, instance: &str) -> (String, Option<Value>) {
        if !super::node_label::is_current_machine(machine) {
            tracing::debug!(
                "Dropping send:i:{}|{} callback on node {} — targeted for different machine",
                machine,
                instance,
                local_machine_label()
            );
            return (String::new(), None);
        }
    ```
  - **REMOVAL REQUIRED**: Delete this guard entirely. The coordinator receiving the Telegram callback MUST handle the selection of remote machines and instances.
  - When `!super::node_label::is_current_machine(machine)`:
    - Save the remote machine and instance in `SEND_FLOWS`.
    - If remote project conversation trees are not locally cached, transition flow to `SendStage::TypeConversation` or query remote projects via GitMap cluster exec.
    - Advance to stage `SendStage::PickConversation` or allow prompt typing (`AwaitPrompt`).

#### 4.2.2 Accurate Target Machine Resolution
- **Target Files:**
  - `src-tauri/src/modules/telegram_inbound/node_label.rs`
  - `src-tauri/src/modules/telegram_inbound/cluster.rs`
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
- **Specification:**
  - Update `is_current_machine(alias)` to accurately handle:
    - Hostname case-insensitivity.
    - Local IP addresses across multiple network interfaces (Ethernet, Wi-Fi, VPN, WSL adapters).
    - Aliases mapped via `gitmap cluster nodes --json`.
  - Ensure `resolve_node_network_info` never returns `"Unknown IP"` when an SSH connection record exists in `load_ssh_connections()`.

#### 4.2.3 Fixing Remote Command Syntax in `injection.rs`
- **Target Files:**
  - `src-tauri/src/modules/telegram_inbound/injection.rs`
- **Specification:**
  - In `src-tauri/src/modules/telegram_inbound/injection.rs:295`, replace the broken command string:
    ```rust
    // FORBIDDEN (broken syntax and unsafe shell escaping):
    let remote_cmd = format!("agm prompt {}{} \"{}\"", target_project, inst_flag, prompt_text);
    ```
  - With canonical AGM CLI syntax:
    ```rust
    // REQUIRED:
    // Format: agm ssh prompt <node> <instance> "<prompt_text>"
    // or when targeting a project: agm prompt send <target_project> --instance <inst> -- "<prompt_text>"
    ```
  - Base64-encode or properly escape prompt text when dispatching across remote shells (especially Windows cmd.exe / PowerShell) to prevent prompt truncation, argument breaking on newlines, or command injection.

---

### 4.3 UI Send Button: Chrome DevTools Protocol (CDP) Prompt Injection

#### 4.3.1 Active IDE Window Detection & CDP-First Dispatch
- **Target Files:**
  - `src-tauri/src/modules/repo_db/agy.rs`
  - `src-tauri/src/modules/instance/cdp_onboarding.rs`
  - `src-tauri/src/commands/instance.rs`
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- **Specification:**
  - Currently in `src-tauri/src/modules/repo_db/agy.rs` (`send_prompt_now_for_instance`):
    - It immediately tries `spawn_or_replace_prompt_via_agy(&active_prompt, true)`, which launches an external headless `agy` process.
    - It only falls back to `submit_prompt_via_cdp_for_instance` if `agy` spawn returns an error.
  - **Inversion of Dispatch Order when IDE is Active**:
    1. Check if the target instance has an active running process and an accessible DevTools port (`read_active_ports(&data_dir)`).
    2. If DevTools port is active and an IDE window is open:
       - Execute `submit_prompt_via_cdp_for_instance(&canonical_inst, fallback_prompt)` **FIRST**.
       - Focus the IDE window (`focus_running_instance`).
       - If CDP injection succeeds, record the active prompt in `repo_db` as `dispatched` / `running`, update `.antigravity_resume_task.json`, and return success **WITHOUT spawning a separate `agy` CLI worker process**.
    3. If DevTools port is not active or CDP injection returns false (e.g. IDE composer element not found):
       - Fall back to spawning `agy` CLI worker (`spawn_or_replace_prompt_via_agy`).
  - This guarantees that clicking the UI Send button immediately types and sends the prompt inside the developer's visible IDE window rather than spawning background headless agents that fight for the workspace.

---

## 5. Interface Schemas & DTO Contracts

### 5.1 SSH Cluster Balance Item Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInstanceBalanceItem {
    pub machine_alias: String,
    pub machine_ip: String,
    pub ip_address: Option<String>,
    pub error_message: Option<String>,
    pub latency_ms: Option<u64>,
    pub is_local: bool,
    pub instance_id: String,
    pub instance_name: String,
    pub running: bool,
    pub bound_email: String,
    pub tier: String,
    pub quota_4h_pct: f64,
    pub quota_weekly_pct: f64,
    pub active_model: String,
    pub status_label: String,
}
```

### 5.2 Auth Repair Parameter Contract

```rust
// In src-tauri/src/modules/ssh_manager/exec.rs
pub fn fix_node_auth_via_gitmap(
    target: &str,
    pubkey_path: Option<&str>,
) -> Result<String, String>;

// In src-tauri/src/modules/ssh_manager/mesh.rs
pub fn deploy_auth_key_to_target(
    target: &str,
    pubkey_path_opt: Option<&str>,
) -> Result<Vec<KeyDeployNodeReport>, String>;
```

### 5.3 CDP Prompt Injection Result Contract

```rust
// In src-tauri/src/modules/instance/cdp_onboarding.rs
pub fn submit_prompt_via_cdp_for_instance(
    instance_id: &str,
    prompt_text: &str,
) -> bool;

pub fn is_instance_devtools_available(
    instance_id: &str,
) -> bool;
```

---

## 6. Error Handling & Invariants

### 6.1 No Silent Errors Invariant
- **Rule**: Every fallible network call, JSON deserialization, or process spawning must propagate via `AppResult<T>` or `Result<T, String>` and be recorded via `tracing::warn!` / `crate::modules::logger::log_warn`.
- **Prohibited**: Bare `let _ = ...`, `.ok()`, or `.unwrap_or_default()` when querying remote instances or SSH keys.

### 6.2 FIFO Invariant for Enqueued Prompts
- All queued prompts in SQLite must preserve strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`.

### 6.3 Strict Relative Git Paths Invariant
- No absolute paths (e.g. `C:\...`, `D:\...`, `/home/...`) may ever appear in logs, error toasts, or exported envelope manifests. All paths must be relative to the repository root.

---

## 7. Verification Contracts & Quality Gates

| Verification Gate | Command / Target | Pass Criteria |
| :--- | :--- | :--- |
| **Rust Code Formatting** | `cargo fmt -- --check` (in `src-tauri`) | Zero formatting diffs |
| **Rust Compiler & Lints** | `cargo clippy --all-targets --all-features` | Zero errors, zero warnings |
| **Frontend Production Build** | `npm run build` | Clean Vite/TypeScript compilation |
| **Auth Repair Key Forwarding** | `agm ssh fix-auth <node> -i <key>` | `-i <key>` forwarded to GitMap CLI without dropping |
| **Telegram Callback Handling** | Inline keyboard tap on remote instance | Callback processed without dropping; enters conversation/prompt flow |
| **UI Send CDP Injection** | Click "Send" with open Antigravity IDE | Prompt injected into active window composer without spawning conflicting agy CLI |
