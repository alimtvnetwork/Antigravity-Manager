# Subtask Plan 02: Backend Command & Service Integration

- **Worker:** Worker 02
- **Target Files:**
  * `src-tauri/src/commands/cmd_ssh.rs`
  * `src/services/sshService.ts`

## Objectives
1. In `src-tauri/src/commands/cmd_ssh.rs`:
   - Update `check_cluster_balances` command to return a comprehensive diagnostic struct (`ClusterBalanceCheckReport` or enhanced `Vec<ClusterInstanceBalanceItem>` with diagnostic fields `error_message`, `latency_ms`, `is_online`).
   - Add command `ssh_test_node(alias: String) -> Result<NodeTestResult, AppError>`.
   - Ensure all errors flow into `AppError` and are logged via `tracing` without silent discards.
2. In `src/services/sshService.ts`:
   - Update interfaces `ClusterInstanceBalanceItem` to include `ip_address?: string`, `machine_ip: string`, `error_message?: string`, `latency_ms?: number`.
   - Update `refreshClusterBalances` and `testSshNode` service methods to handle the enhanced responses.
