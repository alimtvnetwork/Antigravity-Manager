# Issue 37: Windows Test Runner Manifest Entrypoint Resolution & CI Rate Limit Protection

## Part 1: Symptoms and Blast Radius
1. **Windows Test Runner Crash (`0xc0000139`)**:
   In GitHub Actions CI workflow run `#36015043093` on commit `38ab6b6`, the `Check Rust Code (windows-2025)` job failed during the `Run Rust tests` step with:
   ```text
   error: test failed, to rerun pass `--lib`
   Caused by:
     process didn't exit successfully: `antigravity_tools_lib-*.exe` (exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND)
   ```
2. **GitHub Pages Deployment Failure**:
   Workflow run `#36015043116` failed on `Setup Pages` (`actions/configure-pages@v5`) with:
   ```text
   Error 403: API rate limit exceeded for installation.
   ```
3. **Release Publishing Rate Limit Collision**:
   Workflow run `#36015048779` on tag `v4.72.0` failed during `Create or Update GitHub Release` with `Error 403: API rate limit exceeded for installation`.

---

## Part 2: Proximate Cause
1. **Missing Common-Controls 6.0 Manifest in Test Executable**:
   In `src-tauri/build.rs`, commit `22ea77db` switched `WindowsAttributes::new_without_app_manifest()` to `WindowsAttributes::new()`, but simultaneously removed the MSVC link argument `/MANIFESTINPUT:windows-test.manifest`. While `WindowsAttributes::new()` embeds a manifest into the primary GUI binary (`agm-alim.exe`), Cargo compiles library tests into a standalone test executable (`antigravity_tools_lib-*.exe`) without linking `resource.lib`. Because `antigravity_tools_lib-*.exe` lacked an embedded application manifest specifying `Microsoft.Windows.Common-Controls` 6.0.0.0, the Windows NT dynamic loader bound to legacy `comctl32.dll` v5.82, which does not export `TaskDialogIndirect` (invoked by `tauri-plugin-dialog`), causing the test process to terminate immediately on launch with `STATUS_ENTRYPOINT_NOT_FOUND` (`0xc0000139`).
2. **Unfiltered Trigger on Pages Workflow**:
   `.github/workflows/deploy-pages.yml` ran on every commit pushed to `main` without `paths:` filtering, repeatedly querying GitHub API for Pages configuration even when zero web assets were modified.
3. **Post-Release Purge API Quota Depletion**:
   In `03-ai-scripts/34-purge-github-actions-artifacts.py`, artifact purging executed unthrottled DELETE API calls across 12 concurrent worker threads without inspecting remaining rate limits. When multiple release tags were pushed within a 1-hour window, the `GITHUB_TOKEN` hourly installation rate limit was exhausted, breaking downstream steps.

---

## Part 3: Root Cause Analysis
1. **Cargo Test Target vs Binary Manifest Linking**:
   `tauri_build::build()` and `WindowsAttributes::new()` embed the application manifest inside Windows resource file `resource.rc`, compiled into `resource.lib` and linked via `cargo:rustc-link-lib`. In Cargo's architecture, build script `rustc-link-lib` outputs are consumed by `[[bin]]` targets, but Cargo's library test harness (`cargo test --lib`) links the library crate as a standalone test runner. To provide Common-Controls 6.0 to both targets without causing MSVC `CVT1100: duplicate resource` errors, `WindowsAttributes::new_without_app_manifest()` must be paired with `cargo:rustc-link-arg=/MANIFEST:EMBED` and `/MANIFESTINPUT:windows-test.manifest`.
2. **Lack of Rate Limit Throttling in AI Automation Scripts**:
   API-driven cleanup tools must guard against burning user API tokens by checking `/rate_limit` before batch deletion and pausing when remaining calls drop below a safe threshold.

---

## Part 4: Preventive and Remediating Actions
1. **Restored Universal Manifest Embedding (`src-tauri/build.rs`)**:
   Switched to `WindowsAttributes::new_without_app_manifest()` and emitted `cargo:rustc-link-arg=/MANIFEST:EMBED` with `/MANIFESTINPUT:windows-test.manifest`. This embeds the manifest with `Microsoft.Windows.Common-Controls` 6.0.0.0 into both `agm-alim.exe` and `antigravity_tools_lib-*.exe` with zero resource collisions.
2. **Added OS Compatibility Section to `src-tauri/windows-test.manifest`**:
   Added `compatibility` block with `supportedOS` entries for Windows 10/11/Server 2025 (`{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}`).
3. **Added Targeted `--lib` Flag to CI (`.github/workflows/ci.yml`)**:
   Updated the test step to `cargo test --manifest-path src-tauri/Cargo.toml --lib`.
4. **Scoped Pages Workflow (`.github/workflows/deploy-pages.yml`)**:
   Added `paths: ['web_site/**', '.github/workflows/deploy-pages.yml']` and `enablement: true`.
5. **Rate Limit Safety Guard in Purge Script (`03-ai-scripts/34-purge-github-actions-artifacts.py`)**:
   Added `check_rate_limit_safety(threshold=150)` to immediately halt purging if API quota drops below 150 requests.
6. **Clippy Code Quality Fixes**:
   Resolved empty doc comment warnings in `claude/request.rs`, `claude/utils.rs`, `upstream/client.rs`, and cleaned up identity `map_err` and collapsed `if let` in `commands/proxy.rs` and `commands/patch.rs`.

---

## Part 5: Verification Gate
- `cargo fmt -- --check`: Passed cleanly (code 0).
- `cargo check`: Passed with clean profile compilation in 30.37s (code 0).
- `cargo test test_version --lib`: Verified test binary `antigravity_tools_lib-*.exe` launched and passed cleanly on Windows with zero `0xc0000139` entrypoint faults (code 0).
- `npm run build`: Passed cleanly with zero TypeScript errors (code 0).
