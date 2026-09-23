# CI/CD RCA 35: macOS Universal App Bundle Missing Auxiliary Binary in Multi-Target Release Pipeline

## 1. Reproduction & Symptoms

In GitHub Actions workflow run `35819936677` (`Release` workflow on tag `v4.63.0`), the matrix job `build-tauri (macos-latest, --target universal-apple-darwin --bundles app)` failed during the `Build the app` step:

```text
       Built application at: /Users/runner/work/Antigravity-Manager/Antigravity-Manager/src-tauri/target/universal-apple-darwin/release/agm-alim
    Bundling agm-alim.app (/Users/runner/work/Antigravity-Manager/Antigravity-Manager/src-tauri/target/universal-apple-darwin/release/bundle/macos/agm-alim.app)
failed to bundle project Failed to copy binary from "/Users/runner/work/Antigravity-Manager/Antigravity-Manager/src-tauri/target/universal-apple-darwin/release/agm": `"/Users/runner/work/Antigravity-Manager/Antigravity-Manager/src-tauri/target/universal-apple-darwin/release/agm" does not exist`
       Error failed to bundle project Failed to copy binary from "/Users/runner/work/Antigravity-Manager/Antigravity-Manager/src-tauri/target/universal-apple-darwin/release/agm": `"/Users/runner/work/Antigravity-Manager/Antigravity-Manager/src-tauri/target/universal-apple-darwin/release/agm" does not exist`
##[error]Process completed with exit code 1.
```

All other runners (`windows-2025`, `ubuntu-22.04`, `ubuntu-24.04-arm`, `macos-latest aarch64`, and `macos-latest x86_64`) passed legitimately.

## 2. Root Cause Analysis

Tauri v2's CLI builds universal macOS applications for `--target universal-apple-darwin` by separately compiling `x86_64-apple-darwin` and `aarch64-apple-darwin` architectures and running `lipo` on the primary binary (`agm-alim`). However, when bundling the macOS `.app` bundle, Tauri's bundler iterates over all binaries declared under `[[bin]]` sections in `src-tauri/Cargo.toml` (which included both `agm-alim` and the standalone CLI `agm`). Because Tauri's CLI only lipo-merges the primary binary, `src-tauri/target/universal-apple-darwin/release/agm` was never created, causing the macOS bundler to fail with `Failed to copy binary: does not exist`.

Additionally, the compiler emitted `warning: use of async fn in public traits is discouraged as auto trait bounds cannot be specified` for `SystemIntegration::on_account_switch` in `src-tauri/src/modules/integration.rs`.

## 3. Code Fix & Remediation

1. **Created Before-Bundle Hook (`scripts/before-bundle.js`)**:
   - Implemented an automated pre-bundling hook invoked via `"beforeBundleCommand": "node scripts/before-bundle.js"` in `src-tauri/tauri.conf.json`.
   - When targeting `universal-apple-darwin`, the script verifies if `target/universal-apple-darwin/release/agm` exists.
   - If missing, it ensures `x86_64` and `aarch64` architectures are compiled and invokes `lipo -create -output` to assemble the universal Mach-O binary for `agm` before Tauri's bundler copies it into `.app/Contents/MacOS/`.
   - Supports dual root resolution for execution from repository root or `src-tauri`.
2. **Explicit Primary Binary (`src-tauri/Cargo.toml`)**:
   - Added `default-run = "agm-alim"` under `[package]` to explicitly mark the GUI application as the primary binary.
3. **Suppressed Trait Warning (`src-tauri/src/modules/integration.rs`)**:
   - Added `#[allow(async_fn_in_trait)]` above `pub trait SystemIntegration` to eliminate compiler diagnostic noise.

## 4. Prevention & Quality Guard

When adding auxiliary CLI or companion binaries to Tauri crates, always ensure that either:
- A `beforeBundleCommand` hook manages universal binary assembly via `lipo` before packaging, or
- Companion CLIs reside in dedicated sub-crates within a Cargo workspace so Tauri bundlers only package the GUI application binary.
