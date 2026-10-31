# Specification: Multi-Machine Fleet Synchronization & GitMap Connection Engine

## User Request (Verbatim)

```text
# High Priority Instruction

I think you need to consider fixing the fleet option and also connecting with the other machines. Here you have shared the information from the other machine. However, in your case, this information does not match with the Gitmap. Gitmap can easily connect with the machine. You can also use initially Gitmap to get connected and try to get the idea. So you can try to run the end-to-end commands to see where you are missing this stuff, why your tool cannot connect and communicate with your CLI from the other machine. What is missing there? Because Gitmap can. So you can write Gitmap nodes and other stuff to check how Gitmap can communicate. Then you can fix your issues, why it does not show the W2 machine, because W2 machine is connected, W2 machine is running right now. So these type of things, I think you need to fix it. Also, at the same time, you need to consider the fleet option, that should automatically shared cluster should actually show up something. The check again does not do anything or show any error. That is something I want you to investigate right now in this machine by running the code. So CLI should have most of the methods so that you can test out these things and also confirm with the UI later on. So these are still not fixed. And have you created the log methods for the CLI to, log methods for the AI to make it decent and enhanced? And also the button on top for the SSH and Gitmap section inside the settings. The top right-hand button, I will highlight that. It looks pretty terrible, actually. The button does not have the right-hand side import SSH, Gitmap deploy encrypted credential. So this feels like unprofessional. Also, the copy install command, very unprofessional. Need to fix it, make it professional. The text, I can highlight some of the parts. You can make it enhanced better, look better. So these sections, I have highlighted with red so that you can understand where things are an issue. Multi-machine instance is not working. I think you need to look deep into this. And one more point, can you spawn three agent rather than two? And can we correct this into the execute parent task in v6? Instead of two, we'll deploy three agents, each one of them will have two async operation ability. Can we do that? How many places do we have to change in order to achieve that? Do we have to update the coding guideline? I think we do. And we also have to update the coding guideline, Gitmap skills, and the execute v6 skills. These are the things you should do right now, not the other repository. So in Gitmap and coding guideline. Do you understand this? Can you please follow through?

DO a minor bump and release please
```

---

## 1. Executive Summary & Root Cause Analysis (RCA)

### 1.1 Root Cause 1: Empty IP Address `()` in Multi-Machine Matrix
- **Symptom:** The table column `Machine Alias / IP` renders as `W3 ()`, `main ()`, `w2 ()`, `w4 ()`, `u1 ()` with empty parentheses.
- **Root Cause:** In `src-tauri/src/modules/ssh_manager/exec.rs`, the backend populates `ClusterInstanceBalanceItem.machine_ip`, but the frontend `src/components/settings/EmailNotifications/SshGitmapSection.tsx` attempted to read `item.ip_address`. Since `item.ip_address` was `undefined`, it rendered `({item.ip_address})` as literal empty `()`.
- **Remediation:**
  1. Backend: Populate both `machine_ip` and `ip_address` fields on `ClusterInstanceBalanceItem` to maintain bi-directional serialization parity.
  2. Frontend: Resolve IP dynamically using `item.ip_address || item.machine_ip || conn?.ip_address || '127.0.0.1'`.

### 1.2 Root Cause 2: Literal `${adminUser}` and `${linuxUser}` Placeholders
- **Symptom:** Account Email column in the matrix renders `${adminUser}@main`, `${adminUser}@w2`, `${linuxUser}@u1` instead of actual usernames or resolved account emails.
- **Root Cause:** SSH connection templates imported from default configurations contain un-interpolated template variables `${adminUser}` and `${linuxUser}`.
- **Remediation:**
  1. Backend `src-tauri/src/modules/ssh_manager/connections.rs`: Implement variable interpolation that resolves `${adminUser}` to the local/remote administrative user (`Administrator`) or queries GitMap's encrypted credential vault (`gitmap ssh pass ls`).
  2. Resolve `${linuxUser}` to standard Linux usernames (`root`, `ubuntu`, `admin`) or vault-stored users.

### 1.3 Root Cause 3: W2 Machine Reported as `Idle` Despite Being Active
- **Symptom:** Node `w2` displays `w2:default Idle` and static `100.0%` credits, while GitMap commands (`gitmap ssh nodes`) confirm `w2` is online and accessible.
- **Root Cause:** `check_tcp_liveness(&conn.ip_address, port, 600)` in `exec.rs` utilized an aggressive 600ms timeout on static IP addresses without fallback to GitMap node discovery or ARP retry. When TCP connect timed out, `running` was set to `false`, defaulting to `Idle` with dummy balances.
- **Remediation:**
  1. Increase connection timeout to 1500ms with a 2-stage retry ladder.
  2. Implement GitMap node synchronization fallback: if direct TCP fails, invoke `gitmap ssh exec <alias> "hostname"` to verify live execution and update connectivity state.
  3. Show genuine connection error states (e.g. `Connection Timeout: 192.168.1.7:22`, `Auth Failed`) rather than silent failure.

### 1.4 Root Cause 4: Silent Failure on "Check Cluster Balances"
- **Symptom:** Clicking "Check Cluster Balances" or "Check again" spins momentarily and disappears without feedback, error messages, or latency indicators.
- **Root Cause:** `handleRefreshClusterBalances` in `SshGitmapSection.tsx` caught errors without displaying an actionable toast, and the backend command did not return structured reachability diagnostics.
- **Remediation:**
  1. Backend `check_cluster_balances`: Return structured payload containing `checked_count`, `online_count`, `offline_count`, `latency_ms`, and node-by-node error descriptions.
  2. Frontend: Emit success/warning toasts (e.g., `Checked 6 cluster nodes: 2 online, 4 unreachable`) and badge unreachable nodes with error popovers in the matrix.

---

## 2. Architecture & Data Contracts

### 2.1 Backend Models (`src-tauri/src/modules/ssh_manager/models.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNodeDiagnostic {
    pub alias: String,
    pub ip_address: String,
    pub is_online: bool,
    pub latency_ms: u64,
    pub error_message: Option<String>,
    pub quota_4h_percent: f64,
    pub quota_weekly_percent: f64,
    pub active_account_email: Option<String>,
    pub running_instances: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterBalanceCheckReport {
    pub total_nodes: usize,
    pub online_nodes: usize,
    pub offline_nodes: usize,
    pub duration_ms: u64,
    pub nodes: Vec<ClusterNodeDiagnostic>,
}
```

### 2.2 CLI Commands (`src-tauri/src/bin/agm/`)

```text
agm ssh test <alias>         # Probes TCP + SSH connectivity and measures round-trip latency
agm ssh logs <alias>         # Fetches last 50 lines of remote Antigravity / AGM execution logs
agm ssh check-balances       # Executes full cluster balance interrogation with structured table output
```

---

## 3. Acceptance Criteria

1. **AC-FLEET-01 (IP Visibility):** All rows in the Multi-Machine Matrix display their genuine IP address; zero empty `()` occur.
2. **AC-FLEET-02 (Variable Resolution):** `${adminUser}` and `${linuxUser}` are 100% resolved to valid accounts or vault usernames.
3. **AC-FLEET-03 (W2 Liveness):** W2 node displays accurate live status when running; timeouts include retry and GitMap fallback.
4. **AC-FLEET-04 (Error Visibility):** "Check Cluster Balances" displays a descriptive toast with online/offline counts and highlights unreachable nodes.
5. **AC-FLEET-05 (CLI & AI Logs):** `agm ssh test` and `agm ssh check-balances` CLI commands provide rich JSON and formatted table outputs.
