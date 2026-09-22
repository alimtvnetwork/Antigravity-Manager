# CI/CD RCA 33: MSVC Duplicate Resource Link Failure (CVT1100 / LNK1123) & Release Asset Decoupling

## Summary

- **Pipeline Run:** GitHub Actions Release Workflow #35737415089
- **Commit:** `3131f07`
- **Severity:** 🔴 Blocker (Build Matrix & Release Assets)
- **Status:** ✅ Fixed & Verified in `v4.60.0`

---

## 1. Why It Happened

1. **Windows MSVC Linker Error:**
   During the `npm run tauri build` execution on Windows runners, MSVC linker aborted with:
   ```text
   CVTRES : fatal error CVT1100: duplicate resource. type:VERSION, name:1, language:0x0409
   LINK : fatal error LNK1123: failure during conversion to COFF: file invalid or corrupt
   ```
2. **Compiler Lint Warnings:**
   Unused assignment and unused variable warnings in Rust proxy handlers (`claude.rs`, `gemini.rs`, `openai.rs`, `token_manager.rs`) cluttered build output.
3. **Empty Release Publishing:**
   Because all matrix compilation jobs failed, zero binaries reached `publish-release`. Downstream packaging published only `install.ps1`, `install.sh`, and empty checksums to GitHub Releases.

---

## 2. How It Happened

1. `tauri_build::build()` generates `resource.rc` which embeds the default manifest `1 24` into `resource.lib` for the application binary.
2. When `src-tauri/build.rs` also emitted `/MANIFEST:EMBED` with `/MANIFESTINPUT:windows-test.manifest`, MSVC's linker received two copies of the manifest resource, causing `CVTRES : fatal error CVT1100: duplicate resource. type:MANIFEST, name:1, language:0x0409`.
3. When `/MANIFEST:EMBED` was omitted, the `[lib]` test runner executable (`antigravity_tools_lib-*.exe`) did not receive any manifest because Cargo does not pass native static libs from build scripts to library test harnesses. Missing Common-Controls 6.0 caused Windows loader to crash with `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)` due to missing `TaskDialogIndirect` in `comctl32.dll` v5.82.
4. In `.github/workflows/release.yml`, `install.ps1` and `install.sh` were being copied into `release-files/`, and no binary artifact validation step existed before calling `ncipollo/release-action`.

---

## 3. Root Cause Analysis

1. **Manifest Collision Between `tauri-build` and MSVC Linker:** Supplying a manifest inside `resource.lib` while also requesting `/MANIFEST:EMBED` causes `cvtres.exe` to fail on duplicate manifest IDs. The clean solution is to invoke `tauri_build::WindowsAttributes::new_without_app_manifest()` on Windows MSVC. This leaves `resource.lib` containing only icon and version resources, allowing `/MANIFEST:EMBED` to provide `windows-test.manifest` (Common-Controls 6.0) across BOTH the application binary and the library test runner with zero duplicate resource conflicts.
2. **Missing Binary Asset Gate in Release CI/CD:** The release job must verify that at least one `.exe`, `.dmg`, `.AppImage`, `.deb`, or `.rpm` file is present in `release-files/` before creating a GitHub release.
3. **Installer Asset Misplacement:** Bootstrap installer scripts belong at the repository root and on git tags; they must not be bundled into binary release assets.

---

## 4. Corrective Action & Verification

1. **`src-tauri/build.rs`:**
   Configured `tauri_build::WindowsAttributes::new_without_app_manifest()` on Windows MSVC, and emitted `/MANIFEST:EMBED` with `/MANIFESTINPUT:windows-test.manifest`:
   ```rust
   if target_os == "windows" && target_env == "msvc" {
       let windows = tauri_build::WindowsAttributes::new_without_app_manifest();
       let attrs = tauri_build::Attributes::new().windows_attributes(windows);
       tauri_build::try_build(attrs).expect("failed to run tauri-build");

       let manifest_path = std::path::Path::new("windows-test.manifest");
       if manifest_path.exists() {
           println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
           if let Ok(abs_path) = manifest_path.canonicalize() {
               println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", abs_path.display());
           }
       }
       ...
   ```
2. **`src-tauri/src/proxy/handlers/*.rs`:** Fixed compiler warnings (`#[allow(unused_assignments)]` and underscore prefixes).
3. **`.github/workflows/release.yml` & `workflows/release.yml`:**
   - Removed copying `install.ps1` and `install.sh` into `release-files/`.
   - Added mandatory binary asset presence assertion.
   - Separated release notes into 4 individual code blocks for 1-click copying.
4. **Verification:**
   - Local `cargo check`: 0 warnings, 0 errors.
   - Local `cargo test --lib`: Succeeded (code 0), Common-Controls 6.0 loaded, 0xc0000139 eliminated.
   - Local `npm run tauri build -- --debug --no-bundle`: Succeeded (code 0), zero CVT1100 errors.
   - `install.ps1 -DryRun`: Verified clean execution.
