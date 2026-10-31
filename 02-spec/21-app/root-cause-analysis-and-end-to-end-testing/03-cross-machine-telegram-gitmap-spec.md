# Component Specification: Telegram Relay, Date/Time Acknowledgment & GitMap SSH Parity

## 1. Scope & Objective
This specification governs cross-machine prompt relay and acknowledgment in:
1. `src-tauri/src/modules/telegram_inbound/injection.rs`
2. `src-tauri/src/modules/telegram_inbound/send_flow.rs`
3. `src-tauri/src/modules/telegram_inbound/ui_remote.rs`
4. `src-tauri/src/modules/telegram_inbound/open_shot.rs`
5. `src-tauri/src/modules/ssh_manager/` & `src-tauri/src/modules/supabase_sync/`

---

## 2. Telegram Acknowledgment Protocol & Timestamp Attribution

### 2.1 Problem Formulation
When a prompt is dispatched (locally or across machines), the Telegram response message currently formats:
`✅ <b>Prompt sent successfully</b>\n• <b>Instance:</b> ...\n• <b>Reference:</b> <id8>`
It completely omits the Date and Time when the prompt was sent. The user cannot correlate when the action was taken.

### 2.2 Detailed Remediation Protocol
Every prompt acknowledgment envelope (in `injection.rs`, `send_flow.rs`, and `ui_remote.rs`) MUST include:
- ISO-8601 UTC timestamp and Local Time:
  ```rust
  let now_utc = chrono::Utc::now();
  let time_str = now_utc.format("%Y-%m-%d %H:%M:%S UTC").to_string();
  let local_str = chrono::Local::now().format("%Y-%m-%d %I:%M:%S %p %Z").to_string();
  ```
- Formatted HTML block:
  ```html
  ✅ <b>Prompt Dispatched & Acknowledged</b>
  • <b>Timestamp:</b> <code>{time_str}</code> ({local_str})
  • <b>Target Node:</b> <code>{target_node}</code>
  • <b>Instance:</b> <code>{instance_name}</code>
  • <b>Reference:</b> <code>{ref_id}</code>
  • <b>Attached Media:</b> {media_count} image(s)
  ```

---

## 3. Remote Proof Shot & Session 0 Windows Bypass

### 3.1 Problem Formulation
When sending a prompt across machines via SSH, the target machine runs the SSH daemon in Windows Session 0 (non-interactive background service).
Attempting to run PowerShell `System.Drawing.Graphics.CopyFromScreen` in Session 0 immediately throws `Win32Exception: The handle is invalid` because Session 0 has no desktop window station. This causes visual proof capture to fail, and Telegram only sends an error message.

### 3.2 Detailed Remediation Protocol
1. **Fallback Ladder for Remote Screenshot**:
   - **Tier 1 (AGM Local API Proxy)**: If AGM is running on the target machine (port 8045), invoke `http://<node_ip>:8045/api/remote/screenshot`. Because AGM runs in the user's interactive Session 1, it captures the real desktop display.
   - **Tier 2 (Chrome DevTools Protocol - CDP)**: Connect to the running IDE's remote debugging port (e.g. `9222`) and invoke `Page.captureScreenshot`, which renders the DOM buffer headlessly without requiring a desktop DC.
   - **Tier 3 (Safe Fallback Notice)**: If neither is reachable, return an explicit informative acknowledgment explaining that the prompt was successfully injected into `repo_db.rs` / `agy` on the target machine, citing exact PID and timestamp, without failing the entire command.

---

## 4. GitMap SSH Cluster Parity (Offline Supabase Resilience)

### 4.1 Problem Formulation
GitMap successfully connects two machines (e.g. `w1` @ 192.168.1.3 and `main` @ 192.168.1.20) in ~279ms using direct peer-to-peer SSH on Port 22 (`~/.ssh/id_ed25519`).
In contrast, AGM's fleet synchronization was hard-coupled to Supabase PostgreSQL, which is currently paused (NXDOMAIN). When Supabase is paused, the AGM Fleet UI displays an error or empty state, even though the machines are fully connected via GitMap SSH.

### 4.2 Detailed Remediation Protocol
1. In `src-tauri/src/modules/supabase_sync/query.rs` and `src/components/instances/fleet/useFleetMachines.ts`:
   - When Supabase is unreachable or paused, read `~/.gitmap/gitmap-ssh-nodes.json` and active SSH config.
   - Populate `FleetMachinesTable` with the live GitMap SSH nodes (`main`, `w1`), their IP addresses, and ping latency.
   - Allow remote prompt dispatch directly over `gitmap ssh exec <node> 'agm prompt ...'` or `ssh_manager::exec_ssh_command`.
