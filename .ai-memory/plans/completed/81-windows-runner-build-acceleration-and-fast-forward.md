# Master Plan: Windows Runner Build Acceleration & Fast-Forward Toolchain Optimization

> **/goal** Accelerate Windows build times in GitHub Actions from ~55+ minutes down to 10-15 minutes by adopting `rust-lld.exe`, bounded C concurrency (`CMAKE_BUILD_PARALLEL_LEVEL`), stable runner `windows-2022`, multi-directory Defender exclusions, and target dependency caching.

**Plan ID:** `81-windows-runner-build-acceleration-and-fast-forward`
**Created:** 2026-09-27
**Target Release:** Current / Fast-Forward Maintenance
**Status:** Completed

---

## Subtask Execution Roadmap

1. [x] **Subtask 1: Target-Specific Linker Configuration (`src-tauri/.cargo/config.toml`)**
   - Configure `rust-lld.exe` for `x86_64-pc-windows-msvc`.
   - Verified local compilation with `rust-lld.exe` in 2m 06s.
2. [x] **Subtask 2: GitHub Actions Workflow Acceleration (`release.yml`)**
   - Migrate runner from `windows-2025` to `windows-2022`.
   - Add comprehensive Defender exclusions (`$env:USERPROFILE\.cargo`, `$env:USERPROFILE\.rustup`, `$env:TEMP`, workspace).
   - Register `rust-lld.exe` to `GITHUB_PATH`.
   - Set `CMAKE_BUILD_PARALLEL_LEVEL: 4` and `CARGO_BUILD_JOBS: 4`.
   - Set `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER: "rust-lld.exe"` and `RUSTFLAGS: -C linker=rust-lld.exe`.
   - Configure `cache-all-crates: "true"` and `cache-targets: "true"` in `Swatinem/rust-cache@v2`.
3. [x] **Subtask 3: CI Workflow Parity (`ci.yml`)**
   - Apply matching Windows runner optimizations to `ci.yml`.
4. [x] **Subtask 4: Spec & Memory Indices Update**
   - Update `02-spec/21-app/01-index.md` and `02-spec/22-app-issues/01-index.md`.
5. [x] **Subtask 5: Pre-Flight Verification & GitMap Commit**
   - Run `cargo fmt -- --check` (Passed).
   - Run `cargo clippy --all-targets --all-features` (Passed).
   - Run `npm run build` (Passed).
