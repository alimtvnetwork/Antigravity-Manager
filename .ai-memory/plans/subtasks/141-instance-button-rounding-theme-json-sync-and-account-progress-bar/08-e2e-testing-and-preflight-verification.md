# Subtask 08: E2E Testing, Quality Gates & Pre-flight Verification

## Objective
Author and execute end-to-end automated tests verifying settings transfer, project copying, and format integrity (`scripts/test-instance-sync-e2e.ps1`). Execute pre-flight compilation and linter gates.

## Target Files
- `scripts/test-instance-sync-e2e.ps1`

## Verification
- E2E script runs and passes all test assertions.
- `cargo fmt -- --check` passes.
- `cargo clippy --all-targets --all-features` passes.
- `npm run build` succeeds.
- Relative path linter passes.
