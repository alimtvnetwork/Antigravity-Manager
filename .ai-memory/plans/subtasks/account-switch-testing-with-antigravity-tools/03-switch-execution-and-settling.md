# Subtask 03: Switch Execution and Settling

## Objective
Execute the account switch targeting the test instance using both `agm switch` and `gitmap switch` CLI commands. Observe and enforce the mandatory 10-second settling period to confirm child IDE process lifecycle and readiness.

## Execution Steps
1. Execute switch via `agm` CLI:
   - Command: `cargo run --bin agm -- switch -i <test_instance> <target_account>`
2. Execute/validate switch command via `gitmap switch` (verifying CLI parity).
3. Observe mandatory 10-second settling delay:
   - Track PID creation, child process tree, and memory footprint.
   - Verify prompt communication channel or readiness probe.
4. Confirm caller IDE (`default-copy-5597`) was never touched or stopped.
