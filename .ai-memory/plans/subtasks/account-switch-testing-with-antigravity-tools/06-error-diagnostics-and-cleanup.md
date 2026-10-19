# Subtask 06: Error Diagnostics, Self-Healing, and Test Cleanup

## Objective
Inspect runtime errors and logs across `errors.db` and log directories, run `agm errors fix` self-healing verification, execute `agm instances test-clean` to purge all test sandboxes, and run pre-flight code health checks.

## Execution Steps
1. Inspect runtime error logs:
   - Query `errors.db` via `cargo run --bin agm -- errors list --json`.
   - Inspect diagnostic logs in `%APPDATA%\antigravity-manager\logs\`.
2. Run self-healing repair ladder:
   - Execute `cargo run --bin agm -- errors fix`.
   - Verify 8/8 repair checks pass including `PRAGMA quick_check(1)`.
3. Purge test sandbox:
   - Execute `cargo run --bin agm -- instances test-clean switch-test --json`.
   - Verify 0 leftover processes and directories.
4. Verify caller IDE preservation:
   - Verify developer's working instance (`default-copy-5597`) remained uninterrupted.
5. Pre-flight verification:
   - Check `cargo fmt -- --check`, `npm run build`, and `cargo check --bin agm`.
