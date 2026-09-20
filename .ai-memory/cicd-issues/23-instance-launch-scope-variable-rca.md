# Root Cause Analysis: Rust Compilation Error `target_data_path` Undefined in Scope

## 1. Symptom

In GitHub Actions CI on pipeline run `35483541630` (commit `61eda74f9cee175b8af89584e5ced3754baa3fb3`):
All 6 Rust compilation and Clippy jobs across macOS (`macos-latest`), Windows (`windows-2025`), and Ubuntu (`ubuntu-latest`) failed with exit code 1 / 101:
```text
error[E0425]: cannot find value `target_data_path` in this scope
  --> src/modules/instance.rs:437:26
   |
437|             let db_dir = target_data_path.join("User").join("globalStorage");
   |                          ^^^^^^^^^^^^^^^^ not found in this scope
error: could not compile `agm-alim` (lib) due to 1 previous error
```

## 2. Root Cause

In `src-tauri/src/modules/instance.rs`, the account credential token injection block (lines 433–461) was introduced into `launch_instance` to inject auth tokens directly into the isolated instance's SQLite database (`state.vscdb`).
This block referenced `target_data_path`, but `let target_data_path = PathBuf::from(&data_dir);` was only defined further down at line 464 (inside the lockfile cleanup section).
Because `target_data_path` was referenced prior to its declaration, the Rust compiler aborted with `cannot find value target_data_path in this scope [E0425]`.

## 3. Resolution

1. **Scoped Declaration Reordering in `src-tauri/src/modules/instance.rs`**:
   - Lifted `let target_data_path = PathBuf::from(&data_dir);` directly above the bound account credential injection block.
   - Removed the duplicate subsequent declaration at line 464, allowing `target_data_path` to be in scope for both the auth token database injection and the lockfile purge routines.
2. **Quality Verification**:
   - Ran `run.ps1 -Check` validating TypeScript compilation and cargo fmt.
   - Ran `python 03-ai-scripts/06-cicd-local-runner.py` verifying all 36 local quality gates passed cleanly (`36 gates in 8.89s`).

## 4. Prevention & Learnings

1. Whenever introducing token injection or file operations in `src-tauri/`, ensure all path variables are initialized immediately after config extraction.
2. Maintain local compiler verification workflows in CI runner to catch lexical scope bugs before git release tagging.
