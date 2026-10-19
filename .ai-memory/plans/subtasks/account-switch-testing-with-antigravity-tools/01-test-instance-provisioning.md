# Subtask 01: Test Instance Provisioning

## Objective
Provision an isolated test instance sandbox using `agm instances test-create <name> --json` to verify sandboxing, clean profile directories, and zero collision with the developer's working instance (`default-copy-5597`).

## Execution Steps
1. Execute `cargo run --bin agm -- instances test-create switch-test --json`.
2. Inspect JSON output:
   - Confirm `instance_id` begins with `test-`.
   - Confirm profile directory paths are isolated.
   - Confirm keyring bypass flag is enabled.
3. Validate that caller IDE protection (`detect_caller_instance_id()`) actively guards `default-copy-5597`.
4. Output execution status and record instance metadata.
