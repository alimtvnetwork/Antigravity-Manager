# Subtask 07: Fix Rust QuotaData Struct Initializer & Rustfmt

## Objective
Fix compilation errors and formatting drift identified via `gitmap pe`:
1. `src-tauri/src/modules/account.rs:2329`: Add missing field `subscription_tier_fetched_at: None` or use `QuotaData::new()`.
2. `src-tauri/src/modules/auto_switcher.rs:2630`: Add missing field `subscription_tier_fetched_at: None`.
3. `src-tauri/src/modules/quota.rs` & `src-tauri/src/commands/fleet.rs`: Fix rustfmt formatting drift.

## Target Files
- `src-tauri/src/modules/account.rs`
- `src-tauri/src/modules/auto_switcher.rs`
- `src-tauri/src/modules/quota.rs`
- `src-tauri/src/commands/fleet.rs`

## Verification
- `cargo fmt -- --check` passes cleanly.
- `cargo clippy --all-targets --all-features` compiles with 0 errors.
