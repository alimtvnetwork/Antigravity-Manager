# Subtask 004: E2E Cloning Sync Test and Preflight Verification

## Objective
1. Author an end-to-end integration test in `src-tauri/tests/instance_cloning_and_sync_test.rs` (marked with `#[ignore]`) verifying:
   - Duplicating an instance copies `settings.json`, `keybindings.json`, `security_presets.json`, and `workspaceStorage/`.
   - Theme and Antigravity preferences (`turboMode`, `planReviewAlwaysProceed`, policies) are successfully duplicated and preserved.
2. Run pre-flight checks:
   - `cd src-tauri && cargo fmt -- --check`
   - `npm run build`
3. Execute atomic GitMap commit and push.

## Target Files
- `src-tauri/tests/instance_cloning_and_sync_test.rs`

## Acceptance Criteria
- [ ] Integration test passes with `cargo test --test instance_cloning_and_sync_test -- --ignored`.
- [ ] Rust formatting checks pass (`cargo fmt -- --check`).
- [ ] Frontend build passes without TypeScript errors (`npm run build`).
- [ ] Changes are cleanly committed using `gitmap cpf`.
