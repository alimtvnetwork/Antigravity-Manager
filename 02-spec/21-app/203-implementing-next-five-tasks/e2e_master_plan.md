# Sequence 203 E2E Master Test Plan: Preflight, Caller Protection, PR Template, Thinking Budget, and FIFO Query Validation

## Objectives
Validate Batch 203 end-to-end integration across preflight release checks, caller IDE self-preservation exit code enforcement, PR template compliance, thinking budget filtering, and FIFO prompt query validation.

## Test Scenarios
1. **Preflight Checks Gate**:
   - Verify `cargo fmt -- --check`, `cargo clippy`, and `npm run build` execution prerequisites.
   - Detect unformatted Rust files and compiler warnings, rejecting tag creation.
2. **Caller IDE Exit Code Enforcer**:
   - Verify that commands targeting caller IDE instance ID fail-fast with exit code 2.
   - Verify that `--force` flag bypasses caller check and allows execution.
3. **PR Template Compliance Guard**:
   - Parse PR descriptions against `.github/pull_request_template.md`.
   - Verify presence of Problem Classification, Behavior Alterations, Unverified Paths, and Rollback Strategy.
4. **Thinking Budget Filter Pipeline**:
   - Model thinking capability resolution.
   - Backfill default budget for thinking-enabled models, strip/filter budget for non-thinking models.
5. **FIFO Prompt Restore Validator**:
   - SQL query inspection for `ORDER BY created_at ASC, id ASC`.
   - Inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly intercepted and rejected.
