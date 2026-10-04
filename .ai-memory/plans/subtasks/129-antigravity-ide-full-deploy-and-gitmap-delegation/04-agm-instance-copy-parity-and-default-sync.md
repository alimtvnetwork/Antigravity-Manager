# Subtask 04: Antigravity Manager Instance Copy Parity and Default Sync

- **Subtask ID**: `129-subtask-04`
- **Traceability ID**: `Task-05`
- **Spec References**:
  - `02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/01-architecture-spec.md`
  - `02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/02-component-spec.md`
  - `02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/03-root-cause-analysis.md`
- **Target Files**:
  - `src-tauri/src/modules/instance.rs`
- **Status**: Ready for Execution

---

## 1. Objective

Resolve the core code defects in `src-tauri/src/modules/instance.rs` causing cloned Antigravity instances to lose the Default instance's theme, preset mode (Turbo autonomous mode), plugins, skills, and installed extensions. Ensure 100% feature and visual parity between source and target profiles across all creation and duplication pathways.

---

## 2. Granular Implementation Steps

### Step 1: Extensions Directory Discovery & Propagation
- In `copy_instance_with_options` (~line 1102):
  - Check if `source.extensions_dir` is `Some(...)`.
  - If `None` (standard for the Default instance), dynamically discover the host extensions directory from candidate locations:
    - `%USERPROFILE%/.vscode/extensions`
    - `%USERPROFILE%/.antigravity/extensions`
    - `$HOME/.vscode/extensions`
    - `$HOME/.antigravity/extensions`
  - Assign the discovered directory to `new_instance.extensions_dir` and persist it in `instances.json`.
  - Replicate or seed the extensions directory into `<dst_home>/.vscode/extensions` so that isolated home instances retain access to theme extensions and tools when launched with custom `$env:USERPROFILE`.

### Step 2: Refine Aggressive Cache Filter in `copy_dir_recursive`
- In `copy_dir_recursive` (~line 2024):
  - Replace the overly broad `name_str.contains("cache")` with exact string matching for known Chromium/Electron volatile cache folders:
    ```rust
    let is_lock = name_str == "lockfile"
        || name_str.ends_with(".lock")
        || name_str.starts_with("singleton");

    let is_exact_cache_dir = name_str == "cache"
        || name_str == "code cache"
        || name_str == "gpucache"
        || name_str == "dawngraphitecache"
        || name_str == "dawnwebgpucache"
        || name_str == "cacheddata"
        || name_str == "crashpad"
        || name_str == "blob_storage"
        || name_str.starts_with(".org.chromium");

    if is_lock || is_exact_cache_dir {
        continue;
    }
    ```
  - This guarantees skills, documentation, and config files with "cache" in their names (e.g. `dbt-bigquery-cache`) are never dropped.

### Step 3: Expand `GEMINI_CLONE_DIRS`
- In `src-tauri/src/modules/instance.rs` (~line 1545):
  - Expand the constant slice:
    ```rust
    pub const GEMINI_CLONE_DIRS: &[&str] = &[
        "antigravity",
        "antigravity-ide",
        "antigravity-cli",
        "policies",
        "config",
        "skills",
        "plugins",
    ];
    ```

### Step 4: Internal Call to `enforce_default_settings` Post-Clone
- At the end of `copy_instance_with_options`:
  - Invoke `enforce_default_settings(Some(&new_instance.id))` to guarantee that `antigravity.turboMode = true`, `antigravity.planReviewAlwaysProceed = true`, `antigravity.browserExecutionPolicy = "auto"`, and baseline policies are stamped into the target profile.

### Step 5: Implement `sync_instance_ide_parity` Engine
- Create `pub fn sync_instance_ide_parity(target_id: &str, source_id: Option<&str>) -> Result<(), String>`:
  - Pillar 1 (Theme): Extract `workbench.colorTheme` and inject fallback `"Default Dark+"` if not present.
  - Pillar 2 (Presets): Enforce `security_presets.json` (Turbo Mode) and `antigravity_policies.json`.
  - Pillar 3 (Plugins): Replicate official 4 plugins into `<dst_home>/.gemini/config/plugins/`.
  - Pillar 4 (Skills): Replicate 9 builtin skills into `<dst_home>/.gemini/antigravity/builtin/skills/`.
  - Invoke `sync_instance_ide_parity` inside `copy_instance_with_options` and `create_instance_with_account`.

### Step 6: Unit Test Coverage in `instance.rs`
- Add targeted unit tests:
  - `test_copy_instance_extensions_propagation`: Asserts `extensions_dir` is populated when cloning from Default.
  - `test_copy_instance_theme_parity`: Asserts `workbench.colorTheme` is present in cloned `settings.json`.
  - `test_copy_instance_turbo_presets`: Asserts `security_presets.json` exists in cloned user folder.
  - `test_copy_dir_recursive_cache_filter`: Asserts non-volatile files containing "cache" are not skipped.

---

## 3. Acceptance Criteria

1. Cloning any instance creates target `settings.json` files with `"workbench.colorTheme"` explicitly set.
2. Cloned instance has `security_presets.json` containing Turbo Mode (`"active_preset": "turbo"`) and `security.workspace.trust.enabled: false`.
3. Target instance home contains the 4 official plugins and 9 builtin skills.
4. `cargo clippy --all-targets --all-features` in `src-tauri` compiles cleanly with zero warnings.
5. All new unit tests pass in `src-tauri`.

---

## 4. Targeted Verification

- Run targeted Rust tests:
  ```powershell
  cd src-tauri
  cargo test test_copy_instance -- --nocapture
  cargo clippy --all-targets --all-features
  ```
