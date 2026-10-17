# Subtask Plan 01: Backend SSH Manager, W2 Resolution & CLI Diagnostics

- **Worker:** Worker 01
- **Target Files:**
  * `src-tauri/src/modules/ssh_manager/connections.rs`
  * `src-tauri/src/modules/ssh_manager/config.rs`
  * `src-tauri/src/modules/ssh_manager/exec.rs`
  * `src-tauri/src/modules/ssh_manager/models.rs`
  * `src-tauri/src/bin/agm/mod.rs` (or `src-tauri/src/bin/agm/ssh.rs`)

## Objectives
1. In `src-tauri/src/modules/ssh_manager/models.rs`:
   - Ensure `ClusterInstanceBalanceItem` has both `machine_ip: String` and `#[serde(alias = "ip_address")] ip_address: Option<String>` (or serializes both).
2. In `src-tauri/src/modules/ssh_manager/connections.rs` and `config.rs`:
   - Implement variable interpolation: replace `${adminUser}` with the active administrator name (`Administrator`) and `${linuxUser}` with `root` or vault user.
   - When loading connections, resolve variables dynamically.
3. In `src-tauri/src/modules/ssh_manager/exec.rs`:
   - In `check_tcp_liveness`: increase timeout to 1500ms and add a 2-stage retry ladder.
   - If TCP connect fails, try running a quick probe command via GitMap (`gitmap ssh exec <alias> "hostname"`).
   - In `get_cluster_instance_balances`: set `ip_address` on each item and return diagnostic reachability.
4. In `src-tauri/src/bin/agm/`:
   - Implement `agm ssh test <alias>` to test TCP + SSH connectivity and display latency.
   - Implement `agm ssh logs <alias>` to fetch remote logs.
   - Implement `agm ssh check-balances` to query cluster balances and output structured table.
