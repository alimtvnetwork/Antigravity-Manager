# Subtask 02: Steps 2 & 4 - Upstream Connection Backoff Matrix, TCP Keepalive & Claude Turn Validator Unit Tests

- **Parent Plan**: `.ai-memory/plans/pending/175-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/175-test-case-implementation-and-task-management/spec.md#step-2-unit-test-suite-for-upstream-connection-backoff-matrix--tcp-keepalive-probe`
- **Target Files**:
  - `src-tauri/src/proxy/upstream/tests_backoff_and_probe.rs`
  - `src-tauri/src/proxy/handlers/claude/tests_message_validator.rs`
- **Status**: [COMPLETED]

## Completed Work
1. Implemented `tests_backoff_and_probe.rs` covering:
   - Config default validation (500ms initial, 30s max, 2.0x multiplier, 0.25 jitter)
   - Zero-jitter deterministic exponential progression
   - Max delay ceiling clamping
   - ±20% jitter factor bounds
   - Attempt overflow safety (attempt 100)
   - TCP keepalive defaults (30s idle, 10s probe, count 3)
   - Custom idle/interval socket configurations
2. Implemented `tests_message_validator.rs` covering:
   - Empty messages array rejection
   - Invalid role validation (`system`, `tool`)
   - Trailing whitespace trimming from text contents
   - Empty user content minimal dot (`.`) backfilling
   - Structured array content preservation
   - Alternating turn verification (`user -> assistant -> user`)
   - Consecutive role concatenation (`sanitize_claude_role_alternation`)
3. Registered modules in `src-tauri/src/proxy/upstream/mod.rs` and `src-tauri/src/proxy/handlers/claude/mod.rs`.
