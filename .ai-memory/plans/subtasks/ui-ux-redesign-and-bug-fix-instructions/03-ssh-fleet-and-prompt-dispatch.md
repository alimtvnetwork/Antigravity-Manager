---
plan: ui-ux-redesign-and-bug-fix-instructions
subtask: "03"
title: SSH Instance & Google Account Discovery, IP Demotion, and Remote Prompt Dispatch
domain: fullstack-ssh-fleet
depends_on: []
target_files:
  - src/components/settings/EmailNotifications/SshGitmapSection.tsx
  - src/components/instances/FleetMachinesTable.tsx
  - src/services/sshService.ts
  - src-tauri/src/modules/ssh_manager/types.rs
  - src-tauri/src/modules/ssh_manager/exec.rs
status: pending
---

# Subtask 03: SSH Instance & Google Account Discovery, IP Demotion, and Remote Prompt Dispatch

## 1. Context & Root Cause Analysis

### 1.1 The Operational Problem
In Antigravity-Manager, operators manage multi-node development environments consisting of local machines and remote workstations connected via SSH and GitMap mesh networking. Multiple IDE profiles/instances run simultaneously across these machines, each tied to a specific Google account for quota usage and task execution.

Previously, operators encountered severe friction when attempting to understand and control this fleet:
1. **Misplaced Visual Focus**: The UI displayed raw IP addresses as prominent table columns, cluttering the view. Operators' primary mental model is: **Which Google account is bound to which instance, on which machine?** The IP address is secondary network infrastructure metadata that should only appear on demand.
2. **Superficial Cluster Balance Polling**: When operators clicked "Refresh Matrix" in `SshGitmapSection.tsx`, `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` only ran a TCP port 22 handshake. It never queried remote instances or bound accounts, returning synthetic dummy items (`instance_id: "default"`, empty `bound_email`).
3. **Prompt Dispatch Interface Mismatch**: Sending prompts to remote instances failed or degraded because `RemotePromptDispatchPayload` in `src-tauri/src/modules/ssh_manager/types.rs` lacked field serialization aliases (`instance_target`), and `RemotePromptDispatchResult` lacked execution telemetry fields (`stdout`, `stderr`, `duration_ms`, `command_executed`). Furthermore, operators had to manually type instance names into an unguided text box rather than selecting from discovered instances.

---

## 2. Disjoint Target Files & Symbols Inventory

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src-tauri/src/modules/ssh_manager/types.rs` | `RemotePromptDispatchPayload`, `RemotePromptDispatchResult` | Add `#[serde(alias = "instance_target", alias = "instance_id")]` to `instance_name`; align `RemotePromptDispatchResult` fields with `command_executed`, `stdout`, `stderr`, and `duration_ms`. |
| `src-tauri/src/modules/ssh_manager/exec.rs` | `get_cluster_instances_and_balances`, `send_remote_ssh_prompt` | 1. Query remote nodes concurrently via SSH with `agm instances list --json` to discover real instances and bound accounts.<br/>2. Sanitize admin credentials from outputs.<br/>3. Populate full execution envelope with duration and stdout/stderr in `send_remote_ssh_prompt`. |
| `src/services/sshService.ts` | `ClusterInstanceBalanceItem`, `RemotePromptDispatchPayload`, `RemotePromptDispatchResult`, `sshService` | Align TypeScript types with backend Rust structures; ensure safe fallback handling and no silent errors. |
| `src/components/instances/FleetMachinesTable.tsx` | `FleetMachinesTable`, `FleetMachineRow` | Reorganize columns: elevate Connected Google Account Email and Active Instances; demote IP address into an interactive hover popover/tooltip with 1-click copy. |
| `src/components/settings/EmailNotifications/SshGitmapSection.tsx` | `SshGitmapSection` | 1. Replace raw text instance input with an intelligent `<select>` dropdown populated with `<instance_name> — <bound_email>`.<br/>2. Demote IP addresses in the nodes and credit matrix tables into compact tooltip badges with 1-click copy.<br/>3. Wire 1-click "Prompt This Instance" buttons to lock target machine/instance and focus prompt input. |

---

## 3. Step-by-Step Implementation Ladder

### 3.1 Step 1: Rust Type Alignment in `src-tauri/src/modules/ssh_manager/types.rs`

1. **Update `RemotePromptDispatchPayload`**:
   Add serde aliases so both frontend property conventions (`instance_target` and `instance_name`) deserialize cleanly:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct RemotePromptDispatchPayload {
       pub node_alias: String,
       #[serde(alias = "instance_target", alias = "instance_id")]
       pub instance_name: String,
       pub prompt_text: String,
       #[serde(default)]
       pub conversation_id: Option<String>,
   }
   ```

2. **Update `RemotePromptDispatchResult`**:
   Add execution envelope fields:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct RemotePromptDispatchResult {
       pub success: bool,
       pub node_alias: String,
       #[serde(alias = "instance_target")]
       pub instance_name: String,
       #[serde(default)]
       pub dispatch_route: String,
       #[serde(default)]
       pub message: String,
       #[serde(default)]
       pub command_executed: String,
       #[serde(default)]
       pub stdout: String,
       #[serde(default)]
       pub stderr: String,
       #[serde(default)]
       pub duration_ms: u128,
   }
   ```

---

### 3.2 Step 2: Remote Discovery & Execution Ladder in `src-tauri/src/modules/ssh_manager/exec.rs`

1. **Upgrade `get_cluster_instances_and_balances`**:
   - Collect local instances using `load_registry()` and `list_accounts()`.
   - For remote SSH nodes in `ssh_conns`:
     Execute concurrent SSH checks across nodes using `std::thread::scope`:
     ```rust
     let remote_items: Vec<Vec<ClusterInstanceBalanceItem>> = std::thread::scope(|s| {
         let handles: Vec<_> = remote_conns
             .iter()
             .map(|conn| {
                 let target_model = target_model.clone();
                 s.spawn(move || {
                     let mut node_items = Vec::new();
                     let port = if conn.port == 0 { 22 } else { conn.port };
                     let start = Instant::now();
                     let reachable = check_tcp_liveness(&conn.ip_address, port, 1500);
                     let elapsed_ms = start.elapsed().as_millis() as u64;

                     if !reachable {
                         node_items.push(ClusterInstanceBalanceItem {
                             machine_alias: conn.alias.clone(),
                             machine_ip: conn.ip_address.clone(),
                             ip_address: Some(conn.ip_address.clone()),
                             error_message: Some(format!("TCP port {} unreachable on {}", port, conn.ip_address)),
                             latency_ms: Some(elapsed_ms),
                             is_local: false,
                             instance_id: "default".to_string(),
                             instance_name: format!("{}:default", conn.alias),
                             running: false,
                             bound_email: String::new(),
                             tier: "SSH-NODE".to_string(),
                             quota_4h_pct: 100.0,
                             quota_weekly_pct: 100.0,
                             active_model: target_model.clone(),
                             status_label: "offline".to_string(),
                         });
                         return node_items;
                     }

                     // Node reachable: query real instances via SSH
                     let remote_cmd = vec!["instances".to_string(), "list".to_string(), "--json".to_string()];
                     let exec_res = exec_ssh_command(&conn.alias, &remote_cmd, Some(port), None, None);
                     if let Ok(res_list) = exec_res {
                         if let Some(res) = res_list.first().filter(|r| r.success && !r.stdout.is_empty()) {
                             if let Ok(val) = serde_json::from_str::<serde_json::Value>(&res.stdout) {
                                 let raw_instances = val.get("data").and_then(|d| d.as_array()).or_else(|| val.as_array());
                                 if let Some(inst_array) = raw_instances {
                                     for inst_val in inst_array {
                                         let inst_id = inst_val.get("id").and_then(|v| v.as_str()).unwrap_or("default").to_string();
                                         let inst_name = inst_val.get("name").and_then(|v| v.as_str()).unwrap_or(&inst_id).to_string();
                                         let is_running = inst_val.get("is_running").and_then(|v| v.as_bool()).unwrap_or(false);
                                         let raw_email = inst_val.get("bound_email").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                         let clean_email = if raw_email.is_empty() || raw_email.contains("Administrator") || raw_email.contains("root") {
                                             "(unbound)".to_string()
                                         } else {
                                             raw_email
                                         };

                                         node_items.push(ClusterInstanceBalanceItem {
                                             machine_alias: conn.alias.clone(),
                                             machine_ip: conn.ip_address.clone(),
                                             ip_address: Some(conn.ip_address.clone()),
                                             error_message: None,
                                             latency_ms: Some(elapsed_ms),
                                             is_local: false,
                                             instance_id: inst_id,
                                             instance_name: inst_name,
                                             running: is_running,
                                             bound_email: clean_email,
                                             tier: "PRO".to_string(),
                                             quota_4h_pct: 100.0,
                                             quota_weekly_pct: 100.0,
                                             active_model: target_model.clone(),
                                             status_label: if is_running { "running".to_string() } else { "idle".to_string() },
                                         });
                                     }
                                 }
                             }
                         }
                     }

                     if node_items.is_empty() {
                         node_items.push(ClusterInstanceBalanceItem {
                             machine_alias: conn.alias.clone(),
                             machine_ip: conn.ip_address.clone(),
                             ip_address: Some(conn.ip_address.clone()),
                             error_message: None,
                             latency_ms: Some(elapsed_ms),
                             is_local: false,
                             instance_id: "default".to_string(),
                             instance_name: format!("{}:default", conn.alias),
                             running: true,
                             bound_email: "(unbound)".to_string(),
                             tier: "SSH-NODE".to_string(),
                             quota_4h_pct: 100.0,
                             quota_weekly_pct: 100.0,
                             active_model: target_model.clone(),
                             status_label: "reachable".to_string(),
                         });
                     }

                     node_items
                 })
             })
             .collect();
         handles.into_iter().filter_map(|h| h.join().ok()).collect()
     });
     ```

2. **Enrich `send_remote_ssh_prompt`**:
   - Record start timestamp with `Instant::now()`.
   - Dispatch via `execute_prompt_injection(&cmd_str)`.
   - Populate `command_executed`, `stdout`, `stderr`, and `duration_ms` in `RemotePromptDispatchResult`.

---

### 3.3 Step 3: TypeScript Service Alignment in `src/services/sshService.ts`

1. Update `RemotePromptDispatchPayload` and `RemotePromptDispatchResult` interfaces to support both `instance_target` and `instance_name`.
2. Ensure `getClusterInstancesAndBalances()` maps `bound_email` and sanitized instance names smoothly without swallowing errors.

---

### 3.4 Step 4: IP Demotion & Account Prominence in `FleetMachinesTable.tsx` and `FleetMachineRow.tsx`

1. **`FleetMachinesTable.tsx` Table Headers**:
   Re-sequence table headers to prioritize accounts and instances:
   ```tsx
   <th className="px-3.5 py-2.5 min-w-[180px]">Machine & Status</th>
   <th className="px-3.5 py-2.5 min-w-[220px]">Connected Google Account(s)</th>
   <th className="px-3.5 py-2.5 min-w-[200px]">Active Instance(s)</th>
   <th className="px-3.5 py-2.5 min-w-[120px]">Running Prompts</th>
   <th className="px-3.5 py-2.5 text-right min-w-[110px]">Last Seen</th>
   ```
2. **`FleetMachineRow.tsx`**:
   - Embed the IP address into an interactive subtle hover pill next to the machine alias with a 1-click copy icon.
   - Promote `Bound Account Email(s)` to the second column with primary typography and mask toggles.
   - Render instance profile pills with live running indicator dots (`animate-pulse`).

---

### 3.5 Step 5: Intelligent Instance Dropdown & 1-Click Dispatch in `SshGitmapSection.tsx`

1. **Intelligent Instance Dropdown**:
   Replace the raw text input with a dynamic `<select>`:
   ```tsx
   {/* Segmented Capsule for Machine + Instance Selection */}
   <div className="inline-flex items-center rounded-full border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 px-3 py-1 shadow-xs">
       <span className="text-[11px] font-semibold text-gray-400 mr-1.5">Machine:</span>
       <select
           value={targetMachine}
           onChange={(e) => {
               const newMachine = e.target.value;
               setTargetMachine(newMachine);
               const matching = clusterBalances.filter(
                   (item) => item.machine_alias.toLowerCase() === newMachine.toLowerCase()
               );
               if (matching.length > 0) {
                   setTargetInstance(matching[0].instance_name || matching[0].instance_id);
               } else {
                   setTargetInstance('default');
               }
           }}
           className="text-xs font-mono text-gray-900 dark:text-slate-100 bg-transparent border-0 focus:ring-0 focus:outline-none pr-2 cursor-pointer"
       >
           <option value="local">local (This Machine)</option>
           {nodes.map((n) => (
               <option key={n.alias} value={n.alias}>
                   {n.alias}
               </option>
           ))}
       </select>
       <div className="h-4 w-px bg-gray-200 dark:bg-slate-700 mx-2" />
       <span className="text-[11px] font-semibold text-gray-400 mr-1.5">Instance:</span>
       <select
           value={targetInstance}
           onChange={(e) => setTargetInstance(e.target.value)}
           className="text-xs font-mono text-gray-900 dark:text-slate-100 bg-transparent border-0 focus:ring-0 focus:outline-none pr-2 cursor-pointer max-w-[200px]"
       >
           {clusterBalances
               .filter((item) => item.machine_alias.toLowerCase() === targetMachine.toLowerCase())
               .map((item) => (
                   <option key={`${item.machine_alias}-${item.instance_id}`} value={item.instance_name || item.instance_id}>
                       {item.instance_name} — {item.bound_email || 'unbound'} ({item.running ? 'Running' : 'Idle'})
                   </option>
               ))}
           {clusterBalances.filter((item) => item.machine_alias.toLowerCase() === targetMachine.toLowerCase()).length === 0 && (
               <option value="default">default — (Primary Instance)</option>
           )}
       </select>
   </div>
   ```

2. **Demote IP Address in Tables**:
   - In both the configured nodes table and cluster credit matrix table, wrap the IP address in a compact interactive chip with tooltip and 1-click copy feedback.

3. **1-Click "Prompt This Instance" Dispatch**:
   - Clicking "Prompt This Instance" in the matrix locks `targetMachine` and `targetInstance`, shows a confirmation toast, and scrolls/focuses the prompt input field.

---

## 4. Edge Cases & Safety Gates

1. **Unreachable Remote Node**:
   If an SSH worker node is offline, `check_tcp_liveness` times out within 1500ms. The backend safely emits an offline status record without hanging the UI.
2. **Missing `agm` on Remote Machine**:
   If the remote worker lacks `agm` in its system PATH, `agm instances list --json` returns non-zero. The backend catches the failure and falls back to a default reachable placeholder.
3. **Empty or Unbound Accounts**:
   If an instance has not yet completed Google OAuth binding, it is explicitly rendered as `(unbound)`, preventing null pointer exceptions or misleading blank cells.
4. **No Nested White Pills**:
   All `<select>` dropdowns inside segmented capsules strictly utilize `bg-transparent border-0 focus:ring-0 focus:outline-none` in adherence to repository UI conventions.

---

## 5. Verification Protocol

- **Targeted Rust Compilation**:
  `cd src-tauri && cargo clippy --all-targets --all-features`
- **Frontend Verification**:
  `npm run build`
- **Manual Verification Matrix**:
  1. Load `SshGitmapSection.tsx`: Verify the instance selector is a dropdown populated with discovered instances.
  2. Switch machine in dropdown: Verify instance options update dynamically to match the selected machine.
  3. Click "Prompt This Instance" on a matrix row: Verify target machine and instance update immediately.
  4. Inspect `FleetMachinesTable.tsx`: Verify Connected Google Accounts are prominent, and IP is demoted to a subtle hover pill.
