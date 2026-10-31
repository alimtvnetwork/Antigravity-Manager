---
plan: ssh-and-prompt-management-testing
subtask: "01"
title: SSH Fleet Card & Instance Telemetry Hygiene
domain: fullstack-telemetry
depends_on: []
target_files:
  - src/components/instances/fleet/FleetMachineRow.tsx
  - src/components/settings/EmailNotifications/SshGitmapSection.tsx
  - src-tauri/src/modules/ssh_manager/exec.rs
status: pending
---

# Subtask 01: SSH Fleet Card & Instance Telemetry Hygiene

## 1. Context & Motivation
In the Antigravity-Manager Instance view and SSH fleet management views, remote node cards and tables previously exposed administrative operating system usernames (`Administrator`, `root`, `${adminUser}`, `${linuxUser}`) and password authentication indicators. In team environments, live demos, and screen recordings, displaying raw OS administrative credentials presents a security and privacy risk.

Furthermore, users require high operational clarity regarding their cluster:
1. **Prominent Operational Attributes**: Users must immediately see the **Node IP address**, **Node Alias**, **Running Instance Count**, and the **Bound Account Emails** associated with each instance.
2. **Credential Redaction**: Administrative usernames and password references must be completely eliminated from the visual display and telemetry payloads.
3. **Hygiene Across Frontend & Backend**: The redaction must be enforced end-to-end: in `FleetMachineRow.tsx`, in `SshGitmapSection.tsx`, and in backend SSH telemetry execution in `src-tauri/src/modules/ssh_manager/exec.rs`.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src/components/instances/fleet/FleetMachineRow.tsx` | `FleetMachineRow`, `FleetMachineRowProps` | 1. Sanitize `cleanOsInfo` and `node_alias` to eliminate `Administrator`, `root`, `${adminUser}`, and `${linuxUser}`.<br/>2. Scrub instance profile names and tooltips of administrative strings.<br/>3. Prominently display node IP with one-click copy, node alias, active running instance count badge, and masked/unmasked bound account emails.<br/>4. Ensure zero password references or credentials are displayed. |
| `src/components/settings/EmailNotifications/SshGitmapSection.tsx` | `SshGitmapSection`, `handleSaveNodeRsa`, `handleRefreshClusterBalances` | 1. Replace default username state `'Administrator'` with empty string `''` and friendly placeholder `OS user (e.g. deploy or worker)`.<br/>2. Ensure SSH node table and cluster balance matrix display IP, alias, running instance count, and bound account emails without leaking admin usernames.<br/>3. In Multi-Machine Instance matrix, ensure prompt delegation displays `node.alias (ip)` without raw credentials. |
| `src-tauri/src/modules/ssh_manager/exec.rs` | `exec_ssh_command`, `SshExecNodeResult`, `GITMAP_CLUSTER_NODES`, `get_cluster_instances_and_balances` | 1. Redact administrative usernames in `SshExecNodeResult` target string (return `node.alias` or `node.ip_address` instead of `Administrator@192.168.1.3`).<br/>2. Sanitize error messages and command outputs to ensure vaulted passwords or admin accounts do not leak into telemetry logs.<br/>3. Ensure `get_cluster_instances_and_balances` filters admin usernames from bound email fields. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Sanitize and Highlight Operational Attributes in `FleetMachineRow.tsx`

1. **Scrub Administrative References**:
   Expand the scrubbing logic so that `cleanOsInfo`, `machine.node_alias`, and `active_instances` profile labels never render `Administrator`, `root`, `${adminUser}`, or `${linuxUser}`:
   ```tsx
   // Sanitize OS info and node alias
   const cleanOsInfo = machine.os_info
       ?.replace(/\b(administrator|root|\$\{adminUser\}|\$\{linuxUser\})\b/gi, '')
       .replace(/\s+/g, ' ')
       .trim();

   const cleanNodeAlias = machine.node_alias
       ?.replace(/\b(administrator|root|\$\{adminUser\}|\$\{linuxUser\})\b/gi, '')
       .trim() || 'Worker Node';
   ```

2. **Sanitize Bound Account Emails and Instance Tooltips**:
   Filter out any administrative usernames appearing in emails or profile summaries:
   ```tsx
   const emailList = Array.from(
       new Set([
           ...(machine.bound_emails || []),
           ...(machine.active_instances || [])
               .map((i) => i.bound_account_email)
               .filter(Boolean) as string[],
       ])
   ).filter(
       (email) =>
           !email.toLowerCase().startsWith('administrator@') &&
           !email.toLowerCase().startsWith('root@') &&
           !email.toLowerCase().includes('${adminuser}') &&
           !email.toLowerCase().includes('${linuxuser}') &&
           !email.toLowerCase().includes('administrator') &&
           !email.toLowerCase().includes('root')
   );
   ```

3. **Prominent Display Hierarchy**:
   - **Machine & Status**: Display `cleanNodeAlias`, online/offline pulse indicator, Mesh badge, and clean OS label.
   - **IP Address**: Highlight with monospace font, dark-glass pill container, and copy button with confirmation tooltip.
   - **Running Instance(s)**: Render a bold count badge (e.g. `2 insts`) accompanied by profile capsules with pulsing green indicator dots for running instances.
   - **Bound Account Email(s)**: Render account email list with individual or global mask toggle (`Eye`/`EyeOff`).
   - **Zero Password Display**: Ensure no password hints, hashes, or credentials are shown.

---

### 3.2 Step 2: Redact Administrative Defaults & Table Views in `SshGitmapSection.tsx`

1. **Remove Privileged Default Username**:
   In `SshGitmapSection.tsx`:
   ```tsx
   // Update initial state from 'Administrator' to empty string
   const [username, setUsername] = useState('');
   ```
   Update the input element placeholder:
   ```tsx
   <input
       type="text"
       value={username}
       onChange={(e) => setUsername(e.target.value)}
       placeholder="OS user (e.g. deploy or worker)"
       className="w-full h-8 px-2.5 text-xs rounded-lg border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500"
   />
   ```

2. **Clean Display in Configured Nodes Table**:
   Ensure table columns highlight:
   - **Node & Alias**: `node.alias` with OS tag.
   - **IP / Host:Port**: `node.ip_address:node.port` with copy button.
   - **Running Instance(s)**: `runningCount Running` with pulse dot.
   - **Bound Account Email(s)**: `nodeEmails` with admin filtering.
   - **Security & Auth**: Badged as `RSA-2048 Vault` or `SSH Key Auth` with zero password text or credential hashes.

3. **Multi-Machine Instance & Account Credit Matrix**:
   Ensure `item.machine_alias`, `item.ip_address`, `item.instance_name`, `displayEmail`, and quota percentage bars are prominently displayed, with administrative usernames fully stripped from `displayEmail`.

---

### 3.3 Step 3: Telemetry Hygiene in `src-tauri/src/modules/ssh_manager/exec.rs`

1. **Sanitize `target` in `SshExecNodeResult`**:
   Currently, `exec_ssh_command` constructs the target field as:
   `target: format!("{}@{}", node.username, node.ip_address)`
   When `node.username` is `Administrator` or `root`, this leaks administrative credentials to the caller. Modify the target formatting helper:
   ```rust
   fn format_sanitized_target(node: &SshConnectionRecord) -> String {
       let user = node.username.trim();
       let is_admin = user.eq_ignore_ascii_case("administrator")
           || user.eq_ignore_ascii_case("root")
           || user.contains("${adminUser}")
           || user.contains("${linuxUser}");

       if is_admin || user.is_empty() {
           format!("{} ({})", node.alias, node.ip_address)
       } else {
           format!("{}@{}", user, node.ip_address)
       }
   }
   ```
   Apply `format_sanitized_target(node)` across all `SshExecNodeResult` constructor sites in `exec.rs`.

2. **Sanitize Error Messages & Stderr**:
   Ensure command failure messages in `exec_ssh_command` reference `node.alias` and `node.ip_address` rather than printing `Administrator@<ip>` or raw vaulted passwords.

---

## 4. Acceptance Criteria & Edge Cases

- [ ] **AC-1 (No Admin Usernames in Fleet Row)**: `FleetMachineRow.tsx` never displays `Administrator`, `root`, `${adminUser}`, or `${linuxUser}` in `node_alias`, `os_info`, or bound email list.
- [ ] **AC-2 (Prominent IP & Copy)**: Every fleet row displays the node IP address in a clean monospace badge with a working copy-to-clipboard button.
- [ ] **AC-3 (Prominent Instance Count)**: Fleet row displays the number of active instances running on that node (e.g. `1 inst`, `3 insts`) with live pulse indicators.
- [ ] **AC-4 (Prominent Bound Emails)**: Fleet row displays the genuine OAuth account emails bound to those instances, with working mask/reveal toggle.
- [ ] **AC-5 (Zero Password Exposure)**: Neither fleet cards nor instance views render password inputs, password values, or raw credential strings.
- [ ] **AC-6 (Unprivileged Default in Form)**: `SshGitmapSection.tsx` input form defaults `username` to empty string with `"OS user (e.g. deploy or worker)"` placeholder instead of hardcoded `"Administrator"`.
- [ ] **AC-7 (Backend Telemetry Redaction)**: `SshExecNodeResult` emitted from `exec_ssh_command` reports `target` as `alias (ip)` or sanitized user, never exposing `Administrator@<ip>`.
- [ ] **Edge Case (Unassigned / Idle Nodes)**: Nodes with 0 running instances cleanly display `"No active profiles"` and `"Unassigned"` without rendering errors.
- [ ] **Edge Case (OS Info Substrings)**: Operating systems reported as `"Windows Server Administrator"` are scrubbed cleanly to `"Windows Server"`.

---

## 5. Verification Commands

Run the following commands to verify that code changes adhere to project standards:

```bash
# 1. Frontend build verification
npm run build

# 2. Rust formatting gate
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# 3. Rust compiler and clippy gate
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features
```
