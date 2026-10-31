# Subtask Plan: SSH Nodes, Telegram Machine Resolution & UI CDP Injection

- **Task Identifier:** `gitmap-node-and-prompt-system-issues`
- **Subtask Plan Document:** `.ai-memory/plans/subtasks/gitmap-node-and-prompt-system-issues/01-ssh-nodes-and-telegram.md`
- **Parent Specification:** `02-spec/21-app/gitmap-node-and-prompt-system-issues/01-architecture-spec.md`
- **Status:** `[PENDING IMPLEMENTATION]`
- **Target Subsystems:**
  - `src-tauri/src/modules/ssh_manager/`
  - `src-tauri/src/bin/agm/`
  - `src-tauri/src/modules/telegram_inbound/`
  - `src-tauri/src/modules/repo_db/`
  - `src-tauri/src/modules/instance/`
  - `src/components/instances/PromptTreeView/`

---

## 1. Actionable Scope & Implementation Checklist

### Phase 1: GitMap Nodes & SSH Cluster Management
- [ ] **1.1 Forward `-i <pubkey>` in Auth Repair**:
  - Update `fix_node_auth_via_gitmap(target: &str, pubkey_path: Option<&str>)` in `src-tauri/src/modules/ssh_manager/exec.rs`.
  - Append `["-i", pubkey_path]` to the `gitmap ssh fix-auth` command invocation when `pubkey_path` is present.
  - Update caller in `src-tauri/src/bin/agm/ssh_handlers.rs` (`handle_ssh_fix_auth`) to pass `pubkey_path`.
  - Update caller in `src-tauri/src/modules/ssh_manager/mesh.rs` (`deploy_auth_key_to_target`) to pass `pubkey_path_opt`.
- [ ] **1.2 Eliminate Hardcoded Single-Node Bypasses**:
  - Remove hardcoded comments and special-casing (e.g. `w2 via password vault`) in `src-tauri/src/modules/ssh_manager/mesh.rs`.
  - Generalize fallback ladder in `deploy_auth_key_to_target` so all offline/unreachable nodes attempt uniform vaulted credential delegation.
  - Decouple static IP lists in `GITMAP_CLUSTER_NODES` from dynamic cluster discovery.
- [ ] **1.3 Real Remote Instance & Quota Balance Ingestion**:
  - Refactor `get_cluster_instances_and_balances()` in `src-tauri/src/modules/ssh_manager/exec.rs`.
  - Replace the bare `["agm", "instances", "list", "--json"]` call with a multi-step query or unified quota query (`agm quota show --json` / `agm accounts list --json`).
  - Parse real `subscription_tier`, `quota_4h_pct`, and `quota_weekly_pct`.
  - Eliminate hardcoded `100.0%` quotas; mark unverified nodes clearly as `"unverified-balance"` or report errors in `error_message`.
- [ ] **1.4 Node Naming & Aliasing Unification**:
  - Standardize node resolution across `load_ssh_connections()`, `gitmap cluster nodes --json`, and compound `alias|ip` tokens.

---

### Phase 2: Telegram Machine Resolution & Inbound Callback Fixes
- [ ] **2.1 Remove Fatal Callback Drop Guard**:
  - Delete `if !super::node_label::is_current_machine(machine) { return (String::new(), None); }` at line 572 of `src-tauri/src/modules/telegram_inbound/send_flow.rs`.
  - Enable Telegram bot coordinator to process instance picks for remote cluster nodes.
  - Transition flow state to `SendStage::PickConversation` (if remote conversations cached) or `SendStage::TypeConversation` / `AwaitPrompt`.
- [ ] **2.2 Accurate Target Machine Resolution**:
  - Enhance `is_current_machine(alias)` in `src-tauri/src/modules/telegram_inbound/node_label.rs` to compare against all local interface IPs (IPv4 loopback, LAN, WSL, VPN) and detected machine names.
  - Ensure `resolve_node_network_info` never returns `"Unknown IP"` when the node exists in `load_ssh_connections()`.
- [ ] **2.3 Fix Remote Command Syntax in `injection.rs`**:
  - In `src-tauri/src/modules/telegram_inbound/injection.rs:295`, fix the remote execution command string.
  - Use canonical AGM CLI syntax: `agm ssh prompt <node> <instance> "<prompt_text>"` or `agm prompt send <project> --instance <inst> -- "<prompt_text>"`.
  - Ensure safe escaping of prompt text across remote Windows cmd.exe / PowerShell environments.

---

### Phase 3: UI Send Button & CDP Active-Window Injection
- [ ] **3.1 DevTools Active Port & Process Liveness Check**:
  - Expose helper `is_instance_devtools_available(instance_id: &str) -> bool` in `src-tauri/src/modules/instance/cdp_onboarding.rs`.
  - Read `DevToolsActivePort` from instance data dir and verify local TCP socket connectivity.
- [ ] **3.2 Invert Dispatch Order in `send_prompt_now_for_instance`**:
  - In `src-tauri/src/modules/repo_db/agy.rs`, check if instance DevTools is available and window is active before spawning `agy`.
  - If active:
    - Invoke `submit_prompt_via_cdp_for_instance(&canonical_inst, prompt_content)`.
    - Focus running IDE window (`focus_running_instance`).
    - If CDP succeeds, update `ActivePrompt` status to `dispatched`, write `.antigravity_resume_task.json`, and return Ok.
  - If inactive or CDP fails:
    - Fall back to `spawn_or_replace_prompt_via_agy(&active_prompt, true)`.
- [ ] **3.3 Frontend Action Feedback & Clipboard Sync**:
  - Verify `usePromptTreePromptActions.ts` (`handleResendPrompt`) and `instanceService/lifecycle.ts` (`sendPromptNow`) display accurate status banners (`"Prompt typed into active IDE window"` vs `"Prompt dispatched to background agy worker"`).

---

### Phase 4: Verification & Quality Gates
- [ ] **4.1 Rust Format & Clippy Gate**:
  - Run `cd src-tauri && cargo fmt -- --check`.
  - Run `cd src-tauri && cargo clippy --all-targets --all-features`.
- [ ] **4.2 Frontend Build Gate**:
  - Run `npm run build` from repository root.
- [ ] **4.3 Integration Verification**:
  - Verify `agm ssh fix-auth <node> -i ~/.ssh/id_ed25519.pub` passes `-i` to GitMap.
  - Verify Telegram `/send` inline keyboard selection on a remote machine proceeds to prompt input instead of dropping silently.
  - Verify UI Send button directly inputs prompt into visible IDE editor via CDP when IDE is open.

---

## 2. File-by-File Modification Specifications

### 2.1 `src-tauri/src/modules/ssh_manager/exec.rs`
- **Functions to Modify:**
  - `fix_node_auth_via_gitmap(target: &str, pubkey_path: Option<&str>) -> Result<String, String>`:
    ```rust
    // Before:
    pub fn fix_node_auth_via_gitmap(target: &str) -> Result<String, String> {
        ...
        cmd.args(["ssh", "fix-auth", target.trim()]);
        ...
    }

    // After:
    pub fn fix_node_auth_via_gitmap(target: &str, pubkey_path: Option<&str>) -> Result<String, String> {
        let gitmap_bin = resolve_gitmap_bin()
            .ok_or_else(|| "GitMap CLI is not installed or detected in PATH".to_string())?;

        let mut cmd = Command::new(gitmap_bin);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        cmd.args(["ssh", "fix-auth", target.trim()]);
        if let Some(pk) = pubkey_path.map(str::trim).filter(|s| !s.is_empty()) {
            cmd.args(["-i", pk]);
        }
        ...
    }
    ```
  - `get_cluster_instances_and_balances() -> Result<Vec<ClusterInstanceBalanceItem>, String>`:
    - Query remote `agm accounts list --json` and `agm instances list --json`.
    - Extract real subscription tier and 4h/weekly quota values from JSON.
    - Set `status_label: "unverified-balance"` if quotas cannot be parsed, avoiding false 100% metrics.

---

### 2.2 `src-tauri/src/bin/agm/ssh_handlers.rs`
- **Function to Modify:** `handle_ssh_fix_auth(args: &[String])`
- **Changes:**
  - Line 147 and 171:
    ```rust
    // Before:
    ssh_manager::fix_node_auth_via_gitmap(target)

    // After:
    ssh_manager::fix_node_auth_via_gitmap(target, pubkey_path)
    ```

---

### 2.3 `src-tauri/src/modules/ssh_manager/mesh.rs`
- **Function to Modify:** `deploy_auth_key_to_target(target: &str, pubkey_path_opt: Option<&str>)`
- **Changes:**
  - Pass `pubkey_path_opt` to `fix_node_auth_via_gitmap(&node.alias, pubkey_path_opt)` across all retry and fallback paths.
  - Remove `// Check if gitmap fix-auth can reach it anyway (e.g. w2 via password vault)` and treat all nodes uniformly.

---

### 2.4 `src-tauri/src/modules/telegram_inbound/send_flow.rs`
- **Function to Modify:** `on_instance_picked(chat_id: i64, machine: &str, instance: &str)`
- **Changes:**
  - Delete lines 572-580:
    ```rust
    // REMOVE:
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
  - Record the target machine and instance into `SEND_FLOWS` regardless of whether `machine` is local or remote.
  - If the machine is remote, prompt for project/conversation or prompt text directly.

---

### 2.5 `src-tauri/src/modules/telegram_inbound/injection.rs`
- **Function to Modify:** `execute_remote_prompt_injection`
- **Changes:**
  - Sanitize and reformat `remote_cmd`:
    ```rust
    // Format command safely with agm ssh prompt
    let remote_cmd = format!(
        "agm ssh prompt {} {} {}",
        target_node,
        explicit_instance.unwrap_or("default"),
        escape_remote_shell_arg(prompt_text)
    );
    ```

---

### 2.6 `src-tauri/src/modules/repo_db/agy.rs`
- **Function to Modify:** `send_prompt_now_for_instance`
- **Changes:**
  - Inspect `is_instance_devtools_available(&canonical_inst)`:
    ```rust
    let cdp_available = crate::modules::instance::cdp_onboarding::is_instance_devtools_available(&canonical_inst);
    let mut dispatched_via_cdp = false;

    if cdp_available {
        let clean_prompt = extract_clean_user_prompt(prompt_content);
        let prompt_text = if clean_prompt.trim().is_empty() { prompt_content.trim() } else { clean_prompt.as_str() };
        if crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance(&canonical_inst, prompt_text) {
            dispatched_via_cdp = true;
            let (_, _, pids) = crate::modules::instance::is_instance_process_running_smart(&canonical_inst);
            if !pids.is_empty() {
                crate::modules::instance::focus_running_instance(&pids, Some(repo_path));
            }
        }
    }

    if !dispatched_via_cdp {
        // Fallback to agy CLI background worker process
        spawn_or_replace_prompt_via_agy(&active_prompt, true)?;
    }
    ```

---

## 3. Rollback & Safety Strategy

1. **Atomic Branch & Revert Isolation**:
   - Each phase touches strictly decoupled files.
   - If CDP injection fails on certain Antigravity versions, the fallback to `spawn_or_replace_prompt_via_agy` remains completely intact.
2. **Key Forwarding Backward Compatibility**:
   - `fix_node_auth_via_gitmap(target, None)` behaves identically to the legacy signature when no key is specified, ensuring zero regression for existing callers.
3. **No Process Termination**:
   - Prompt dispatch never terminates caller IDE instances (`detect_caller_instance_id` preservation).

---

## 4. Verification Script & Test Commands

```powershell
# 1. Rust code format gate
cd src-tauri
cargo fmt -- --check

# 2. Rust clippy gate
cargo clippy --all-targets --all-features

# 3. Frontend compilation gate
cd ..
npm run build
```
