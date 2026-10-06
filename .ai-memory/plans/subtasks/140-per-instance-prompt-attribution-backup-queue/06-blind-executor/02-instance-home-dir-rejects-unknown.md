# Step 02: get_instance_home_dir rejects unknown ids

Today `get_instance_home_dir("anything")` silently creates `<data dir>/instances/anything/home` when the id is not registered (bug B03). After this step it returns `Err` for an unknown id and still creates `home` for a registered id.

## 1. Depends on

Step 01 (uses `canonical_instance_id_in`, and the test helpers `scope_cfg` / `scope_registry`).

## 2. Files you may edit

- `src-tauri/src/modules/instance.rs`
- `src-tauri/tests/per_instance_prompt_liveness_test.rs` (two assertions that expect a bare suffix to resolve; Change C)

## 3. Find it

| Change | Anchor (`fn` signature) | Unique search literal | Line hint |
|---|---|---|---|
| A. Replace `get_instance_home_dir` | `pub fn get_instance_home_dir(instance_id: &str) -> Result<PathBuf, String> {` | `pub fn get_instance_home_dir` | 24 to 34 |
| B. Add test | `mod tests {` | `fn resolve_instance_input_requires_input_and_uses_active_only_when_typed` (added in step 01) | near 6613 plus the step 01 block |
| C1. Integration test assertion | `async fn test_e2e_suffix_matching_for_cloned_instances() {` in `src-tauri/tests/per_instance_prompt_liveness_test.rs` | `get_instance_home_dir("8159")` (first hit) | 874 to 876 |
| C2. Integration test helper | `fn assert_suffix_resolutions() {` in the same file | `let home_8159 = get_instance_home_dir("8159")` | 2152 |

Search commands (GitMap does not descend into `src-tauri/src/bin` or `src-tauri/tests` when you search `src-tauri/src`, so the full-path searches are required):

```text
gitmap aum search "pub fn get_instance_home_dir" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "fn resolve_instance_input_requires_input_and_uses_active_only_when_typed" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "get_instance_home_dir" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "get_instance_home_dir" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "get_instance_home_dir" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "get_instance_home_dir" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
```

Expected on 2026-10-06: `agm.rs` has 1 hit (`:14262`, `if let Ok(inst_home) = instance::get_instance_home_dir(&new_inst.id) {`, a registry id; no change). The first two test files have 0 hits. `per_instance_prompt_liveness_test.rs` has 4 hits (`:874` comment, `:875` call, `:2137` `use`, `:2152` call); `:875` and `:2152` pass the bare suffix `"8159"` and expect `default-copy-8159/home`. After this step that call returns `Err`, so those two assertions are rewritten in Change C. The signature does not change, so the test file still compiles either way; Change C keeps it passing.

## 4. Current code

### Change A (lines 24 to 34)

```rust
/// Resolve or create the isolated home directory for an instance
pub fn get_instance_home_dir(instance_id: &str) -> Result<PathBuf, String> {
    let resolved_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let instances_root = get_instances_dir()?;
    let home_dir = instances_root.join(&resolved_id).join("home");
    if !home_dir.exists() {
        fs::create_dir_all(&home_dir)
            .map_err(|e| format!("Failed to create instance home directory: {}", e))?;
    }
    Ok(home_dir)
}
```

Facts you can rely on (already verified):

- `use std::path::{Path, PathBuf};` and `use std::fs;` are at the top of `instance.rs` (lines 4 and 5).
- `get_instances_dir()` is at line 13 and returns `Result<PathBuf, String>`.
- `create_instance_with_account` (line 820) saves the new instance to the registry (`save_registry`, line 1045) before any caller asks for its home dir. `copy_instance_with_options` (line 1144) calls `create_instance` first. So creating and cloning instances still work after this change.

### Callers (no change needed; listed so you can confirm)

`gitmap aum search "get_instance_home_dir(" src-tauri/src --ext .rs` returns 32 hits. Every caller already handles `Err` with `if let Ok(...)` or `.ok()`:

- `src-tauri/src/modules/agy_cleaner.rs:108`
- `src-tauri/src/modules/instance.rs:1235`, `:1300`, `:1366`, `:1448`, `:1476`, `:1645`, `:1654`, `:1657`, `:1705`, `:1743`, `:2701`, `:2958`, `:3249`, `:3314`, `:3395`, `:3591`, `:4602`, `:5242`, `:5295`, `:5440`, `:5491`, `:5558`, `:5650`, `:5714`, `:5807`, `:5846`, `:5956`, `:6016`
- `src-tauri/src/modules/repo_db.rs:833` (replaced in step 04), `:3166` (`spawn_prompt_via_agy`)

- `src-tauri/src/bin/agm.rs:14262` (`if let Ok(...)`, registry id)

If the count is not 32 (plus the definition) in `src-tauri/src`, or a new caller uses `?` or `.unwrap()` on the result, STOP and report it.

### C1. `src-tauri/tests/per_instance_prompt_liveness_test.rs` lines 874 to 876 (inside `test_e2e_suffix_matching_for_cloned_instances`)

```rust
    // Verify get_instance_home_dir targeting "8159" resolves to default-copy-8159/home
    let home_8159 = antigravity_tools_lib::modules::instance::get_instance_home_dir("8159")
        .expect("get instance home dir");
```

`instances_dir` (defined at line 829 as `sandbox.path().join("instances")`) is in scope here.

### C2. Same file, line 2152 (inside `fn assert_suffix_resolutions()`)

```rust
    let home_8159 = get_instance_home_dir("8159").expect("get home dir");
```

## 5. New code

### Change A: replace the whole block from section 4 with

```rust
/// Resolve or create the isolated home directory for a registered instance.
/// Unknown ids return `Err`; no folder is created for them.
pub fn instance_home_dir_in(
    registry: &InstanceRegistry,
    instances_root: &Path,
    instance_id: &str,
) -> Result<PathBuf, String> {
    let resolved_id = canonical_instance_id_in(registry, instance_id)?;
    let home_dir = instances_root.join(&resolved_id).join("home");
    if !home_dir.exists() {
        fs::create_dir_all(&home_dir)
            .map_err(|e| format!("Failed to create instance home directory: {}", e))?;
    }
    Ok(home_dir)
}

/// Resolve or create the isolated home directory for an instance
pub fn get_instance_home_dir(instance_id: &str) -> Result<PathBuf, String> {
    let registry = load_registry()?;
    let instances_root = get_instances_dir()?;
    instance_home_dir_in(&registry, &instances_root, instance_id)
}
```

### C1. Replace the three lines from 4 C1 with

```rust
    // get_instance_home_dir takes a registry id; a bare suffix is rejected and creates no folder
    assert!(antigravity_tools_lib::modules::instance::get_instance_home_dir("8159").is_err());
    assert!(!instances_dir.join("8159").exists());
    let home_8159 =
        antigravity_tools_lib::modules::instance::get_instance_home_dir("default-copy-8159")
            .expect("get instance home dir");
```

The following `home_path_str` / `assert!(home_path_str.contains("default-copy-8159/home"), ...)` lines stay unchanged.

### C2. Replace the line from 4 C2 with

```rust
    assert!(get_instance_home_dir("8159").is_err());
    let home_8159 = get_instance_home_dir("default-copy-8159").expect("get home dir");
```

The `resolve_instance_id("8159")` assertions above it stay unchanged (`resolve_instance_id` is not modified by this plan).

## 6. Tests

Paste this test inside `mod tests` of `instance.rs`, directly after the test `resolve_instance_input_requires_input_and_uses_active_only_when_typed` from step 01. It uses a temp folder for `instances_root`; it does not touch `HOME`, `USERPROFILE`, `APPDATA`, or `ABV_DATA_DIR`. `tempfile` is already a dev-dependency (`src-tauri/Cargo.toml` line 104).

```rust
    #[test]
    fn get_instance_home_dir_does_not_create_unknown_folder() {
        let root = tempfile::tempdir().unwrap();
        let registry = scope_registry("default");

        assert!(instance_home_dir_in(&registry, root.path(), "__ghost__").is_err());
        assert!(!root.path().join("__ghost__").exists());
        assert!(instance_home_dir_in(&registry, root.path(), "all").is_err());
        assert!(!root.path().join("all").exists());

        let home = instance_home_dir_in(&registry, root.path(), "WORK-1234").unwrap();
        assert_eq!(home, root.path().join("work-1234").join("home"));
        assert!(home.is_dir());

        let default_home = instance_home_dir_in(&registry, root.path(), "__default__").unwrap();
        assert_eq!(default_home, root.path().join("default").join("home"));
    }
```

The original subtask described this test with `get_instance_home_dir("__ghost__")` and a temp `HOME`. It is redesigned against the pure core `instance_home_dir_in` because the wrapper calls `load_registry()`, which tests must never call.

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 get_instance_home_dir_does_not_create_unknown_folder; cargo test --lib -- --test-threads=1 canonical_instance_id; cd ..
```

## 8. Commit

```text
Fix: prompts - reject unknown ids in get_instance_home_dir
```

Stage exactly: `src-tauri/src/modules/instance.rs`, `src-tauri/tests/per_instance_prompt_liveness_test.rs`.

## 9. Done when

- [ ] `get_instance_home_dir` no longer calls `resolve_instance_id`.
- [ ] `instance_home_dir_in` exists, is `pub`, and takes `&InstanceRegistry` and `&Path`.
- [ ] The caller count is unchanged and every caller still uses `if let Ok` or `.ok()`.
- [ ] Callers were searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file; `gitmap aum search "get_instance_home_dir" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs` shows 6 hits (new comment, two `.is_err()` asserts, two `"default-copy-8159"` calls, the `use` line) and none of them reads `get_instance_home_dir("8159").expect`.
- [ ] `get_instance_home_dir_does_not_create_unknown_folder` passes.
- [ ] Gate exited 0; commit pushed with the exact message.
