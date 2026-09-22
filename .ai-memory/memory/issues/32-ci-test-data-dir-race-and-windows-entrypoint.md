# Issue 32: CI Test Data Dir Concurrency Race and Windows Entrypoint Failure

## Part 1: Why It Happened
During GitHub Actions CI workflow run #35737415089 on commit `3131f07`, three jobs failed across all matrix platforms:
1. **Ubuntu & macOS Runners:** `cargo test` failed two unit tests with assertion mismatches:
   - `modules::account::tests::test_set_current_account_id_with_target`: panicked at `src/modules/account.rs:444:9` (`left == right` failed, `left: None`, `right: Some("acc-1")`).
   - `modules::config::tests::test_load_app_config_recovers_from_backup`: panicked at `src/modules/config.rs:321:9` (`left == right` failed, `left: "en"`, `right: "zh-TW"`).
2. **Windows Runner (`windows-2025`):** The `Run Rust tests` step exited abruptly with code `0xc0000139 (STATUS_ENTRYPOINT_NOT_FOUND)`.
3. **Compiler Warnings:** Rust compiler emitted unused import warnings across `models/mod.rs`, `modules/integration.rs`, `modules/email_inbound.rs`, and `modules/repo_db.rs`.

## Part 2: How It Happened
1. **Concurrency Race on `ABV_DATA_DIR` in Tests:**
   - In Cargo test runs, unit tests execute concurrently across multiple worker threads in a single process.
   - `test_set_current_account_id_with_target` in `src-tauri/src/modules/account.rs` acquired `TEST_DATA_DIR_MUTEX` and set `std::env::set_var("ABV_DATA_DIR", dir.path())`.
   - Concurrently on another thread, `test_load_app_config_recovers_from_backup` and `test_load_app_config_self_heals_empty_string` in `src-tauri/src/modules/config.rs` mutated `std::env::set_var("ABV_DATA_DIR", temp_dir)` without acquiring `TEST_DATA_DIR_MUTEX`.
   - Thread A (`account.rs`) created its dummy account index, but before `set_current_account_id_with_target` resolved `get_data_dir()`, Thread B (`config.rs`) overwrote `ABV_DATA_DIR` with its own temporary directory.
   - Consequently, `set_current_account_id_with_target` saved the updated account ID into Thread B's directory. When Thread A loaded back from its own `dir.path()`, `current_account_id` remained `None`, triggering the test assertion failure.
   - Conversely, when Thread B attempted to load its backup configuration, Thread A's environment cleanup caused `load_app_config()` to look in an empty directory and generate a default `AppConfig` with `language: "en"` instead of the expected `"zh-TW"`.
2. **Windows Entrypoint Failure (`0xc0000139`):**
   - The CI matrix targeted `windows-2025`, a preview Windows Server image.
   - The test step invoked `cargo test --manifest-path src-tauri/Cargo.toml` without the `--lib` flag.
   - In Tauri applications, `src/main.rs` is a GUI subsystem binary (`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`) that contains zero unit tests. Without `--lib`, Cargo builds and attempts to execute a test harness for the binary target without a console subsystem entrypoint and without runtime dependencies such as `WebView2Loader.dll`.
   - Furthermore, `windows-test.manifest` (stipulated in `02-spec/20-instance-management/04-ide-crash-recovery-and-prompt-resume-spec.md` Rule 5) was omitted from `src-tauri/build.rs`, preventing proper Common-Controls 6.0 manifest embedding.
3. **Platform-Conditional Import Warning:**
   - In `email_inbound.rs`, `CommandExtWrapper` provides `creation_flags_windows()`, which is guarded by `#[cfg(target_os = "windows")]`.
   - On Linux (`ubuntu-latest`), the method call was compiled out, causing rustc to warn that `CommandExtWrapper` was an unused import.

## Part 3: Root Cause Analysis
1. **Unsynchronized Process Environment Mutation:** In Rust, `std::env::set_var` is process-global. While `account.rs` established `TEST_DATA_DIR_MUTEX`, `config.rs` unit tests omitted locking this mutex, resulting in data directory collisions between concurrent test threads.
2. **Missing Common-Controls 6.0 Manifest in Test Executable:** `tauri-plugin-dialog` references `TaskDialogIndirect` from `comctl32.dll`. Because `comctl32.dll` v5.82 does not export `TaskDialogIndirect` (it only exists in v6.0), Windows loader terminates unmanifested test executables before entry with `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)`. Tauri's build step embeds the manifest into `agm-alim.exe`, but `cargo test` runs `antigravity_tools_lib-*.exe` which lacked the manifest.
3. **Architecture Collision in Build Script DLL Copy:** In `src-tauri/build.rs`, `copy_dll_recursive` copied all `WebView2Loader.dll` files in directory traversal order. `webview2-com-sys` generates `x86`, `x64`, and `arm64` DLLs. On Windows, `x86/WebView2Loader.dll` was copied after `x64/WebView2Loader.dll`, overwriting the 64-bit DLL with a 32-bit binary in `deps/`.
4. **Duplicate Manifest Linking on Binary Test Targets:** Specifying `cargo:rustc-link-arg=/MANIFEST:EMBED` caused MSVC `link.exe` to fail with `CVT1100: duplicate resource` on `agm-alim` binary tests because Tauri already attached a manifest to the binary. Disabling `test = false` on `[[bin]]` avoids redundant compilation of `main.rs` as a test.
5. **Platform-Conditional Import in Integration Module:** `std::process::Command` is unused on Windows but required on Linux and macOS for `secret-tool` and `kill`. Removing it caused compilation errors on Unix runners.
6. **Concurrent State Mutation in Thinking Budget Tests:** `test_default_max_tokens` ran in parallel with tests mutating the global `ThinkingBudgetConfig` to `Adaptive`, which unexpectedly set `maxOutputTokens: 64000`.

## Part 4: Corrective Action & Verification
1. **Thread-Safe Test State Synchronization:**
   - Updated `test_load_app_config_self_heals_empty_string` and `test_load_app_config_recovers_from_backup` in `src-tauri/src/modules/config.rs` to acquire `crate::modules::account::TEST_DATA_DIR_MUTEX`.
   - Added `TEST_THINKING_BUDGET_MUTEX` locking and reset in `test_default_max_tokens` in `src-tauri/src/proxy/mappers/claude/request.rs`.
2. **Architecture-Aware DLL Copying in `build.rs`:**
   - Modified `copy_dll_recursive` to verify the parent directory matches `expected_arch_dir` (`x64` for `x86_64`), preventing 32-bit overwrites.
3. **Disable Redundant Binary Test Harness:**
   - Explicitly configured `[[bin]]` with `name = "agm-alim"` and `test = false` in `src-tauri/Cargo.toml`.
4. **Embed Windows Test Manifest:**
   - Re-enabled `/MANIFEST:EMBED` and `/MANIFESTINPUT` for `windows-test.manifest` in `src-tauri/build.rs` for MSVC test executables.
5. **Cross-Platform Trait & Command Imports:**
   - Added `#[allow(unused_imports)] use std::process::Command;` in `src-tauri/src/modules/integration.rs`.
   - Guarded Windows-only traits in `email_inbound.rs` with `#[cfg(target_os = "windows")]`.
6. **Verification:**
   - Executed `cargo test --manifest-path src-tauri/Cargo.toml`: **All 752 unit tests passed (0 failed)**.
   - Executed `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: **Clean formatting (code 0)**.
   - Executed `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`: **Zero errors (code 0)**.
   - Executed `npm run build`: **Built successfully (code 0)**.
