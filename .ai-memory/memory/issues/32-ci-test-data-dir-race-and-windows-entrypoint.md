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
2. **Binary Test Target Execution on Headless CI:** Invoking `cargo test` without `--lib` compiles test harnesses for the binary crate `agm-alim` (`src/main.rs`). On Windows CI runners lacking GUI subsystem scaffolding, launching GUI test harnesses causes `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)`.
3. **Missing Manifest Linking in Build Script:** `build.rs` did not link `windows-test.manifest` for MSVC targets, leaving test executables vulnerable to common control DLL entrypoint failures.
4. **Unconditional Import of Target-Specific Trait:** `CommandExtWrapper` was imported unconditionally in `email_inbound.rs`, triggering unused import warnings on Unix platforms.

## Part 4: Corrective Action & Verification
1. **Thread-Safe Data Dir Test Synchronization:**
   - Updated `test_load_app_config_self_heals_empty_string` and `test_load_app_config_recovers_from_backup` in `src-tauri/src/modules/config.rs` to acquire `crate::modules::account::TEST_DATA_DIR_MUTEX`.
   - Added `data_dir_override_slot()` clearing in `test_set_current_account_id_with_target` to guarantee complete state isolation.
2. **Re-Embed Windows Test Manifest:**
   - In `src-tauri/build.rs`, added MSVC manifest embedding for `windows-test.manifest` via `cargo:rustc-link-arg=/MANIFEST:EMBED` and `/MANIFESTINPUT`.
3. **Harden CI Test Step and Platform Matrix:**
   - Updated `.github/workflows/ci.yml` matrix from `windows-2025` to stable `windows-latest`.
   - Appended `--lib` to the `cargo test` command so that all 752 library unit tests execute without attempting to invoke the GUI binary target.
4. **Clean Compiler Warnings & Conditional Imports:**
   - Added `#[cfg(target_os = "windows")]` to `CommandExtWrapper` in `email_inbound.rs`.
   - Cleaned unused imports and variables in `models/mod.rs`, `modules/integration.rs`, `modules/repo_db.rs`, `modules/cloudflared.rs`, `commands/patch.rs`, `commands/supabase.rs`, and `proxy/server.rs`.
5. **Local Verification:**
   - Executed `cargo test --manifest-path src-tauri/Cargo.toml --lib -j 2`: **All 752 tests passed (0 failed)**.
   - Executed `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: **Clean formatting (code 0)**.
   - Executed `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -j 2`: **Zero errors (code 0)**.
   - Executed `npx tsc --noEmit` and `npm run build`: **Built successfully (code 0)**.
