# 4-Part RCA: Rust Compilation Failure in Release #35514458062 and Release Asset Pinned Version Stamping

## 1. Symptom

In GitHub Actions Release workflow run `#35514458062` on commit `8d8c17491ef74b44eb324665ad669f70df43d008`, all 6 build-tauri jobs (macOS x86_64, Windows 2025, macOS universal, macOS aarch64, Ubuntu 22.04, Ubuntu 24.04-arm) failed during `Build the app` step:

```text
error: could not compile `agm-alim` (lib) due to 3 previous errors; 53 warnings emitted
failed to build app: failed to build app
Error failed to build app: failed to build app
Process completed with exit code 1.
```

Targeted compile logs revealed:
1. `src-tauri/src/modules/auto_switcher.rs`: mismatched type evaluation where `acc.last_used` was treated as an `Option<i64>` rather than an `i64`.
2. `src-tauri/src/proxy/upstream/retry.rs`: missing legacy forwarder `parse_legacy_retry_delay` expected by callers.
3. In addition, when users attempted to install historical pinned versions via one-liners such as `irm https://github.com/.../releases/download/v4.34.0/install.ps1 | iex` or `curl -fsSL https://github.com/.../releases/download/v4.34.0/install.sh | bash`, the installers failed to recognize the pinned version and resolved to the latest release because `__PINNED_VERSION__` was left unstamped in release assets, and piped execution left invocation lines empty.

## 2. Root Cause

1. **Rust Library Signature Drift**: Recent upstream proxy and auto-switcher changes introduced signature and field type alterations (`acc.last_used` typed as `i64` instead of `Option<i64>`, and a refactored `parse_retry_delay` without backward-compatible shim `parse_legacy_retry_delay`).
2. **Release Asset Stamping Omission in CI/CD**: In `workflows/release.yml`, standalone installer scripts `install.ps1`, `install.sh`, and `deploy/arch/install.sh` were copied into `release-files/` without substituting `__PINNED_VERSION__` with the target release tag `${VER}`.
3. **Piped Invocation Blindspot in Installers**: When running via `irm ... | iex` or `curl ... | bash`, standard shell invocation variables (`$MyInvocation.Line` and `BASH_EXECUTION_STRING`) are unpopulated because execution occurs via piped standard input or `Invoke-Expression`, requiring interrogation of process command lines, PSReadLine console history, and process groups to detect the source URL.

## 3. Resolution

1. **Rust Compile Fixes (Resolved in commit `e49b9030` / `v4.41.0`)**:
   - Corrected `acc.last_used` evaluation in `src-tauri/src/modules/auto_switcher.rs`.
   - Restored `pub fn parse_legacy_retry_delay(error_text: &str) -> Option<u64>` forwarding to `parse_retry_delay(error_text, None)` in `src-tauri/src/proxy/upstream/retry.rs`.
2. **Release Workflow Asset Stamping (`workflows/release.yml`)**:
   - Replaced plain file copies with `sed "s/__PINNED_VERSION__/${VER}/g"` when packaging `install.ps1`, `install.sh`, and `deploy/arch/install.sh` into `release-files/`.
   - Added dedicated `### Pinned Version (${VERSION})` section in `release_notes.md` with explicit Windows and Linux/macOS code blocks.
3. **Multi-Source Pinned Version Detection in Installers**:
   - **PowerShell (`install.ps1`)**: Added `Resolve-PinnedVersion` checking explicit `-Version`, baked `$PinnedVersion`, `$MyInvocation`, persistent PSReadLine history (`ConsoleHost_history.txt`), in-memory PSReadLine history, `Get-History`, `[System.Environment]::CommandLine`, and CIM `Win32_Process` (process + parent process).
   - **Bash (`install.sh`)**: Added multi-candidate discovery querying `$VERSION`, baked `PINNED_VERSION`, `BASH_EXECUTION_STRING`, `/proc/$PPID/cmdline`, `/proc/$$/cmdline`, `ps -p "$PPID" -o args=`, process group arguments, and `~/.bash_history` / `~/.zsh_history`.
   - **Arch (`deploy/arch/install.sh`)**: Added matching URL detection and pinned version respect.

## 4. Prevention & Learnings

1. **Mandatory Pre-Release Local Compilation**: Always execute `cargo check --lib` or `cargo test --no-run` across crates before triggering release ceremonies to catch type mismatches before CI runners invoke them.
2. **Release Asset Artifact Stamping Contract**: Standalone installer scripts packaged into GitHub release assets must always be stamped with the immutable concrete version of that release.
3. **Multi-Source Piped Invocation Protocol**: Never assume `$MyInvocation.Line` or `BASH_EXECUTION_STRING` is populated when scripts can be piped via `iex` or `bash`. Always inspect persistent shell history and process metadata as fallbacks.
